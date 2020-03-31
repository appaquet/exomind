use std::ops::Deref;
use std::path::Path;
use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{TimeZone, Utc};
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

use exocore_core::protos::generated::exocore_index::{entity_query::Predicate, EntityQuery};
use exocore_core::protos::prost::ProstTimestampExt;
use exocore_core::protos::reflect;
use exocore_core::protos::reflect::{FieldType, FieldValue, ReflectMessage};
use exocore_core::protos::registry::Registry;
use exocore_data::block::BlockOffset;

use crate::error::Error;

mod config;
mod mutation;
mod results;
#[cfg(test)]
mod tests;

pub use config::*;
pub use mutation::*;
pub use results::*;

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
    schemas: Arc<Registry>,
    fields: Fields,
}

impl MutationIndex {
    /// Creates or opens a disk persisted index.
    pub fn open_or_create_mmap(
        config: MutationIndexConfig,
        schemas: Arc<Registry>,
        directory: &Path,
    ) -> Result<MutationIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(schemas.as_ref());
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
            schemas,
            fields,
        })
    }

    /// Creates or opens a in-memory index.
    pub fn create_in_memory(
        config: MutationIndexConfig,
        schemas: Arc<Registry>,
    ) -> Result<MutationIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(schemas.as_ref());
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
            schemas,
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
            reflect::from_prost_any(self.schemas.as_ref(), message).map_err(Error::Proto)?;

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

/// Paging information for mutations querying.
#[derive(Debug, Clone)]
pub struct QueryPaging {
    pub after_score: Option<u64>,
    pub before_score: Option<u64>,
    pub count: usize,
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
