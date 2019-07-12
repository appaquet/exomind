use super::entity::{FieldValue, Record, Struct, Trait};
use super::schema::{Schema, SchemaRecord};
use serde::de::{MapAccess, Visitor};
use serde::export::Formatter;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    // used to pass current schema down to the deserializer
    #[allow(unused)]
    static SCHEMA: RefCell<Option<Arc<Schema>>> = RefCell::new(None);
}

///
/// Record serialization
///
impl Serialize for Trait {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serialize_record(self, serializer)
    }
}

impl Serialize for Struct {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serialize_record(self, serializer)
    }
}

fn serialize_record<R: Record, S>(
    record: &R,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
{
    let record_schema = record.record_schema();
    let mut ser = serializer.serialize_map(Some(record.values().len() + 1))?;
    ser.serialize_entry(
        &"_type",
        &format!("{}.{}", record.schema().name, record.record_schema().name()),
    )?;
    for (field_id, field_value) in record.values().iter() {
        if let Some(field) = record_schema.field_by_id(*field_id) {
            ser.serialize_entry(&field.name, field_value)?;
        }
    }
    ser.end()
}

///
/// Field value serialization
///
impl Serialize for FieldValue {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            FieldValue::String(v) => v.serialize(serializer),
            FieldValue::Struct(v) => v.serialize(serializer),
            FieldValue::Int(v) => v.serialize(serializer),
            FieldValue::Map(v) => v.serialize(serializer),
        }
    }
}

///
/// Trait deserialization
///
impl<'de> Deserialize<'de> for Trait {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let mut values = deserializer.deserialize_map(MapVisitor)?;
        let schema = SCHEMA.with(|sc| sc.borrow().clone()).ok_or_else(|| {
            serde::de::Error::custom("Schema not set in thread local".to_string())
        })?;

        let trait_type = extract_record_type(&mut values)?;
        let mut new_trait = Trait::new(schema, &trait_type);
        for (field_name, value) in values {
            new_trait = new_trait.with_value_by_name(&field_name, value);
        }

        Ok(new_trait)
    }
}

struct MapVisitor;
impl<'de> Visitor<'de> for MapVisitor {
    type Value = HashMap<String, FieldValue>;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter.write_str("Trait deserializer")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = HashMap::new();

        while let Some((key, value)) = map_access.next_entry::<String, FieldValue>()? {
            values.insert(key, value);
        }

        Ok(values)
    }
}

///
/// Struct deserialization
///
impl<'de> Deserialize<'de> for Struct {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let values = deserializer.deserialize_map(MapVisitor)?;
        struct_from_values::<D>(values)
    }
}

fn extract_record_type<E: serde::de::Error>(
    values: &mut HashMap<String, FieldValue>,
) -> Result<String, E> {
    let struct_type = values
        .remove("_type")
        .ok_or_else(|| serde::de::Error::custom("No _type field found"))?;

    let struct_type = if let FieldValue::String(typ) = struct_type {
        typ
    } else {
        return Err(serde::de::Error::custom("Field _type was not a string"));
    };

    let struct_type = struct_type.split('.').nth(1).ok_or_else(|| {
        serde::de::Error::custom(format!(
            "Invalid _type field in JSON for struct: {}",
            struct_type
        ))
    })?;

    Ok(struct_type.to_string())
}

fn struct_from_values<'de, D: Deserializer<'de>>(
    mut values: HashMap<String, FieldValue>,
) -> Result<Struct, <D as Deserializer<'de>>::Error> {
    let schema = SCHEMA
        .with(|sc| sc.borrow().clone())
        .ok_or_else(|| serde::de::Error::custom("Schema not set in thread local".to_string()))?;

    let struct_type = extract_record_type(&mut values)?;
    let mut new_struct = Struct::new(schema, &struct_type);
    for (field_name, value) in values {
        new_struct = new_struct.with_value_by_name(&field_name, value);
    }

    Ok(new_struct)
}

///
/// Field value deserialization
///
impl<'de> Deserialize<'de> for FieldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FieldValueVisitor)
    }
}

struct FieldValueVisitor;
impl<'de> Visitor<'de> for FieldValueVisitor {
    type Value = FieldValue;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter.write_str("FieldValue deserializer")
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(i64::from(v)))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(i64::from(v)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(i64::from(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(i64::from(v)))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(i64::from(v)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(i64::from(v)))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::Int(v as i64))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FieldValue::String(v.to_string()))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = MapVisitor.visit_map(map)?;
        if values.contains_key("_type") {
            let schema = SCHEMA.with(|sc| sc.borrow().clone()).ok_or_else(|| {
                serde::de::Error::custom("Schema not set in thread local".to_string())
            })?;

            let struct_type = extract_record_type(&mut values)?;
            let mut new_struct = Struct::new(schema, &struct_type);
            for (field_name, value) in values {
                new_struct = new_struct.with_value_by_name(&field_name, value);
            }

            Ok(FieldValue::Struct(new_struct))
        } else {
            Ok(FieldValue::Map(values))
        }
    }
}

pub fn with_schema<F, R>(schema: &Arc<Schema>, fun: F) -> R
where
    F: Fn() -> R,
{
    SCHEMA.with(|sc| {
        *sc.borrow_mut() = Some(schema.clone());
    });

    fun()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_deserialize_primitive_field_value() -> Result<(), failure::Error> {
        let value = FieldValue::String("hello world".to_string());
        let value_ser = serde_json::to_string(&value)?;
        let value_deser = serde_json::from_str(&value_ser)?;
        assert_eq!(value, value_deser);

        let value = FieldValue::Int(4012);
        let value_ser = serde_json::to_string(&value)?;
        let value_deser = serde_json::from_str(&value_ser)?;
        assert_eq!(value, value_deser);

        let value = FieldValue::Map(hashmap!(
            "key".to_string() => "value".into()
        ));
        let value_ser = serde_json::to_string(&value)?;
        let value_deser = serde_json::from_str(&value_ser)?;
        assert_eq!(value, value_deser);

        Ok(())
    }

    #[test]
    fn serialize_deserialize_struct() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        let value = Struct::new(schema.clone(), "struct1").with_value_by_name("field1", 1234);
        let value_ser = serde_json::to_string(&value)?;
        let value_deser = with_schema(&schema, || serde_json::from_str::<Struct>(&value_ser))?;
        assert_eq!(
            FieldValue::Int(1234),
            *value_deser.value_by_name("field1").unwrap()
        );
        assert_eq!(value, value_deser);

        Ok(())
    }

    #[test]
    fn serialize_deserialize_trait() -> Result<(), failure::Error> {
        let schema = create_test_schema();

        let value = Trait::new(schema.clone(), "trait1")
            .with_value_by_name("field1", "hey you")
            .with_value_by_name("field2", Struct::new(schema.clone(), "struct1"));
        let value_ser = serde_json::to_string(&value)?;
        let value_deser = with_schema(&schema, || serde_json::from_str::<Trait>(&value_ser))?;
        assert_eq!(
            FieldValue::String("hey you".to_string()),
            *value_deser.value_by_name("field1").unwrap()
        );
        assert_eq!(value, value_deser);

        Ok(())
    }

    fn create_test_schema() -> Arc<Schema> {
        Arc::new(
            Schema::parse(
                r#"
        name: schema1
        traits:
            - id: 0
              name: trait1
              fields:
                - id: 0
                  name: field1
                  type: string
                - id: 1
                  name: field2
                  type:
                      struct: 0
        structs:
            - id: 0
              name: struct1
              fields:
                - id: 0
                  name: field1
                  type: int 
        "#,
            )
            .unwrap(),
        )
    }
}
