use std::ops::Deref;
use std::path::Path;
use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, TimeZone, Utc};
use tantivy::collector::{Collector, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, QueryParser, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, Schema, SchemaBuilder, FAST, INDEXED, STORED, STRING, TEXT,
};
use tantivy::{
    DocAddress, Document, Index as TantivyIndex, IndexReader, IndexWriter, Searcher, SegmentReader,
    Term,
};

use exocore_core::protos::generated::exocore_index::{entity_query::Predicate, EntityQuery, Trait};
use exocore_core::protos::prost::ProstTimestampExt;
use exocore_core::protos::reflect;
use exocore_core::protos::reflect::{FieldType, FieldValue, ReflectMessage};
use exocore_core::protos::registry::Registry;
use exocore_data::block::BlockOffset;
use exocore_data::operation::OperationId;

use crate::entity::{EntityId, TraitId};
use crate::error::Error;

const SCORE_TO_U64_MULTIPLIER: f32 = 10_000_000_000.0;
const UNIQUE_SORT_TO_U64_DIVIDER: f32 = 100_000_000.0;
const SEARCH_ENTITY_ID_LIMIT: usize = 1_000_000;

/// Index (full-text & fields) for entities & traits mutations stored in the data layer. Each
/// mutation is individually indexed as a single document.
///
/// This index is used to index both the chain, and the pending store mutations. The chain
/// index is stored on disk, while the pending is stored in-memory. Deletions are handled
/// by using tombstones that are eventually compacted by the store once the number of mutations
/// is too high on an entity.
pub struct MutationIndex {
    config: MutationIndexConfig,
    index: TantivyIndex,
    index_reader: IndexReader,
    index_writer: Mutex<IndexWriter>,
    registry: Arc<Registry>,
    fields: Fields,
}

