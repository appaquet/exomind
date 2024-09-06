use std::{
    io::{Read, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use super::{
    ram::{RamFile, RamFileData},
    Directory, DynDirectory, Error, FileRead, FileStat, FileWrite,
};

const DIR_PREFIX: &str = "_dir";

pub struct WebDirectory {
    storage: Arc<Mutex<Storage>>,
}

impl WebDirectory {
    pub fn new(storage: web_sys::Storage) -> Self {
        Self {
            storage: Arc::new(Mutex::new(Storage(storage))),
        }
    }
}

impl Directory for WebDirectory {
    fn open_read(&self, path: &Path) -> Result<Box<dyn FileRead>, Error> {
        let key = path_to_key(path, true)?;
        let file = WebFile::new(self.storage.clone(), key, false, true)?;
        Ok(Box::new(file))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn FileWrite>, Error> {
        let key = path_to_key(path, true)?;
        let file = WebFile::new(self.storage.clone(), key, false, false)?;
        Ok(Box::new(file))
    }

    fn open_create(&self, path: &Path) -> Result<Box<dyn FileWrite>, Error> {
        let key = path_to_key(path, true)?;
        let file = WebFile::new(self.storage.clone(), key, true, false)?;
        Ok(Box::new(file))
    }

    fn list(&self, prefix: Option<&Path>) -> Result<Vec<Box<dyn FileStat>>, Error> {
        let prefix = path_to_key(prefix.unwrap_or_else(|| Path::new("")), false)?;

        let storage = self.storage.lock().unwrap();

        let mut files: Vec<Box<dyn FileStat>> = Vec::new();
        let nb_items = storage.length().unwrap_or_default();
        for i in 0..nb_items {
            let key = storage
                .key(i)
                .map_err(|err| anyhow!("failed to get storage item {}: {:?}", i, err))?;
            let Some(key) = key else { continue };

            if key.starts_with(&prefix) {
                let file = WebFileStat::new(&storage, key)?;
                files.push(Box::new(file));
            }
        }

        Ok(files)
    }

    fn stat(&self, path: &Path) -> Result<Box<dyn FileStat>, Error> {
        let key = path_to_key(path, true)?;
        let storage = self.storage.lock().unwrap();
        let stat = WebFileStat::new(&storage, key)?;
        Ok(Box::new(stat))
    }

    fn exists(&self, path: &Path) -> bool {
        let Ok(key) = path_to_key(path, false) else {
            return false;
        };

        let storage = self.storage.lock().unwrap();
        storage.get(&key).unwrap_or_default().is_some()
    }

    fn delete(&self, path: &Path) -> Result<(), Error> {
        let key = path_to_key(path, true)?;
        let storage = self.storage.lock().unwrap();
        storage
            .delete(&key)
            .map_err(|err| anyhow!("failed to delete file from storage: {:?}", err))?;
        Ok(())
    }

    fn clone(&self) -> DynDirectory {
        WebDirectory {
            storage: self.storage.clone(),
        }
        .into()
    }

    fn as_os_path(&self) -> Result<PathBuf, Error> {
        Err(Error::NotOsDirectory)
    }
}

// Represents a file stored in web local storage.
// On first access, the file is loaded into memory and flushed to the web
// storage on every flushed write.
pub struct WebFile {
    storage: Arc<Mutex<Storage>>,
    key: String,
    readonly: bool,
    ram_data: RamFileData,
    ram_file: RamFile,
}

impl WebFile {
    fn new(
        storage: Arc<Mutex<Storage>>,
        key: String,
        new: bool,
        readonly: bool,
    ) -> Result<Self, Error> {
        // load data into a ram file
        let data = {
            let storage = storage.lock().unwrap();
            let data = if !new { storage.get_bytes(&key) } else { None };

            if let Some(data) = data {
                data
            } else {
                storage.set_bytes(&key, Vec::new())?;
                Vec::new()
            }
        };

        let ram_data = RamFileData::from(data);
        Ok(Self {
            storage,
            key,
            readonly,
            ram_data: ram_data.clone(),
            ram_file: RamFile::new(ram_data),
        })
    }
}

impl FileRead for WebFile {}
impl FileWrite for WebFile {}

impl Read for WebFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.ram_file.read(buf)
    }
}

impl Seek for WebFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.ram_file.seek(pos)
    }
}

impl Write for WebFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.readonly {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "file is readonly",
            ));
        }

        let written = self.ram_file.write(buf)?;

        let data = self.ram_data.bytes.read().unwrap();
        let storage = self.storage.lock().unwrap();
        storage.set_bytes(&self.key, data.clone())?;

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct WebFileStat {
    path: PathBuf,
    size: u64,
}

impl WebFileStat {
    fn new(storage: &Storage, key: String) -> Result<Self, Error> {
        let path = key_to_path(&key);
        let size = storage
            .get_bytes(&key)
            .ok_or_else(|| Error::NotFound(path.clone()))?
            .len() as u64;

        Ok(Self { size, path })
    }
}

impl FileStat for WebFileStat {
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn size(&self) -> u64 {
        self.size
    }
}

// Wraps a web_sys::Storage to make it send. This is needed because it contains
// a inner pointer which is not Send. In WASM, there is no multi-threading yet,
// so we can assume that it's safe to send this across threads.
// See https://github.com/rustwasm/wasm-bindgen/issues/1505
struct Storage(web_sys::Storage);

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Storage {}

impl Deref for Storage {
    type Target = web_sys::Storage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Storage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Storage {
    fn get_bytes(&self, key: &str) -> Option<Vec<u8>> {
        let data = self.0.get(key).unwrap_or_default()?;
        Some(data.into_bytes())
    }

    fn set_bytes(&self, key: &str, data: Vec<u8>) -> std::io::Result<()> {
        let data_str = String::from_utf8(data).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("couldn't convert data to string: {}", err),
            )
        })?;

        self.0.set(key, data_str.as_str()).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("couldn't write to web storage: {:?}", err),
            )
        })?;

        Ok(())
    }
}

fn key_to_path(key: &str) -> PathBuf {
    let path = key
        .strip_prefix(DIR_PREFIX)
        .unwrap_or_default()
        .strip_prefix('/')
        .unwrap_or_default();
    PathBuf::from(path)
}

fn path_to_key(path: &Path, expect_file: bool) -> Result<String, Error> {
    if expect_file && path.parent().is_none() {
        return Err(Error::Path(anyhow!("expected a non-root path to a file")));
    }

    Ok(format!("{}/{}", DIR_PREFIX, path.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_path_conversion() {
        // empty path denied for files
        assert!(path_to_key(Path::new(""), true).is_err());

        // empty path ok for prefixes
        assert!(path_to_key(Path::new(""), false).is_ok());

        let path = Path::new("foo");
        let key = path_to_key(path, false).unwrap();
        assert_eq!("_dir/foo", key);
        assert_eq!(path, key_to_path("_dir/foo"));

        let path = Path::new("foo/bar");
        let key = path_to_key(path, false).unwrap();
        assert_eq!("_dir/foo/bar", key);
        assert_eq!(path, key_to_path("_dir/foo/bar"));
    }
}
