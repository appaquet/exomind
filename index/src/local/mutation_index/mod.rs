use std::ops::Deref;
use std::path::Path;
use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{TimeZone, Utc};
use tantivy::collector::{Collector, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, PhraseQuery, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::{
    DocAddress, Document, Index as TantivyIndex, IndexReader, IndexWriter, Searcher, SegmentReader,
    Term,
};

use exocore_chain::block::BlockOffset;
use exocore_core::protos::generated::exocore_index::{
    entity_query::Predicate, EntityQuery, MatchPredicate, Paging, ReferencePredicate, Sorting,
    TraitPredicate,
};
use exocore_core::protos::generated::index::sorting::Value as SortingValue;
use exocore_core::protos::prost::{Any, ProstTimestampExt};
use exocore_core::protos::reflect;
use exocore_core::protos::reflect::{FieldValue, ReflectMessage};
use exocore_core::protos::registry::Registry;

use crate::error::Error;

mod config;
mod mutation;
mod results;
mod schema;
#[cfg(test)]
mod tests;

use crate::query::SortToken;
pub use config::*;
pub use mutation::*;
pub use results::*;

const SEARCH_ENTITY_ID_LIMIT: usize = 1_000_000;

/// Index (full-text & fields) for entities & traits mutations stored in the chain layer. Each
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
    fields: schema::Fields,
}

impl MutationIndex {
    /// Creates or opens a disk persisted index.
    pub fn open_or_create_mmap(
        config: MutationIndexConfig,
        schemas: Arc<Registry>,
        directory: &Path,
    ) -> Result<MutationIndex, Error> {
        let (tantivy_schema, fields) = schema::build_tantivy_schema(schemas.as_ref());
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
        let (tantivy_schema, fields) = schema::build_tantivy_schema(schemas.as_ref());
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
    pub fn search(&self, query: &EntityQuery) -> Result<MutationResults, Error> {
        let predicate = query
            .predicate
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("predicate"))?;

        let paging = query.paging.clone();
        let sorting = query.sorting.as_ref();

        match predicate {
            Predicate::Trait(inner) => self.search_with_trait(inner, paging, sorting),
            Predicate::Match(inner) => self.search_matches(inner, paging, sorting),
            Predicate::Id(inner) => self.search_entity_id(&inner.id),
            Predicate::Reference(inner) => self.search_reference(inner, paging, sorting),
            Predicate::Test(_inner) => Err(Error::Other("Query failed for tests".to_string())),
        }
    }

    /// Execute a query on the index and return an iterator over all matching
    /// mutations.
    pub fn search_all<'i, 'q>(
        &'i self,
        query: &'q EntityQuery,
    ) -> Result<ResultsIterator<'i, 'q>, Error> {
        let results = self.search(query)?;

        Ok(ResultsIterator {
            index: self,
            query,
            total_results: results.total_results,
            current_results: results.results.into_iter(),
            next_page: results.next_page,
        })
    }

    /// Execute a search by trait type query and return traits in operations id descending order.
    pub fn search_with_trait(
        &self,
        predicate: &TraitPredicate,
        paging: Option<Paging>,
        sorting: Option<&Sorting>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let term = Term::from_field_text(self.fields.trait_type, &predicate.trait_name);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        let mut sorting = sorting.cloned().unwrap_or_else(Sorting::default);
        if sorting.value.is_none() {
            sorting.value = Some(SortingValue::OperationId(true));
        }

        self.execute_results_tantivy_query(
            searcher,
            &query,
            paging,
            sorting,
            Some(&predicate.trait_name),
        )
    }

