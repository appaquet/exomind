use std::io::{Read, Result, Write};

pub mod msg;

pub trait Serializable<T> {
    fn serialize(&self, writer: &Write) -> Result<usize>;
    fn deserialize(reader: &Read) -> Result<T>;
}