impl MutationIndex {
    /// Creates or opens a disk persisted index.
    pub fn open_or_create_mmap(
        config: MutationIndexConfig,
        registry: Arc<Registry>,
        directory: &Path,
    ) -> Result<MutationIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(registry.as_ref());
        let directory = MmapDirectory::open(directory)?;
        let index = TantivyIndex::open_or_create(directory, tantivy_schema)?;
        let index_reader = index.reader()?;
        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        Ok(MutationIndex {
            config,
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            registry,
            fields,
        })
    }

    /// Creates or opens a in-memory index.
    pub fn create_in_memory(
        config: MutationIndexConfig,
        registry: Arc<Registry>,
    ) -> Result<MutationIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(registry.as_ref());
        let index = TantivyIndex::create_in_ram(tantivy_schema);
        let index_reader = index.reader()?;
        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        Ok(MutationIndex {
            config,
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            registry,
            fields,
        })
    }

    /// Apply a single mutation. A costly commit & refresh is done at each
    /// mutation, so `apply_mutations` should be used for multiple
    /// mutations.
    #[cfg(test)]
    fn apply_mutation(&mut self, mutation: IndexMutation) -> Result<(), Error> {
        self.apply_mutations(Some(mutation).into_iter())
    }

    /// Apply an iterator of mutations, with a single atomic commit at the end
    /// of the iteration.
    pub fn apply_mutations<T>(&mut self, mutations: T) -> Result<(), Error>
    where
        T: Iterator<Item = IndexMutation>,
    {
        let mut index_writer = self.index_writer.lock()?;

        debug!("Starting applying mutations to index...");
        let mut nb_mutations = 0;
        for mutation in mutations {
            nb_mutations += 1;

            match mutation {
                IndexMutation::PutTrait(trait_put) => {
                    let entity_trait_id = format!("{}_{}", trait_put.entity_id, trait_put.trt.id);
                    trace!(
                        "Putting trait {} with op {}",
                        entity_trait_id,
                        trait_put.operation_id
                    );
                    let doc = self.trait_put_to_document(&trait_put)?;
                    index_writer.add_document(doc);
                }
                IndexMutation::PutTraitTombstone(trait_tombstone) => {
                    trace!(
                        "Putting tombstone for trait {}_{} with op {}",
                        trait_tombstone.entity_id,
                        trait_tombstone.trait_id,
                        trait_tombstone.operation_id
                    );
                    let doc = self.trait_tombstone_to_document(&trait_tombstone);
                    index_writer.add_document(doc);
                }
                IndexMutation::PutEntityTombstone(entity_tombstone) => {
                    trace!(
                        "Putting tombstone for entity {} with op {}",
                        entity_tombstone.entity_id,
                        entity_tombstone.operation_id
                    );
                    let doc = self.entity_tombstone_to_document(&entity_tombstone);
                    index_writer.add_document(doc);
                }
                IndexMutation::DeleteOperation(operation_id) => {
                    trace!("Deleting op from index {}", operation_id);
                    index_writer
                        .delete_term(Term::from_field_u64(self.fields.operation_id, operation_id));
                }
            }
        }

        if nb_mutations > 0 {
            debug!("Applied {} mutations, now committing", nb_mutations);
            index_writer.commit()?;
            // it may take milliseconds for reader to see committed changes, so we force
            // reload
            self.index_reader.reload()?;
        }

        Ok(())
    }

    /// Return the highest block offset found in the index.
    /// This is used to know from which point we need to re-index.
    pub fn highest_indexed_block(&self) -> Result<Option<BlockOffset>, Error> {
        let searcher = self.index_reader.searcher();

        let query = AllQuery;
        let top_collector = TopDocs::with_limit(1).order_by_u64_field(self.fields.block_offset);
        let search_results = searcher.search(&query, &top_collector)?;

        Ok(search_results
            .first()
            .map(|(block_offset, _doc_addr)| *block_offset))
    }

    /// Execute a query on the index and return a page of mutations matching the
    /// query.
    pub fn search(
        &self,
        query: &EntityQuery,
        paging: Option<QueryPaging>,
    ) -> Result<MutationResults, Error> {
        let predicate = query
            .predicate
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("predicate"))?;

        match predicate {
            Predicate::Trait(inner) => self.search_with_trait(&inner.trait_name, paging),
            Predicate::Match(inner) => self.search_matches(&inner.query, paging),
            Predicate::Id(inner) => self.search_entity_id(&inner.id),
            Predicate::Test(_inner) => Err(Error::Other("Query failed for tests".to_string())),
        }
    }

    /// Execute a query on the index and return an iterator over all matching
    /// mutations.
    pub fn search_all<'i, 'q>(
        &'i self,
        query: &'q EntityQuery,
    ) -> Result<ResultsIterator<'i, 'q>, Error> {
        let results = self.search(query, None)?;

        Ok(ResultsIterator {
            index: self,
            query,
            total_results: results.total_results,
            current_results: results.results.into_iter(),
            next_page: results.next_page,
        })
    }

    /// Converts a trait put / update to Tantivy document
    fn trait_put_to_document(&self, mutation: &PutTraitMutation) -> Result<Document, Error> {
        let message = mutation
            .trt
            .message
            .as_ref()
            .ok_or_else(|| Error::ProtoFieldExpected("Trait message"))?;
        let dyn_message =
            reflect::from_prost_any(self.registry.as_ref(), message).map_err(Error::Proto)?;

        let mut doc = Document::default();
        doc.add_text(self.fields.trait_type, dyn_message.full_name());
        doc.add_text(self.fields.trait_id, &mutation.trt.id);
        doc.add_text(self.fields.entity_id, &mutation.entity_id);
        doc.add_text(
            self.fields.entity_trait_id,
            &format!("{}_{}", mutation.entity_id, &mutation.trt.id),
        );

        doc.add_u64(self.fields.operation_id, mutation.operation_id);
        if let Some(block_offset) = mutation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        if let Some(creation_date) = &mutation.trt.creation_date {
            doc.add_u64(
                self.fields.creation_date,
                creation_date.to_timestamp_nanos(),
            );
        }
        if let Some(modification_date) = &mutation.trt.modification_date {
            doc.add_u64(
                self.fields.modification_date,
                modification_date.to_timestamp_nanos(),
            );
        }

        // random value added to each document to make sorting by documents of the same
        // score deterministic
        doc.add_u64(
            self.fields.sort_tie_breaker,
            u64::from(crc::crc16::checksum_usb(
                &mutation.operation_id.to_be_bytes(),
            )),
        );

        doc.add_u64(
            self.fields.document_type,
            MutationMetadataType::TRAIT_PUT_ID,
        );

        for field in dyn_message.fields() {
            if !field.indexed_flag {
                continue;
            }

            match &field.field_type {
                FieldType::String => {
                    if let Ok(FieldValue::String(val)) = dyn_message.get_field_value(field) {
                        doc.add_text(self.fields.text, &val);
                    }
                }
                ft => {
                    warn!("Unsupported indexed field type: {:?}", ft);
                }
            }
        }

        Ok(doc)
    }

    /// Converts a trait tombstone mutation to Tantivy document
    fn trait_tombstone_to_document(&self, mutation: &PutTraitTombstone) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.fields.trait_id, &mutation.trait_id);
        doc.add_text(self.fields.entity_id, &mutation.entity_id);
        doc.add_text(
            self.fields.entity_trait_id,
            &format!("{}_{}", mutation.entity_id, mutation.trait_id),
        );
        doc.add_u64(self.fields.operation_id, mutation.operation_id);

        if let Some(block_offset) = mutation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        doc.add_u64(
            self.fields.document_type,
            MutationMetadataType::TRAIT_TOMBSTONE_ID,
        );

        doc
    }

    /// Converts an entity tombstone mutation to Tantivy document
    fn entity_tombstone_to_document(&self, mutation: &PutEntityTombstone) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.fields.entity_id, &mutation.entity_id);
        doc.add_u64(self.fields.operation_id, mutation.operation_id);

        if let Some(block_offset) = mutation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        doc.add_u64(
            self.fields.document_type,
            MutationMetadataType::ENTITY_TOMBSTONE_ID,
        );

        doc
    }

    /// Execute a search by trait type query and return traits in operations id descending order.
    pub fn search_with_trait(
        &self,
        trait_name: &str,
        paging: Option<QueryPaging>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();
        let paging = paging.unwrap_or_else(|| QueryPaging {
            after_score: None,
            before_score: None,
            count: self.config.iterator_page_size,
        });

        let term = Term::from_field_text(self.fields.trait_type, trait_name);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        let after_op = paging.after_score.unwrap_or(0);
        let before_op = paging.before_score.unwrap_or(std::u64::MAX);

        let total_count = Arc::new(AtomicUsize::new(0));
        let matching_count = Arc::new(AtomicUsize::new(0));
        let top_collector = {
            let total_count = total_count.clone();
            let matching_count = matching_count.clone();
            let op_id_field = self.fields.operation_id;

            TopDocs::with_limit(paging.count as usize).custom_score(
                move |segment_reader: &SegmentReader| {
                    let total_docs = total_count.clone();
                    let remaining_count = matching_count.clone();

                    let op_id_fast_field = segment_reader
                        .fast_fields()
                        .u64(op_id_field)
                        .expect("Field requested is not a i64/u64 fast field.");
                    move |doc_id| {
                        let op_id = op_id_fast_field.get(doc_id);

                        total_docs.fetch_add(1, Ordering::SeqCst);
                        if op_id > after_op && op_id < before_op {
                            remaining_count.fetch_add(1, Ordering::SeqCst);
                            op_id
                        } else {
                            0
                        }
                    }
                },
            )
        };

        let rescorer = |op_id| {
            if op_id > 0 {
                Some(op_id)
            } else {
                None
            }
        };

        let results = self.execute_tantivy_query(searcher, &query, &top_collector, rescorer)?;
        let total_results = total_count.load(Ordering::Relaxed);
        let remaining_results = matching_count
            .load(Ordering::Relaxed)
            .saturating_sub(results.len());
        let next_page = if remaining_results > 0 {
            Some(Self::extract_next_page(&paging, &results))
        } else {
            None
        };

        Ok(MutationResults {
            results,
            total_results,
            remaining_results,
            next_page,
        })
    }

    /// Execute a search by text query
    pub fn search_matches(
        &self,
        query: &str,
        paging: Option<QueryPaging>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();
        let paging = paging.unwrap_or_else(|| QueryPaging {
            after_score: None,
            before_score: None,
            count: self.config.iterator_page_size,
        });

        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.text]);
        let query = query_parser.parse_query(query)?;

        let after_score = paging.after_score.map(score_from_u64).unwrap_or(0.0);

        let before_score = paging
            .before_score
            .map(score_from_u64)
            .unwrap_or(std::f32::MAX);

        let total_count = Arc::new(AtomicUsize::new(0));
        let matching_count = Arc::new(AtomicUsize::new(0));
        let top_collector = {
            let total_count = total_count.clone();
            let matching_count = matching_count.clone();
            let sort_tie_breaker_field = self.fields.sort_tie_breaker;

            TopDocs::with_limit(paging.count as usize).tweak_score(
                move |segment_reader: &SegmentReader| {
                    let total_docs = total_count.clone();
                    let remaining_count = matching_count.clone();
                    let sort_tie_breaker_fast_field = segment_reader
                        .fast_fields()
                        .u64(sort_tie_breaker_field)
                        .expect("Field requested is not a i64/u64 fast field.");

                    move |doc_id, score| {
                        total_docs.fetch_add(1, Ordering::SeqCst);

                        // add stable randomness to score so that documents with same score don't
                        // equal
                        let sort_tie_breaker = sort_tie_breaker_fast_field.get(doc_id) as f32
                            / UNIQUE_SORT_TO_U64_DIVIDER;
                        let rescored = score + sort_tie_breaker;
                        if rescored > after_score && rescored < before_score {
                            remaining_count.fetch_add(1, Ordering::SeqCst);
                            rescored
                        } else {
                            std::f32::MIN
                        }
                    }
                },
            )
        };

        let rescorer = |score| {
            // top collector changes score to negative if result shouldn't be considered
            if score > 0.0 {
                Some(score_to_u64(score))
            } else {
                None
            }
        };

        let results = self.execute_tantivy_query(searcher, &query, &top_collector, rescorer)?;
        let total_results = total_count.load(Ordering::Relaxed);
        let remaining_results = matching_count
            .load(Ordering::Relaxed)
            .saturating_sub(results.len());
        let next_page = if remaining_results > 0 {
            Some(Self::extract_next_page(&paging, &results))
        } else {
            None
        };

        Ok(MutationResults {
            results,
            total_results,
            remaining_results,
            next_page,
        })
    }

    /// Execute a search by entity id query
    pub fn search_entity_id(&self, entity_id: &str) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let term = Term::from_field_text(self.fields.entity_id, &entity_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let top_collector = TopDocs::with_limit(SEARCH_ENTITY_ID_LIMIT);
        let rescorer = |score| Some(score_to_u64(score));

        let results = self.execute_tantivy_query(searcher, &query, &top_collector, rescorer)?;
        let total_results = results.len();

        Ok(MutationResults {
            results,
            total_results,
            remaining_results: 0,
            next_page: None,
        })
    }

    /// Execute query on Tantivy index and build mutations result
    fn execute_tantivy_query<S, C, SC, FS>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        top_collector: &C,
        rescorer: FS,
    ) -> Result<Vec<MutationMetadata>, Error>
    where
        S: Deref<Target = Searcher>,
        C: Collector<Fruit = Vec<(SC, DocAddress)>>,
        SC: Send + 'static,
        FS: Fn(SC) -> Option<u64>,
    {
        let search_results = searcher.search(query, top_collector)?;

        let mut results = Vec::new();
        for (score, doc_addr) in search_results {
            if let Some(score) = rescorer(score) {
                let doc = searcher.doc(doc_addr)?;
                let block_offset = get_doc_opt_u64_value(&doc, self.fields.block_offset);
                let operation_id = get_doc_u64_value(&doc, self.fields.operation_id);
                let entity_id = get_doc_string_value(&doc, self.fields.entity_id);
                let opt_trait_id = get_doc_opt_string_value(&doc, self.fields.trait_id);
                let document_type_id = get_doc_u64_value(&doc, self.fields.document_type);

                let mut mutation_type = MutationMetadataType::new(document_type_id, opt_trait_id)?;

                if let MutationMetadataType::TraitPut(put_trait) = &mut mutation_type {
                    put_trait.creation_date =
                        get_doc_opt_u64_value(&doc, self.fields.creation_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                    put_trait.modification_date =
                        get_doc_opt_u64_value(&doc, self.fields.modification_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                }

                let result = MutationMetadata {
                    block_offset,
                    operation_id,
                    entity_id,
                    mutation_type,
                    score,
                };
                results.push(result);
            }
        }

        Ok(results)
    }

    fn extract_next_page(previous_page: &QueryPaging, results: &[MutationMetadata]) -> QueryPaging {
        let last_result = results.last().expect("Should had results, but got none");
        QueryPaging {
            after_score: None,
            before_score: Some(last_result.score),
            count: previous_page.count,
        }
    }

    /// Builds Tantivy schema based on the domain schema
    fn build_tantivy_schema(_registry: &Registry) -> (Schema, Fields) {
        let mut schema_builder = SchemaBuilder::default();
        schema_builder.add_text_field("trait_type", STRING | STORED);
        schema_builder.add_text_field("entity_id", STRING | STORED);
        schema_builder.add_text_field("trait_id", STRING | STORED);
        schema_builder.add_text_field("entity_trait_id", STRING);
        schema_builder.add_u64_field("sort_tie_breaker", STORED | FAST);
        schema_builder.add_u64_field("creation_date", STORED);
        schema_builder.add_u64_field("modification_date", STORED);
        schema_builder.add_u64_field("block_offset", STORED | FAST);
        schema_builder.add_u64_field("operation_id", INDEXED | STORED | FAST);
        schema_builder.add_u64_field("document_type", STORED);
        schema_builder.add_text_field("text", TEXT);

        let schema = schema_builder.build();

        let fields = Fields {
            trait_type: schema.get_field("trait_type").unwrap(),
            entity_id: schema.get_field("entity_id").unwrap(),
            trait_id: schema.get_field("trait_id").unwrap(),
            entity_trait_id: schema.get_field("entity_trait_id").unwrap(),
            sort_tie_breaker: schema.get_field("sort_tie_breaker").unwrap(),
            creation_date: schema.get_field("creation_date").unwrap(),
            modification_date: schema.get_field("modification_date").unwrap(),
            block_offset: schema.get_field("block_offset").unwrap(),
            operation_id: schema.get_field("operation_id").unwrap(),
            document_type: schema.get_field("document_type").unwrap(),
            text: schema.get_field("text").unwrap(),
        };

        (schema, fields)
    }
}

/// Trait index configuration
#[derive(Clone, Copy, Debug)]
pub struct MutationIndexConfig {
    pub indexer_num_threads: Option<usize>,
    pub indexer_heap_size_bytes: usize,
    pub iterator_page_size: usize,
}

impl Default for MutationIndexConfig {
    fn default() -> Self {
        MutationIndexConfig {
            indexer_num_threads: None,
            indexer_heap_size_bytes: 50_000_000,
            iterator_page_size: 50,
        }
    }
}

/// Tantitvy schema fields
struct Fields {
    trait_type: Field,
    entity_id: Field,
    trait_id: Field,
    entity_trait_id: Field,
    sort_tie_breaker: Field,
    creation_date: Field,
    modification_date: Field,
    block_offset: Field,
    operation_id: Field,
    document_type: Field,
    text: Field,
}

/// Mutation of the index.
pub enum IndexMutation {
    /// New version of a trait at a new position in chain or pending
    PutTrait(PutTraitMutation),

    /// Mark a trait has being deleted without deleting it.
    PutTraitTombstone(PutTraitTombstone),

    /// Mark an entity has being deleted without deleting it.
    PutEntityTombstone(PutEntityTombstone),

    /// Delete a document by its operation id.
    DeleteOperation(OperationId),
}

pub struct PutTraitMutation {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
    pub trt: Trait,
}

pub struct PutTraitTombstone {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
    pub trait_id: TraitId,
}

pub struct PutEntityTombstone {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: EntityId,
}

/// Collection of `MutationMetadata`
pub struct MutationResults {
    pub results: Vec<MutationMetadata>,
    pub total_results: usize,
    pub remaining_results: usize,
    pub next_page: Option<QueryPaging>,
}

/// Indexed trait / entity mutation metadata returned as a result of a query.
#[derive(Debug, Clone)]
pub struct MutationMetadata {
    pub operation_id: OperationId,
    pub block_offset: Option<BlockOffset>,
    pub entity_id: EntityId,
    pub score: u64,
    pub mutation_type: MutationMetadataType,
}

#[derive(Debug, Clone)]
pub enum MutationMetadataType {
    TraitPut(PutTraitMetadata),
    TraitTombstone(TraitId),
    EntityTombstone,
}

#[derive(Debug, Clone)]
pub struct PutTraitMetadata {
    pub trait_id: TraitId,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
}

impl MutationMetadataType {
    const TRAIT_TOMBSTONE_ID: u64 = 0;
    const TRAIT_PUT_ID: u64 = 1;
    const ENTITY_TOMBSTONE_ID: u64 = 2;

    fn new(
        document_type_id: u64,
        opt_trait_id: Option<TraitId>,
    ) -> Result<MutationMetadataType, Error> {
        match document_type_id {
            Self::TRAIT_TOMBSTONE_ID => {
                Ok(MutationMetadataType::TraitTombstone(opt_trait_id.unwrap()))
            }
            Self::TRAIT_PUT_ID => Ok(MutationMetadataType::TraitPut(PutTraitMetadata {
                trait_id: opt_trait_id.unwrap(),
                creation_date: None,
                modification_date: None,
            })),
            Self::ENTITY_TOMBSTONE_ID => Ok(MutationMetadataType::EntityTombstone),
            _ => Err(Error::Fatal(format!(
                "Invalid document type id {}",
                document_type_id
            ))),
        }
    }
}

/// Paging information for mutations querying.
#[derive(Debug, Clone)]
pub struct QueryPaging {
    pub after_score: Option<u64>,
    pub before_score: Option<u64>,
    pub count: usize,
}

/// Iterates through all results matching a given initial query using the
/// next_page score when a page got emptied.
pub struct ResultsIterator<'i, 'q> {
    index: &'i MutationIndex,
    query: &'q EntityQuery,
    pub total_results: usize,
    current_results: std::vec::IntoIter<MutationMetadata>,
    next_page: Option<QueryPaging>,
}

impl<'i, 'q> Iterator for ResultsIterator<'i, 'q> {
    type Item = MutationMetadata;

    fn next(&mut self) -> Option<Self::Item> {
        let next_result = self.current_results.next();
        if let Some(next_result) = next_result {
            Some(next_result)
        } else {
            let next_page = self.next_page.clone()?;
            let results = self
                .index
                .search(self.query, Some(next_page))
                .expect("Couldn't get another page from initial iterator query");
            self.next_page = results.next_page;
            self.current_results = results.results.into_iter();

            self.current_results.next()
        }
    }
}

/// Convert Tantivy f32 score to u64
fn score_to_u64(score: f32) -> u64 {
    (score * SCORE_TO_U64_MULTIPLIER) as u64
}

/// Convert u64 score to Tantivy f32
fn score_from_u64(value: u64) -> f32 {
    value as f32 / SCORE_TO_U64_MULTIPLIER
}

/// Extracts string value from Tantivy document
fn get_doc_string_value(doc: &Document, field: Field) -> String {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::Str(v)) => v.to_string(),
        _ => panic!("Couldn't find field of type string"),
    }
}

