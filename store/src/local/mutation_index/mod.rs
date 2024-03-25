use std::{
    borrow::Borrow,
    collections::BTreeMap,
    convert::TryInto,
    num::NonZeroUsize,
    ops::Deref,
    path::Path,
    result::Result,
    sync::{Arc, Mutex},
};

use chrono::{TimeZone, Utc};
pub use config::*;
use entity_cache::EntityMutationsCache;
use exocore_chain::block::BlockOffset;
use exocore_core::time::Instant;
use exocore_protos::{
    generated::exocore_store::{
        ordering, ordering_value, EntityQuery, Ordering, OrderingValue, Paging,
    },
    prost::{Any, ProstTimestampExt},
    reflect,
    reflect::{DynamicMessage, FieldDescriptor, FieldValue, ReflectMessage},
    registry::Registry,
};
pub use operations::*;
pub use results::*;
use tantivy::{
    collector::{Collector, Count, MultiCollector, TopDocs},
    directory::MmapDirectory,
    query::{AllQuery, TermQuery},
    schema::{Field, IndexRecordOption},
    DocAddress, Document, Index as TantivyIndex, IndexReader, IndexSettings, IndexSortByField,
    IndexWriter, Order, ReloadPolicy, Searcher, SegmentReader, Term,
};

use self::{
    query::{ParsedQuery, QueryParser},
    schema::MutationIndexSchema,
};
use crate::{
    entity::EntityIdRef, error::Error, mutation::OperationId, ordering::OrderingValueWrapper,
};

mod config;
mod entity_cache;
mod operations;
mod query;
mod results;
mod schema;
#[cfg(test)]
mod tests;

const ENTITY_MAX_TRAITS: u32 = 10_000;

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
    schema: MutationIndexSchema,
    storage: Storage,
    entity_cache: EntityMutationsCache,
    full_text_boost: f32,
    schema_registry: Arc<Registry>,
}

#[derive(Debug, PartialEq, Eq)]
enum Storage {
    Memory,
    Disk,
}

impl MutationIndex {
    /// Creates or opens a disk persisted index.
    pub fn open_or_create_mmap(
        config: MutationIndexConfig,
        schema_registry: Arc<Registry>,
        directory: &Path,
    ) -> Result<MutationIndex, Error> {
        let schema = MutationIndexSchema::new(config, schema_registry.as_ref());

        let directory = MmapDirectory::open(directory)?;
        let index = TantivyIndex::builder()
            .schema(schema.tantivy.clone())
            .settings(index_settings())
            .open_or_create(directory)?;
        schema.register_tokenizers(&index);

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual) // we do our own reload after each commit for faster availability
            .try_into()?;

        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        let entity_cache_size: NonZeroUsize = config
            .entity_mutations_cache_size
            .try_into()
            .map_err(|err| Error::Other(anyhow!("Invalid entity mutations cache size: {}", err)))?;

