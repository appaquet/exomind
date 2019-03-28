pub use capnp;

use std::io::{Read, Result, Write};

pub mod framed;
pub mod protos;

pub trait Serializable<T> {
    fn serialize(&self, writer: &dyn Write) -> Result<usize>;
    fn deserialize(reader: &dyn Read) -> Result<T>;
}
