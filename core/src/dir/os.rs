use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use super::*;

#[derive(Clone)]
pub struct OsDirectory {
    base_path: PathBuf,
}

impl OsDirectory {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn resolve_path(&self, path: &Path, expect_file: bool) -> Result<PathBuf, Error> {
        if expect_file && path.parent().is_none() {
            return Err(Error::Path(anyhow!("expected a non-root path to a file")));
        }

        let joined = self.base_path.join(path);
        if !joined.starts_with(&self.base_path) {
            return Err(Error::Path(anyhow!(
                "resolved path {:?} is not under base path {:?}",
                joined,
                self.base_path
            )));
        }

        Ok(joined)
    }
}

impl Directory for OsDirectory {
    fn open_read(&self, path: &Path) -> Result<Box<dyn FileRead>, Error> {
        let path = self.resolve_path(path, true)?;
        create_parent_path(&path)?;

        let file = File::open(path)?;
        Ok(Box::new(OsFile { file }))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn FileWrite>, Error> {
        let path = self.resolve_path(path, true)?;
        create_parent_path(&path)?;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .read(true)
            .open(path)?;
        Ok(Box::new(OsFile { file }))
    }

    fn open_create(&self, path: &Path) -> Result<Box<dyn FileWrite>, Error> {
        let path = self.resolve_path(path, true)?;
        create_parent_path(&path)?;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .truncate(true)
            .open(path)?;
        Ok(Box::new(OsFile { file }))
    }

    fn list(&self, prefix: Option<&Path>) -> Result<Vec<Box<dyn FileStat>>, Error> {
        let prefix = if let Some(prefix) = prefix {
            let _ = self.resolve_path(prefix, false)?; // validate
            Some(prefix)
        } else {
            None
        };

        let has_prefix = |path: &Path| {
            if let Some(prefix) = prefix {
                path.starts_with(prefix)
            } else {
                true
            }
        };

        fn walk_dir(
            entries: &mut Vec<Box<dyn FileStat + 'static>>,
            has_prefix: impl Fn(&Path) -> bool + Copy,
            base_path: &Path,
            path: &Path,
        ) -> Result<(), Error> {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                let path_no_prefix = path.strip_prefix(base_path).unwrap();
                if !has_prefix(path_no_prefix) {
                    continue;
                }

                let metadata = std::fs::metadata(&path)?;
                if !metadata.is_dir() {
                    let mut path = PathBuf::from("/");
                    path.push(path_no_prefix);

                    entries.push(Box::new(OsFileStat { path, metadata }));
                } else {
                    walk_dir(entries, has_prefix, base_path, &path)?;
                }
            }
            Ok(())
        }

        let mut entries = Vec::<Box<dyn FileStat>>::new();
        walk_dir(&mut entries, has_prefix, &self.base_path, &self.base_path)?;

        Ok(entries)
    }

    fn stat(&self, path: &Path) -> Result<Box<dyn FileStat>, Error> {
        let metadata = std::fs::metadata(self.resolve_path(path, true)?)?;
        Ok(Box::new(OsFileStat {
            path: path.to_path_buf(),
            metadata,
        }))
    }

    fn exists(&self, path: &Path) -> bool {
        match self.resolve_path(path, true) {
            Ok(res) => res.exists(),
            Err(_) => false,
        }
    }

    fn delete(&self, path: &Path) -> Result<(), Error> {
        let path = self.resolve_path(path, true)?;
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn clone(&self) -> DynDirectory {
        OsDirectory {
            base_path: self.base_path.clone(),
        }
        .into()
    }

    fn as_os_path(&self) -> Result<PathBuf, Error> {
        create_parent_path(&self.base_path)?;
        Ok(self.base_path.to_path_buf())
    }
}

fn create_parent_path(path: &Path) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::Path(anyhow!("expected parent on resolved path {:?}", path)))?;

    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}

pub struct OsFile {
    file: File,
}

impl FileRead for OsFile {}
impl FileWrite for OsFile {}

impl Read for OsFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for OsFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl Write for OsFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

pub struct OsFileStat {
    path: PathBuf,
    metadata: std::fs::Metadata,
}

impl FileStat for OsFileStat {
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn size(&self) -> u64 {
        self.metadata.len()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_write_read_file() {
        let tmp = tempdir().unwrap();
        let dir = OsDirectory::new(tmp.into_path());
        super::super::tests::test_write_read_file(dir);
    }

    #[test]
    fn test_list() {
        let tmp = tempdir().unwrap();
        let dir = OsDirectory::new(tmp.into_path());
        super::super::tests::test_list(dir);
    }

    #[test]
    fn test_delete() {
        let tmp = tempdir().unwrap();
        let dir = OsDirectory::new(tmp.into_path());
        super::super::tests::test_delete(dir);
    }

    #[test]
    fn test_as_os_path() {
        let tmp = tempdir().unwrap();
        let dir = OsDirectory::new(tmp.path().to_path_buf());

        let os_path = dir.as_os_path().unwrap();
        assert_eq!(tmp.path(), os_path.as_path());
    }
}