        Ok(MutationIndex {
            config,
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            schema,
            storage: Storage::Disk,
            entity_cache: EntityMutationsCache::new(entity_cache_size),
            full_text_boost: 1.0,
            schema_registry,
        })
    }

    /// Creates or opens a in-memory index.
    pub fn create_in_memory(
        config: MutationIndexConfig,
        schema_registry: Arc<Registry>,
    ) -> Result<MutationIndex, Error> {
        let schema = MutationIndexSchema::new(config, schema_registry.as_ref());

        let index = TantivyIndex::builder()
            .schema(schema.tantivy.clone())
            .settings(index_settings())
            .create_in_ram()?;
        schema.register_tokenizers(&index);

        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual) // we do our own reload after each commit for faster availability
            .try_into()?;

        let index_writer = if let Some(nb_threads) = config.indexer_num_threads {
            index.writer_with_num_threads(nb_threads, config.indexer_heap_size_bytes)?
        } else {
            index.writer(config.indexer_heap_size_bytes)?
        };

        let entity_cache_size: NonZeroUsize = config
            .entity_mutations_cache_size
            .try_into()
            .map_err(|err| Error::Other(anyhow!("Invalid entity mutations cache size: {}", err)))?;

        Ok(MutationIndex {
            config,
            index,
            index_reader,
            index_writer: Mutex::new(index_writer),
            schema,
            storage: Storage::Memory,
            entity_cache: EntityMutationsCache::new(entity_cache_size),
            full_text_boost: 1.0,
            schema_registry,
        })
    }

    /// Boosts full-text result scores by multiplying it by the given boost
    /// value.
    pub fn set_full_text_boost(&mut self, boost: f32) {
        self.full_text_boost = boost;
    }

    /// Apply a single operation on the index. A costly commit & refresh is done
    /// at each operation, so `apply_operations` should be used for multiple
    /// operations.
    #[cfg(test)]
    fn apply_operation(&self, operation: IndexOperation) -> Result<usize, Error> {
        self.apply_operations(Some(operation).into_iter())
    }

    /// Apply an iterator of operations on the index, with a single atomic
    /// commit at the end of the iteration.
    pub fn apply_operations<T>(&self, operations: T) -> Result<usize, Error>
    where
        T: Iterator<Item = IndexOperation>,
    {
        let mut index_writer = self.index_writer.lock()?;

        let begin_time = Instant::now();

        trace!("Starting applying operations to index...");
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

                    self.entity_cache.remove(&trait_put.entity_id);

                    let doc = self.trait_put_to_document(&trait_put)?;
                    index_writer.add_document(doc)?;
                }
                IndexOperation::PutTraitTombstone(trait_tombstone) => {
                    trace!(
                        "Putting tombstone for trait {}_{} with op {}",
                        trait_tombstone.entity_id,
                        trait_tombstone.trait_id,
                        trait_tombstone.operation_id
                    );

                    self.entity_cache.remove(&trait_tombstone.entity_id);

                    let doc = self.trait_tombstone_to_document(&trait_tombstone);
                    index_writer.add_document(doc)?;
                }
                IndexOperation::PutEntityTombstone(entity_tombstone) => {
                    trace!(
                        "Putting tombstone for entity {} with op {}",
                        entity_tombstone.entity_id,
                        entity_tombstone.operation_id
                    );

                    self.entity_cache.remove(&entity_tombstone.entity_id);

                    let doc = self.entity_tombstone_to_document(&entity_tombstone);
                    index_writer.add_document(doc)?;
                }
                IndexOperation::PendingDeletionMarker(entity_id, operation_id) => {
                    trace!(
                        "Putting pending deletion marker for entity {} with op {}",
                        entity_id,
                        operation_id
                    );

                    self.entity_cache.remove(&entity_id);

                    let doc = self.pending_deletion_marker_to_document(&entity_id, operation_id);
                    index_writer.add_document(doc)?;
                }
                IndexOperation::DeleteEntityOperation(entity_id, operation_id) => {
                    trace!(
                        "Deleting operation {} from entity {} from index",
                        operation_id,
                        entity_id
                    );

                    self.entity_cache.remove(&entity_id);

                    index_writer
                        .delete_term(Term::from_field_u64(self.schema.operation_id, operation_id));
                }
            }

            if nb_operations % 10000 == 0 {
                info!("Indexed {} operations so far...", nb_operations);
            }
        }

        if nb_operations > 0 {
            index_writer.commit()?;
            // it may take milliseconds for reader to see committed changes, so we force
            // reload
            self.index_reader.reload()?;

            debug!(
                "Applied {} mutations to index in {:?} ({:?})",
                nb_operations,
                begin_time.elapsed(),
                self.storage
            );
        }

        Ok(nb_operations)
    }

    /// Return the highest block offset found in the index.
    /// This is used to know from which point we need to re-index.
    pub fn highest_indexed_block(&self) -> Result<Option<BlockOffset>, Error> {
        let searcher = self.index_reader.searcher();

        let query = AllQuery;
        let top_collector = TopDocs::with_limit(1).order_by_u64_field(self.schema.block_offset);
        let search_results = searcher.search(&query, &top_collector)?;

        Ok(search_results
            .first()
            .map(|(block_offset, _doc_addr)| *block_offset))
    }

    /// Execute a query on the index and return a page of mutations matching the
    /// query.
    pub fn search<Q: Borrow<EntityQuery>>(&self, query: Q) -> Result<MutationResults, Error> {
        let parsed_query =
            QueryParser::parse(&self.index, &self.schema, &self.config, query.borrow())?;

        let searcher = self.index_reader.searcher();
        let results = self.execute_tantivy_query_with_paging(&searcher, parsed_query)?;

        Ok(results)
    }

    /// Execute a query on the index and return an iterator over all matching
    /// mutations.
    pub fn search_iter<Q: Borrow<EntityQuery>>(
        &self,
        query: Q,
    ) -> Result<MutationResultIterator<Q>, Error> {
        let results = self.search(query.borrow())?;

        Ok(MutationResultIterator {
            index: self,
            query,
            total_results: results.total,
            current_results: results.mutations.into_iter(),
            next_page: results.next_page,
            max_pages: self.config.iterator_max_pages,
        })
    }

    /// Fetch all mutations for a given entity id.
    ///
    /// This method is in a very hot path since it's called to get all the
    /// mutations of an entity that go returned in a search query. Therefor,
    /// we use a cache to store all the mutations that we fetch here and
    /// bust them when we index a new mutation for an entity.
    pub fn fetch_entity_mutations(
        &self,
        entity_id: EntityIdRef,
    ) -> Result<EntityMutationResults, Error> {
        if let Some(results) = self.entity_cache.get(entity_id) {
            return Ok(results);
        }

        let searcher = self.index_reader.searcher();

        let term = Term::from_field_text(self.schema.entity_id, entity_id);
        let tantivy = TermQuery::new(term, IndexRecordOption::Basic);

        let ordering = Ordering {
            ascending: true,
            value: Some(ordering::Value::OperationId(true)),
            ..Default::default()
        };
        let paging = Paging {
            count: ENTITY_MAX_TRAITS,
            ..Default::default()
        };

        let parsed_query = ParsedQuery {
            tantivy: Box::new(tantivy),
            paging,
            ordering,
            trait_name: None,
        };

        let mut results = self.execute_tantivy_query_with_paging(&searcher, parsed_query)?;

        // because of the way we index pending (we may have pending store events after
        // indexing it after first), we need to make sure we don't include any
        // duplicate operations
        dedup_results(&self.storage, &mut results);

        let entity_mutations = EntityMutationResults {
            mutations: results.mutations,
        };
        self.entity_cache.put(entity_id, entity_mutations.clone());

        Ok(entity_mutations)
    }

    /// Converts a trait put / update to Tantivy document
    fn trait_put_to_document(&self, operation: &PutTraitMutation) -> Result<Document, Error> {
        let message_any = operation
            .trt
            .message
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("Trait message"))?;

        let mut doc = Document::default();

        let message_full_name = reflect::any_url_to_full_name(&message_any.type_url);
        doc.add_text(self.schema.trait_type, &message_full_name);
        doc.add_text(self.schema.trait_id, &operation.trt.id);
        doc.add_text(self.schema.entity_id, &operation.entity_id);
        doc.add_text(
            self.schema.entity_trait_id,
            &format!("{}_{}", operation.entity_id, &operation.trt.id),
        );

        doc.add_u64(self.schema.operation_id, operation.operation_id);
        if let Some(block_offset) = operation.block_offset {
            doc.add_u64(self.schema.block_offset, block_offset);
        }

        if let Some(creation_date) = &operation.trt.creation_date {
            doc.add_u64(
                self.schema.creation_date,
                creation_date.to_timestamp_nanos(),
            );
        }
        if let Some(modification_date) = &operation.trt.modification_date {
            doc.add_u64(
                self.schema.modification_date,
                modification_date.to_timestamp_nanos(),
            );
        }

        doc.add_u64(self.schema.document_type, MutationType::TRAIT_PUT_ID);

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
        let dyn_message = match reflect::from_prost_any(self.schema_registry.as_ref(), message_any)
        {
            Ok(dyn_message) => dyn_message,
            Err(err) => {
                error!(
                    "Error reflecting message of type '{message_full_name}'. No fields will be indexed. Error: {err}",
                );
                return;
            }
        };

        let Some(message_mappings) = self.schema.dynamic_fields.get(dyn_message.full_name()) else {
            // field is not indexed if we don't have a mapping for it
            return;
        };

        let mut has_reference = false;

        for field in dyn_message.fields().values() {
            self.add_trait_message_document_field(
                doc,
                &dyn_message,
                None,
                field,
                message_mappings,
                &mut has_reference,
            );
        }

        // the message has at least one reference field
        if has_reference {
            doc.add_u64(
                self.schema.has_reference,
                schema::bool_to_u64(has_reference),
            );
        }
    }

    fn add_trait_message_document_field(
        &self,
        doc: &mut Document,
        dyn_message: &DynamicMessage,
        prefix: Option<&str>,
        field_desc: &FieldDescriptor,
        message_mappings: &BTreeMap<String, schema::MappedDynamicField>,
        has_reference: &mut bool,
    ) {
        let field_value = match dyn_message.get_field_value(field_desc.id) {
            Ok(fv) => fv,
            Err(err) => {
                trace!("Couldn't get value of field {}: {}", field_desc.name, err);
                return;
            }
        };

        let field_name = if let Some(prefix) = prefix {
            format!("{}.{}", prefix, field_desc.name)
        } else {
            field_desc.name.to_string()
        };

        // special case for fields of type message. we always recurse into them even if
        // they don't have a mapping directly since mapping is done on
        // sub-fields (ex: mapped into `field.sub_field`)
        if let FieldValue::Message(_, _) = &field_value {
            match field_value.into_message(&self.schema_registry) {
                Ok(dyn_msg) => {
                    for sub_field in dyn_msg.fields().values() {
                        self.add_trait_message_document_field(
                            doc,
                            &dyn_msg,
                            Some(&field_name),
                            sub_field,
                            message_mappings,
                            has_reference,
                        );
                    }
                }
                Err(err) => {
                    error!("Error getting message descriptor for sub field: {err}");
                }
            }
            return;
        }

        let Some(mapped_field) = message_mappings.get(&field_name) else {
            // field is not indexed if we don't have a mapping for it
            return;
        };

        match field_value {
            FieldValue::String(value) if field_desc.text_flag => {
                doc.add_text(mapped_field.field, &value);
                doc.add_text(self.schema.all_text, &value);
            }
            FieldValue::String(value) if field_desc.indexed_flag => {
                doc.add_text(mapped_field.field, &value);
            }
            FieldValue::Reference(value) if field_desc.indexed_flag => {
                let ref_value = format!("entity{} trait{}", value.entity_id, value.trait_id);
                doc.add_text(mapped_field.field, &ref_value);
                doc.add_text(self.schema.all_refs, &ref_value);
                *has_reference = true;
            }
            FieldValue::DateTime(value) if field_desc.indexed_flag || field_desc.sorted_flag => {
                doc.add_u64(
                    mapped_field.field,
                    value.timestamp_nanos_opt().unwrap_or_default() as u64,
                );
            }
            FieldValue::Int64(value) if field_desc.indexed_flag || field_desc.sorted_flag => {
                doc.add_i64(mapped_field.field, value);
            }
            FieldValue::Int32(value) if field_desc.indexed_flag || field_desc.sorted_flag => {
                doc.add_i64(mapped_field.field, i64::from(value));
            }
            FieldValue::Uint64(value) if field_desc.indexed_flag || field_desc.sorted_flag => {
                doc.add_u64(mapped_field.field, value);
            }
            FieldValue::Uint32(value) if field_desc.indexed_flag || field_desc.sorted_flag => {
                doc.add_u64(mapped_field.field, u64::from(value));
            }
            other => {
                warn!(
                    "Unsupported indexed field type / value: type={:?} value={:?} mapping={:?}",
                    field_desc.field_type, other, mapped_field
                );
            }
        }
    }

    /// Converts a trait tombstone mutation to Tantivy document
    fn trait_tombstone_to_document(&self, operation: &PutTraitTombstoneMutation) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.schema.trait_id, &operation.trait_id);
        doc.add_text(self.schema.entity_id, &operation.entity_id);
        doc.add_text(
            self.schema.entity_trait_id,
            &format!("{}_{}", operation.entity_id, operation.trait_id),
        );
        doc.add_u64(self.schema.operation_id, operation.operation_id);

        if let Some(block_offset) = operation.block_offset {
            doc.add_u64(self.schema.block_offset, block_offset);
        }

        doc.add_u64(self.schema.document_type, MutationType::TRAIT_TOMBSTONE_ID);

        doc
    }

    /// Converts an entity tombstone mutation to Tantivy document
    fn entity_tombstone_to_document(&self, operation: &PutEntityTombstoneMutation) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.schema.entity_id, &operation.entity_id);
        doc.add_u64(self.schema.operation_id, operation.operation_id);

        if let Some(block_offset) = operation.block_offset {
            doc.add_u64(self.schema.block_offset, block_offset);
        }

        doc.add_u64(self.schema.document_type, MutationType::ENTITY_TOMBSTONE_ID);

        doc
    }

    /// Converts a marker used to indicate that an entity has pending deletions.
    /// Only used in pending index, since chain index actually deletes the
    /// operations.
    fn pending_deletion_marker_to_document(
        &self,
        entity_id: EntityIdRef,
        operation_id: OperationId,
    ) -> Document {
        let mut doc = Document::default();

        doc.add_text(self.schema.entity_id, entity_id);
        doc.add_u64(self.schema.operation_id, operation_id);
        doc.add_u64(self.schema.document_type, MutationType::PENDING_DELETION_ID);

        doc
    }

    /// Execute query on Tantivy index by taking paging, ordering into
    /// consideration and returns paged results.
    fn execute_tantivy_query_with_paging<S>(
        &self,
        searcher: S,
        query: ParsedQuery,
    ) -> Result<MutationResults, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let ordering_value = query
            .ordering
            .value
            .ok_or(Error::ProtoFieldExpected("ordering.value"))?;
        let (mutations, total) = match ordering_value {
            ordering::Value::Score(_) => {
                let collector = self.match_score_collector(
                    &query.paging,
                    query.ordering.ascending,
                    query.ordering.no_recency_boost,
                );
                self.execute_tantity_query_with_collector(
                    searcher,
                    query.tantivy.as_ref(),
                    collector,
                )?
            }
            ordering::Value::OperationId(_) => {
                let sort_field = self.schema.operation_id;
                let collector = self.sorted_field_collector(
                    &query.paging,
                    sort_field,
                    query.ordering.ascending,
                );
                self.execute_tantity_query_with_collector(
                    searcher,
                    query.tantivy.as_ref(),
                    collector,
                )?
            }
            ordering::Value::CreatedAt(_) => {
                let collector = self.sorted_field_collector(
                    &query.paging,
                    self.schema.creation_date,
                    query.ordering.ascending,
                );
                self.execute_tantity_query_with_collector(
                    searcher,
                    query.tantivy.as_ref(),
                    collector,
                )?
            }
            ordering::Value::UpdatedAt(_) => {
                let collector = self.sorted_field_collector(
                    &query.paging,
                    self.schema.modification_date,
                    query.ordering.ascending,
                );
                self.execute_tantity_query_with_collector(
                    searcher,
                    query.tantivy.as_ref(),
                    collector,
                )?
            }
            ordering::Value::Field(field_name) => {
                let trait_name = query.trait_name.ok_or_else(|| {
                    Error::QueryParsing(anyhow!("Ordering by field only supported in trait query",))
                })?;

                let sort_field = self
                    .schema
                    .get_dynamic_trait_field(&trait_name, &field_name)?;
                if !sort_field.is_fast_field {
                    return Err(Error::QueryParsing(anyhow!(
                        "Cannot sort by field '{}' as it's not sortable in  trait '{}'",
                        field_name,
                        trait_name,
                    )));
                }

                let collector = self.sorted_field_collector(
                    &query.paging,
                    sort_field.field,
                    query.ordering.ascending,
                );
                self.execute_tantity_query_with_collector(
                    searcher,
                    query.tantivy.as_ref(),
                    collector,
                )?
            }
        };

        let next_page = if mutations.len() >= query.paging.count as usize {
            Some(Paging {
                count: query.paging.count,
                offset: query.paging.offset + mutations.len() as u32,
                ..Default::default()
            })
        } else {
            None
        };

        let remaining = total - query.paging.offset as usize - mutations.len();
        Ok(MutationResults {
            mutations,
            total,
            remaining,
            next_page,
        })
    }

    /// Execute query on Tantivy index and build mutations metadata results.
    fn execute_tantity_query_with_collector<S, C>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        top_collector: C,
    ) -> Result<(Vec<MutationMetadata>, usize), Error>
    where
        S: Deref<Target = Searcher>,
        C: Collector<Fruit = Vec<(OrderingValueWrapper, DocAddress)>>,
    {
        let mut multi_collector = MultiCollector::new();
        let top_collector = multi_collector.add_collector(top_collector);
        let count_collector = multi_collector.add_collector(Count);

        let mut fruits = searcher.search(query, &multi_collector)?;

        let mut results = Vec::new();
        for (sort_value, doc_addr) in top_collector.extract(&mut fruits) {
            // ignored results were out of the requested paging
            if !sort_value.ignore {
                let doc = searcher.doc(doc_addr)?;
                let block_offset = schema::get_doc_opt_u64_value(&doc, self.schema.block_offset);
                let operation_id = schema::get_doc_u64_value(&doc, self.schema.operation_id);
                let entity_id = schema::get_doc_string_value(&doc, self.schema.entity_id);
                let opt_trait_id = schema::get_doc_opt_string_value(&doc, self.schema.trait_id);
                let document_type_id = schema::get_doc_u64_value(&doc, self.schema.document_type);

                let mut mutation_type = MutationType::new(document_type_id, opt_trait_id)?;
                if let MutationType::TraitPut(put_trait) = &mut mutation_type {
                    put_trait.creation_date =
                        schema::get_doc_opt_u64_value(&doc, self.schema.creation_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                    put_trait.modification_date =
                        schema::get_doc_opt_u64_value(&doc, self.schema.modification_date)
                            .map(|ts| Utc.timestamp_nanos(ts as i64));
                    put_trait.trait_type =
                        schema::get_doc_opt_string_value(&doc, self.schema.trait_type);
                    put_trait.has_reference =
                        schema::get_doc_opt_bool_value(&doc, self.schema.has_reference)
                            .unwrap_or(false);
                }

                let result = MutationMetadata {
                    operation_id,
                    block_offset,
                    entity_id,
                    mutation_type,
                    sort_value,
                };
                results.push(result);
            }
        }

        let total_count = count_collector.extract(&mut fruits);

        Ok((results, total_count))
    }

    /// Creates a Tantivy top document collectors that sort by the given fast
    /// field and limits the result by the requested paging.
    fn sorted_field_collector(
        &self,
        paging: &Paging,
        sort_field: Field,
        ascending: bool,
    ) -> impl Collector<Fruit = Vec<(OrderingValueWrapper, DocAddress)>> {
        let operation_id_field = self.schema.operation_id;
        TopDocs::with_limit(paging.count as usize)
            .and_offset(paging.offset as usize)
            .custom_score(move |segment_reader: &SegmentReader| {
                let operation_id_reader = segment_reader
                    .fast_fields()
                    .u64(operation_id_field)
                    .unwrap();

                let sort_fast_field = segment_reader
                    .fast_fields()
                    .u64(sort_field)
                    .expect("Field requested is not a i64/u64 fast field.");
                move |doc_id| OrderingValueWrapper {
                    value: OrderingValue {
                        value: Some(ordering_value::Value::Uint64(
                            sort_fast_field.get_val(doc_id),
                        )),
                        operation_id: operation_id_reader.get_val(doc_id),
                    },
                    reverse: ascending,
                    ignore: false,
                }
            })
    }

    /// Creates a Tantivy top document collectors that sort by full text
    /// matching score and limits the result by the requested paging.
    fn match_score_collector(
        &self,
        paging: &Paging,
        ascending: bool,
        no_recency_boost: bool,
    ) -> impl Collector<Fruit = Vec<(OrderingValueWrapper, DocAddress)>> {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or_default() as u64;
        let operation_id_field = self.schema.operation_id;
        let modification_date_field = self.schema.modification_date;
        let boost = self.full_text_boost;
        TopDocs::with_limit(paging.count as usize)
            .and_offset(paging.offset as usize)
            .tweak_score(move |segment_reader: &SegmentReader| {
                let operation_id_reader = segment_reader
                    .fast_fields()
                    .u64(operation_id_field)
                    .unwrap();

                let modification_date_reader = segment_reader
                    .fast_fields()
                    .u64(modification_date_field)
                    .unwrap();

                move |doc_id, score| {
                    let operation_id = operation_id_reader.get_val(doc_id);

                    // boost by date if needed
                    let score = if !no_recency_boost {
                        let mut modification_date = modification_date_reader.get_val(doc_id);
                        if modification_date == 0 {
                            modification_date = operation_id;
                        }
                        score * boost_recent(now, modification_date) * boost
                    } else {
                        score
                    };

                    OrderingValueWrapper {
                        value: OrderingValue {
                            value: Some(ordering_value::Value::Float(score)),
                            operation_id,
                        },
                        reverse: ascending,
                        ignore: false,
                    }
                }
            })
    }
}

