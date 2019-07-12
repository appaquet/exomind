use crate::error::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub type SchemaTraitId = u16;
pub type SchemaStructId = u16;
pub type SchemaFieldId = u16;

// TODO: To be completed in https://github.com/appaquet/exocore/issues/104

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Schema {
    pub name: String,
    pub traits: Vec<TraitSchema>,
    #[serde(skip)]
    pub traits_id: HashMap<SchemaTraitId, usize>,
    #[serde(skip)]
    pub traits_name: HashMap<String, usize>,
    #[serde(default = "Vec::new")]
    pub structs: Vec<StructSchema>,
    #[serde(skip)]
    pub structs_id: HashMap<SchemaStructId, usize>,
    #[serde(skip)]
    pub structs_name: HashMap<String, usize>,
}

impl Schema {
    pub fn parse(yaml: &str) -> Result<Schema, crate::error::Error> {
        let mut schema: Schema =
            serde_yaml::from_str(yaml).map_err(|err| Error::Schema(err.to_string()))?;

        for (stc_pos, stc) in schema.structs.iter_mut().enumerate() {
            if let Some(_other_struct) = schema.structs_id.insert(stc.id, stc_pos) {
                return Err(Error::Schema(format!(
                    "A struct with id {} already exists in schema",
                    stc.id
                )));
            }

            if let Some(_other_struct) = schema.structs_name.insert(stc.name.clone(), stc_pos) {
                return Err(Error::Schema(format!(
                    "A struct with name {} already exists in schema",
                    stc.name
                )));
            }

            for (field_pos, field) in stc.fields.iter_mut().enumerate() {
                if let Some(_other_field) = stc.fields_id.insert(field.id, field_pos) {
                    return Err(Error::Schema(format!(
                        "Struct {} already had a field with id {}",
                        stc.name, field.id
                    )));
                }

                if let Some(_other_field) = stc.fields_name.insert(field.name.clone(), field_pos) {
                    return Err(Error::Schema(format!(
                        "Struct {} already had a field with name {}",
                        stc.name, field.name
                    )));
                }
            }
        }

        for (trt_pos, trt) in schema.traits.iter_mut().enumerate() {
            let default_fields = TraitSchema::default_fields();
            for field in default_fields {
                trt.fields.push(field);
            }

            if let Some(_other_trait) = schema.traits_id.insert(trt.id, trt_pos) {
                return Err(Error::Schema(format!(
                    "A trait with id {} already exists in schema",
                    trt.id
                )));
            }

            if let Some(_other_trait) = schema.traits_name.insert(trt.name.clone(), trt_pos) {
                return Err(Error::Schema(format!(
                    "A trait with name {} already exists in schema",
                    trt.name
                )));
            }

            for (field_pos, field) in trt.fields.iter_mut().enumerate() {
                if let Some(_other_field) = trt.fields_id.insert(field.id, field_pos) {
                    return Err(Error::Schema(format!(
                        "Trait {} already had a field with id {}",
                        trt.name, field.id
                    )));
                }

                if let Some(_other_field) = trt.fields_name.insert(field.name.clone(), field_pos) {
                    return Err(Error::Schema(format!(
                        "Trait {} already had a field with name {}",
                        trt.name, field.name
                    )));
                }
            }
        }

        Ok(schema)
    }

    pub fn trait_by_id(&self, id: SchemaTraitId) -> Option<&TraitSchema> {
        self.traits_id
            .get(&id)
            .and_then(|pos| self.traits.get(*pos))
    }

    pub fn trait_by_name(&self, name: &str) -> Option<&TraitSchema> {
        self.traits_name
            .get(name)
            .and_then(|pos| self.traits.get(*pos))
    }

    pub fn struct_by_id(&self, id: SchemaStructId) -> Option<&StructSchema> {
        self.structs_id
            .get(&id)
            .and_then(|pos| self.structs.get(*pos))
    }

    pub fn struct_by_name(&self, name: &str) -> Option<&StructSchema> {
        self.structs_name
            .get(name)
            .and_then(|pos| self.structs.get(*pos))
    }
}

pub trait SchemaRecord {
    fn name(&self) -> &str;

