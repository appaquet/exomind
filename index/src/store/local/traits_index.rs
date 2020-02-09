use std::ops::Deref;
use std::path::Path;
use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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

use crate::error::Error;

const SCORE_TO_U64_MULTIPLIER: f32 = 10_000_000_000.0;
const UNIQUE_SORT_TO_U64_DIVIDER: f32 = 100_000_000.0;
const SEARCH_ENTITY_ID_LIMIT: usize = 1_000_000;

/// Traits index configuration
#[derive(Clone, Copy, Debug)]
pub struct TraitsIndexConfig {
    pub indexer_num_threads: Option<usize>,
    pub indexer_heap_size_bytes: usize,
    pub iterator_page_size: usize,
}

impl Default for TraitsIndexConfig {
    fn default() -> Self {
        TraitsIndexConfig {
            indexer_num_threads: None,
            indexer_heap_size_bytes: 50_000_000,
            iterator_page_size: 50,
        }
    }
}

/// Index (full-text & fields) for traits in the given schema. Each trait is individually
/// indexed as a single document.
///
/// This index is used to index both the chain, and the pending store. The chain index
/// is stored on disk, while the pending is stored in-memory. Deletion in each index
/// is handled differently. On the disk persisted, we delete using Tantivy document deletion,
/// while the pending-store uses a tombstone approach. This is needed since the pending store
/// "applies" mutation that may not be definitive onto the chain.
pub struct TraitsIndex {
    config: TraitsIndexConfig,
    index: TantivyIndex,
    index_reader: IndexReader,
    index_writer: Mutex<IndexWriter>,
    registry: Arc<Registry>,
    fields: Fields,
}

