use std::collections::{BTreeMap, HashMap};

use exocore_protos::{
    reflect::{FieldDescriptor, FieldType, ReflectMessageDescriptor},
    registry::Registry,
};
use tantivy::{schema::*, tokenizer::*, Document};

use super::MutationIndexConfig;
use crate::error::Error;

/// Schema that contains Tantivy fields for mutations and indexed messages.
///
/// Tantivy doesn't support dynamic schema yet: https://github.com/tantivy-search/tantivy/issues/301
/// Because of this, we need to pre-allocate fields that will be used
/// sequentially by fields of each registered messages. This means that we only
/// support a limited amount of indexed/sorted fields per message.
pub(crate) struct MutationIndexSchema {
    pub tantivy: Schema,

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

    pub has_reference: Field,

    // mapping for indexed/sorted fields of messages in registry
    // message type -> field name -> tantivy field
    pub dynamic_fields: DynamicFieldsMapping,
    pub short_names: HashMap<String, String>,

    pub references_tokenizer: TextAnalyzer,
}

impl MutationIndexSchema {
    pub(crate) fn new(config: MutationIndexConfig, registry: &Registry) -> MutationIndexSchema {
        let mut schema_builder = SchemaBuilder::default();

        let trait_type = schema_builder.add_text_field("trait_type", STRING | STORED);
        let entity_id = schema_builder.add_text_field("entity_id", STRING | STORED);
        let trait_id = schema_builder.add_text_field("trait_id", STRING | STORED);
        let entity_trait_id = schema_builder.add_text_field("entity_trait_id", STRING);
        let creation_date = schema_builder.add_u64_field("creation_date", STORED | FAST);
        let modification_date = schema_builder.add_u64_field("modification_date", STORED | FAST);
        let block_offset = schema_builder.add_u64_field("block_offset", STORED | FAST);
        let operation_id = schema_builder.add_u64_field(
            "operation_id",
            NumericOptions::default()
                .set_indexed()
                .set_stored()
                .set_fast(Cardinality::SingleValue),
        );
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
        let has_reference = schema_builder.add_u64_field("has_reference", STORED);

        let dynamic_fields = build_dynamic_fields_tantivy_schema(
            &config,
            registry,
            &mut schema_builder,
            references_options,
        );

        let short_names = build_schema_short_type_mapping(registry);

        MutationIndexSchema {
            tantivy: schema_builder.build(),

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

            has_reference,

            dynamic_fields,
            short_names,

            references_tokenizer,
        }
    }

    pub fn get_dynamic_trait_field(
        &self,
        trait_name: &str,
        field_name: &str,
    ) -> Result<&MappedDynamicField, Error> {
        let trait_fields = self.trait_fields(trait_name)?;

        let field = trait_fields.get(field_name).ok_or_else(|| {
            Error::QueryParsing(anyhow!(
                "Trait '{}' doesn\'t have any dynamic field with name '{}'",
                trait_name,
                field_name
            ))
        })?;

        Ok(field)
    }

    pub fn get_dynamic_trait_field_prefix(
        &self,
        trait_name: &str,
        field_or_prefix: &str,
    ) -> Result<Vec<&MappedDynamicField>, Error> {
        let trait_fields = self.trait_fields(trait_name)?;

        let field_or_prefix_clone = field_or_prefix.to_string();
        let field_prefix = format!("{}.", field_or_prefix);

        trait_fields
            .range(field_or_prefix_clone..)
            .take_while(|field| field.0 == field_or_prefix || field.0.starts_with(&field_prefix))
            .map(|(_field_name, field)| Ok(field))
            .collect()
    }

    pub fn register_tokenizers(&self, index: &tantivy::Index) {
        index
            .tokenizers()
            .register("references", self.references_tokenizer.clone());
    }

    pub fn get_message_name_from_short(&self, short_name: &str) -> Option<&str> {
        self.short_names
            .get(&short_name.to_lowercase())
            .map(|name| name.as_str())
    }

