use super::MutationIndexConfig;
use crate::error::Error;
use exocore_core::protos::reflect::FieldType;
use exocore_core::protos::registry::Registry;
use std::collections::HashMap;
use tantivy::schema::*;
use tantivy::tokenizer::*;
use tantivy::Document;

/// Tantitvy schema fields
pub(crate) struct Fields {
    pub config: MutationIndexConfig,

    pub trait_type: Field,
    pub entity_id: Field,
    pub trait_id: Field,
    pub entity_trait_id: Field,
    pub creation_date: Field,
    pub modification_date: Field,
    pub block_offset: Field,
    pub operation_id: Field,
    pub document_type: Field,

    pub all_text: Field,
    pub all_refs: Field,

    // mapping for indexed/sorted fields of messages in registry
    // message type -> field name -> tantivy field
    pub _dynamic_fields: DynamicFields,
    pub dynamic_mappings: DynamicFieldsMapping,

    pub references_tokenizer: TextAnalyzer,
}

impl Fields {
    pub fn get_dynamic_trait_field(
        &self,
        trait_name: &str,
        field_name: &str,
    ) -> Result<&MappedDynamicField, Error> {
        let fields_mapping = self.dynamic_mappings.get(trait_name).ok_or_else(|| {
            Error::QueryParsing(format!(
                "Trait '{}' doesn\'t have any dynamic fields",
                trait_name
            ))
        })?;

        let field = fields_mapping.get(field_name).ok_or_else(|| {
            Error::QueryParsing(format!(
                "Trait '{}' doesn\'t have any dynamic field with name '{}'",
                trait_name, field_name
            ))
        })?;

        Ok(field)
    }

    pub fn register_tokenizers(&self, index: &tantivy::Index) {
        index
            .tokenizers()
            .register("references", self.references_tokenizer.clone());
    }
}

pub(crate) type DynamicFieldsMapping = HashMap<String, HashMap<String, MappedDynamicField>>;

#[derive(Default)]
pub(crate) struct DynamicFields {
    reference: Vec<Field>,
    text: Vec<Field>,
    string: Vec<Field>,
    i64: Vec<Field>,
    i64_fast: Vec<Field>,
    u64: Vec<Field>,
    u64_fast: Vec<Field>,
}

#[derive(Debug)]
pub(crate) struct MappedDynamicField {
    pub field: Field,
    pub field_type: FieldType,
    pub is_fast_field: bool,
}

/// Builds Tantivy schema required for mutations related queries and registered
/// messages fields.
///
/// Tantivy doesn't support dynamic schema yet: https://github.com/tantivy-search/tantivy/issues/301
/// Because of this, we need to pre-allocate fields that will be used
/// sequentially by fields of each registered messages. This means that we only
/// support a limited amount of indexed/sorted fields per message.
pub(crate) fn build_tantivy_schema(
    config: MutationIndexConfig,
    registry: &Registry,
) -> (Schema, Fields) {
    let mut schema_builder = SchemaBuilder::default();

    let trait_type = schema_builder.add_text_field("trait_type", STRING | STORED);
    let entity_id = schema_builder.add_text_field("entity_id", STRING | STORED);
    let trait_id = schema_builder.add_text_field("trait_id", STRING | STORED);
    let entity_trait_id = schema_builder.add_text_field("entity_trait_id", STRING);
    let creation_date = schema_builder.add_u64_field("creation_date", STORED);
    let modification_date = schema_builder.add_u64_field("modification_date", STORED);
    let block_offset = schema_builder.add_u64_field("block_offset", STORED | FAST);
    let operation_id = schema_builder.add_u64_field("operation_id", INDEXED | STORED | FAST);
    let document_type = schema_builder.add_u64_field("document_type", STORED);

    // Tokenize references by space, but no stemming, case folding or length limit
    let references_tokenizer = TextAnalyzer::from(SimpleTokenizer);
    let references_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("references")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();

    let all_text = schema_builder.add_text_field("all_text", TEXT);
    let all_refs = schema_builder.add_text_field("all_refs", references_options.clone());

    let (dynamic_fields, dynamic_mappings) = build_dynamic_fields_tantivy_schema(
        &config,
        registry,
        &mut schema_builder,
        references_options,
    );

    let schema = schema_builder.build();

    let fields = Fields {
        config,
        trait_type,
        entity_id,
        trait_id,
        entity_trait_id,
        creation_date,
        modification_date,
        block_offset,
        operation_id,
        document_type,

        all_text,
        all_refs,

        _dynamic_fields: dynamic_fields,
        dynamic_mappings,

        references_tokenizer,
    };

    (schema, fields)
}