/// Extracts optional string value from Tantivy document
fn get_doc_opt_string_value(doc: &Document, field: Field) -> Option<String> {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::Str(v)) => Some(v.to_string()),
        _ => None,
    }
}

/// Extracts optional u46 value from Tantivy document
fn get_doc_opt_u64_value(doc: &Document, field: Field) -> Option<u64> {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::U64(v)) => Some(*v),
        _ => None,
    }
}

/// Extracts u46 value from Tantivy document
fn get_doc_u64_value(doc: &Document, field: Field) -> u64 {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::U64(v)) => *v,
        _ => panic!("Couldn't find field of type u64"),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use itertools::Itertools;

    use exocore_core::protos::generated::exocore_test::{TestMessage, TestMessage2};
    use exocore_core::protos::prost::{ProstAnyPackMessageExt, ProstDateTimeExt};

    use crate::query::QueryBuilder;

    use super::*;

    #[test]
    fn search_by_entity_id() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1),
            operation_id: 10,
            entity_id: "entity_id1".to_string(),
            trt: Trait {
                id: "foo1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Foo Foo Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        let trait2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(2),
            operation_id: 20,
            entity_id: "entity_id2".to_string(),
            trt: Trait {
                id: "foo2".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Foo Foo Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        let trait3 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(3),
            operation_id: 21,
            entity_id: "entity_id2".to_string(),
            trt: Trait {
                id: "foo3".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Foo Foo Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });
        index.apply_mutations(vec![trait1, trait2, trait3].into_iter())?;

        let results = index.search_entity_id("entity_id1")?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].block_offset, Some(1));
        assert_eq!(results.results[0].operation_id, 10);
        assert_eq!(results.results[0].entity_id, "entity_id1");
        assert_is_put_trait(&results.results[0].mutation_type, "foo1");

        let results = index.search_entity_id("entity_id2")?;
        assert_eq!(results.results.len(), 2);
        find_put_trait(&results, "foo2");
        find_put_trait(&results, "foo3");

        // search all should return an iterator all results
        let query = QueryBuilder::with_entity_id("entity_id2").build();
        let iter = index.search_all(&query)?;
        assert_eq!(iter.total_results, 2);
        let results = iter.collect_vec();
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[test]
    fn search_query_matches() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1),
            operation_id: 10,
            entity_id: "entity_id1".to_string(),
            trt: Trait {
                id: "foo1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Foo Foo Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        let trait2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(2),
            operation_id: 20,
            entity_id: "entity_id2".to_string(),
            trt: Trait {
                id: "foo2".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Bar Bar Bar Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });
        index.apply_mutations(vec![trait1, trait2].into_iter())?;

        let results = index.search_matches("foo", None)?;
        assert_eq!(results.results.len(), 2);
        assert_eq!(results.results[0].entity_id, "entity_id1"); // foo is repeated in entity 1

        let results = index.search_matches("bar", None)?;
        assert_eq!(results.results.len(), 2);
        assert!(results.results[0].score > score_to_u64(0.30));
        assert!(results.results[1].score > score_to_u64(0.18));
        assert_eq!(results.results[0].entity_id, "entity_id2"); // foo is repeated in entity 2

        // with limit
        let paging = QueryPaging {
            after_score: None,
            before_score: None,
            count: 1,
        };
        let results = index.search_matches("foo", Some(paging))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 1);
        assert_eq!(results.total_results, 2);

        // only results from given score
        let paging = QueryPaging {
            after_score: Some(score_to_u64(0.30)),
            before_score: None,
            count: 10,
        };
        let results = index.search_matches("bar", Some(paging))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.total_results, 2);
        assert_eq!(results.results[0].entity_id, "entity_id2");

        // only results before given score
        let paging = QueryPaging {
            after_score: None,
            before_score: Some(score_to_u64(0.30)),
            count: 10,
        };
        let results = index.search_matches("bar", Some(paging))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.total_results, 2);
        assert_eq!(results.results[0].entity_id, "entity_id1");

        Ok(())
    }

    #[test]
    fn search_query_matches_paging() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let traits = (0..30).map(|i| {
            IndexMutation::PutTrait(PutTraitMutation {
                block_offset: Some(i),
                operation_id: i,
                entity_id: format!("entity_id{}", i),
                trt: Trait {
                    id: format!("entity_id{}", i),
                    message: Some(
                        TestMessage {
                            string1: "Foo Bar".to_string(),
                            ..Default::default()
                        }
                        .pack_to_any()
                        .unwrap(),
                    ),
                    ..Default::default()
                },
            })
        });
        index.apply_mutations(traits)?;

        let paging = QueryPaging {
            after_score: None,
            before_score: None,
            count: 10,
        };
        let results1 = index.search_matches("foo", Some(paging))?;
        assert_eq!(results1.total_results, 30);
        assert_eq!(results1.results.len(), 10);
        assert_eq!(results1.remaining_results, 20);
        find_put_trait(&results1, "id29");
        find_put_trait(&results1, "id20");

        let results2 = index.search_matches("foo", Some(results1.next_page.clone().unwrap()))?;
        assert_eq!(results2.total_results, 30);
        assert_eq!(results2.results.len(), 10);
        assert_eq!(results2.remaining_results, 10);
        find_put_trait(&results1, "id19");
        find_put_trait(&results1, "id10");

        let results3 = index.search_matches("foo", Some(results2.next_page.unwrap()))?;
        assert_eq!(results3.total_results, 30);
        assert_eq!(results3.results.len(), 10);
        assert_eq!(results3.remaining_results, 0);
        find_put_trait(&results1, "id9");
        find_put_trait(&results1, "id0");

        // search all should return an iterator over all results
        let query = QueryBuilder::match_text("foo").build();
        let iter = index.search_all(&query)?;
        assert_eq!(iter.total_results, 30);
        let results = iter.collect_vec();
        assert_eq!(results.len(), 30);

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1,
            entity_id: "entity_id1".to_string(),
            trt: Trait {
                id: "trt1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        let trait2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id2".to_string(),
            trt: Trait {
                id: "trait2".to_string(),
                message: Some(
                    TestMessage2 {
                        string1: "Some subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        let trait3 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 3,
            entity_id: "entity_id3".to_string(),
            trt: Trait {
                id: "trait3".to_string(),
                message: Some(
                    TestMessage2 {
                        string1: "Some subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        let trait4 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 4,
            entity_id: "entity_id4".to_string(),
            trt: Trait {
                id: "trait4".to_string(),
                message: Some(
                    TestMessage2 {
                        string1: "Some subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });

        index.apply_mutations(vec![trait4, trait3, trait2, trait1].into_iter())?;

        let results = index.search_with_trait("exocore.test.TestMessage", None)?;
        assert_eq!(results.results.len(), 1);
        assert!(find_put_trait(&results, "trt1").is_some());

        // ordering of multiple traits is operation id
        let results = index.search_with_trait("exocore.test.TestMessage2", None)?;
        assert_eq!(
            extract_traits_id(results),
            vec!["trait4", "trait3", "trait2"]
        );

        // with limit
        let paging = QueryPaging {
            after_score: None,
            before_score: None,
            count: 1,
        };
        let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
        assert_eq!(results.results.len(), 1);

        // only results after given modification date
        let paging = QueryPaging {
            after_score: Some(2),
            before_score: None,
            count: 10,
        };
        let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
        assert_eq!(extract_traits_id(results), vec!["trait4", "trait3"]);

        // only results before given modification date
        let paging = QueryPaging {
            after_score: None,
            before_score: Some(3),
            count: 10,
        };
        let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
        assert_eq!(extract_traits_id(results), vec!["trait2"]);

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type_paging() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let traits = (0..30).map(|i| {
            IndexMutation::PutTrait(PutTraitMutation {
                block_offset: Some(i),
                operation_id: 30 - i,
                entity_id: format!("entity_id{}", i),
                trt: Trait {
                    id: format!("entity_id{}", i),
                    message: Some(
                        TestMessage {
                            string1: "Some Subject".to_string(),
                            ..Default::default()
                        }
                        .pack_to_any()
                        .unwrap(),
                    ),
                    ..Default::default()
                },
            })
        });
        index.apply_mutations(traits)?;

        let paging = QueryPaging {
            after_score: None,
            before_score: None,
            count: 10,
        };

        let results1 = index.search_with_trait("exocore.test.TestMessage", Some(paging))?;
        assert_eq!(results1.total_results, 30);
        assert_eq!(results1.remaining_results, 20);
        assert_eq!(results1.results.len(), 10);
        find_put_trait(&results1, "id29");
        find_put_trait(&results1, "id20");

        let results2 = index.search_with_trait(
            "exocore.test.TestMessage",
            Some(results1.next_page.clone().unwrap()),
        )?;
        assert_eq!(results2.total_results, 30);
        assert_eq!(results2.remaining_results, 10);
        assert_eq!(results2.results.len(), 10);
        find_put_trait(&results1, "id19");
        find_put_trait(&results1, "id10");

        let results3 = index.search_with_trait(
            "exocore.test.TestMessage",
            Some(results2.next_page.unwrap()),
        )?;
        assert_eq!(results3.total_results, 30);
        assert_eq!(results3.remaining_results, 0);
        assert_eq!(results3.results.len(), 10);
        find_put_trait(&results1, "id9");
        find_put_trait(&results1, "id0");

        // search all should return an iterator over all results
        let query = QueryBuilder::with_trait("exocore.test.TestMessage").build();
        let iter = index.search_all(&query)?;
        assert_eq!(iter.total_results, 30);
        let results = iter.collect_vec();
        assert_eq!(results.len(), 30);

        Ok(())
    }

    #[test]
    fn highest_indexed_block() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        assert_eq!(index.highest_indexed_block()?, None);

        index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1234),
            operation_id: 1,
            entity_id: "et1".to_string(),
            trt: Trait {
                id: "trt1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Some Subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        }))?;
        assert_eq!(index.highest_indexed_block()?, Some(1234));

        index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(120),
            operation_id: 2,
            entity_id: "et1".to_string(),
            trt: Trait {
                id: "trt2".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Some Subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        }))?;
        assert_eq!(index.highest_indexed_block()?, Some(1234));

        index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(9999),
            operation_id: 3,
            entity_id: "et1".to_string(),
            trt: Trait {
                id: "trt1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Some Subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        }))?;
        assert_eq!(index.highest_indexed_block()?, Some(9999));

        Ok(())
    }

    #[test]
    fn delete_operation_id_mutation() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
            trt: Trait {
                id: "foo1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });
        index.apply_mutation(trait1)?;

        assert_eq!(index.search_matches("foo", None)?.results.len(), 1);

        index.apply_mutation(IndexMutation::DeleteOperation(1234))?;

        assert_eq!(index.search_matches("foo", None)?.results.len(), 0);

        Ok(())
    }

    #[test]
    fn put_trait_tombstone() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let contact_mutation = IndexMutation::PutTraitTombstone(PutTraitTombstone {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
            trait_id: "foo1".to_string(),
        });
        index.apply_mutation(contact_mutation)?;

        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2345,
            entity_id: "entity_id2".to_string(),
            trt: Trait {
                id: "foo2".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                ..Default::default()
            },
        });
        index.apply_mutation(trait1)?;

        let res = index.search_entity_id("entity_id1")?;
        assert_is_trait_tombstone(&res.results.first().unwrap().mutation_type, "foo1");

        let res = index.search_entity_id("entity_id2")?;
        assert_is_put_trait(&res.results.first().unwrap().mutation_type, "foo2");

        Ok(())
    }

    #[test]
    fn put_entity_tombstone() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let trait1 = IndexMutation::PutEntityTombstone(PutEntityTombstone {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
        });
        index.apply_mutation(trait1)?;

        let res = index.search_entity_id("entity_id1")?;
        assert_is_entity_tombstone(&res.results.first().unwrap().mutation_type);

        Ok(())
    }

    #[test]
    fn trait_dates() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = MutationIndex::create_in_memory(config, registry)?;

        let creation_date = "2019-08-01T12:00:00Z".parse::<DateTime<Utc>>()?;
        let modification_date = "2019-12-03T12:00:00Z".parse::<DateTime<Utc>>()?;

        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1),
            operation_id: 10,
            entity_id: "entity_id1".to_string(),
            trt: Trait {
                id: "foo1".to_string(),
                message: Some(
                    TestMessage {
                        string1: "Foo Foo Foo Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
                creation_date: Some(creation_date.to_proto_timestamp()),
                modification_date: Some(modification_date.to_proto_timestamp()),
            },
        });
        index.apply_mutation(trait1)?;

        let res = index.search_entity_id("entity_id1")?;
        let trait_meta = find_put_trait(&res, "foo1").unwrap();
        let trait_put = assert_is_put_trait(&trait_meta.mutation_type, "foo1");
        assert_eq!(creation_date, trait_put.creation_date.unwrap());
        assert_eq!(modification_date, trait_put.modification_date.unwrap());

        Ok(())
    }

    fn test_config() -> MutationIndexConfig {
        MutationIndexConfig {
            iterator_page_size: 7,
            ..MutationIndexConfig::default()
        }
    }

    fn find_put_trait<'r>(
        results: &'r MutationResults,
        trait_id: &str,
    ) -> Option<&'r MutationMetadata> {
        results.results.iter().find(|t| match &t.mutation_type {
            MutationMetadataType::TraitPut(put_trait) if put_trait.trait_id == trait_id => true,
            _ => false,
        })
    }

    fn assert_is_put_trait<'r>(
        document_type: &'r MutationMetadataType,
        trait_id: &str,
    ) -> &'r PutTraitMetadata {
        match document_type {
            MutationMetadataType::TraitPut(put_trait) if put_trait.trait_id == trait_id => {
                put_trait
            }
            other => panic!("Expected TraitPut type, but got {:?}", other),
        }
    }

    fn assert_is_trait_tombstone(document_type: &MutationMetadataType, trait_id: &str) {
        match document_type {
            MutationMetadataType::TraitTombstone(trt_id) if trt_id == trait_id => {}
            other => panic!("Expected TraitTombstone type, but got {:?}", other),
        }
    }

    fn assert_is_entity_tombstone(document_type: &MutationMetadataType) {
        match document_type {
            MutationMetadataType::EntityTombstone => {}
            other => panic!("Expected EntityTombstone type, but got {:?}", other),
        }
    }

    fn extract_traits_id(results: MutationResults) -> Vec<String> {
        results
            .results
            .iter()
            .map(|res| match &res.mutation_type {
                MutationMetadataType::TraitPut(put_trait) => put_trait.trait_id.clone(),
                other => panic!("Expected trait put, got something else: {:?}", other),
            })
            .collect()
    }
}