fn dedup_results(storage: &Storage, results: &mut MutationResults) {
    let mut i = 0;
    let mut prev_operation = None;
    while i < results.mutations.len() {
        let op_id = results.mutations[i].operation_id;
        if prev_operation == Some(op_id) {
            if *storage == Storage::Disk {
                error!(
                    "duplicate operation in disk index: op={} block={:?}",
                    op_id, results.mutations[i].block_offset
                );
            }
            results.mutations.swap_remove(i);
        } else {
            prev_operation = Some(op_id);
            i += 1;
        }
    }
}

fn index_settings() -> IndexSettings {
    IndexSettings {
        sort_by_field: Some(IndexSortByField {
            field: "operation_id".to_string(),
            order: Order::Desc,
        }),
        ..Default::default()
    }
}

fn boost_recent(now_nano: u64, date_nano: u64) -> f32 {
    // From: https://www.elastic.co/guide/en/elasticsearch/reference/current/query-dsl-function-score-query.html#function-decay
    // exp(λ * max(0, |value - origin| - offset))
    // λ = ln(decay) / scale
    //
    // See curve at https://www.desmos.com/calculator/ktjtz8yeln

    const OFFSET: f32 = 15.0; // time between [now - offset, now] at which score = 1.0
    const DECAY_LN: f32 = -1.609_437; // = ln(0.2), where 0.2 = decay
    const SCALE: f32 = 365.0; // score at (now - scale) = decay value
    const LAMBDA: f32 = DECAY_LN / SCALE;

    let now_days = (now_nano / 86_400_000_000_000) as f32;
    let date_days = (date_nano / 86_400_000_000_000) as f32;

    (LAMBDA * ((now_days - date_days).abs() - OFFSET).max(0.0)).exp()
}