/// Adds all dynamic fields to the tantivy schema based on the registered
/// messages and creates a mapping of registered messages' fields to dynamic
/// fields.
fn build_dynamic_fields_tantivy_schema(
    config: &MutationIndexConfig,
    registry: &Registry,
    builder: &mut SchemaBuilder,
    references_options: TextOptions,
) -> (DynamicFields, DynamicFieldsMapping) {
    let mut dyn_fields = DynamicFields::default();

    // create dynamic fields that will be usable by defined messages
    for i in 0..config.dynamic_reference_fields {
        dyn_fields
            .reference
            .push(builder.add_text_field(&format!("references_{}", i), references_options.clone()));
    }
    for i in 0..config.dynamic_text_fields {
        dyn_fields
            .text
            .push(builder.add_text_field(&format!("text_{}", i), TEXT));
    }
    for i in 0..config.dynamic_string_fields {
        dyn_fields
            .string
            .push(builder.add_text_field(&format!("string_{}", i), STRING));
    }
    for i in 0..config.dynamic_i64_fields {
        dyn_fields
            .i64
            .push(builder.add_i64_field(&format!("i64_{}", i), STORED));
    }
    for i in 0..config.dynamic_i64_sortable_fields {
        dyn_fields
            .i64_fast
            .push(builder.add_i64_field(&format!("i64_fast_{}", i), STORED | FAST));
    }
    for i in 0..config.dynamic_u64_fields {
        dyn_fields
            .u64
            .push(builder.add_u64_field(&format!("u64_{}", i), STORED));
    }
    for i in 0..config.dynamic_u64_sortable_fields {
        dyn_fields
            .u64_fast
            .push(builder.add_u64_field(&format!("u64_fast_{}", i), STORED | FAST));
    }

    // map fields of each message in registry that need to be indexed / sortable to
    // dynamic fields
    let mut dyn_mappings = HashMap::new();
    let message_descriptors = registry.message_descriptors();
    for message_descriptor in message_descriptors {
        let mut ref_fields_count = 0;
        let mut text_fields_count = 0;
        let mut string_fields_count = 0;
        let mut i64_fields_count = 0;
        let mut i64_fast_fields_count = 0;
        let mut u64_fields_count = 0;
        let mut u64_fast_fields_count = 0;
        let mut field_mapping = HashMap::new();

        for field in message_descriptor.fields.values() {
            if !field.indexed_flag && !field.text_flag && !field.sorted_flag {
                continue;
            }

            let ft = field.field_type.clone();
            if ft == FieldType::Uint64 || ft == FieldType::Uint32 || ft == FieldType::DateTime {
                if field.sorted_flag && u64_fast_fields_count < dyn_fields.u64_fast.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.u64_fast[u64_fast_fields_count],
                        field_type: ft,
                        is_fast_field: true,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    u64_fast_fields_count += 1;
                    continue;
                } else if field.indexed_flag && u64_fields_count < dyn_fields.u64.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.u64[u64_fields_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    u64_fields_count += 1;
                    continue;
                }
            } else if ft == FieldType::Int64 || ft == FieldType::Int32 {
                if field.sorted_flag && i64_fast_fields_count < dyn_fields.i64_fast.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.i64_fast[i64_fast_fields_count],
                        field_type: ft,
                        is_fast_field: true,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    i64_fast_fields_count += 1;
                    continue;
                } else if field.indexed_flag && i64_fields_count < dyn_fields.i64.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.i64[i64_fields_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    i64_fields_count += 1;
                    continue;
                }
            } else if ft == FieldType::Reference {
                if field.indexed_flag && ref_fields_count < dyn_fields.reference.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.reference[ref_fields_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    ref_fields_count += 1;
                    continue;
                }
            } else if ft == FieldType::String {
                if field.text_flag && text_fields_count < dyn_fields.text.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.text[text_fields_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    text_fields_count += 1;
                    continue;
                } else if field.indexed_flag && string_fields_count < dyn_fields.string.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.string[string_fields_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    field_mapping.insert(field.name.clone(), mapped_field);
                    string_fields_count += 1;
                    continue;
                }
            }

            error!(
                "Invalid index option / type for field {:?} of message {} or ran out of fields",
                field, message_descriptor.name
            );
        }

        if !field_mapping.is_empty() {
            dyn_mappings.insert(message_descriptor.name.clone(), field_mapping);
        }
    }
    (dyn_fields, dyn_mappings)
}

/// Extracts string value from Tantivy document
pub(crate) fn get_doc_string_value(doc: &Document, field: Field) -> String {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::Str(v)) => v.to_string(),
        _ => panic!("Couldn't find field of type string"),
    }
}

/// Extracts optional string value from Tantivy document
pub(crate) fn get_doc_opt_string_value(doc: &Document, field: Field) -> Option<String> {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::Str(v)) => Some(v.to_string()),
        _ => None,
    }
}

/// Extracts optional u46 value from Tantivy document
pub(crate) fn get_doc_opt_u64_value(doc: &Document, field: Field) -> Option<u64> {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::U64(v)) => Some(*v),
        _ => None,
    }
}

/// Extracts u46 value from Tantivy document
pub(crate) fn get_doc_u64_value(doc: &Document, field: Field) -> u64 {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::U64(v)) => *v,
        _ => panic!("Couldn't find field of type u64"),
    }
}
