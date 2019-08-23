use super::schema::RecordSchema;
use crate::error::Error;
use crate::schema::{
    FieldSchema, Namespace, Schema, SchemaFieldId, StructSchema, TraitIdValue, TraitSchema,
};
use chrono::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use uuid::Uuid;

pub type EntityId = String;
pub type EntityIdRef = str;
pub type TraitId = String;
pub type TraitIdRef = str;

///
/// An entity is an object on which traits can be added to shape what it represents.
///   Ex: an email is an entity on which we added a trait "Email"
///       a note could be added to an email by adding a "Note" trait also
///
/// Traits can also represent relationship.
///   Ex: a "Child" trait can be used to add an entity into a collection
///
#[derive(Serialize, Deserialize, Debug)]
pub struct Entity {
    pub id: EntityId,
    pub traits: Vec<Trait>,
}

impl Entity {
    pub fn new<I: Into<EntityId>>(id: I) -> Entity {
        Entity {
            id: id.into(),
            traits: Vec::new(),
        }
    }

    pub fn with_trait(mut self, trt: Trait) -> Self {
        self.traits.push(trt);
        self
    }
}

///
/// Common trait between `Struct` and `Trait`, since both are a collection of fields.
///
pub trait Record: Sized {
    type SchemaType: RecordSchema;

    fn schema(&self) -> &Arc<Schema>;

    fn namespace(&self) -> &Arc<Namespace>;

    fn record_schema(&self) -> &Arc<Self::SchemaType>;

    fn full_name(&self) -> String {
        format!(
            "{}.{}",
            self.namespace().name(),
            self.record_schema().name()
        )
    }

    fn values(&self) -> &HashMap<SchemaFieldId, FieldValue>;

    fn value(&self, field: &FieldSchema) -> Option<&FieldValue> {
        self.values().get(&field.id)
    }

    fn get<'s, T: TryFrom<&'s FieldValue, Error = Error>>(
        &'s self,
        field_name: &str,
    ) -> Result<T, Error> {
        let schema = self.record_schema();
        let field = schema
            .field_by_name(field_name)
            .ok_or_else(|| Error::NamedFieldNotInSchema(field_name.to_owned()))?;

        let value = self
            .values()
            .get(&field.id)
            .ok_or_else(|| Error::FieldEmptyValue(field.id, schema.id()))?;

        T::try_from(value)
    }

    fn get_string(&self, field_name: &str) -> Result<&str, Error> {
        self.get(field_name)
    }

    fn get_int(&self, field_name: &str) -> Result<i64, Error> {
        self.get(field_name)
    }

    fn get_bool(&self, field_name: &str) -> Result<bool, Error> {
        self.get(field_name)
    }

    fn get_datetime(&self, field_name: &str) -> Result<&DateTime<Utc>, Error> {
        self.get(field_name)
    }

    fn get_struct(&self, field_name: &str) -> Result<&Struct, Error> {
        self.get(field_name)
    }

    fn debug_fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let record_schema = self.record_schema();
        let name = self.full_name();
        let mut str_fmt = f.debug_struct(&name);
        for (field_id, value) in self.values() {
            let field = record_schema
                .field_by_id(*field_id)
                .map(|f| f.name.to_string())
                .unwrap_or_else(|| format!("_{}", field_id));
            str_fmt.field(&field, value);
        }
        str_fmt.finish()
    }
}

///
/// Record builder implemented by both `TraitBuilder` and `StructBuilder`
///
pub trait RecordBuilder: Sized {
    type SchemaType: RecordSchema;

    fn schema(&self) -> &Arc<Schema>;

    fn namespace(&self) -> &Arc<Namespace>;

    fn record_schema(&self) -> &Arc<Self::SchemaType>;

    fn full_name(&self) -> String {
        format!(
            "{}.{}",
            self.namespace().name(),
            self.record_schema().name()
        )
    }

    fn values(&self) -> &HashMap<SchemaFieldId, FieldValue>;

    fn values_mut(&mut self) -> &mut HashMap<SchemaFieldId, FieldValue>;

    fn value(&self, field: &FieldSchema) -> Option<&FieldValue> {
        self.values().get(&field.id)
    }

    fn set<V: Into<FieldValue>>(self, field_name: &str, value: V) -> Self {
        if let Some(field_id) = self.record_schema().field_by_name(field_name).map(|f| f.id) {
            self.set_by_id(field_id, value.into())
        } else {
            self
        }
    }

