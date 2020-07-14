use super::*;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

pub struct JsonDiskStore<T: Serialize + DeserializeOwned> {
    path: PathBuf,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> JsonDiskStore<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<JsonDiskStore<T>> {
        let meta = std::fs::metadata(path.as_ref());
        if meta.map(|m| m.is_dir()).unwrap_or(false) {
            error!("Path {:?} is a directory, not a file.", path.as_ref());
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Path was a directory, not a file",
            ));
        }

        Ok(JsonDiskStore {
            path: path.as_ref().to_path_buf(),
            phantom: std::marker::PhantomData,
        })
    }
}

impl<T: Serialize + DeserializeOwned> SimpleStore<T> for JsonDiskStore<T> {
    fn read(&self) -> Result<Option<T>> {
        let meta = std::fs::metadata(&self.path);
        if meta.is_err() {
            return Ok(None);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&self.path)?;

        serde_json::from_reader(file).map_err(|err| {
            error!("Couldn't decode stored JSON file: {}", err);
            Error::new(ErrorKind::InvalidData, "Invalid json data")
        })
    }

    fn write(&self, value: &T) -> Result<()> {
        let tmp_file = self.path.with_extension("tmp");
        let file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .open(&tmp_file)?;
        serde_json::to_writer(file, value)?;

        std::fs::rename(&tmp_file, &self.path)
    }
}

#[cfg(test)]
mod test {
    use serde_derive::{Deserialize, Serialize};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_not_dir() {
        let dir = tempdir().unwrap();
        let store = JsonDiskStore::<String>::new(dir.path());
        assert!(store.is_err());

        let file = dir.path().join("file");
        let store = JsonDiskStore::<String>::new(&file);
        assert!(store.is_ok());
    }

    #[test]
    fn test_write_read() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("store");
        let store = JsonDiskStore::<TestStruct>::new(&file).unwrap();

        let data = TestStruct {
            data: "hello".to_string(),
        };
        store.write(&data).unwrap();

        let read_data = store.read().unwrap();
        assert_eq!(read_data, Some(data));
    }

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestStruct {
        data: String,
    }
}
