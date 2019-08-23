use crate::schema::{SchemaFieldId, SchemaRecordId};

///
/// Schema related error
///
#[derive(Debug, Fail, Clone)]
pub enum Error {
    #[fail(display = "Error parsing schema: {}", _0)]
    Schema(String),

    #[fail(display = "Data integrity error: {}", _0)]
    DataIntegrity(String),

    #[fail(display = "Field id {} of record id {} didn't have a value", _0, _1)]
    FieldEmptyValue(SchemaRecordId, SchemaFieldId),

    #[fail(display = "Record field invalid type error: {}", _0)]
    FieldInvalidType(String),

    #[fail(display = "Field named {} was not in schema", _0)]
    NamedFieldNotInSchema(String),
}
