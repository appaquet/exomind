use std::path::{Path, PathBuf};

use super::{Directory, DynDirectory, Error, FileStat};

pub struct ScopedDirectory {
    inner: DynDirectory,
    base_path: PathBuf,
}

impl ScopedDirectory {
    pub fn new(dir: impl Into<DynDirectory>, base_path: PathBuf) -> Self {
        ScopedDirectory {
            inner: dir.into(),
            base_path,
        }
    }

    fn join_path(&self, path: &Path, expect_file: bool) -> Result<PathBuf, Error> {
        let joined = self.base_path.join(path);
        if expect_file && path.parent().is_none() {
            return Err(Error::Path(anyhow!("expected a non-root path to a file")));
        }

        Ok(joined)
    }
}

impl Directory for ScopedDirectory {
    fn open_read(&self, path: &std::path::Path) -> Result<Box<dyn super::FileRead>, super::Error> {
        let path = self.join_path(path, true)?;
        self.inner.open_read(&path)
    }

    fn open_write(
        &self,
        path: &std::path::Path,
    ) -> Result<Box<dyn super::FileWrite>, super::Error> {
        let path = self.join_path(path, true)?;
        self.inner.open_write(&path)
    }

    fn open_create(
        &self,
        path: &std::path::Path,
    ) -> Result<Box<dyn super::FileWrite>, super::Error> {
        let path = self.join_path(path, true)?;
        self.inner.open_create(&path)
    }

    fn list(
        &self,
        prefix: Option<&std::path::Path>,
    ) -> Result<Vec<Box<dyn super::FileStat>>, super::Error> {
        let path = prefix.map(|p| self.join_path(p, false)).transpose()?;
        self.inner.list(path.as_deref())
    }

    fn stat(&self, path: &std::path::Path) -> Result<Box<dyn super::FileStat>, super::Error> {
        let resolved_path = self.join_path(path, true)?;
        let stat = self.inner.stat(&resolved_path)?;

        Ok(Box::new(ScopedFileStat {
            path: path.to_path_buf(),
            inner: stat,
        }))
    }

    fn exists(&self, path: &std::path::Path) -> bool {
        let Ok(path) = self.join_path(path, true) else {
            return false;
        };

        self.inner.exists(&path)
    }

    fn delete(&self, path: &std::path::Path) -> Result<(), super::Error> {
        let path = self.join_path(path, true)?;
        self.inner.delete(&path)
    }

    fn clone(&self) -> DynDirectory {
        ScopedDirectory {
            inner: self.inner.clone(),
            base_path: self.base_path.clone(),
        }
        .into()
    }

    fn as_os_path(&self) -> Result<std::path::PathBuf, super::Error> {
        let path = self.inner.as_os_path()?;
        Ok(path.join(&self.base_path))
    }
}

pub struct ScopedFileStat {
    path: PathBuf,
    inner: Box<dyn super::FileStat>,
}

impl FileStat for ScopedFileStat {
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn size(&self) -> u64 {
        self.inner.size()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::dir::{os::OsDirectory, ram::RamDirectory};

    #[test]
    fn test_write_read_file() {
        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from("sub"));
        super::super::tests::test_write_read_file(scoped);

        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from("sub/sub"));
        super::super::tests::test_write_read_file(scoped);

        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from(""));
        super::super::tests::test_write_read_file(scoped);
    }

    #[test]
    fn test_list() {
        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from("sub"));
        super::super::tests::test_list(scoped);

        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from("sub/sub"));
        super::super::tests::test_list(scoped);

        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from(""));
        super::super::tests::test_list(scoped);
    }

    #[test]
    fn test_delete() {
        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from("sub"));
        super::super::tests::test_delete(scoped);

        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from("sub/sub"));
        super::super::tests::test_delete(scoped);

        let ram = RamDirectory::new();
        let scoped = ScopedDirectory::new(ram, PathBuf::from(""));
        super::super::tests::test_delete(scoped);
    }

    #[test]
    fn test_as_os_path() {
        let dir = tempdir().unwrap();
        let scoped = ScopedDirectory::new(
            OsDirectory::new(dir.path().to_path_buf()),
            PathBuf::from("sub"),
        );

        let os_path = scoped.as_os_path().unwrap();
        assert_eq!(dir.path().join("sub"), os_path.as_path());
    }
}
