use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::result::Result;
use std::sync::{atomic, atomic::AtomicUsize};
use std::sync::{Arc, Mutex};

use chrono::{TimeZone, Utc};
use tantivy::collector::{Collector, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, BooleanQuery, Occur, PhraseQuery, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::{
    DocAddress, Document, Index as TantivyIndex, IndexReader, IndexWriter, ReloadPolicy, Searcher,
    SegmentReader, Term,
};

pub use config::*;
use exocore_chain::block::BlockOffset;
use exocore_core::protos::generated::exocore_index::{
    entity_query::Predicate, ordering, ordering_value, trait_field_predicate, trait_query,
    EntityQuery, IdsPredicate, MatchPredicate, OperationsPredicate, Ordering, OrderingValue,
    Paging, ReferencePredicate, TraitFieldPredicate, TraitFieldReferencePredicate, TraitPredicate,
};
use exocore_core::protos::prost::{Any, ProstTimestampExt};
use exocore_core::protos::reflect;
use exocore_core::protos::reflect::{FieldValue, ReflectMessage};
use exocore_core::protos::{index::AllPredicate, registry::Registry};
pub use operations::*;
pub use results::*;

use crate::error::Error;
use crate::ordering::OrderingValueWrapper;
use std::borrow::Borrow;

mod config;
mod operations;
mod results;
mod schema;
#[cfg(test)]
mod tests;

const ENTITY_MAX_TRAITS: u32 = 1_000_000;

/// Index (full-text & fields) for entities & traits mutations stored in the
/// chain. Each mutation is individually indexed as a single document.
///
/// This index is used to index both the chain, and the pending store mutations.
/// The chain index is stored on disk, while the pending is stored in-memory.
/// Deletions are handled by using tombstones that are eventually compacted by
/// the store once the number of mutations is too high on an entity.
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
        let (tantivy_schema, fields) = schema::build_tantivy_schema(config, schemas.as_ref());

        let directory = MmapDirectory::open(directory)?;
        let index = TantivyIndex::open_or_create(directory, tantivy_schema)?;

        fields.register_tokenizers(&index);

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual) // we do our own reload after each commit for faster availability
            .try_into()?;

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
        let (tantivy_schema, fields) = schema::build_tantivy_schema(config, schemas.as_ref());

        let index = TantivyIndex::create_in_ram(tantivy_schema);
        fields.register_tokenizers(&index);

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual) // we do our own reload after each commit for faster availability
            .try_into()?;

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

    /// Apply a single operation on the index. A costly commit & refresh is done
    /// at each operation, so `apply_operations` should be used for multiple
    /// operations.
    #[cfg(test)]
    fn apply_operation(&mut self, operation: IndexOperation) -> Result<(), Error> {
        self.apply_operations(Some(operation).into_iter())
    }

    /// Apply an iterator of operations on the index, with a single atomic
    /// commit at the end of the iteration.
    pub fn apply_operations<T>(&mut self, operations: T) -> Result<(), Error>
    where
        T: Iterator<Item = IndexOperation>,
    {
        let mut index_writer = self.index_writer.lock()?;

        debug!("Starting applying operations to index...");
        let mut nb_operations = 0;
        for operation in operations {
            nb_operations += 1;

            match operation {
                IndexOperation::PutTrait(trait_put) => {
                    let entity_trait_id = format!("{}_{}", trait_put.entity_id, trait_put.trt.id);
                    trace!(
                        "Putting trait {} with op {}",
                        entity_trait_id,
                        trait_put.operation_id
                    );
                    let doc = self.trait_put_to_document(&trait_put)?;
                    index_writer.add_document(doc);
                }
                IndexOperation::PutTraitTombstone(trait_tombstone) => {
                    trace!(
                        "Putting tombstone for trait {}_{} with op {}",
                        trait_tombstone.entity_id,
                        trait_tombstone.trait_id,
                        trait_tombstone.operation_id
                    );
                    let doc = self.trait_tombstone_to_document(&trait_tombstone);
                    index_writer.add_document(doc);
                }
                IndexOperation::PutEntityTombstone(entity_tombstone) => {
                    trace!(
                        "Putting tombstone for entity {} with op {}",
                        entity_tombstone.entity_id,
                        entity_tombstone.operation_id
                    );
                    let doc = self.entity_tombstone_to_document(&entity_tombstone);
                    index_writer.add_document(doc);
                }
                IndexOperation::DeleteOperation(operation_id) => {
                    trace!("Deleting op from index {}", operation_id);
                    index_writer
                        .delete_term(Term::from_field_u64(self.fields.operation_id, operation_id));
                }
            }
        }

        if nb_operations > 0 {
            debug!("Applied {} operations, now committing", nb_operations);
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
    pub fn search<Q: Borrow<EntityQuery>>(&self, query: Q) -> Result<MutationResults, Error> {
        let query = query.borrow();
        let predicate = query
            .predicate
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("predicate"))?;

        let paging = query.paging.as_ref();
        let ordering = query.ordering.as_ref();

        let results = match predicate {
            Predicate::Trait(inner) => self.search_with_trait(inner, paging, ordering),
            Predicate::Match(inner) => self.search_matches(inner, paging, ordering),
            Predicate::Ids(inner) => self.search_entity_ids(inner, paging, ordering),
            Predicate::Reference(inner) => self.search_reference(inner, paging, ordering),
            Predicate::Operations(inner) => self.search_operations(inner, paging, ordering),
            Predicate::All(inner) => self.search_all(inner, paging, ordering),
            Predicate::Test(_inner) => Err(Error::Other("Query failed for tests".to_string())),
        }?;

        Ok(results)
    }

    /// Execute a query on the index and return an iterator over all matching
    /// mutations.
    pub fn search_iter<Q: Borrow<EntityQuery>>(
        &self,
        query: Q,
    ) -> Result<MutationResultsIterator<Q>, Error> {
        let results = self.search(query.borrow())?;

        Ok(MutationResultsIterator {
            index: self,
            query,
            total_results: results.total,
            current_results: results.mutations.into_iter(),
            next_page: results.next_page,
        })
    }

    /// Fetch all mutations for a given entity id.
    pub fn fetch_entity_mutations(&self, entity_id: &str) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let term = Term::from_field_text(self.fields.entity_id, &entity_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        let ordering = Ordering {
            ascending: true,
            value: Some(ordering::Value::OperationId(true)),
        };
        let paging = Paging {
            count: ENTITY_MAX_TRAITS,
            ..Default::default()
        };

        self.execute_tantivy_with_paging(searcher, &query, Some(&paging), ordering, None)
    }

    /// Execute a search by trait type query and return traits in operations id
    /// descending order.
    fn search_with_trait(
        &self,
        predicate: &TraitPredicate,
        paging: Option<&Paging>,
        ordering: Option<&Ordering>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let mut ordering = ordering.cloned().unwrap_or_else(Ordering::default);
        if ordering.value.is_none() {
            ordering.value = Some(ordering::Value::OperationId(true));
        }

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        let trait_type = Term::from_field_text(self.fields.trait_type, &predicate.trait_name);
        let trait_type_query = TermQuery::new(trait_type, IndexRecordOption::Basic);
        queries.push((Occur::Must, Box::new(trait_type_query)));

        if let Some(trait_query) = &predicate.query {
            match &trait_query.predicate {
                Some(trait_query::Predicate::Match(trait_pred)) => {
                    queries.push((Occur::Must, self.match_predicate_to_query(trait_pred)?));
                }
                Some(trait_query::Predicate::Field(trait_pred)) => {
                    queries.push((
                        Occur::Must,
                        self.trait_field_predicate_to_query(&predicate.trait_name, trait_pred)?,
                    ));
                }
                Some(trait_query::Predicate::Reference(trait_pred)) => {
                    queries.push((
                        Occur::Must,
                        self.trait_field_reference_predicate_to_query(
                            &predicate.trait_name,
                            trait_pred,
                        )?,
                    ));
                }
                None => {}
            }
        }

        let query = BooleanQuery::from(queries);
        self.execute_tantivy_with_paging(
            searcher,
            &query,
            paging,
            ordering,
            Some(&predicate.trait_name),
        )
    }

    /// Execute a search by text query
    fn search_matches(
        &self,
        predicate: &MatchPredicate,
        paging: Option<&Paging>,
        ordering: Option<&Ordering>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let mut ordering = ordering.cloned().unwrap_or_else(Ordering::default);
        if ordering.value.is_none() {
            ordering.value = Some(ordering::Value::Score(true));
        }

        let query = self.match_predicate_to_query(predicate)?;
        self.execute_tantivy_with_paging(searcher, &query, paging, ordering, None)
    }

    /// Executes a search for mutations with the given operations ids.
    pub fn search_operations(
        &self,
        predicate: &OperationsPredicate,
        paging: Option<&Paging>,
        ordering: Option<&Ordering>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for operation_id in &predicate.operation_ids {
            let op_term = Term::from_field_u64(self.fields.operation_id, *operation_id);
            let op_query = TermQuery::new(op_term, IndexRecordOption::Basic);
            queries.push((Occur::Should, Box::new(op_query)));
        }
        let query = BooleanQuery::from(queries);

        let mut ordering = ordering.cloned().unwrap_or_else(Ordering::default);
        if ordering.value.is_none() {
            ordering.value = Some(ordering::Value::OperationId(true));
            ordering.ascending = true;
        }

        self.execute_tantivy_with_paging(searcher, &query, paging, ordering, None)
    }

    /// Executes a search for mutations on the given entities ids.
    pub fn search_entity_ids(
        &self,
        predicate: &IdsPredicate,
        paging: Option<&Paging>,
        ordering: Option<&Ordering>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for entity_id in &predicate.ids {
            let term = Term::from_field_text(self.fields.entity_id, entity_id);
            let query = TermQuery::new(term, IndexRecordOption::Basic);
            queries.push((Occur::Should, Box::new(query)));
        }
        let query = BooleanQuery::from(queries);

        let mut ordering = ordering.cloned().unwrap_or_else(Ordering::default);
        if ordering.value.is_none() {
            ordering.value = Some(ordering::Value::OperationId(true));
        }

        self.execute_tantivy_with_paging(searcher, &query, paging, ordering, None)
    }

    /// Executes a search for traits that have the given reference to another
    /// entity and optionally trait.
    fn search_reference(
        &self,
        predicate: &ReferencePredicate,
        paging: Option<&Paging>,
        ordering: Option<&Ordering>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let query = self.reference_predicate_to_query(self.fields.all_refs, predicate);

        let mut ordering = ordering.cloned().unwrap_or_else(Ordering::default);
        if ordering.value.is_none() {
            ordering.value = Some(ordering::Value::OperationId(true));
        }

        self.execute_tantivy_with_paging(searcher, &query, paging, ordering, None)
    }

    /// Returns all mutations.
    pub fn search_all(
        &self,
        _predicate: &AllPredicate,
        paging: Option<&Paging>,
        ordering: Option<&Ordering>,
    ) -> Result<MutationResults, Error> {
        let searcher = self.index_reader.searcher();

        let mut ordering = ordering.cloned().unwrap_or_else(Ordering::default);
        if ordering.value.is_none() {
            ordering.value = Some(ordering::Value::OperationId(true));
            ordering.ascending = false;
        }

        self.execute_tantivy_with_paging(searcher, &AllQuery, paging, ordering, None)
    }

    /// Converts a trait put / update to Tantivy document
    fn trait_put_to_document(&self, operation: &PutTraitMutation) -> Result<Document, Error> {
        let message_any = operation
            .trt
            .message
            .as_ref()
            .ok_or_else(|| Error::ProtoFieldExpected("Trait message"))?;

        let mut doc = Document::default();

        let message_full_name = reflect::any_url_to_full_name(&message_any.type_url);
        doc.add_text(self.fields.trait_type, &message_full_name);
        doc.add_text(self.fields.trait_id, &operation.trt.id);
        doc.add_text(self.fields.entity_id, &operation.entity_id);
        doc.add_text(
            self.fields.entity_trait_id,
            &format!("{}_{}", operation.entity_id, &operation.trt.id),
        );

        doc.add_u64(self.fields.operation_id, operation.operation_id);
        if let Some(block_offset) = operation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        if let Some(creation_date) = &operation.trt.creation_date {
            doc.add_u64(
                self.fields.creation_date,
                creation_date.to_timestamp_nanos(),
            );
        }
        if let Some(modification_date) = &operation.trt.modification_date {
            doc.add_u64(
                self.fields.modification_date,
                modification_date.to_timestamp_nanos(),
            );
        }

        doc.add_u64(self.fields.document_type, MutationType::TRAIT_PUT_ID);

        self.trait_message_to_document(&mut doc, message_any, message_full_name);

        Ok(doc)
    }

    /// Fills a Tantivy document to be indexed with indexable/sortable fields of
    /// the given registered message.
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

        for (field_id, field) in message_dyn.fields() {
            let field_value = match message_dyn.get_field_value(*field_id) {
                Ok(fv) => fv,
                Err(err) => {
                    debug!("Couldn't get value of field {:?}: {}", field, err);
                    continue;
                }
            };

            let field_mapping =
                if let Some(field_mapping) = message_mappings.and_then(|m| m.get(&field.name)) {
                    field_mapping
                } else {
                    // field is not indexed if we don't have a mapping for it
                    continue;
                };

            match field_value {
                FieldValue::String(value) if field.text_flag => {
                    doc.add_text(field_mapping.field, &value);
                    doc.add_text(self.fields.all_text, &value);
                }
                FieldValue::String(value) if field.indexed_flag => {
                    doc.add_text(field_mapping.field, &value);
                }
                FieldValue::Reference(value) if field.indexed_flag => {
                    let ref_value = format!("entity{} trait{}", value.entity_id, value.trait_id);
                    doc.add_text(field_mapping.field, &ref_value);
                    doc.add_text(self.fields.all_refs, &ref_value);
                }
                FieldValue::DateTime(value) if field.indexed_flag || field.sorted_flag => {
                    doc.add_u64(field_mapping.field, value.timestamp_nanos() as u64);
                }
                FieldValue::Int64(value) if field.indexed_flag || field.sorted_flag => {
                    doc.add_i64(field_mapping.field, value);
                }
                FieldValue::Int32(value) if field.indexed_flag || field.sorted_flag => {
                    doc.add_i64(field_mapping.field, i64::from(value));
                }
                FieldValue::Uint64(value) if field.indexed_flag || field.sorted_flag => {
                    doc.add_u64(field_mapping.field, value);
                }
                FieldValue::Uint32(value) if field.indexed_flag || field.sorted_flag => {
                    doc.add_u64(field_mapping.field, u64::from(value));
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
    fn trait_tombstone_to_document(&self, operation: &PutTraitTombstoneMutation) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.fields.trait_id, &operation.trait_id);
        doc.add_text(self.fields.entity_id, &operation.entity_id);
        doc.add_text(
            self.fields.entity_trait_id,
            &format!("{}_{}", operation.entity_id, operation.trait_id),
        );
        doc.add_u64(self.fields.operation_id, operation.operation_id);

        if let Some(block_offset) = operation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        doc.add_u64(self.fields.document_type, MutationType::TRAIT_TOMBSTONE_ID);

        doc
    }

    /// Converts an entity tombstone mutation to Tantivy document
    fn entity_tombstone_to_document(&self, operation: &PutEntityTombstoneMutation) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.fields.entity_id, &operation.entity_id);
        doc.add_u64(self.fields.operation_id, operation.operation_id);

        if let Some(block_offset) = operation.block_offset {
            doc.add_u64(self.fields.block_offset, block_offset);
        }

        doc.add_u64(self.fields.document_type, MutationType::ENTITY_TOMBSTONE_ID);

        doc
    }

    /// Transforms a text match predicate to Tantivy query.
    fn match_predicate_to_query(
        &self,
        predicate: &MatchPredicate,
    ) -> Result<Box<dyn tantivy::query::Query>, Error> {
        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.all_text]);
        let query = query_parser.parse_query(&predicate.query)?;

        Ok(query)
    }

    /// Transforms a trait's field predicate to Tantivy query.
    fn trait_field_predicate_to_query(
        &self,
        trait_name: &str,
        predicate: &TraitFieldPredicate,
    ) -> Result<Box<dyn tantivy::query::Query>, Error> {
        let field = self
            .fields
            .get_dynamic_trait_field(trait_name, &predicate.field)?;

        use reflect::FieldType as FT;
        use trait_field_predicate::Value as PV;
        match (&field.field_type, &predicate.value) {
            (FT::String, Some(PV::String(value))) => {
                let term = Term::from_field_text(field.field, &value);
                Ok(Box::new(TermQuery::new(term, IndexRecordOption::Basic)))
            }
            (ft, pv) => {
                Err(
                    Error::QueryParsing(
                        format!(
                            "Incompatible field type vs field value in predicate: trait_name={} field={}, field_type={:?}, value={:?}",
                            trait_name,
                            predicate.field,
                            ft,
                            pv,
                        ))
                )
            }
        }
    }

    /// Transforms a trait's field reference predicate to Tantivy query.
    fn trait_field_reference_predicate_to_query(
        &self,
        trait_name: &str,
        predicate: &TraitFieldReferencePredicate,
    ) -> Result<Box<dyn tantivy::query::Query>, Error> {
        let field = self
            .fields
            .get_dynamic_trait_field(trait_name, &predicate.field)?;

        let reference = predicate
            .reference
            .as_ref()
            .ok_or_else(|| Error::ProtoFieldExpected("reference"))?;

        Ok(self.reference_predicate_to_query(field.field, reference))
    }

    /// Transforms a reference predicate to Tantivy query.
    fn reference_predicate_to_query(
        &self,
        field: Field,
        predicate: &ReferencePredicate,
    ) -> Box<dyn tantivy::query::Query> {
        let query: Box<dyn tantivy::query::Query> = if !predicate.trait_id.is_empty() {
            let terms = vec![
                Term::from_field_text(field, &format!("entity{}", predicate.entity_id)),
                Term::from_field_text(field, &format!("trait{}", predicate.trait_id)),
            ];
            Box::new(PhraseQuery::new(terms))
        } else {
            Box::new(TermQuery::new(
                Term::from_field_text(field, &format!("entity{}", predicate.entity_id)),
                IndexRecordOption::Basic,
            ))
        };
        query
    }

    /// Execute query on Tantivy index by taking paging, ordering into
    /// consideration and returns paged results.
    fn execute_tantivy_with_paging<S>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        paging: Option<&Paging>,
        ordering: Ordering,
        trait_name: Option<&str>,
    ) -> Result<MutationResults, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let paging = paging.cloned().unwrap_or_else(|| Paging {
            after_ordering_value: None,
            before_ordering_value: None,
            count: self.config.iterator_page_size,
        });

        let total_count = Arc::new(AtomicUsize::new(0));
        let match_count = Arc::new(AtomicUsize::new(0));

        let ordering_value = ordering
            .value
            .ok_or_else(|| Error::ProtoFieldExpected("ordering.value"))?;
        let mutations = match ordering_value {
            ordering::Value::Score(_) => {
                let collector = self.match_score_collector(
                    total_count.clone(),
                    match_count.clone(),
                    &paging,
                    ordering.ascending,
                );
                self.execute_tantity_query_with_collector(searcher, query, &collector)?
            }
            ordering::Value::OperationId(_) => {
                let sort_field = self.fields.operation_id;
                let collector = self.sorted_field_collector(
                    total_count.clone(),
                    match_count.clone(),
                    &paging,
                    sort_field,
                    ordering.ascending,
                );
                self.execute_tantity_query_with_collector(searcher, query, &collector)?
            }
            ordering::Value::Field(field_name) => {
                let trait_name = trait_name.ok_or_else(|| {
                    Error::QueryParsing(String::from(
                        "Ordering by field only supported in trait query",
                    ))
                })?;

                let sort_field = self
                    .fields
                    .get_dynamic_trait_field(trait_name, &field_name)?;
                if !sort_field.is_fast_field {
                    return Err(Error::QueryParsing(format!(
                        "Cannot sort by field '{}' as it's not sortable in  trait '{}'",
                        field_name, trait_name,
                    )));
                }

                let collector = self.sorted_field_collector(
                    total_count.clone(),
                    match_count.clone(),
                    &paging,
                    sort_field.field,
                    ordering.ascending,
                );
                self.execute_tantity_query_with_collector(searcher, query, &collector)?
            }
        };

        let total_results = total_count.load(atomic::Ordering::Relaxed);
        let remaining_results = match_count
            .load(atomic::Ordering::Relaxed)
            .saturating_sub(mutations.len());

        let next_page = if remaining_results > 0 {
            let last_result = mutations.last().expect("Should had results, but got none");
            let mut next_page = Paging {
                before_ordering_value: None,
                after_ordering_value: None,
                count: paging.count,
            };

            if ordering.ascending {
                next_page.after_ordering_value = Some(last_result.sort_value.value.clone());
            } else {
                next_page.before_ordering_value = Some(last_result.sort_value.value.clone());
            }

            Some(next_page)
        } else {
            None
        };

        Ok(MutationResults {
            mutations,
            total: total_results,
            remaining: remaining_results,
            next_page,
        })
    }

    /// Execute query on Tantivy index and build mutations metadata results.
    fn execute_tantity_query_with_collector<S, C>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        top_collector: &C,
    ) -> Result<Vec<MutationMetadata>, Error>
    where
        S: Deref<Target = Searcher>,
        C: Collector<Fruit = Vec<(OrderingValueWrapper, DocAddress)>>,
    {
        let search_results = searcher.search(query, top_collector)?;

        let mut results = Vec::new();
        for (sort_wrapper, doc_addr) in search_results {
            // ignored results were out of the requested paging
            if !sort_wrapper.ignore {
                let doc = searcher.doc(doc_addr)?;
                let block_offset = schema::get_doc_opt_u64_value(&doc, self.fields.block_offset);
                let operation_id = schema::get_doc_u64_value(&doc, self.fields.operation_id);
                let entity_id = schema::get_doc_string_value(&doc, self.fields.entity_id);
                let opt_trait_id = schema::get_doc_opt_string_value(&doc, self.fields.trait_id);
                let document_type_id = schema::get_doc_u64_value(&doc, self.fields.document_type);

                let mut mutation_type = MutationType::new(document_type_id, opt_trait_id)?;
                if let MutationType::TraitPut(put_trait) = &mut mutation_type {
                    put_trait.creation_date =
                        schema::get_doc_opt_u64_value(&doc, self.fields.creation_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                    put_trait.modification_date =
                        schema::get_doc_opt_u64_value(&doc, self.fields.modification_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                    put_trait.trait_type =
                        schema::get_doc_opt_string_value(&doc, self.fields.trait_type);
                }

                let result = MutationMetadata {
                    block_offset,
                    operation_id,
                    entity_id,
                    mutation_type,
                    sort_value: Rc::new(sort_wrapper),
                };
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Creates a Tantivy top document collectors that sort by the given fast
    /// field and limits the result by the requested paging.
    fn sorted_field_collector(
        &self,
        total_count: Arc<AtomicUsize>,
        matching_count: Arc<AtomicUsize>,
        paging: &Paging,
        sort_field: Field,
        ascending: bool,
    ) -> impl Collector<Fruit = Vec<(OrderingValueWrapper, DocAddress)>> {
        let after_ordering_value = Arc::new(paging.after_ordering_value.clone().unwrap_or_else(
            || OrderingValue {
                value: Some(ordering_value::Value::Min(true)),
                operation_id: 0,
            },
        ));
        let before_ordering_value = Arc::new(paging.before_ordering_value.clone().unwrap_or_else(
            || OrderingValue {
                value: Some(ordering_value::Value::Max(true)),
                operation_id: 0,
            },
        ));

        let operation_id_field = self.fields.operation_id;
        TopDocs::with_limit(paging.count as usize).custom_score(
            move |segment_reader: &SegmentReader| {
                let after_ordering_value = after_ordering_value.clone();
                let before_ordering_value = before_ordering_value.clone();
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
                    total_docs.fetch_add(1, atomic::Ordering::SeqCst);

                    let mut ordering_value_wrapper = OrderingValueWrapper {
                        value: OrderingValue {
                            value: Some(ordering_value::Value::Uint64(sort_fast_field.get(doc_id))),
                            operation_id: operation_id_reader.get(doc_id),
                        },
                        reverse: ascending,
                        ignore: false,
                    };

                    // we ignore the result if it's out of the requested pages
                    if ordering_value_wrapper
                        .is_within_bound(&after_ordering_value, &before_ordering_value)
                    {
                        remaining_count.fetch_add(1, atomic::Ordering::SeqCst);
                    } else {
                        ordering_value_wrapper.ignore = true;
                    }

                    ordering_value_wrapper
                }
            },
        )
    }

    /// Creates a Tantivy top document collectors that sort by full text
    /// matching score and limits the result by the requested paging.
    fn match_score_collector(
        &self,
        total_count: Arc<AtomicUsize>,
        matching_count: Arc<AtomicUsize>,
        paging: &Paging,
        ascending: bool,
    ) -> impl Collector<Fruit = Vec<(OrderingValueWrapper, DocAddress)>> {
        let after_score =
            Arc::new(
                paging
                    .after_ordering_value
                    .clone()
                    .unwrap_or_else(|| OrderingValue {
                        value: Some(ordering_value::Value::Min(true)),
                        operation_id: 0,
                    }),
            );
        let before_score =
            Arc::new(
                paging
                    .before_ordering_value
                    .clone()
                    .unwrap_or_else(|| OrderingValue {
                        value: Some(ordering_value::Value::Max(true)),
                        operation_id: 0,
                    }),
            );

        let operation_id_field = self.fields.operation_id;
        TopDocs::with_limit(paging.count as usize).tweak_score(
            move |segment_reader: &SegmentReader| {
                let after_score = after_score.clone();
                let before_score = before_score.clone();
                let total_docs = total_count.clone();
                let remaining_count = matching_count.clone();

                let operation_id_reader = segment_reader
                    .fast_fields()
                    .u64(operation_id_field)
                    .unwrap();

                move |doc_id, score| {
                    total_docs.fetch_add(1, atomic::Ordering::SeqCst);

                    let mut ordering_value_wrapper = OrderingValueWrapper {
                        value: OrderingValue {
                            value: Some(ordering_value::Value::Float(score)),
                            operation_id: operation_id_reader.get(doc_id),
                        },
                        reverse: ascending,
                        ignore: false,
                    };

                    // we ignore the result if it's out of the requested pages
                    if ordering_value_wrapper.is_within_bound(&after_score, &before_score) {
                        remaining_count.fetch_add(1, atomic::Ordering::SeqCst);
                    } else {
                        ordering_value_wrapper.ignore = true;
                    }

                    ordering_value_wrapper
                }
            },
        )
    }
}