impl TraitsIndex {
    /// Creates or opens a disk persisted traits index
    pub fn open_or_create_mmap(
        config: TraitsIndexConfig,
        registry: Arc<Registry>,
        directory: &Path,
    ) -> Result<TraitsIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(registry.as_ref());
        let directory = MmapDirectory::open(directory)?;
        let index = TantivyIndex::open_or_create(directory, tantivy_schema)?;
        let index_reader = index.reader()?;
        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        Ok(TraitsIndex {
            config,
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            registry,
            fields,
        })
    }

    /// Creates or opens a in-memory traits index
    pub fn create_in_memory(
        config: TraitsIndexConfig,
        registry: Arc<Registry>,
    ) -> Result<TraitsIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(registry.as_ref());
        let index = TantivyIndex::create_in_ram(tantivy_schema);
        let index_reader = index.reader()?;
        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        Ok(TraitsIndex {
            config,
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            registry,
            fields,
        })
    }

    /// Apply a single trait mutation. A costly commit & refresh is done at each mutation,
    /// so `apply_mutations` should be used for multiple mutations.
    pub fn apply_mutation(&mut self, mutation: IndexMutation) -> Result<(), Error> {
        self.apply_mutations(Some(mutation).into_iter())
    }

    /// Apply an iterator of mutations, with a single atomic commit at the end of the iteration.
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
                IndexMutation::PutTrait(new_trait) => {
                    // delete older versions of the trait first
                    let entity_trait_id = format!("{}_{}", new_trait.entity_id, new_trait.trt.id);
                    trace!(
                        "Putting trait {} with op {}",
                        entity_trait_id,
                        new_trait.operation_id
                    );
                    index_writer.delete_term(Term::from_field_text(
                        self.fields.entity_trait_id,
                        &entity_trait_id,
                    ));

                    let doc = self.put_mutation_to_document(&new_trait)?;
                    index_writer.add_document(doc);
                }
                IndexMutation::PutTraitTombstone(trait_tombstone) => {
                    trace!(
                        "Putting tombstone for {}_{} with op {}",
                        trait_tombstone.entity_id,
                        trait_tombstone.trait_id,
                        trait_tombstone.operation_id
                    );
                    let doc = self.tombstone_mutation_to_document(&trait_tombstone);
                    index_writer.add_document(doc);
                }
                IndexMutation::DeleteTrait(entity_id, trait_id) => {
                    let entity_trait_id = format!("{}_{}", entity_id, trait_id);
                    trace!("Deleting trait {}", entity_trait_id,);
                    index_writer.delete_term(Term::from_field_text(
                        self.fields.entity_trait_id,
                        &entity_trait_id,
                    ));
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
            // it may take milliseconds for reader to see committed changes, so we force reload
            self.index_reader.reload()?;
        } else {
            debug!("Applied 0 mutations, not committing");
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

    /// Execute a query on the index and return a page of traits matching the query.
    pub fn search(
        &self,
        query: &EntityQuery,
        paging: Option<TraitPaging>,
    ) -> Result<TraitResults, Error> {
        let predicate = query
            .predicate
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("predicate"))?;

        match predicate {
            Predicate::Trait(inner_query) => {
                self.search_with_trait(&inner_query.trait_name, paging)
            }
            Predicate::Match(inner_query) => self.search_matches(&inner_query.query, paging),
            Predicate::Id(inner_query) => self.search_entity_id(&inner_query.id),
            Predicate::Test(_query) => Err(Error::Other("Query failed for tests".to_string())),
        }
    }

    /// Execute a query on the index and return an iterator over all matching traits.
    pub fn search_all<'i, 'q>(
        &'i self,
        query: &'q EntityQuery,
    ) -> Result<TraitResultsIterator<'i, 'q>, Error> {
        let results = self.search(query, None)?;

        Ok(TraitResultsIterator {
            index: self,
            query,
            total_results: results.total_results,
            current_results: results.results.into_iter(),
            next_page: results.next_page,
        })
    }

    /// Converts a put mutation to Tantivy document
    fn put_mutation_to_document(&self, mutation: &PutTraitMutation) -> Result<Document, Error> {
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

        let creation_ts = mutation
            .trt
            .creation_date
            .as_ref()
            .map(|d| d.to_timestamp_nanos())
            .unwrap_or(0);
        doc.add_u64(self.fields.creation_date, creation_ts);

        let modification_ts = mutation
            .trt
            .modification_date
            .as_ref()
            .map(|d| d.to_timestamp_nanos())
            .unwrap_or(creation_ts);
        doc.add_u64(self.fields.modification_date, modification_ts);

        // value added as stable randomness to stabilize order for documents with same score
        doc.add_u64(
            self.fields.sort_unique,
            u64::from(crc::crc16::checksum_usb(
                &mutation.operation_id.to_be_bytes(),
            )),
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

    /// Converts a tombstone mutation to Tantivy document
    fn tombstone_mutation_to_document(&self, mutation: &PutTraitTombstone) -> Document {
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

        doc.add_u64(self.fields.tombstone, 1);

        doc
    }

    /// Execute a search by trait type query
    pub fn search_with_trait(
        &self,
        trait_name: &str,
        paging: Option<TraitPaging>,
    ) -> Result<TraitResults, Error> {
        let searcher = self.index_reader.searcher();
        let paging = paging.unwrap_or_else(|| TraitPaging {
            after_score: None,
            before_score: None,
            count: self.config.iterator_page_size,
        });

        let term = Term::from_field_text(self.fields.trait_type, trait_name);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        let after_date = paging.after_score.unwrap_or(0);

        let before_date = paging.before_score.unwrap_or(std::u64::MAX);

        let total_count = Arc::new(AtomicUsize::new(0));
        let matching_count = Arc::new(AtomicUsize::new(0));
        let top_collector = {
            let total_count = total_count.clone();
            let matching_count = matching_count.clone();
            let mod_date_field = self.fields.modification_date;

            TopDocs::with_limit(paging.count as usize).custom_score(
                move |segment_reader: &SegmentReader| {
                    let total_docs = total_count.clone();
                    let remaining_count = matching_count.clone();

                    let mod_date_fast_field = segment_reader
                        .fast_fields()
                        .u64(mod_date_field)
                        .expect("Field requested is not a i64/u64 fast field.");
                    move |doc_id| {
                        let mod_date = mod_date_fast_field.get(doc_id);

                        total_docs.fetch_add(1, Ordering::SeqCst);
                        if mod_date > after_date && mod_date < before_date {
                            remaining_count.fetch_add(1, Ordering::SeqCst);
                            mod_date
                        } else {
                            0
                        }
                    }
                },
            )
        };

        let rescorer = |mod_date| {
            if mod_date > 0 {
                Some(mod_date)
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

        Ok(TraitResults {
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
        paging: Option<TraitPaging>,
    ) -> Result<TraitResults, Error> {
        let searcher = self.index_reader.searcher();
        let paging = paging.unwrap_or_else(|| TraitPaging {
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
            let sort_unique_field = self.fields.sort_unique;

            TopDocs::with_limit(paging.count as usize).tweak_score(
                move |segment_reader: &SegmentReader| {
                    let total_docs = total_count.clone();
                    let remaining_count = matching_count.clone();
                    let sort_unique_fast_field = segment_reader
                        .fast_fields()
                        .u64(sort_unique_field)
                        .expect("Field requested is not a i64/u64 fast field.");

                    move |doc_id, score| {
                        total_docs.fetch_add(1, Ordering::SeqCst);

                        // add stable randomness to score so that documents with same score don't equal
                        let sort_unique =
                            sort_unique_fast_field.get(doc_id) as f32 / UNIQUE_SORT_TO_U64_DIVIDER;
                        let rescored = score + sort_unique;
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

        Ok(TraitResults {
            results,
            total_results,
            remaining_results,
            next_page,
        })
    }

    /// Execute a search by entity id query
    pub fn search_entity_id(&self, entity_id: &str) -> Result<TraitResults, Error> {
        let searcher = self.index_reader.searcher();

        let term = Term::from_field_text(self.fields.entity_id, &entity_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let top_collector = TopDocs::with_limit(SEARCH_ENTITY_ID_LIMIT);
        let rescorer = |score| Some(score_to_u64(score));

        let results = self.execute_tantivy_query(searcher, &query, &top_collector, rescorer)?;
        let total_results = results.len();

        Ok(TraitResults {
            results,
            total_results,
            remaining_results: 0,
            next_page: None,
        })
    }

    /// Execute query on Tantivy index and build trait result
    fn execute_tantivy_query<S, C, SC, FS>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        top_collector: &C,
        rescorer: FS,
    ) -> Result<Vec<TraitResult>, Error>
    where
        S: Deref<Target = Searcher>,
        C: Collector<Fruit = Vec<(SC, DocAddress)>>,
        SC: Send + 'static,
        FS: Fn(SC) -> Option<u64>,
    {
        let search_results = searcher.search(query, top_collector)?;

        let mut results = Vec::new();
        for (score, doc_addr) in search_results {
            if let Some(score_u64) = rescorer(score) {
                let doc = searcher.doc(doc_addr)?;
                let block_offset = self.get_doc_opt_u64_value(&doc, self.fields.block_offset);
                let operation_id = self.get_doc_u64_value(&doc, self.fields.operation_id);
                let entity_id = self.get_doc_string_value(&doc, self.fields.entity_id);
                let trait_id = self.get_doc_string_value(&doc, self.fields.trait_id);
                let tombstone = self
                    .get_doc_opt_u64_value(&doc, self.fields.tombstone)
                    .map_or(false, |v| v == 1);

                let result = TraitResult {
                    block_offset,
                    operation_id,
                    entity_id,
                    trait_id,
                    tombstone,
                    score: score_u64,
                };
                results.push(result);
            }
        }

        Ok(results)
    }

    fn extract_next_page(previous_page: &TraitPaging, results: &[TraitResult]) -> TraitPaging {
        let last_result = results.last().expect("Should had results, but got none");
        TraitPaging {
            after_score: None,
            before_score: Some(last_result.score),
            count: previous_page.count,
        }
    }

    /// Extracts optional string value from Tantivy document
    fn get_doc_string_value(&self, doc: &Document, field: Field) -> String {
        match doc.get_first(field) {
            Some(tantivy::schema::Value::Str(v)) => v.to_string(),
            _ => panic!("Couldn't find field of type string"),
        }
    }

    /// Extracts optional u46 value from Tantivy document
    fn get_doc_opt_u64_value(&self, doc: &Document, field: Field) -> Option<u64> {
        match doc.get_first(field) {
            Some(tantivy::schema::Value::U64(v)) => Some(*v),
            _ => None,
        }
    }

    /// Extracts u46 value from Tantivy document
    fn get_doc_u64_value(&self, doc: &Document, field: Field) -> u64 {
        match doc.get_first(field) {
            Some(tantivy::schema::Value::U64(v)) => *v,
            _ => panic!("Couldn't find field of type u64"),
        }
    }

    /// Builds Tantivy schema based on the domain schema
    fn build_tantivy_schema(_registry: &Registry) -> (Schema, Fields) {
        let mut schema_builder = SchemaBuilder::default();
        schema_builder.add_text_field("trait_type", STRING | STORED);
        schema_builder.add_text_field("entity_id", STRING | STORED);
        schema_builder.add_text_field("trait_id", STRING | STORED);
        schema_builder.add_text_field("entity_trait_id", STRING);
        schema_builder.add_u64_field("creation_date", STORED | FAST);
        schema_builder.add_u64_field("modification_date", STORED | FAST);

        schema_builder.add_u64_field("block_offset", STORED | FAST);
        schema_builder.add_u64_field("operation_id", INDEXED | FAST | STORED);

        schema_builder.add_u64_field("tombstone", STORED);
        schema_builder.add_u64_field("sort_unique", STORED | FAST);

        schema_builder.add_text_field("text", TEXT);

        let schema = schema_builder.build();

        let fields = Fields {
            trait_type: schema.get_field("trait_type").unwrap(),
            entity_id: schema.get_field("entity_id").unwrap(),
            trait_id: schema.get_field("trait_id").unwrap(),
            entity_trait_id: schema.get_field("entity_trait_id").unwrap(),

            creation_date: schema.get_field("creation_date").unwrap(),
            modification_date: schema.get_field("modification_date").unwrap(),

            block_offset: schema.get_field("block_offset").unwrap(),
            operation_id: schema.get_field("operation_id").unwrap(),

            tombstone: schema.get_field("tombstone").unwrap(),
            sort_unique: schema.get_field("sort_unique").unwrap(),

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
    creation_date: Field,
    modification_date: Field,

    block_offset: Field,
    operation_id: Field,

    tombstone: Field,
    sort_unique: Field,

    text: Field,
}

pub enum IndexMutation {
    /// New version of a trait at a new position in chain or pending
    PutTrait(PutTraitMutation),

    /// Mark a trait has being delete without deleting it. This only used in pending index to notify
    /// the entities index that the trait was deleted (but it may be reverted, hence the
    /// tombstone)
    PutTraitTombstone(PutTraitTombstone),

    /// Delete a trait by its entity id and trait id from index
    DeleteTrait(String, String),

    /// Delete a trait by its operation id
    DeleteOperation(OperationId),
}

pub struct PutTraitMutation {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: String,
    pub trt: Trait,
}

pub struct PutTraitTombstone {
    pub block_offset: Option<BlockOffset>,
    pub operation_id: OperationId,
    pub entity_id: String,
    pub trait_id: String,
}

/// Collection of `TraitResult`
pub struct TraitResults {
    pub results: Vec<TraitResult>,
    pub total_results: usize,
    pub remaining_results: usize,
    pub next_page: Option<TraitPaging>,
}

/// Indexed trait returned as a result of a query
#[derive(Debug)]
pub struct TraitResult {
    pub operation_id: OperationId,
    pub block_offset: Option<BlockOffset>,
    pub entity_id: String,
    pub trait_id: String,
    pub tombstone: bool,
    pub score: u64,
}

#[derive(Debug, Clone)]
pub struct TraitPaging {
    pub after_score: Option<u64>,
    pub before_score: Option<u64>,
    pub count: usize,
}

/// Iterates through all results matching a given initial query using the next_page score
/// when a page got emptied.
pub struct TraitResultsIterator<'i, 'q> {
    index: &'i TraitsIndex,
    query: &'q EntityQuery,
    pub total_results: usize,
    current_results: std::vec::IntoIter<TraitResult>,
    next_page: Option<TraitPaging>,
}

impl<'i, 'q> Iterator for TraitResultsIterator<'i, 'q> {
    type Item = TraitResult;

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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use itertools::Itertools;

    use exocore_core::node::LocalNode;
    use exocore_core::protos::generated::exocore_test::{TestMessage, TestMessage2};
    use exocore_core::protos::prost::{ProstAnyPackMessageExt, ProstDateTimeExt};
    use exocore_core::time::Clock;

    use crate::query::{QueryBuilder, SortToken};

    use super::*;

    #[test]
    fn search_by_entity_id() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut indexer = TraitsIndex::create_in_memory(config, registry)?;

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
        indexer.apply_mutation(trait1)?;

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
        indexer.apply_mutation(trait2)?;

        let contact3 = IndexMutation::PutTrait(PutTraitMutation {
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
        indexer.apply_mutation(contact3)?;

        let results = indexer.search_entity_id("entity_id1")?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].block_offset, Some(1));
        assert_eq!(results.results[0].operation_id, 10);
        assert_eq!(results.results[0].entity_id, "entity_id1");
        assert_eq!(results.results[0].trait_id, "foo1");

        let results = indexer.search_entity_id("entity_id2")?;
        assert_eq!(results.results.len(), 2);
        find_trait_result(&results, "foo2");
        find_trait_result(&results, "foo3");

        // search all should return an iterator all results
        let query = QueryBuilder::with_entity_id("entity_id2").build();
        let iter = indexer.search_all(&query)?;
        assert_eq!(iter.total_results, 2);
        let results = iter.collect_vec();
        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[test]
    fn search_query_matches() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut indexer = TraitsIndex::create_in_memory(config, registry)?;

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
        indexer.apply_mutation(trait1)?;

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
        indexer.apply_mutation(trait2)?;

        let results = indexer.search_matches("foo", None)?;
        assert_eq!(results.results.len(), 2);
        assert_eq!(results.results[0].entity_id, "entity_id1"); // foo is repeated in entity 1

        let results = indexer.search_matches("bar", None)?;
        assert_eq!(results.results.len(), 2);
        assert!(results.results[0].score > score_to_u64(0.30));
        assert!(results.results[1].score > score_to_u64(0.18));
        assert_eq!(results.results[0].entity_id, "entity_id2"); // foo is repeated in entity 2

        // with limit
        let paging = TraitPaging {
            after_score: None,
            before_score: None,
            count: 1,
        };
        let results = indexer.search_matches("foo", Some(paging))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 1);
        assert_eq!(results.total_results, 2);

        // only results from given score
        let paging = TraitPaging {
            after_score: Some(score_to_u64(0.30)),
            before_score: None,
            count: 10,
        };
        let results = indexer.search_matches("bar", Some(paging))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.total_results, 2);
        assert_eq!(results.results[0].entity_id, "entity_id2");

        // only results before given score
        let paging = TraitPaging {
            after_score: None,
            before_score: Some(score_to_u64(0.30)),
            count: 10,
        };
        let results = indexer.search_matches("bar", Some(paging))?;
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
        let mut indexer = TraitsIndex::create_in_memory(config, registry)?;
        let clock = Clock::new();
        let node = LocalNode::generate();

        let traits = (0..30).map(|i| {
            let op_id = clock.consistent_time(node.node()).into();
            IndexMutation::PutTrait(PutTraitMutation {
                block_offset: Some(i),
                operation_id: op_id,
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
        indexer.apply_mutations(traits)?;

        let paging = TraitPaging {
            after_score: None,
            before_score: None,
            count: 10,
        };
        let results1 = indexer.search_matches("foo", Some(paging))?;
        assert_eq!(results1.total_results, 30);
        assert_eq!(results1.results.len(), 10);
        assert_eq!(results1.remaining_results, 20);
        find_trait_result(&results1, "id29");
        find_trait_result(&results1, "id20");

        let results2 = indexer.search_matches("foo", Some(results1.next_page.clone().unwrap()))?;
        assert_eq!(results2.total_results, 30);
        assert_eq!(results2.results.len(), 10);
        assert_eq!(results2.remaining_results, 10);
        find_trait_result(&results1, "id19");
        find_trait_result(&results1, "id10");

        let results3 = indexer.search_matches("foo", Some(results2.next_page.unwrap()))?;
        assert_eq!(results3.total_results, 30);
        assert_eq!(results3.results.len(), 10);
        assert_eq!(results3.remaining_results, 0);
        find_trait_result(&results1, "id9");
        find_trait_result(&results1, "id0");

        // search all should return an iterator over all results
        let query = QueryBuilder::match_text("foo").build();
        let iter = indexer.search_all(&query)?;
        assert_eq!(iter.total_results, 30);
        let results = iter.collect_vec();
        assert_eq!(results.len(), 30);

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = TraitsIndex::create_in_memory(config, registry)?;

        let date = "2019-08-01T12:00:00Z"
            .parse::<DateTime<Utc>>()?
            .to_proto_timestamp();
        let trait1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1,
            entity_id: "entity_id1".to_string(),
            trt: Trait {
                id: "trt1".to_string(),
                creation_date: Some(date.clone()),
                modification_date: Some(date),
                message: Some(
                    TestMessage {
                        string1: "Foo Bar".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
            },
        });

        let date = "2019-09-01T12:00:00Z"
            .parse::<DateTime<Utc>>()?
            .to_proto_timestamp();
        let trait2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id2".to_string(),
            trt: Trait {
                id: "trait2".to_string(),
                creation_date: Some(date.clone()),
                modification_date: Some(date),
                message: Some(
                    TestMessage2 {
                        string1: "Some subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
            },
        });

        let date = "2019-09-03T12:00:00Z"
            .parse::<DateTime<Utc>>()?
            .to_proto_timestamp();
        let trait3 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 3,
            entity_id: "entity_id3".to_string(),
            trt: Trait {
                id: "trait3".to_string(),
                creation_date: Some(date.clone()),
                modification_date: Some(date),
                message: Some(
                    TestMessage2 {
                        string1: "Some subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
            },
        });

        let date = "2019-09-02T12:00:00Z"
            .parse::<DateTime<Utc>>()?
            .to_proto_timestamp();
        let trait4 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 4,
            entity_id: "entity_id4".to_string(),
            trt: Trait {
                id: "trait4".to_string(),
                creation_date: Some(date.clone()),
                modification_date: Some(date),
                message: Some(
                    TestMessage2 {
                        string1: "Some subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()?,
                ),
            },
        });

        index.apply_mutations(vec![trait1, trait2, trait3, trait4].into_iter())?;

        let results = index.search_with_trait("exocore.test.TestMessage", None)?;
        assert_eq!(results.results.len(), 1);
        assert!(find_trait_result(&results, "trt1").is_some());

        // ordering of multiple traits is by modification date
        let results = index.search_with_trait("exocore.test.TestMessage2", None)?;
        let traits_ids = results
            .results
            .iter()
            .map(|res| res.trait_id.clone())
            .collect_vec();
        assert_eq!(traits_ids, vec!["trait3", "trait4", "trait2"]);

        // with limit
        let paging = TraitPaging {
            after_score: None,
            before_score: None,
            count: 1,
        };
        let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
        assert_eq!(results.results.len(), 1);

        // only results after given modification date
        let date_token = SortToken::from_datetime("2019-09-02T11:59:00Z".parse::<DateTime<Utc>>()?);
        let date_value = date_token.to_u64()?;
        let paging = TraitPaging {
            after_score: Some(date_value),
            before_score: None,
            count: 10,
        };
        let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
        let traits_ids = results
            .results
            .iter()
            .map(|res| res.trait_id.clone())
            .collect_vec();
        assert_eq!(traits_ids, vec!["trait3", "trait4"]);

        // only results before given modification date
        let paging = TraitPaging {
            after_score: None,
            before_score: Some(date_value),
            count: 10,
        };
        let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
        let traits_ids = results
            .results
            .iter()
            .map(|res| res.trait_id.clone())
            .collect_vec();
        assert_eq!(traits_ids, vec!["trait2"]);

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type_paging() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut indexer = TraitsIndex::create_in_memory(config, registry)?;
        let clock = Clock::new();
        let node = LocalNode::generate();

        let contacts = (0..30).map(|i| {
            let now = clock.consistent_time(node.node());

            IndexMutation::PutTrait(PutTraitMutation {
                block_offset: Some(i),
                operation_id: now.into(),
                entity_id: format!("entity_id{}", i),
                trt: Trait {
                    id: format!("entity_id{}", i),
                    creation_date: Some(now.to_datetime().to_proto_timestamp()),
                    modification_date: Some(now.to_datetime().to_proto_timestamp()),
                    message: Some(
                        TestMessage {
                            string1: "Some Subject".to_string(),
                            ..Default::default()
                        }
                        .pack_to_any()
                        .unwrap(),
                    ),
                },
            })
        });
        indexer.apply_mutations(contacts)?;

        let paging = TraitPaging {
            after_score: None,
            before_score: None,
            count: 10,
        };

        let results1 = indexer.search_with_trait("exocore.test.TestMessage", Some(paging))?;
        assert_eq!(results1.total_results, 30);
        assert_eq!(results1.remaining_results, 20);
        assert_eq!(results1.results.len(), 10);
        find_trait_result(&results1, "id29");
        find_trait_result(&results1, "id20");

        let results2 = indexer.search_with_trait(
            "exocore.test.TestMessage",
            Some(results1.next_page.clone().unwrap()),
        )?;
        assert_eq!(results2.total_results, 30);
        assert_eq!(results2.remaining_results, 10);
        assert_eq!(results2.results.len(), 10);
        find_trait_result(&results1, "id19");
        find_trait_result(&results1, "id10");

        let results3 = indexer.search_with_trait(
            "exocore.test.TestMessage",
            Some(results2.next_page.unwrap()),
        )?;
        assert_eq!(results3.total_results, 30);
        assert_eq!(results3.remaining_results, 0);
        assert_eq!(results3.results.len(), 10);
        find_trait_result(&results1, "id9");
        find_trait_result(&results1, "id0");

        // search all should return an iterator over all results
        let query = QueryBuilder::with_trait("exocore.test.TestMessage").build();
        let iter = indexer.search_all(&query)?;
        assert_eq!(iter.total_results, 30);
        let results = iter.collect_vec();
        assert_eq!(results.len(), 30);

        Ok(())
    }

    #[test]
    fn highest_indexed_block() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = TraitsIndex::create_in_memory(config, registry)?;

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
    fn update_trait() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = TraitsIndex::create_in_memory(config, registry)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1,
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
        index.apply_mutation(contact_mutation)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
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
        index.apply_mutation(contact_mutation)?;

        let results = index.search_matches("foo", None)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.total_results, 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.results[0].operation_id, 2);

        Ok(())
    }

    #[test]
    fn delete_trait_mutation() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = TraitsIndex::create_in_memory(config, registry)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
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
        index.apply_mutation(contact_mutation)?;

        assert_eq!(index.search_matches("foo", None)?.results.len(), 1);

        index.apply_mutation(IndexMutation::DeleteTrait(
            "entity_id1".to_string(),
            "foo1".to_string(),
        ))?;

        assert_eq!(index.search_matches("foo", None)?.results.len(), 0);

        Ok(())
    }

    #[test]
    fn delete_operation_id_mutation() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = TraitsIndex::create_in_memory(config, registry)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
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
        index.apply_mutation(contact_mutation)?;

        assert_eq!(index.search_matches("foo", None)?.results.len(), 1);

        index.apply_mutation(IndexMutation::DeleteOperation(1234))?;

        assert_eq!(index.search_matches("foo", None)?.results.len(), 0);

        Ok(())
    }

    #[test]
    fn put_trait_tombstone() -> Result<(), failure::Error> {
        let registry = Arc::new(Registry::new_with_exocore_types());
        let config = test_config();
        let mut index = TraitsIndex::create_in_memory(config, registry)?;

        let contact_mutation = IndexMutation::PutTraitTombstone(PutTraitTombstone {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
            trait_id: "foo1".to_string(),
        });
        index.apply_mutation(contact_mutation)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
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
        index.apply_mutation(contact_mutation)?;

        let res = index.search_entity_id("entity_id1")?;
        assert!(res.results.first().unwrap().tombstone);

        let res = index.search_entity_id("entity_id2")?;
        assert!(!res.results.first().unwrap().tombstone);

        Ok(())
    }

    fn test_config() -> TraitsIndexConfig {
        TraitsIndexConfig {
            iterator_page_size: 7,
            ..TraitsIndexConfig::default()
        }
    }

    fn find_trait_result<'r>(results: &'r TraitResults, trait_id: &str) -> Option<&'r TraitResult> {
        results.results.iter().find(|t| t.trait_id == trait_id)
    }
}