    fn set_by_id<V: Into<FieldValue>>(mut self, field_id: SchemaFieldId, value: V) -> Self {
        self.values_mut().insert(field_id, value.into());
        self
    }
}

fn check_and_default_record_values<S: RecordSchema>(
    schema: &Arc<S>,
    values: &mut HashMap<SchemaFieldId, FieldValue>,
) -> Result<(), Error> {
    for field in schema.fields() {
        let value = values.get(&field.id);

        match value {
            Some(val) => {
                field.validate_value_type(val)?;
            }
            None => match field.default_value() {
                Some(value) => {
                    values.insert(field.id, value);
                }
                None if !field.optional => {
                    return Err(Error::FieldEmptyValue(field.id, schema.id()));
                }
                _ => {}
            },
        }
    }

    Ok(())
}

///
/// Trait that can be added to an entity, shaping its representation.
///
#[derive(Clone)]
pub struct Trait {
    schema: Arc<Schema>,
    namespace: Arc<Namespace>,
    trait_schema: Arc<TraitSchema>,
    values: HashMap<SchemaFieldId, FieldValue>,
}

impl Trait {
    pub fn id(&self) -> &TraitId {
        let value = self.values.get(&TraitSchema::TRAIT_ID_FIELD);
        match value {
            Some(FieldValue::String(str)) => str,
            _ => panic!("Trait didn't have a valid ID value"),
        }
    }
}

impl Record for Trait {
    type SchemaType = TraitSchema;

    fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    fn namespace(&self) -> &Arc<Namespace> {
        &self.namespace
    }

    fn record_schema(&self) -> &Arc<Self::SchemaType> {
        &self.trait_schema
    }

    fn values(&self) -> &HashMap<SchemaFieldId, FieldValue> {
        &self.values
    }
}

impl std::fmt::Debug for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.debug_fmt(f)
    }
}

impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        if self.namespace.name() != other.namespace.name() {
            return false;
        }

        if self.trait_schema.id() != other.trait_schema.id() {
            return false;
        }

        self.values == other.values
    }
}

///
/// Trait builder
///
pub struct TraitBuilder {
    schema: Arc<Schema>,
    namespace: Arc<Namespace>,
    trait_schema: Arc<TraitSchema>,
    values: HashMap<SchemaFieldId, FieldValue>,
}

impl TraitBuilder {
    pub fn new<N: AsRef<str>, T: AsRef<str>>(
        schema: &Arc<Schema>,
        namespace_name: N,
        trait_name: T,
    ) -> Result<TraitBuilder, Error> {
        let namespace = schema
            .namespace_by_name(namespace_name.as_ref())
            .ok_or_else(|| {
                Error::Schema(format!(
                    "Couldn't find namespace with name '{}'",
                    namespace_name.as_ref()
                ))
            })?
            .clone();

        let trait_schema = namespace
            .trait_by_name(trait_name.as_ref())
            .ok_or_else(|| {
                Error::Schema(format!(
                    "Couldn't find trait with name '{}' in namespace '{}'",
                    trait_name.as_ref(),
                    namespace_name.as_ref()
                ))
            })?
            .clone();

        Ok(TraitBuilder {
            schema: schema.clone(),
            namespace,
            trait_schema,
            values: HashMap::new(),
        })
    }

    pub fn new_full_name<S: AsRef<str>>(
        schema: &Arc<Schema>,
        full_trait_name: S,
    ) -> Result<TraitBuilder, Error> {
        let (ns_name, trait_name) = super::schema::parse_record_full_name(full_trait_name.as_ref())
            .ok_or_else(|| {
                Error::Schema(format!(
                    "Couldn't parse record full trait name '{}'",
                    full_trait_name.as_ref()
                ))
            })?;

        Self::new(schema, ns_name, trait_name)
    }

    pub fn set_id<S: Into<String>>(mut self, value: S) -> Self {
        self.values.insert(
            TraitSchema::TRAIT_ID_FIELD,
            FieldValue::String(value.into()),
        );
        self
    }

    pub fn build(mut self) -> Result<Trait, Error> {
        let trait_id = self.generate_id()?;

        self.values
            .insert(TraitSchema::TRAIT_ID_FIELD, trait_id.clone().into());

        check_and_default_record_values(&self.trait_schema, &mut self.values)?;

        Ok(Trait {
            schema: self.schema,
            namespace: self.namespace,
            trait_schema: self.trait_schema,
            values: self.values,
        })
    }

