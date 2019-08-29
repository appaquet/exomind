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
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, QueryParser, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, Schema as TantivySchema, SchemaBuilder, FAST, INDEXED, STORED,
    STRING, TEXT,
};
use tantivy::{Document, Index as TantivyIndex, IndexReader, IndexWriter, Searcher, Term};

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
    pub fn search(&self, query: &Query, limit: usize) -> Result<Vec<TraitResult>, Error> {
        let searcher = self.index_reader.searcher();
        let res = match query {
            Query::WithTrait(inner_query) => {
                self.search_with_trait(searcher, inner_query, limit)?
            }
            Query::Match(inner_query) => self.search_matches(searcher, inner_query, limit)?,
            Query::IdEqual(inner_query) => self.search_entity_id(searcher, inner_query, limit)?,
            Query::Conjunction(_inner_query) => unimplemented!(),

            #[cfg(test)]
            Query::TestFail(_query) => {
                return Err(Error::Other("Query failed for tests".to_string()))
            }
        };

        Ok(res)
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

        let indexeds = record_schema.fields().iter().filter(|f| f.indexed);
        for field in indexeds {
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
        query: &WithTraitQuery,
        limit: usize,
    ) -> Result<Vec<TraitResult>, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let trait_schema =
            if let Some(trait_schema) = self.schema.trait_by_full_name(&query.trait_name) {
                trait_schema
            } else {
                return Ok(vec![]);
            };

        let term = Term::from_field_u64(self.fields.trait_type, u64::from(trait_schema.id()));
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        self.execute_tantivy_query(searcher, &query, limit)
    }

    /// Execute a search by text query
    fn search_matches<S>(
        &self,
        searcher: S,
        query: &MatchQuery,
        limit: usize,
    ) -> Result<Vec<TraitResult>, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.text]);
        let query = query_parser.parse_query(&query.query)?;
        self.execute_tantivy_query(searcher, &query, limit)
    }

    /// Execute a search by entity id query
    fn search_entity_id<S>(
        &self,
        searcher: S,
        query: &IdEqualQuery,
        limit: usize,
    ) -> Result<Vec<TraitResult>, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let term = Term::from_field_text(self.fields.entity_id, &query.entity_id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        self.execute_tantivy_query(searcher, &query, limit)
    }

    /// Execute query on Tantivy index and build trait result
    fn execute_tantivy_query<S>(
        &self,
        searcher: S,
        query: &dyn tantivy::query::Query,
        limit: usize,
    ) -> Result<Vec<TraitResult>, Error>
    where
        S: Deref<Target = Searcher>,
    {
        let top_collector = TopDocs::with_limit(limit);
        let search_results = searcher.search(query, &top_collector)?;

        let mut results = Vec::new();
        for (score, doc_addr) in search_results {
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
                score,
            };
            results.push(result);
        }

        Ok(results)
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
        schema_builder.add_u64_field("block_offset", STORED | FAST);
        schema_builder.add_u64_field("operation_id", INDEXED | STORED);
        schema_builder.add_u64_field("tombstone", STORED);
        schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();

        let fields = Fields {
            trait_type: schema.get_field("trait_type").unwrap(),
            entity_id: schema.get_field("entity_id").unwrap(),
            trait_id: schema.get_field("trait_id").unwrap(),
            entity_trait_id: schema.get_field("entity_trait_id").unwrap(),
            block_offset: schema.get_field("block_offset").unwrap(),
            operation_id: schema.get_field("operation_id").unwrap(),
            tombstone: schema.get_field("tombstone").unwrap(),
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
    block_offset: Field,
    operation_id: Field,
    tombstone: Field,
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
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_schema::entity::{RecordBuilder, TraitBuilder};
    use exocore_schema::tests_utils::create_test_schema;

    #[test]
    fn search_query_matches() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut indexer = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(1234),
            operation_id: 2345,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set("id", "trudeau1")
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });

        indexer.apply_mutation(contact_mutation)?;

        let query = Query::match_text("justin");
        let results = indexer.search(&query, 10)?;
        assert_eq!(results.len(), 1);

        let result = find_trait_result(&results, "trudeau1").unwrap();
        assert_eq!(result.block_offset, Some(1234));
        assert_eq!(result.operation_id, 2345);
        assert_eq!(result.entity_id, "entity_id1");
        assert_eq!(result.trait_id, "trudeau1");

        Ok(())
    }

    #[test]
    fn search_query_by_trait_type() -> Result<(), failure::Error> {
        let schema = create_test_schema();
        let config = TraitsIndexConfig::default();
        let mut index = TraitsIndex::create_in_memory(config, schema.clone())?;

        let contact_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 1,
            entity_id: "entity_id1".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "contact")?
                .set_id("trt1".to_string())
                .set("name", "Justin Trudeau")
                .set("email", "justin.trudeau@gov.ca")
                .build()?,
        });

        let email_mutation = IndexMutation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: 2,
            entity_id: "entity_id2".to_string(),
            trt: TraitBuilder::new(&schema, "exocore", "email")?
                .set_id("trt2")
                .set("subject", "Some subject")
                .set("body", "Very important body")
                .build()?,
        });

        index.apply_mutations(vec![contact_mutation, email_mutation].into_iter())?;

        let query = Query::with_trait("exocore.email");
        let results = index.search(&query, 10)?;
        assert!(find_trait_result(&results, "trt2").is_some());

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
        let results = index.search(&query, 10)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].operation_id, 2);

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
        assert_eq!(index.search(&query, 10)?.len(), 1);

        index.apply_mutation(IndexMutation::DeleteTrait(
            "entity_id1".to_string(),
            "trudeau1".to_string(),
        ))?;

        assert_eq!(index.search(&query, 10)?.len(), 0);

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
        assert_eq!(index.search(&query, 10)?.len(), 1);

        index.apply_mutation(IndexMutation::DeleteOperation(1234))?;

        assert_eq!(index.search(&query, 10)?.len(), 0);

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
        let res = index.search(&query, 10)?;
        assert!(res.first().unwrap().tombstone);

        let query = Query::with_entity_id("entity_id2");
        let res = index.search(&query, 10)?;
        assert!(!res.first().unwrap().tombstone);

        Ok(())
    }

    fn find_trait_result<'r>(
        results: &'r [TraitResult],
        trait_id: &str,
    ) -> Option<&'r TraitResult> {
        results.iter().find(|t| t.trait_id == trait_id)
    }

}