    fn trait_fields(
        &self,
        trait_name: &str,
    ) -> Result<&BTreeMap<String, MappedDynamicField>, Error> {
        let trait_fields = self.dynamic_fields.get(trait_name).ok_or_else(|| {
            Error::QueryParsing(anyhow!(
                "Trait '{}' doesn\'t have any dynamic fields",
                trait_name
            ))
        })?;

        Ok(trait_fields)
    }
}

pub(crate) type DynamicFieldsMapping = HashMap<String, BTreeMap<String, MappedDynamicField>>;

/// Dynamic fields reused across trait types. Tantivy doesn't allow adding new
/// fields once the index has been created. We pre-alloc a certain number of
/// fields for each field type and reuse them across trait types.
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

impl DynamicFields {
    fn new(
        index_config: &MutationIndexConfig,
        schema_builder: &mut SchemaBuilder,
        references_options: TextOptions,
    ) -> DynamicFields {
        let mut dyn_fields = DynamicFields::default();

        for i in 0..index_config.dynamic_reference_fields {
            dyn_fields.reference.push(
                schema_builder
                    .add_text_field(&format!("references_{}", i), references_options.clone()),
            );
        }
        for i in 0..index_config.dynamic_text_fields {
            dyn_fields
                .text
                .push(schema_builder.add_text_field(&format!("text_{}", i), TEXT));
        }
        for i in 0..index_config.dynamic_string_fields {
            dyn_fields
                .string
                .push(schema_builder.add_text_field(&format!("string_{}", i), STRING));
        }
        for i in 0..index_config.dynamic_i64_fields {
            dyn_fields
                .i64
                .push(schema_builder.add_i64_field(&format!("i64_{}", i), STORED));
        }
        for i in 0..index_config.dynamic_i64_sortable_fields {
            dyn_fields
                .i64_fast
                .push(schema_builder.add_i64_field(&format!("i64_fast_{}", i), STORED | FAST));
        }
        for i in 0..index_config.dynamic_u64_fields {
            dyn_fields
                .u64
                .push(schema_builder.add_u64_field(&format!("u64_{}", i), STORED));
        }
        for i in 0..index_config.dynamic_u64_sortable_fields {
            dyn_fields
                .u64_fast
                .push(schema_builder.add_u64_field(&format!("u64_fast_{}", i), STORED | FAST));
        }

        dyn_fields
    }
}

#[derive(Debug)]
pub(crate) struct MappedDynamicField {
    pub field: Field,
    pub field_type: FieldType,
    pub is_fast_field: bool,
}

/// Adds all dynamic fields to the tantivy schema based on the registered
/// messages and creates a mapping of registered messages' fields to dynamic
/// fields.
fn build_dynamic_fields_tantivy_schema(
    index_config: &MutationIndexConfig,
    registry: &Registry,
    schema_builder: &mut SchemaBuilder,
    references_options: TextOptions,
) -> DynamicFieldsMapping {
    let mut dyn_fields = DynamicFields::new(index_config, schema_builder, references_options);

    // map fields of each message in registry that need to be indexed / sortable to
    // dynamic fields
    let mut dyn_mappings = HashMap::new();
    let message_descriptors = registry.message_descriptors();
    for message_descriptor in message_descriptors {
        let mut msg_fields = MsgFields::default();

        for field in message_descriptor.fields.values() {
            msg_fields.add_field(None, registry, &mut dyn_fields, &message_descriptor, field);
        }

        if !msg_fields.mapping.is_empty() {
            dyn_mappings.insert(message_descriptor.name.clone(), msg_fields.mapping);
        }
    }

    dyn_mappings
}

/// Creates a map of short type names to their full type names.
fn build_schema_short_type_mapping(registry: &Registry) -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    let message_descriptors = registry.message_descriptors();
    for message_descriptor in message_descriptors {
        for short_name in &message_descriptor.short_names {
            if mapping.contains_key(short_name) {
                warn!(
                    "Short type name {} is already mapped to {}. Skipping mapping to {}.",
                    short_name, mapping[short_name], message_descriptor.name
                );
                continue;
            }

            mapping.insert(short_name.to_lowercase(), message_descriptor.name.clone());
        }
    }
    mapping
}