    fn generate_id(&self) -> Result<String, Error> {
        let current_id_value =
            self.values
                .get(&TraitSchema::TRAIT_ID_FIELD)
                .and_then(|fv| match fv {
                    FieldValue::String(s) => Some(s.clone()),
                    _ => None,
                });
        if let Some(current_id_value) = current_id_value {
            return Ok(current_id_value);
        }

        match self.trait_schema.id_field() {
            TraitIdValue::Specified => {
                Err(Error::DataIntegrity(format!(
                    "Trait with schema_trait_id={} didn't have a valid ID, but should have been specified",
                    self.trait_schema.id()
                )))
            }
            TraitIdValue::Generated => {
                match current_id_value {
                    Some(id) => Ok(id),
                    None =>
                        Ok(Uuid::new_v4().to_string()),
                }
            }
            TraitIdValue::Static(id) => {
                Ok(id.clone())
            }
            TraitIdValue::Field(id) => self.generate_id_from_field(*id),
            TraitIdValue::Fields(ids) => {
                Ok(ids.iter()
                    .map(|id| self.generate_id_from_field(*id))
                    .collect::<Result<Vec<String>, Error>>()?
                    .join("_"))
            }
        }
    }

    fn generate_id_from_field(&self, id: u16) -> Result<String, Error> {
        let value = self.values.get(&id);
        if let Some(id_value) = value.and_then(|v| v.to_id_string()) {
            Ok(id_value)
        } else {
            Err(Error::DataIntegrity(format!(
                "Trait with schema_trait_id={} didn't have a valid value for id with field {}: value={:?}",
                self.trait_schema.id(), id, value,
            )))
        }
    }
}

impl RecordBuilder for TraitBuilder {
    type SchemaType = TraitSchema;

    fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    fn namespace(&self) -> &Arc<Namespace> {
        &self.namespace
    }

    fn record_schema(&self) -> &Arc<Self::SchemaType> {
        &self.trait_schema
    }

    fn values(&self) -> &HashMap<SchemaFieldId, FieldValue> {
        &self.values
    }

    fn values_mut(&mut self) -> &mut HashMap<SchemaFieldId, FieldValue> {
        &mut self.values
    }
}

///
///
///
pub struct StructBuilder {
    schema: Arc<Schema>,
    namespace: Arc<Namespace>,
    struct_schema: Arc<StructSchema>,
    values: HashMap<SchemaFieldId, FieldValue>,
}

impl StructBuilder {
    pub fn new<N: AsRef<str>, T: AsRef<str>>(
        schema: &Arc<Schema>,
        namespace_name: N,
        struct_name: T,
    ) -> Result<StructBuilder, Error> {
        let namespace = schema
            .namespace_by_name(namespace_name.as_ref())
            .ok_or_else(|| {
                Error::Schema(format!(
                    "Couldn't find namespace with name '{}'",
                    namespace_name.as_ref()
                ))
            })?
            .clone();

        let struct_schema = namespace
            .struct_by_name(struct_name.as_ref())
            .ok_or_else(|| {
                Error::Schema(format!(
                    "Couldn't find struct with name '{}' in namespace '{}'",
                    struct_name.as_ref(),
                    namespace_name.as_ref()
                ))
            })?
            .clone();

        Ok(StructBuilder {
            schema: schema.clone(),
            namespace,
            struct_schema,
            values: HashMap::new(),
        })
    }

    pub fn new_full_name<S: AsRef<str>>(
        schema: &Arc<Schema>,
        full_struct_name: S,
    ) -> Result<StructBuilder, Error> {
        let (ns_name, struct_name) =
            super::schema::parse_record_full_name(full_struct_name.as_ref()).ok_or_else(|| {
                Error::Schema(format!(
                    "Couldn't parse record full struct name '{}'",
                    full_struct_name.as_ref()
                ))
            })?;

        Self::new(schema, ns_name, struct_name)
    }

    pub fn build(mut self) -> Result<Struct, Error> {
        check_and_default_record_values(&self.struct_schema, &mut self.values)?;

        Ok(Struct {
            schema: self.schema,
            namespace: self.namespace,
            struct_schema: self.struct_schema,
            values: self.values,
        })
    }
}

impl RecordBuilder for StructBuilder {
    type SchemaType = StructSchema;

    fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    fn namespace(&self) -> &Arc<Namespace> {
        &self.namespace
    }

    fn record_schema(&self) -> &Arc<Self::SchemaType> {
        &self.struct_schema
    }

