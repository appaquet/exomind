use std::io::Result;

use serde::{de::DeserializeOwned, Serialize};

pub mod json_disk_store;

pub trait SimpleStore<T: Serialize + DeserializeOwned> {
    fn read(&self) -> Result<Option<T>>;
    fn write(&self, value: &T) -> Result<()>;

    fn read_or_default(&self) -> Result<T>
    where
        T: Default,
    {
        self.read().map(|v| v.unwrap_or_default())
    }
}