    /// Execute a search by text query
    pub fn search_matches(
        &self,
        predicate: &MatchPredicate,
        paging: Option<Paging>,
        sorting: Option<&Sorting>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.all_text]);
        let query = query_parser.parse_query(&predicate.query)?;

        let mut sorting = sorting.cloned().unwrap_or_else(Sorting::default);
        if sorting.value.is_none() {
            sorting.value = Some(SortingValue::Score(true));
        }

        self.execute_results_tantivy_query(searcher, &query, paging, sorting, None)
    }

    /// Execute a search by entity id query
    pub fn search_entity_id(&self, entity_id: &str) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let term = Term::from_field_text(self.fields.entity_id, &entity_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        let top_collector = TopDocs::with_limit(SEARCH_ENTITY_ID_LIMIT).tweak_score(
            move |_segment_reader: &SegmentReader| {
                move |_doc_id, score| ResultScore {
                    score,
                    reverse: false,
                    operation_id: 0,
                    ignore: false,
                }
            },
        );

        let results = self.execute_mutations_tantity_query(searcher, &query, &top_collector)?;
        let total_results = results.len();

        Ok(MutationResults {
            results,
            total_results,
            remaining_results: 0,
            next_page: None,
        })
    }

    /// Executes a search for traits that have the given reference to another entity and optionally trait.
    pub fn search_reference(
        &self,
        predicate: &ReferencePredicate,
        paging: Option<Paging>,
        sorting: Option<&Sorting>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let query = self.reference_predicate_to_query(predicate);

        let mut sorting = sorting.cloned().unwrap_or_else(Sorting::default);
        if sorting.value.is_none() {
            sorting.value = Some(SortingValue::OperationId(true));
        }

        self.execute_results_tantivy_query(searcher, &query, paging, sorting, None)
    }

    /// Converts a trait put / update to Tantivy document
    fn trait_put_to_document(&self, mutation: &PutTraitMutation) -> Result<Document, Error> {
        let message_any = mutation
            .trt
            .message
            .as_ref()
            .ok_or_else(|| Error::ProtoFieldExpected("Trait message"))?;

        let mut doc = Document::default();

        let message_full_name = reflect::any_url_to_full_name(&message_any.type_url);
        doc.add_text(self.fields.trait_type, &message_full_name);
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

        doc.add_u64(
            self.fields.document_type,
            MutationMetadataType::TRAIT_PUT_ID,
        );

        self.trait_message_to_document(&mut doc, message_any, message_full_name);

        Ok(doc)
    }

    /// Fills a Tantivy document to be indexed with indexable/sortable fields of the given registered message.
    fn trait_message_to_document(
        &self,
        doc: &mut Document,
        message_any: &Any,
        message_full_name: String,
    ) {
        let message_dyn = match reflect::from_prost_any(self.schemas.as_ref(), message_any) {
            Ok(message_dyn) => message_dyn,
            Err(err) => {
                error!(
                    "Error reflecting message of type '{}'. No fields will be indexed. Error: {}",
                    message_full_name, err
                );
                return;
            }
        };

        let message_mappings = self.fields.dynamic_mappings.get(message_dyn.full_name());

        for field in message_dyn.fields() {
            let field_value = match message_dyn.get_field_value(field) {
                Ok(fv) => fv,
                Err(err) => {
                    debug!("Couldn't get value of field {:?}: {}", field, err);
                    continue;
                }
            };
            let field_mapping = message_mappings.and_then(|m| m.get(&field.name));

            match (field_value, field_mapping) {
                (FieldValue::String(value), Some(fm)) if field.text_flag => {
                    doc.add_text(fm.field, &value);
                    doc.add_text(self.fields.all_text, &value);
                }
                (FieldValue::String(value), Some(fm)) if field.indexed_flag => {
                    doc.add_text(fm.field, &value);
                }
                (FieldValue::Reference(value), Some(fm)) if field.indexed_flag => {
                    let ref_value = format!("entity{} trait{}", value.entity_id, value.trait_id);
                    doc.add_text(fm.field, &ref_value);
                    doc.add_text(self.fields.all_refs, &ref_value);
                }
                (FieldValue::DateTime(value), Some(fm))
                    if field.indexed_flag || field.sorted_flag =>
                {
                    doc.add_u64(fm.field, value.timestamp_nanos() as u64);
                }
                (FieldValue::Int64(value), Some(fm)) if field.indexed_flag || field.sorted_flag => {
                    doc.add_i64(fm.field, value);
                }
                (FieldValue::Int32(value), Some(fm)) if field.indexed_flag || field.sorted_flag => {
                    doc.add_i64(fm.field, i64::from(value));
                }
                (FieldValue::Uint64(value), Some(fm))
                    if field.indexed_flag || field.sorted_flag =>
                {
                    doc.add_u64(fm.field, value);
                }
                (FieldValue::Uint32(value), Some(fm))
                    if field.indexed_flag || field.sorted_flag =>
                {
                    doc.add_u64(fm.field, u64::from(value));
                }
                other => {
                    warn!(
                        "Unsupported indexed field type / value: type={:?} value={:?} mapping={:?}",
                        field.field_type, other, field_mapping
                    );
                }
            }
        }
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

    /// Transforms a reference predicate to Tantivy query.
    fn reference_predicate_to_query(
        &self,
        predicate: &ReferencePredicate,
    ) -> Box<dyn tantivy::query::Query> {
        let query: Box<dyn tantivy::query::Query> = if !predicate.trait_id.is_empty() {
            let terms = vec![
                Term::from_field_text(
                    self.fields.all_refs,
                    &format!("entity{}", predicate.entity_id),
                ),
                Term::from_field_text(
                    self.fields.all_refs,
                    &format!("trait{}", predicate.trait_id),
                ),
            ];
            Box::new(PhraseQuery::new(terms))
        } else {
            Box::new(TermQuery::new(
                Term::from_field_text(
                    self.fields.all_refs,
                    &format!("entity{}", predicate.entity_id),
                ),
                IndexRecordOption::Basic,
            ))
        };
        query
    }

    /// Execute query on Tantivy index by taking paging, sorting into consideration and
    /// returns paged results.
    fn execute_results_tantivy_query<S>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        paging: Option<Paging>,
        sorting: Sorting,
        trait_name: Option<&str>,
    ) -> Result<MutationResults, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let paging = paging.unwrap_or_else(|| Paging {
            after_token: String::new(),
            before_token: String::new(),
            count: self.config.iterator_page_size,
        });

        let total_count = Arc::new(AtomicUsize::new(0));
        let matching_count = Arc::new(AtomicUsize::new(0));

        let sorting_value = sorting
            .value
            .ok_or_else(|| Error::ProtoFieldExpected("sorting.value"))?;
        let results = match sorting_value {
            SortingValue::Score(_) => {
                let collector = self.match_score_collector(
                    total_count.clone(),
                    matching_count.clone(),
                    &paging,
                    sorting.ascending,
                );
                self.execute_mutations_tantity_query(searcher, query, &collector)?
            }
            SortingValue::OperationId(_) => {
                let sort_field = self.fields.operation_id;
                let collector = self.sorted_field_collector(
                    total_count.clone(),
                    matching_count.clone(),
                    &paging,
                    sort_field,
                    sorting.ascending,
                );
                self.execute_mutations_tantity_query(searcher, query, &collector)?
            }
            SortingValue::Field(field_name) => {
                let trait_name = trait_name.ok_or_else(|| {
                    Error::QueryParsing(String::from(
                        "Sorting by field only supported in trait query",
                    ))
                })?;

                let fields_mapping =
                    self.fields
                        .dynamic_mappings
                        .get(trait_name)
                        .ok_or_else(|| {
                            Error::QueryParsing(String::from(
                            "Sorting by field only supported in trait query with a sortable field",
                        ))
                        })?;

                let sort_field = fields_mapping
                    .get(&field_name)
                    .filter(|p| p.is_fast_field)
                    .ok_or_else(|| {
                        Error::QueryParsing(format!(
                            "Cannot sort by field {} as it's not sortable",
                            field_name
                        ))
                    })?;

                let collector = self.sorted_field_collector(
                    total_count.clone(),
                    matching_count.clone(),
                    &paging,
                    sort_field.field,
                    sorting.ascending,
                );
                self.execute_mutations_tantity_query(searcher, query, &collector)?
            }
        };

        let total_results = total_count.load(Ordering::Relaxed);
        let remaining_results = matching_count
            .load(Ordering::Relaxed)
            .saturating_sub(results.len());

        let next_page = if remaining_results > 0 {
            let last_result = results.last().expect("Should had results, but got none");
            let mut next_page = Paging {
                after_token: String::new(),
                before_token: String::new(),
                count: paging.count,
            };

            if sorting.ascending {
                next_page.after_token = last_result.sort_token.clone().into();
            } else {
                next_page.before_token = last_result.sort_token.clone().into();
            }

            Some(next_page)
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

    /// Execute query on Tantivy index and build mutations metadata results.
    fn execute_mutations_tantity_query<S, C, O>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        top_collector: &C,
    ) -> Result<Vec<MutationMetadata>, Error>
    where
        S: Deref<Target = Searcher>,
        C: Collector<Fruit = Vec<(ResultScore<O>, DocAddress)>>,
        O: ScoreValue + Send + 'static,
    {
        let search_results = searcher.search(query, top_collector)?;

        let mut results = Vec::new();
        for (doc_score, doc_addr) in search_results {
            if !doc_score.ignore {
                let doc = searcher.doc(doc_addr)?;
                let block_offset = schema::get_doc_opt_u64_value(&doc, self.fields.block_offset);
                let operation_id = schema::get_doc_u64_value(&doc, self.fields.operation_id);
                let entity_id = schema::get_doc_string_value(&doc, self.fields.entity_id);
                let opt_trait_id = schema::get_doc_opt_string_value(&doc, self.fields.trait_id);
                let document_type_id = schema::get_doc_u64_value(&doc, self.fields.document_type);

                let mut mutation_type = MutationMetadataType::new(document_type_id, opt_trait_id)?;

                if let MutationMetadataType::TraitPut(put_trait) = &mut mutation_type {
                    put_trait.creation_date =
                        schema::get_doc_opt_u64_value(&doc, self.fields.creation_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                    put_trait.modification_date =
                        schema::get_doc_opt_u64_value(&doc, self.fields.modification_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                }

                let result = MutationMetadata {
                    block_offset,
                    operation_id,
                    entity_id,
                    mutation_type,
                    sort_token: doc_score.score.to_sort_token(),
                };
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Creates a Tantivy top document collectors that sort by the given fast field and limits the result
    /// by the requested paging.
    fn sorted_field_collector(
        &self,
        total_count: Arc<AtomicUsize>,
        matching_count: Arc<AtomicUsize>,
        paging: &Paging,
        sort_field: Field,
        ascending: bool,
    ) -> impl Collector<Fruit = Vec<(ResultScore<u64>, DocAddress)>> {
        let after_sort_value = if !paging.after_token.is_empty() {
            SortToken::from(paging.after_token.clone())
                .to_u64()
                .unwrap_or(0)
        } else {
            0
        };
        let before_sort_value = if !paging.before_token.is_empty() {
            SortToken::from(paging.before_token.clone())
                .to_u64()
                .unwrap_or(0)
        } else {
            std::u64::MAX
        };

        let operation_id_field = self.fields.operation_id;
        TopDocs::with_limit(paging.count as usize).custom_score(
            move |segment_reader: &SegmentReader| {
                let total_docs = total_count.clone();
                let remaining_count = matching_count.clone();
                let operation_id_reader = segment_reader
                    .fast_fields()
                    .u64(operation_id_field)
                    .unwrap();

                let sort_fast_field = segment_reader
                    .fast_fields()
                    .u64(sort_field)
                    .expect("Field requested is not a i64/u64 fast field.");
                move |doc_id| {
                    total_docs.fetch_add(1, Ordering::SeqCst);

                    let sort_value = sort_fast_field.get(doc_id);
                    let operation_id = operation_id_reader.get(doc_id);
                    if sort_value > after_sort_value && sort_value < before_sort_value {
                        remaining_count.fetch_add(1, Ordering::SeqCst);
                        ResultScore {
                            score: sort_value,
                            reverse: ascending,
                            operation_id,
                            ignore: false,
                        }
                    } else {
                        ResultScore {
                            score: sort_value,
                            reverse: ascending,
                            operation_id,
                            ignore: true,
                        }
                    }
                }
            },
        )
    }

    /// Creates a Tantivy top document collectors that sort by full text matching score and limits the result
    /// by the requested paging.
    fn match_score_collector(
        &self,
        total_count: Arc<AtomicUsize>,
        matching_count: Arc<AtomicUsize>,
        paging: &Paging,
        ascending: bool,
    ) -> impl Collector<Fruit = Vec<(ResultScore<f32>, DocAddress)>> {
        let after_score = if !paging.after_token.is_empty() {
            SortToken::from(paging.after_token.clone())
                .to_f32()
                .unwrap_or(0.0)
        } else {
            std::f32::MIN
        };
        let before_score = if !paging.before_token.is_empty() {
            SortToken::from(paging.before_token.clone())
                .to_f32()
                .unwrap_or(0.0)
        } else {
            std::f32::MAX
        };

        let operation_id_field = self.fields.operation_id;
        TopDocs::with_limit(paging.count as usize).tweak_score(
            move |segment_reader: &SegmentReader| {
                let total_docs = total_count.clone();
                let remaining_count = matching_count.clone();
                let operation_id_reader = segment_reader
                    .fast_fields()
                    .u64(operation_id_field)
                    .unwrap();

                move |doc_id, score| {
                    total_docs.fetch_add(1, Ordering::SeqCst);

                    let operation_id = operation_id_reader.get(doc_id);
                    if score > after_score && score < before_score {
                        remaining_count.fetch_add(1, Ordering::SeqCst);
                        ResultScore {
                            score,
                            reverse: ascending,
                            operation_id,
                            ignore: false,
                        }
                    } else {
                        ResultScore {
                            score: 0.0,
                            reverse: ascending,
                            operation_id,
                            ignore: true,
                        }
                    }
                }
            },
        )
    }
}