// Fields mapping for a trait.
#[derive(Default)]
struct MsgFields {
    mapping: BTreeMap<String, MappedDynamicField>,
    ref_count: usize,
    text_count: usize,
    string_count: usize,
    i64_count: usize,
    i64_fast_count: usize,
    u64_count: usize,
    u64_fast_count: usize,
}

impl MsgFields {
    fn add_field(
        &mut self,
        prefix: Option<&str>,
        registry: &Registry,
        dyn_fields: &mut DynamicFields,
        msg_desc: &ReflectMessageDescriptor,
        field_desc: &FieldDescriptor,
    ) {
        if !field_desc.indexed_flag && !field_desc.text_flag && !field_desc.sorted_flag {
            return;
        }

        let field_name = if let Some(prefix) = prefix {
            format!("{}.{}", prefix, field_desc.name)
        } else {
            field_desc.name.to_string()
        };

        let ft = field_desc.field_type.clone();
        match ft {
            FieldType::Uint64 | FieldType::Uint32 | FieldType::DateTime => {
                if field_desc.sorted_flag && self.u64_fast_count < dyn_fields.u64_fast.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.u64_fast[self.u64_fast_count],
                        field_type: ft,
                        is_fast_field: true,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.u64_fast_count += 1;
                    return;
                } else if field_desc.indexed_flag && self.u64_count < dyn_fields.u64.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.u64[self.u64_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.u64_count += 1;
                    return;
                }
            }

            FieldType::Int64 | FieldType::Int32 => {
                if field_desc.sorted_flag && self.i64_fast_count < dyn_fields.i64_fast.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.i64_fast[self.i64_fast_count],
                        field_type: ft,
                        is_fast_field: true,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.i64_fast_count += 1;
                    return;
                } else if field_desc.indexed_flag && self.i64_count < dyn_fields.i64.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.i64[self.i64_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.i64_count += 1;
                    return;
                }
            }

            FieldType::Reference => {
                if field_desc.indexed_flag && self.ref_count < dyn_fields.reference.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.reference[self.ref_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.ref_count += 1;
                    return;
                }
            }

            FieldType::String => {
                if field_desc.text_flag && self.text_count < dyn_fields.text.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.text[self.text_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.text_count += 1;
                    return;
                } else if field_desc.indexed_flag && self.string_count < dyn_fields.string.len() {
                    let mapped_field = MappedDynamicField {
                        field: dyn_fields.string[self.string_count],
                        field_type: ft,
                        is_fast_field: false,
                    };
                    self.mapping.insert(field_name, mapped_field);
                    self.string_count += 1;
                    return;
                }
            }

            FieldType::Message(msg_type) => match registry.get_message_descriptor(&msg_type) {
                Ok(sub_msg_desc) => {
                    for field in sub_msg_desc.fields.values() {
                        self.add_field(
                            Some(&field_name),
                            registry,
                            dyn_fields,
                            msg_desc, /* we add field onto main msg mapping so that we can
                                       * `field.field_sub` */
                            field,
                        );
                    }
                    return;
                }
                Err(err) => {
                    error!("Error getting message descriptor for {}: {}", msg_type, err);
                }
            },

            FieldType::Repeated(_) => {
                // not supported
            }
        }

        error!(
            "Invalid index option / type for field {} of message {} or ran out of fields",
            field_desc.name, msg_desc.name,
        );
    }
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

/// Extracts boolean (via u64) value from Tantivy document
pub(crate) fn get_doc_opt_bool_value(doc: &Document, field: Field) -> Option<bool> {
    match doc.get_first(field) {
        Some(tantivy::schema::Value::U64(v)) => Some(*v == 1),
        _ => None,
    }
}

pub(crate) fn bool_to_u64(v: bool) -> u64 {
    u64::from(v)
}
