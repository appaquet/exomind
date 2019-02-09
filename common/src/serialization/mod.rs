use std::io::{Read, Result, Write};

pub mod framed;
pub mod protos;

pub trait Serializable<T> {
    fn serialize(&self, writer: &Write) -> Result<usize>;
    fn deserialize(reader: &Read) -> Result<T>;
}