    fn values(&self) -> &HashMap<SchemaFieldId, FieldValue> {
        &self.values
    }

    fn values_mut(&mut self) -> &mut HashMap<SchemaFieldId, FieldValue> {
        &mut self.values
    }
}

///
/// Structure with field-value pairs that can be used as a value of any field in a `Record`
///
#[derive(Clone)]
pub struct Struct {
    schema: Arc<Schema>,
    namespace: Arc<Namespace>,
    struct_schema: Arc<StructSchema>,
    values: HashMap<SchemaFieldId, FieldValue>,
}

impl Record for Struct {
    type SchemaType = StructSchema;

    fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    fn namespace(&self) -> &Arc<Namespace> {
        &self.namespace
    }

    fn record_schema(&self) -> &Arc<Self::SchemaType> {
        &self.struct_schema
    }

    fn values(&self) -> &HashMap<SchemaFieldId, FieldValue> {
        &self.values
    }
}

impl std::fmt::Debug for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.debug_fmt(f)
    }
}

impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        if self.namespace.name() != other.namespace.name() {
            return false;
        }

        if self.struct_schema.id() != other.struct_schema.id() {
            return false;
        }

        self.values == other.values
    }
}

///
/// Value of a field of a record
///
#[derive(PartialEq, Debug, Clone)]
pub enum FieldValue {
    String(String),
    Int(i64),
    Bool(bool),
    DateTime(DateTime<Utc>),
    Struct(Struct),
    Map(HashMap<String, FieldValue>),
}

impl FieldValue {
    fn to_id_string(&self) -> Option<String> {
        match self {
            FieldValue::String(v) => Some(v.to_owned()),
            FieldValue::Int(v) => Some(format!("{}", v)),
            _ => None,
        }
    }
}

impl From<&str> for FieldValue {
    fn from(v: &str) -> FieldValue {
        FieldValue::String(v.to_string())
    }
}

impl From<String> for FieldValue {
    fn from(v: String) -> FieldValue {
        FieldValue::String(v)
    }
}

impl From<i64> for FieldValue {
    fn from(v: i64) -> FieldValue {
        FieldValue::Int(v)
    }
}

impl From<DateTime<Utc>> for FieldValue {
    fn from(v: DateTime<Utc>) -> FieldValue {
        FieldValue::DateTime(v)
    }
}

impl From<Struct> for FieldValue {
    fn from(v: Struct) -> FieldValue {
        FieldValue::Struct(v)
    }
}

impl<'s> TryFrom<&'s FieldValue> for &'s str {
    type Error = Error;

    fn try_from(value: &'s FieldValue) -> Result<Self, Error> {
        match value {
            FieldValue::String(value) => Ok(value),
            other => Err(Error::FieldInvalidType(format!(
                "Field was not a string, but was '{:?}'",
                other
            ))),
        }
    }
}

impl TryFrom<&FieldValue> for i64 {
    type Error = Error;

    fn try_from(value: &FieldValue) -> Result<Self, Error> {
        match value {
            FieldValue::Int(value) => Ok(*value),
            other => Err(Error::FieldInvalidType(format!(
                "Field was not an int, but was '{:?}'",
                other
            ))),
        }
    }
}

impl TryFrom<&FieldValue> for bool {
    type Error = Error;

    fn try_from(value: &FieldValue) -> Result<Self, Error> {
        match value {
            FieldValue::Bool(value) => Ok(*value),
            other => Err(Error::FieldInvalidType(format!(
                "Field was not an bool, but was '{:?}'",
                other
            ))),
        }
    }
}

impl<'s> TryFrom<&'s FieldValue> for &'s DateTime<Utc> {
    type Error = Error;

    fn try_from(value: &'s FieldValue) -> Result<Self, Error> {
        match value {
            FieldValue::DateTime(value) => Ok(value),
            other => Err(Error::FieldInvalidType(format!(
                "Field was not a DateTime, but was '{:?}'",
                other
            ))),
        }
    }
}

