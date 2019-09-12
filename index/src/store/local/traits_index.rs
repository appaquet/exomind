use crate::error::Error;
use crate::query::*;
use exocore_data::block::BlockOffset;
use exocore_data::operation::OperationId;
use exocore_schema::entity::{EntityId, FieldValue, Record, Trait, TraitId};
use exocore_schema::schema;
use exocore_schema::schema::RecordSchema;
use std::ops::Deref;
use std::path::Path;
use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tantivy::collector::{Collector, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, QueryParser, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, Schema as TantivySchema, SchemaBuilder, FAST, INDEXED, STORED,
    STRING, TEXT,
};
use tantivy::{
    DocAddress, Document, Index as TantivyIndex, IndexReader, IndexWriter, Searcher, SegmentReader,
    Term,
};

const SCORE_TO_U64_MULTIPLIER: f32 = 10_000_000_000.0;
const UNIQUE_SORT_TO_U64_DIVIDER: f32 = 100_000_000.0;
const SEARCH_ENTITY_ID_LIMIT: usize = 1_000_000;

///
/// Traits index configuration
///
#[derive(Clone, Copy, Debug)]
pub struct TraitsIndexConfig {
    pub indexer_num_threads: Option<usize>,
    pub indexer_heap_size_bytes: usize,
}

impl Default for TraitsIndexConfig {
    fn default() -> Self {
        TraitsIndexConfig {
            indexer_num_threads: None,
            indexer_heap_size_bytes: 50_000_000,
        }
    }
}

///
/// Index (full-text & fields) for traits in the given schema. Each trait is individually
/// indexed as a single document.
///
/// This index is used to index both the chain, and the pending store. The chain index
/// is stored on disk, while the pending is stored in-memory. Deletion in each index
/// is handled differently. On the disk persisted, we delete using Tantivy document deletion,
/// while the pending-store uses a tombstone approach. This is needed since the pending store
/// "applies" mutation that may not be definitive onto the chain.
///
pub struct TraitsIndex {
    index: TantivyIndex,
    index_reader: IndexReader,
    index_writer: Mutex<IndexWriter>,
    schema: Arc<schema::Schema>,
    fields: Fields,
}

