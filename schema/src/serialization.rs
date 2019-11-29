use super::entity::{FieldValue, Record, Struct, Trait};
use super::schema::{RecordSchema, Schema};
use crate::entity::{RecordBuilder, StructBuilder, TraitBuilder};
use crate::schema::{FieldSchema, FieldType};
use chrono::prelude::*;
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
        &format!(
            "{}.{}",
            record.namespace().name(),
            record.record_schema().name()
        ),
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
            FieldValue::Bool(v) => v.serialize(serializer),
            FieldValue::Int(v) => v.serialize(serializer),
            FieldValue::DateTime(v) => v.serialize(serializer),
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

        let mut trait_builder =
            TraitBuilder::new_full_name(&schema, &trait_type).map_err(|err| {
                serde::de::Error::custom(format!(
                    "Couldn't create trait from serialized valued: {}",
                    err
                ))
            })?;
        for (field_name, value) in values {
            let remapped_value = if let Some(field_record) =
                trait_builder.record_schema().field_by_name(&field_name)
            {
                maybe_remap_value(field_record, value)
            } else {
                value
            };
            trait_builder = trait_builder.set(&field_name, remapped_value);
        }
        let new_trait = trait_builder.build().map_err(|err| {
            serde::de::Error::custom(format!(
                "Couldn't create trait from serialized valued: {}",
                err
            ))
        })?;

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

    let record_type = if let FieldValue::String(typ) = struct_type {
        typ
    } else {
        return Err(serde::de::Error::custom("Field _type was not a string"));
    };

    Ok(record_type.to_string())
}

fn struct_from_values<'de, D: Deserializer<'de>>(
    mut values: HashMap<String, FieldValue>,
) -> Result<Struct, <D as Deserializer<'de>>::Error> {
    let schema = SCHEMA
        .with(|sc| sc.borrow().clone())
        .ok_or_else(|| serde::de::Error::custom("Schema not set in thread local".to_string()))?;

    let struct_type = extract_record_type(&mut values)?;

    let mut struct_builder =
        StructBuilder::new_full_name(&schema, &struct_type).map_err(|err| {
            serde::de::Error::custom(format!(
                "Couldn't create struct from serialized valued: {}",
                err
            ))
        })?;

    for (field_name, value) in values {
        let remapped_value =
            if let Some(field_record) = struct_builder.record_schema().field_by_name(&field_name) {
                maybe_remap_value(field_record, value)
            } else {
                value
            };
        struct_builder = struct_builder.set(&field_name, remapped_value);
    }

    let new_struct = struct_builder.build().map_err(|err| {
        serde::de::Error::custom(format!(
            "Couldn't create struct from serialized valued: {}",
            err
        ))
    })?;

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

            let mut struct_builder =
                StructBuilder::new_full_name(&schema, &struct_type).map_err(|err| {
                    serde::de::Error::custom(format!(
                        "Couldn't create struct from serialized valued: {}",
                        err
                    ))
                })?;

            for (field_name, value) in values {
                struct_builder = struct_builder.set(&field_name, value);
            }

            let new_struct = struct_builder.build().map_err(|err| {
                serde::de::Error::custom(format!(
                    "Couldn't create struct from serialized valued: {}",
                    err
                ))
            })?;

            Ok(FieldValue::Struct(new_struct))
        } else {
            Ok(FieldValue::Map(values))
        }
    }
}

pub fn maybe_remap_value(field: &FieldSchema, value: FieldValue) -> FieldValue {
    match (&field.typ, value) {
        (FieldType::DateTime, FieldValue::String(date_str)) => {
            match DateTime::parse_from_rfc3339(&date_str) {
                Ok(res) => FieldValue::DateTime(res.with_timezone(&Utc)),
                Err(err) => {
                    warn!("Error parsing string: {}", err);
                    FieldValue::String(date_str)
                }
            }
        }
        (_, value) => value,
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
        let schema = crate::test_schema::create();

        let now = std::time::SystemTime::now();
        let chrono_now = DateTime::<Utc>::from(now);

        let strt = StructBuilder::new(&schema, "exocore", "struct1")?.build()?;

        let value = StructBuilder::new(&schema, "exocore", "struct1")?
            .set("string_field", "string_value")
            .set("int_field", 1234)
            .set("date_field", chrono_now)
            .set("struct_field", strt.clone())
            .build()?;

        let value_ser = serde_json::to_string(&value)?;
        let value_deser = with_schema(&schema, || serde_json::from_str::<Struct>(&value_ser))?;

        assert_eq!(value_deser.get_string("string_field")?, "string_value");
        assert_eq!(value_deser.get_int("int_field")?, 1234,);
        assert_eq!(value_deser.get_datetime("date_field")?, &chrono_now);
        assert_eq!(value_deser.get_struct("struct_field")?, &strt);
        assert_eq!(value, value_deser);

        Ok(())
    }

    #[test]
    fn serialize_deserialize_trait() -> Result<(), failure::Error> {
        let schema = crate::test_schema::create();

        let now = std::time::SystemTime::now();
        let chrono_now = DateTime::<Utc>::from(now);

        let strt = StructBuilder::new(&schema, "exocore", "struct1")?.build()?;

        let trt = TraitBuilder::new(&schema, "exocore", "trait1")?
            .set("string_field", "string_value")
            .set("int_field", 1234)
            .set("date_field", chrono_now)
            .set("struct_field", strt.clone())
            .build()?;

        let value_ser = serde_json::to_string(&trt)?;
        let value_deser = with_schema(&schema, || serde_json::from_str::<Trait>(&value_ser))?;

        assert_eq!(value_deser.get_string("string_field")?, "string_value");
        assert_eq!(value_deser.get_int("int_field")?, 1234,);
        assert_eq!(value_deser.get_datetime("date_field")?, &chrono_now);
        assert_eq!(value_deser.get_struct("struct_field")?, &strt);
        assert_eq!(trt, value_deser);

        Ok(())
    }
}
