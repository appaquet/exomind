use std::io::Result;
use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

// TODO: Only if feature
pub mod json_disk_store;

pub trait SimpleStore<T: Serialize + DeserializeOwned> {
    fn read(&self) -> Result<Option<T>>;
    fn write(&self, value: &T) -> Result<()>;
}

pub fn get_disk_store<T: Serialize + DeserializeOwned>(
    _path: &Path,
) -> Result<Box<dyn SimpleStore<T>>> {
    // TODO: If WASM, then get data from somewhere
    unimplemented!()
}