impl<'s> TryFrom<&'s FieldValue> for &'s Struct {
    type Error = Error;

    fn try_from(value: &'s FieldValue) -> Result<Self, Error> {
        match value {
            FieldValue::Struct(value) => Ok(value),
            other => Err(Error::FieldInvalidType(format!(
                "Field was not a struct, but was '{:?}'",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_utils::create_test_schema;
    use failure::_core::time::Duration;
    use std::time::SystemTime;

    #[test]
    fn string_field_value() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        let collection = TraitBuilder::new(&schema, "exocore", "collection")?
            .set("name", "some collection")
            .build()?;
        assert_eq!(collection.get_string("name")?, "some collection");
        assert_eq!(collection.get::<&str>("name")?, "some collection");

        let email_contact = StructBuilder::new(&schema.clone(), "exocore", "email_contact")?
            .set("name", "Some Name")
            .build()?;
        assert_eq!(email_contact.get::<&str>("name")?, "Some Name");
        assert_eq!(email_contact.get_string("name")?, "Some Name");

        Ok(())
    }

    #[test]
    fn struct_field_value() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        let email_from = StructBuilder::new(&schema, "exocore", "email_contact")?
            .set("name", "Some Name")
            .build()?;

        let email = TraitBuilder::new(&schema, "exocore", "email")?
            .set_id("email_id")
            .set("name", "some collection")
            .set("from", email_from)
            .build()?;

        assert_eq!(email.get_struct("from")?.get_string("name")?, "Some Name");
        assert_eq!(
            email.get::<&Struct>("from")?.get_string("name")?,
            "Some Name"
        );

        Ok(())
    }

    #[test]
    fn int_field_value() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        let annot = TraitBuilder::new(&schema, "exocore", "annotation")?
            .set("count", 1234)
            .build()?;

        assert_eq!(annot.get_int("count")?, 1234);
        assert_eq!(annot.get::<i64>("count")?, 1234);

        Ok(())
    }

    #[test]
    fn trait_id_generation() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        // email has specified id
        let email_res = TraitBuilder::new(&schema, "exocore", "email")?
            .set("subject", "Some title")
            .set("body", "Some body")
            .build();
        assert!(email_res.is_err());

        let email = TraitBuilder::new(&schema, "exocore", "email")?
            .set_id("email_id")
            .set("subject", "Some title")
            .set("body", "Some body")
            .build()?;
        assert_eq!(email.id(), "email_id");

        // annotation has generated id
        let annot = TraitBuilder::new(&schema, "exocore", "annotation")?
            .set("count", 1234)
            .build()?;
        assert!(annot.id().len() > 10);

        // contact id is based on another field
        let contact = TraitBuilder::new(&schema, "exocore", "contact")?
            .set("id", "some_id")
            .build()?;
        assert_eq!(contact.id(), "some_id");

        let contact_res = TraitBuilder::new(&schema, "exocore", "contact")?.build();
        assert!(contact_res.is_err());

        // contact id is based on multiple fields
        let comb = TraitBuilder::new(&schema, "exocore", "combined_id")?
            .set("id1", "abc")
            .set("id2", "dfe")
            .build()?;
        assert_eq!(comb.id(), "abc_dfe");

        let comb_res = TraitBuilder::new(&schema, "exocore", "combined_id")?.build();
        assert!(comb_res.is_err());

        // collection has static id
        let collection = TraitBuilder::new(&schema, "exocore", "collection")?.build()?;
        assert_eq!(collection.id(), "collection_id");

        Ok(())
    }

    #[test]
    fn entity_build() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        let entity = Entity::new("entity_id").with_trait(
            TraitBuilder::new(&schema, "exocore", "email")?
                .set_id("email_id")
                .set("subject", "Some title")
                .set("body", "Some body")
                .build()?,
        );

        assert_eq!(entity.traits.len(), 1);

        Ok(())
    }

    #[test]
    fn field_default_value() -> Result<(), failure::Error> {
        let schema = Arc::new(Schema::parse(
            r#"
        namespaces:
          - name: test
            traits:
              - id: 0
                name: test
                id_field: generated
                fields:
                  - id: 0
                    name: str_value
                    type: string
                    default: hello world
                  - id: 1
                    name: int_value
                    type: int
                    default: 1234
                  - id: 2
                    name: bool_value
                    type: bool
                    default: true
                  - id: 3
                    name: date_value
                    type: date_time
                    default: now
        "#,
        )?);

        let trt = TraitBuilder::new(&schema, "test", "test")?.build()?;

        assert_eq!(trt.get_string("str_value")?, "hello world");
        assert_eq!(trt.get_int("int_value")?, 1234);
        assert_eq!(trt.get_bool("bool_value")?, true);

        let date_time = SystemTime::from(*trt.get_datetime("date_value")?);
        assert!(date_time.elapsed()? <= Duration::from_millis(100));

        Ok(())
    }
}
