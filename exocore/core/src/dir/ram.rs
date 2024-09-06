use std::{
    collections::BTreeMap,
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, RwLock},
};

use super::*;

#[derive(Default)]
pub struct RamDirectory {
    files: Arc<RwLock<BTreeMap<PathBuf, RamFileData>>>,
}

impl RamDirectory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Directory for RamDirectory {
    fn open_read(&self, path: &Path) -> Result<Box<dyn FileRead>, Error> {
        if path.parent().is_none() {
            return Err(Error::Path(anyhow!("expected a non-root path to a file")));
        }

        let files = self.files.read().unwrap();
        let file = files
            .get(path)
            .ok_or_else(|| Error::NotFound(path.to_path_buf()))?;

        Ok(Box::new(RamFile {
            data: file.clone(),
            cursor: 0,
        }))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn FileWrite>, Error> {
        if path.parent().is_none() {
            return Err(Error::Path(anyhow!("expected a non-root path to a file")));
        }

        let mut files = self.files.write().unwrap();

        let data = files.get(path).cloned();
        let data = if let Some(data) = data {
            data
        } else {
            let data = RamFileData::default();
            files.insert(path.to_path_buf(), data.clone());
            data
        };

        Ok(Box::new(RamFile { data, cursor: 0 }))
    }

    fn open_create(&self, path: &Path) -> Result<Box<dyn FileWrite>, Error> {
        if path.parent().is_none() {
            return Err(Error::Path(anyhow!("expected a non-root path to a file")));
        }

        let mut files = self.files.write().unwrap();

        let data = RamFileData::default();
        files.insert(path.to_path_buf(), data.clone());

        Ok(Box::new(RamFile { data, cursor: 0 }))
    }

    fn list(&self, prefix: Option<&Path>) -> Result<Vec<Box<dyn FileStat>>, Error> {
        let prefix = prefix.unwrap_or_else(|| Path::new(""));
        let files = self.files.read().unwrap();

        let mut result: Vec<Box<dyn FileStat>> = Vec::new();
        for (path, file) in files.iter() {
            if path.starts_with(prefix) {
                result.push(Box::new(RamFileStat {
                    path: path.to_path_buf(),
                    data: file.clone(),
                }));
            }
        }

        Ok(result)
    }

    fn stat(&self, path: &Path) -> Result<Box<dyn FileStat>, Error> {
        let files = self.files.read().unwrap();
        let file = files
            .get(path)
            .ok_or_else(|| Error::NotFound(path.to_path_buf()))?;

        Ok(Box::new(RamFileStat {
            path: path.to_path_buf(),
            data: file.clone(),
        }))
    }

    fn exists(&self, path: &Path) -> bool {
        let files = self.files.read().unwrap();
        files.contains_key(path)
    }

    fn delete(&self, path: &Path) -> Result<(), Error> {
        let mut files = self.files.write().unwrap();
        files.remove(path);
        Ok(())
    }

    fn clone(&self) -> DynDirectory {
        RamDirectory {
            files: self.files.clone(),
        }
        .into()
    }

    fn as_os_path(&self) -> Result<PathBuf, Error> {
        Err(Error::NotOsDirectory)
    }
}

#[derive(Clone, Default)]
pub struct RamFileData {
    pub bytes: Arc<RwLock<Vec<u8>>>,
}

impl From<Vec<u8>> for RamFileData {
    fn from(bytes: Vec<u8>) -> Self {
        RamFileData {
            bytes: Arc::new(RwLock::new(bytes)),
        }
    }
}

#[derive(Default)]
pub struct RamFile {
    data: RamFileData,
    cursor: usize,
}

impl RamFile {
    pub fn new(data: RamFileData) -> Self {
        RamFile { data, cursor: 0 }
    }
}

impl FileRead for RamFile {}

impl FileWrite for RamFile {}

impl Read for RamFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.data.bytes.read().unwrap();
        if self.cursor >= data.len() {
            return Ok(0);
        }

        let to = std::cmp::min(self.cursor + buf.len(), data.len());
        let count = (&data[self.cursor..to]).read(buf)?;
        self.cursor += count;
        Ok(count)
    }
}

impl Seek for RamFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let data = self.data.bytes.read().unwrap();
        let len = data.len();
        let pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::End(pos) => (len as i64 + pos) as u64,
            SeekFrom::Current(pos) => (self.cursor as i64 + pos) as u64,
        };
        self.cursor = pos as usize;
        Ok(pos)
    }
}

impl Write for RamFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut data = self.data.bytes.write().unwrap();

        for i in 0..buf.len() {
            if self.cursor + i >= data.len() {
                data.push(buf[i]);
            } else {
                data[self.cursor + i] = buf[i];
            }
        }

        self.cursor += buf.len();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct RamFileStat {
    path: PathBuf,
    data: RamFileData,
}

impl FileStat for RamFileStat {
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn size(&self) -> u64 {
        self.data.bytes.read().unwrap().len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_file() {
        super::super::tests::test_write_read_file(RamDirectory::new());
    }

    #[test]
    fn test_list() {
        super::super::tests::test_list(RamDirectory::new());
    }

    #[test]
    fn test_delete() {
        super::super::tests::test_delete(RamDirectory::new());
    }
}