impl TraitsIndex {
    /// Creates or opens a disk persisted traits index
    pub fn open_or_create_mmap(
        config: TraitsIndexConfig,
        schema: Arc<schema::Schema>,
        directory: &Path,
    ) -> Result<TraitsIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(schema.as_ref());
        let directory = MmapDirectory::open(directory)?;
        let index = TantivyIndex::open_or_create(directory, tantivy_schema.clone())?;
        let index_reader = index.reader()?;
        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        Ok(TraitsIndex {
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            schema,
            fields,
        })
    }

    /// Creates or opens a in-memory traits index
    pub fn create_in_memory(
        config: TraitsIndexConfig,
        schema: Arc<schema::Schema>,
    ) -> Result<TraitsIndex, Error> {
        let (tantivy_schema, fields) = Self::build_tantivy_schema(schema.as_ref());
        let index = TantivyIndex::create_in_ram(tantivy_schema.clone());
        let index_reader = index.reader()?;
        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        Ok(TraitsIndex {
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            schema,
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
                    let entity_trait_id = format!("{}_{}", new_trait.entity_id, new_trait.trt.id());
                    index_writer.delete_term(Term::from_field_text(
                        self.fields.entity_trait_id,
                        &entity_trait_id,
                    ));

                    let doc = self.put_mutation_to_document(&new_trait);
                    index_writer.add_document(doc);
                }
                IndexMutation::PutTraitTombstone(trait_tombstone) => {
                    let doc = self.tombstone_mutation_to_document(&trait_tombstone);
                    index_writer.add_document(doc);
                }
                IndexMutation::DeleteTrait(entity_id, trait_id) => {
                    let entity_trait_id = format!("{}_{}", entity_id, trait_id);
                    index_writer.delete_term(Term::from_field_text(
                        self.fields.entity_trait_id,
                        &entity_trait_id,
                    ));
                }
                IndexMutation::DeleteOperation(operation_id) => {
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

    /// Execute a query on the index and return the matching traits' information.
    /// The actual data of the traits is stored in the chain.
    pub fn search(&self, query: &Query) -> Result<TraitResults, Error> {
        let searcher = self.index_reader.searcher();

        let paging = query.paging_or_default();
        match &query.inner {
            InnerQuery::WithTrait(inner_query) => {
                self.search_with_trait(searcher, inner_query, paging)
            }
            InnerQuery::Match(inner_query) => self.search_matches(searcher, inner_query, paging),
            InnerQuery::IdEqual(inner_query) => self.search_entity_id(searcher, inner_query),

            #[cfg(test)]
            InnerQuery::TestFail(_query) => Err(Error::Other("Query failed for tests".to_string())),
        }
    }

    /// Converts a put mutation to Tantivy document
    fn put_mutation_to_document(&self, mutation: &PutTraitMutation) -> Document {
        let record_schema: &schema::TraitSchema = mutation.trt.record_schema();
        let entity_trait_id = &format!("{}_{}", mutation.entity_id, mutation.trt.id());

        let mut doc = Document::default();
        doc.add_u64(self.fields.trait_type, u64::from(record_schema.id()));
        doc.add_text(self.fields.trait_id, &mutation.trt.id());
        doc.add_text(self.fields.entity_id, &mutation.entity_id);
        doc.add_text(self.fields.entity_trait_id, entity_trait_id);

        doc.add_u64(self.fields.operation_id, mutation.operation_id);
        if let Some(block_offset) = mutation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        doc.add_u64(
            self.fields.creation_date,
            mutation.trt.creation_date().timestamp_nanos() as u64,
        );
        doc.add_u64(
            self.fields.modification_date,
            mutation.trt.modification_date().timestamp_nanos() as u64,
        );

        // value added as stable randomness to stabilize order for documents with same score
        doc.add_u64(
            self.fields.sort_unique,
            u64::from(crc::crc16::checksum_usb(
                &mutation.operation_id.to_be_bytes(),
            )),
        );

        let indexed_fields = record_schema.fields().iter().filter(|f| f.indexed);
        for field in indexed_fields {
            if let Some(field_value) = mutation.trt.value(field) {
                match (&field.typ, field_value) {
                    (schema::FieldType::String, FieldValue::String(v)) => {
                        doc.add_text(self.fields.text, &v);
                    }
                    _ => panic!(
                        "Type not supported yet: ({:?}, {:?})",
                        field.typ, field_value
                    ),
                }
            }
        }

        doc
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
    fn search_with_trait<S>(
        &self,
        searcher: S,
        inner_query: &WithTraitQuery,
        paging: &QueryPaging,
    ) -> Result<TraitResults, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let trait_schema =
            if let Some(trait_schema) = self.schema.trait_by_full_name(&inner_query.trait_name) {
                trait_schema
            } else {
                warn!(
                    "Tried to search for trait {}, but doesn't exist in schema",
                    inner_query.trait_name
                );
                return Ok(TraitResults::new_empty());
            };

        let term = Term::from_field_u64(self.fields.trait_type, u64::from(trait_schema.id()));
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        let after_date = paging
            .after_token
            .as_ref()
            .map(|token| token.to_u64().map(|value| value))
            .unwrap_or(Ok(0))?;

        let before_date = paging
            .before_token
            .as_ref()
            .map(|token| token.to_u64().map(|value| value))
            .unwrap_or(Ok(std::u64::MAX))?;

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
            .checked_sub(results.len())
            .unwrap_or(0);
        let next_page = if remaining_results > 0 {
            Some(Self::extract_next_page_token(&paging, &results))
        } else {
            None
        };

        Ok(TraitResults {
            results,
            total_results,
            remaining_results,
            current_page: Some(paging.clone()),
            next_page,
        })
    }

    /// Execute a search by text query
    fn search_matches<S>(
        &self,
        searcher: S,
        inner_query: &MatchQuery,
        paging: &QueryPaging,
    ) -> Result<TraitResults, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.text]);
        let query = query_parser.parse_query(&inner_query.query)?;

        let after_score = paging
            .after_token
            .as_ref()
            .map(sort_token_to_score)
            .unwrap_or(Ok(0.0))?;

        let before_score = paging
            .before_token
            .as_ref()
            .map(sort_token_to_score)
            .unwrap_or(Ok(std::f32::MAX))?;

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
            .checked_sub(results.len())
            .unwrap_or(0);
        let next_page = if remaining_results > 0 {
            Some(Self::extract_next_page_token(&paging, &results))
        } else {
            None
        };

        Ok(TraitResults {
            results,
            total_results,
            remaining_results,
            current_page: Some(paging.clone()),
            next_page,
        })
    }

    /// Execute a search by entity id query
    fn search_entity_id<S>(
        &self,
        searcher: S,
        inner_query: &IdEqualQuery,
    ) -> Result<TraitResults, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let term = Term::from_field_text(self.fields.entity_id, &inner_query.entity_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let top_collector = TopDocs::with_limit(SEARCH_ENTITY_ID_LIMIT);
        let rescorer = |score| Some(score_to_u64(score));

        let results = self.execute_tantivy_query(searcher, &query, &top_collector, rescorer)?;
        let total_results = results.len();

        Ok(TraitResults {
            results,
            total_results,
            remaining_results: 0,
            current_page: None,
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

    fn extract_next_page_token(
        previous_page: &QueryPaging,
        results: &[TraitResult],
    ) -> QueryPaging {
        let last_result = results.last().expect("Should had results, but got none");
        let last_token = SortToken::from_u64(last_result.score);
        QueryPaging {
            after_token: None,
            before_token: Some(last_token),
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
    fn build_tantivy_schema(_schema: &schema::Schema) -> (TantivySchema, Fields) {
        let mut schema_builder = SchemaBuilder::default();
        schema_builder.add_u64_field("trait_type", INDEXED);
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

///
/// Tantivy fields used by traits index
///
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

///
/// Mutation to applied to the index
///
pub enum IndexMutation {
    /// New version of a trait at a new position in chain or pending
    PutTrait(PutTraitMutation),

    /// Mark a trait has being delete without deleting it. This only used in pending index to notify
    /// the entities index that the trait was deleted (but it may be reverted, hence the
    /// tombstone)
    PutTraitTombstone(PutTraitTombstone),

    /// Delete a trait by its entity id and trait id from index
    DeleteTrait(EntityId, TraitId),

    /// Delete a trait by its operation id
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

///
/// Indexed trait returned as a result of a query
///
#[derive(Debug)]
pub struct TraitResult {
    pub operation_id: OperationId,
    pub block_offset: Option<BlockOffset>,
    pub entity_id: EntityId,
    pub trait_id: TraitId,
    pub tombstone: bool,
    pub score: u64,
}

///
/// Collection of `TraitResult`
///
#[derive(Debug)]
pub struct TraitResults {
    pub results: Vec<TraitResult>,
    pub total_results: usize,
    pub remaining_results: usize,
    pub current_page: Option<QueryPaging>,
    pub next_page: Option<QueryPaging>,
}

impl TraitResults {
    fn new_empty() -> TraitResults {
        TraitResults {
            results: vec![],
            total_results: 0,
            remaining_results: 0,
            current_page: None,
            next_page: None,
        }
    }
}

/// Convert SortToken string to Tantivy f32 score
fn sort_token_to_score(token: &SortToken) -> Result<f32, Error> {
    token
        .to_u64()
        .map(|score| score as f32 / SCORE_TO_U64_MULTIPLIER)
}

/// Convert Tantivy f32 score to SortToken string
#[cfg(test)]
fn score_to_sort_token(score: f32) -> SortToken {
    SortToken::from_u64(score_to_u64(score))
}

/// Convert Tantivy f32 score to u64
fn score_to_u64(score: f32) -> u64 {
    (score * SCORE_TO_U64_MULTIPLIER) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use exocore_common::node::LocalNode;
    use exocore_common::time::Clock;
    use exocore_schema::entity::{RecordBuilder, TraitBuilder};
    use exocore_schema::tests_utils::create_test_schema;
    use itertools::Itertools;

    #[test]
    fn search_by_entity_id() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut indexer = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1),
            operation_id: 10,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Justin Justin Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        indexer.apply_mutation(contact1)?;

        let contact2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(2),
            operation_id: 20,
            entity_id: "entity_id2".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau2")
                .set("name", "Justin Trudeau Trudeau Trudeau Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        indexer.apply_mutation(contact2)?;

        let results = indexer.search(&Query::with_entity_id("entity_id1"))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].block_offset, Some(1));
        assert_eq!(results.results[0].operation_id, 10);
        assert_eq!(results.results[0].entity_id, "entity_id1");
        assert_eq!(results.results[0].trait_id, "trudeau1");

        let results = indexer.search(&Query::with_entity_id("entity_id2"))?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].block_offset, Some(2));
        assert_eq!(results.results[0].operation_id, 20);
        assert_eq!(results.results[0].entity_id, "entity_id2");
        assert_eq!(results.results[0].trait_id, "trudeau2");

        Ok(())
    }

    #[test]
    fn search_query_matches() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut indexer = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1),
            operation_id: 10,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Justin Justin Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        indexer.apply_mutation(contact1)?;

        let contact2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(2),
            operation_id: 20,
            entity_id: "entity_id2".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau2")
                .set("name", "Justin Trudeau Trudeau Trudeau Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        indexer.apply_mutation(contact2)?;

        let query = Query::match_text("justin");
        let results = indexer.search(&query)?;
        assert_eq!(results.results.len(), 2);
        assert_eq!(results.results[0].entity_id, "entity_id1"); // justin is repeated in entity 1

        let query = Query::match_text("trudeau");
        let results = indexer.search(&query)?;
        assert_eq!(results.results.len(), 2);
        assert!(results.results[0].score > score_to_u64(0.32));
        assert!(results.results[1].score > score_to_u64(0.25));
        assert_eq!(results.results[0].entity_id, "entity_id2"); // trudeau is repeated in entity 2

        // with limit
        let query = Query::match_text("trudeau").with_paging(QueryPaging::new(1));
        let results = indexer.search(&query)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 1);
        assert_eq!(results.total_results, 2);

        // only results from given score
        let query = Query::match_text("trudeau")
            .with_paging(QueryPaging::new(10).with_from_token(score_to_sort_token(0.30)));
        let results = indexer.search(&query)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.total_results, 2);
        assert_eq!(results.results[0].entity_id, "entity_id2");

        // only results before given score
        let query = Query::match_text("trudeau")
            .with_paging(QueryPaging::new(10).with_to_token(score_to_sort_token(0.30)));
        let results = indexer.search(&query)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.total_results, 2);
        assert_eq!(results.results[0].entity_id, "entity_id1");

        Ok(())
    }

    #[test]
    fn search_query_matches_paging() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut indexer = TraitsIndex::create_in_memory(config, schema.clone())?;
        let clock = Clock::new();
        let node = LocalNode::generate();

        let contacts = (0..30).map(|i| {
            let op_id = clock.consistent_time(node.node()).into();
            IndexMutation::PutTrait(PutTraitMutation {
                block_offset: Some(i),
                operation_id: op_id,
                entity_id: format!("entity_id{}", i),
                trt: TraitBuilder::new(&schema, "exocore", "contact")
                    .unwrap()
                    .set("id", format!("entity_id{}", i))
                    .set("name", "Justin Trudeau")
                    .set("email", "justin.trudeau@gov.ca")
                    .build()
                    .unwrap(),
            })
        });
        indexer.apply_mutations(contacts)?;

        let query1 = Query::match_text("trudeau").with_paging(QueryPaging::new(10));
        let results1 = indexer.search(&query1)?;
        assert_eq!(results1.total_results, 30);
        assert_eq!(results1.results.len(), 10);
        assert_eq!(results1.remaining_results, 20);
        find_trait_result(&results1, "id29");
        find_trait_result(&results1, "id20");

        let query2 = query1.with_paging(results1.next_page.clone().unwrap());
        let results2 = indexer.search(&query2)?;
        assert_eq!(results2.total_results, 30);
        assert_eq!(results2.results.len(), 10);
        assert_eq!(results2.remaining_results, 10);
        find_trait_result(&results1, "id19");
        find_trait_result(&results1, "id10");

        let query3 = query2.with_paging(results2.next_page.clone().unwrap());
        let results3 = indexer.search(&query3)?;
        assert_eq!(results3.total_results, 30);
        assert_eq!(results3.results.len(), 10);
        assert_eq!(results3.remaining_results, 0);
        find_trait_result(&results1, "id9");
        find_trait_result(&results1, "id0");

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        let context1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set_id("trt1".to_string())
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });

        let email1 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id2".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "email")?
                .set_id("email1")
                .set_modification_date("2019-09-01T12:00:00Z".parse::<DateTime<Utc>>()?)
                .set("subject", "Some subject")
                .set("body", "Very important body")
                .build()?,
        });

        let email2 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id3".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "email")?
                .set_id("email2")
                .set_modification_date("2019-09-03T12:00:00Z".parse::<DateTime<Utc>>()?)
                .set("subject", "Some subject")
                .set("body", "Very important body")
                .build()?,
        });

        let email3 = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id4".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "email")?
                .set_id("email3")
                .set_modification_date("2019-09-02T12:00:00Z".parse::<DateTime<Utc>>()?)
                .set("subject", "Some subject")
                .set("body", "Very important body")
                .build()?,
        });

        index.apply_mutations(vec![context1, email1, email2, email3].into_iter())?;

        let query = Query::with_trait("exocore.contact");
        let results = index.search(&query)?;
        assert_eq!(results.results.len(), 1);
        assert!(find_trait_result(&results, "trt1").is_some());

        // ordering of multiple traits is by modification date
        let query = Query::with_trait("exocore.email");
        let results = index.search(&query)?;
        let traits_ids = results
            .results
            .iter()
            .map(|res| res.trait_id.clone())
            .collect_vec();
        assert_eq!(traits_ids, vec!["email2", "email3", "email1"]);

        // with limit
        let query = Query::with_trait("exocore.email").with_paging(QueryPaging::new(1));
        let results = index.search(&query)?;
        assert_eq!(results.results.len(), 1);

        // only results after given modification date
        let date_token = SortToken::from_datetime("2019-09-02T11:59:00Z".parse::<DateTime<Utc>>()?);
        let query = Query::with_trait("exocore.email")
            .with_paging(QueryPaging::new(10).with_from_token(date_token.clone()));
        let results = index.search(&query)?;
        let traits_ids = results
            .results
            .iter()
            .map(|res| res.trait_id.clone())
            .collect_vec();
        assert_eq!(traits_ids, vec!["email2", "email3"]);

        // only results before given modification date
        let query = Query::with_trait("exocore.email")
            .with_paging(QueryPaging::new(10).with_to_token(date_token));
        let results = index.search(&query)?;
        let traits_ids = results
            .results
            .iter()
            .map(|res| res.trait_id.clone())
            .collect_vec();
        assert_eq!(traits_ids, vec!["email1"]);

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type_paging() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut indexer = TraitsIndex::create_in_memory(config, schema.clone())?;
        let clock = Clock::new();
        let node = LocalNode::generate();

        let contacts = (0..30).map(|i| {
            let now = clock.consistent_time(node.node());
            IndexMutation::PutTrait(PutTraitMutation {
                block_offset: Some(i),
                operation_id: now.into(),
                entity_id: format!("entity_id{}", i),
                trt: TraitBuilder::new(&schema, "exocore", "contact")
                    .unwrap()
                    .set("id", format!("entity_id{}", i))
                    .set("name", "Justin Trudeau")
                    .set("email", "justin.trudeau@gov.ca")
                    .set_modification_date(now.to_datetime())
                    .build()
                    .unwrap(),
            })
        });
        indexer.apply_mutations(contacts)?;

        let query1 = Query::with_trait("exocore.contact").with_paging(QueryPaging::new(10));
        let results1 = indexer.search(&query1)?;
        assert_eq!(results1.total_results, 30);
        assert_eq!(results1.remaining_results, 20);
        assert_eq!(results1.results.len(), 10);
        find_trait_result(&results1, "id29");
        find_trait_result(&results1, "id20");

        let query2 = query1.with_paging(results1.next_page.clone().unwrap());
        let results2 = indexer.search(&query2)?;
        assert_eq!(results2.total_results, 30);
        assert_eq!(results2.remaining_results, 10);
        assert_eq!(results2.results.len(), 10);
        find_trait_result(&results1, "id19");
        find_trait_result(&results1, "id10");

        let query3 = query2.with_paging(results2.next_page.clone().unwrap());
        let results3 = indexer.search(&query3)?;
        assert_eq!(results3.total_results, 30);
        assert_eq!(results3.remaining_results, 0);
        assert_eq!(results3.results.len(), 10);
        find_trait_result(&results1, "id9");
        find_trait_result(&results1, "id0");

        Ok(())
    }

    #[test]
    fn highest_indexed_block() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        assert_eq!(index.highest_indexed_block()?, None);

        index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1234),
            operation_id: 1,
            entity_id: "et1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trt1")
                .build()?,
        }))?;
        assert_eq!(index.highest_indexed_block()?, Some(1234));

        index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(120),
            operation_id: 2,
            entity_id: "et1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trt2")
                .build()?,
        }))?;
        assert_eq!(index.highest_indexed_block()?, Some(1234));

        index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(9999),
            operation_id: 3,
            entity_id: "et1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trt1")
                .build()?,
        }))?;
        assert_eq!(index.highest_indexed_block()?, Some(9999));

        Ok(())
    }

    #[test]
    fn update_trait() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        index.apply_mutation(contact_mutation)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        index.apply_mutation(contact_mutation)?;

        let query = Query::match_text("justin");
        let results = index.search(&query)?;
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.total_results, 1);
        assert_eq!(results.remaining_results, 0);
        assert_eq!(results.results[0].operation_id, 2);

        Ok(())
    }

    #[test]
    fn delete_trait_mutation() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        index.apply_mutation(contact_mutation)?;

        let query = Query::match_text("justin");
        assert_eq!(index.search(&query)?.results.len(), 1);

        index.apply_mutation(IndexMutation::DeleteTrait(
            "entity_id1".to_string(),
            "trudeau1".to_string(),
        ))?;

        assert_eq!(index.search(&query)?.results.len(), 0);

        Ok(())
    }

    #[test]
    fn delete_operation_id_mutation() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        index.apply_mutation(contact_mutation)?;

        let query = Query::match_text("justin");
        assert_eq!(index.search(&query)?.results.len(), 1);

        index.apply_mutation(IndexMutation::DeleteOperation(1234))?;

        assert_eq!(index.search(&query)?.results.len(), 0);

        Ok(())
    }

    #[test]
    fn put_trait_tombstone() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact_mutation = IndexMutation::PutTraitTombstone(PutTraitTombstone {
            block_offset: None,
            operation_id: 1234,
            entity_id: "entity_id1".to_string(),
            trait_id: "trudeau1".to_string(),
        });
        index.apply_mutation(contact_mutation)?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2345,
            entity_id: "entity_id2".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau2")
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });
        index.apply_mutation(contact_mutation)?;

        let query = Query::with_entity_id("entity_id1");
        let res = index.search(&query)?;
        assert!(res.results.first().unwrap().tombstone);

        let query = Query::with_entity_id("entity_id2");
        let res = index.search(&query)?;
        assert!(!res.results.first().unwrap().tombstone);

        Ok(())
    }

    fn find_trait_result<'r>(results: &'r TraitResults, trait_id: &str) -> Option<&'r TraitResult> {
        results.results.iter().find(|t| t.trait_id == trait_id)
    }
}