#[cfg(test)]
mod unit_tests {
    use chrono::{DateTime, Duration};

    use super::*;

    macro_rules! approx_eq {
        ($left:expr, $right:expr) => {
            let diff = ($left - $right).abs();
            assert!(
                diff < f32::EPSILON,
                "left = {}, right = {}, |left - right| = {}",
                $left,
                $right,
                diff
            );
        };
    }

    #[test]
    fn test_boost_recent() {
        let now = Utc::now();

        approx_eq!(1.0, boost_recent_chrono(now, now)); // same day should be 1.0
        approx_eq!(
            1.0,
            boost_recent_chrono(now, now - Duration::try_days(15).unwrap())
        ); // within 15 days of now should be 1.0

        // at 365d from now should equal decay
        approx_eq!(
            0.2136757,
            boost_recent_chrono(now, now - Duration::try_days(365).unwrap())
        );

        // at 730d from now, should be still decreasing
        approx_eq!(
            0.042735178,
            boost_recent_chrono(now, now - Duration::try_days(730).unwrap())
        );
    }

    fn boost_recent_chrono(now: DateTime<Utc>, date: DateTime<Utc>) -> f32 {
        boost_recent(
            now.timestamp_nanos_opt().unwrap_or_default() as u64,
            date.timestamp_nanos_opt().unwrap_or_default() as u64,
        )
    }
}