    fn field_by_id(&self, id: SchemaFieldId) -> Option<&SchemaField>;
    fn field_by_name(&self, name: &str) -> Option<&SchemaField>;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TraitSchema {
    pub id: u16,
    pub name: String,
    pub fields: Vec<SchemaField>,
    #[serde(skip)]
    pub fields_id: HashMap<SchemaFieldId, usize>,
    #[serde(skip)]
    pub fields_name: HashMap<String, usize>,
}

impl TraitSchema {
    pub const TRAIT_ID_FIELD: &'static str = "_id";
    pub const CREATION_DATE_FIELD: &'static str = "creation_date";
    pub const MODIFICATION_DATE_FIELD: &'static str = "modification_date";

    fn default_fields() -> Vec<SchemaField> {
        vec![
            SchemaField {
                id: 65400,
                name: Self::TRAIT_ID_FIELD.to_owned(),
                typ: FieldType::String,
                indexed: false, // special case, it's indexed & stored in another way
                optional: false,
            },
            SchemaField {
                id: 65401,
                name: Self::CREATION_DATE_FIELD.to_owned(),
                typ: FieldType::Int, // TODO: date
                indexed: true,
                optional: false,
            },
            SchemaField {
                id: 65402,
                name: Self::MODIFICATION_DATE_FIELD.to_owned(),
                typ: FieldType::Int, // TODO: date
                indexed: true,
                optional: false,
            },
        ]
    }
}

impl SchemaRecord for TraitSchema {
    fn name(&self) -> &str {
        &self.name
    }

    fn field_by_id(&self, id: SchemaFieldId) -> Option<&SchemaField> {
        self.fields_id
            .get(&id)
            .and_then(|pos| self.fields.get(*pos))
    }

    fn field_by_name(&self, name: &str) -> Option<&SchemaField> {
        self.fields_name
            .get(name)
            .and_then(|pos| self.fields.get(*pos))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StructSchema {
    pub id: u16,
    pub name: String,
    pub fields: Vec<SchemaField>,
    #[serde(skip)]
    pub fields_id: HashMap<SchemaFieldId, usize>,
    #[serde(skip)]
    pub fields_name: HashMap<String, usize>,
}

impl SchemaRecord for StructSchema {
    fn name(&self) -> &str {
        &self.name
    }

    fn field_by_id(&self, id: SchemaFieldId) -> Option<&SchemaField> {
        self.fields_id
            .get(&id)
            .and_then(|pos| self.fields.get(*pos))
    }

    fn field_by_name(&self, name: &str) -> Option<&SchemaField> {
        self.fields_name
            .get(name)
            .and_then(|pos| self.fields.get(*pos))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SchemaField {
    pub id: u16,
    pub name: String,
    #[serde(default = "default_true")]
    pub optional: bool,
    #[serde(default = "default_false")]
    pub indexed: bool,
    #[serde(rename = "type")]
    pub typ: FieldType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    Int,
    Bool,
    Struct(SchemaStructId),
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn parse_basic() {
        let schema_defaults = Schema::parse(
            r#"
        name: schema2
        traits:
            - id: 0
              name: trait2
              fields:
                - id: 0
                  name: my_field
                  type:
                      struct: 0
        "#,
        )
        .unwrap();
        assert_eq!("schema2", schema_defaults.name);

        let trt = schema_defaults.trait_by_name("trait2").unwrap();
        assert!(trt.field_by_name(TraitSchema::TRAIT_ID_FIELD).is_some());
        assert!(trt
            .field_by_name(TraitSchema::CREATION_DATE_FIELD)
            .is_some());
        assert!(trt
            .field_by_name(TraitSchema::MODIFICATION_DATE_FIELD)
            .is_some());

        assert!(serde_yaml::to_string(&schema_defaults).is_ok());
    }

    pub fn create_test_schema() -> Arc<Schema> {
        Arc::new(
            Schema::parse(
                r#"
        name: myschema
        traits:
            - id: 0
              name: contact
              fields:
                - id: 0
                  name: name
                  type: string
                  indexed: true
                - id: 1
                  name: email
                  type: string
                  indexed: true
            - id: 1
              name: email
              fields:
                - id: 0
                  name: subject
                  type: string
                  indexed: true
                - id: 1
                  name: body
                  type: string
                  indexed: true
        "#,
            )
            .unwrap(),
        )
    }
}
