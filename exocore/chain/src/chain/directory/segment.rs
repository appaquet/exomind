use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    ops::Range,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, Weak},
};

use serde::{Deserialize, Serialize};

use super::{
    tracker::{RegisteredSegment, SegmentTracker},
    DirectoryChainStoreConfig, Error,
};
use crate::{
    block::{Block, BlockOffset, DataBlock},
    chain::ChainData,
    data::{Data, MmapData, RefData},
};

/// A segment of the chain, stored in its own file (`segment_file`) and that
/// should not exceed a size specified by configuration.
///
/// As mmap can only accessed allocated space in the file, we need to
/// pre-allocate space in the file. When a block would exceed this pre-allocated
/// space, we re-size and re-open the file.
pub struct DirectorySegment {
    config: DirectoryChainStoreConfig,
    segment_path: PathBuf,
    segment_file: SegmentFile,
    first_block_offset: BlockOffset,
    next_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    pub fn create<B: Block>(
        config: DirectoryChainStoreConfig,
        directory: &Path,
        block: &B,
        tracker: SegmentTracker,
    ) -> Result<DirectorySegment, Error> {
        let block_header_reader = block.header().get_reader().unwrap();
        let first_block_offset = block_header_reader.get_offset();

        let segment_path = Self::segment_path(directory, first_block_offset);
        if segment_path.exists() {
            return Err(Error::UnexpectedState(anyhow!(
                "Tried to create a new segment at path {:?}, but already existed",
                segment_path
            )));
        }

        info!(
            "Creating new segment at {:?} for offset {}",
            directory, first_block_offset
        );

        let block_size = block.total_size();
        let segment_alloc_size = config.segment_over_allocate_size.max(block_size as u64);

        let mut segment_file = SegmentFile::open(&segment_path, segment_alloc_size, tracker)?;
        segment_file.write_block(0, block)?;

        Ok(DirectorySegment {
            config,
            segment_path,
            segment_file,
            first_block_offset,
            next_block_offset: first_block_offset + block_size as BlockOffset,
            next_file_offset: block_size,
        })
    }

    #[cfg(test)]
    pub fn open_with_first_offset(
        config: DirectoryChainStoreConfig,
        directory: &Path,
        first_offset: BlockOffset,
        tracker: SegmentTracker,
    ) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, first_offset);
        let segment = Self::open(config, &segment_path, tracker)?;

        if segment.first_block_offset != first_offset {
            return Err(Error::Integrity(anyhow!(
                "First block offset != segment first_offset ({} != {})",
                segment.first_block_offset,
                first_offset
            )));
        }

        Ok(segment)
    }

    pub fn open(
        config: DirectoryChainStoreConfig,
        segment_path: &Path,
        tracker: SegmentTracker,
    ) -> Result<DirectorySegment, Error> {
        info!("Opening segment at {:?}", segment_path);

        let metadata = SegmentMetadata::from_segment_file_path(segment_path, tracker.clone())?;

        Self::open_with_metadata(config, segment_path, &metadata, tracker)
    }

    pub fn open_with_metadata(
        config: DirectoryChainStoreConfig,
        segment_path: &Path,
        metadata: &SegmentMetadata,
        tracker: SegmentTracker,
    ) -> Result<DirectorySegment, Error> {
        info!(
            "Opening segment at {:?} with metadata {:?}",
            segment_path, metadata
        );

        let segment_file = SegmentFile::open(segment_path, 0, tracker)?;
        let next_file_offset = (metadata.next_block_offset - metadata.first_block_offset) as usize;

        Ok(DirectorySegment {
            config,
            first_block_offset: metadata.first_block_offset,
            segment_path: segment_path.to_path_buf(),
            segment_file,
            next_block_offset: metadata.next_block_offset,
            next_file_offset,
        })
    }

    pub fn offset_range(&self) -> Range<BlockOffset> {
        self.first_block_offset..self.next_block_offset
    }

    pub fn first_block_offset(&self) -> BlockOffset {
        self.first_block_offset
    }

    pub fn next_block_offset(&self) -> BlockOffset {
        self.next_block_offset
    }

    pub fn next_file_offset(&self) -> usize {
        self.next_file_offset
    }

    pub fn write_block<B: Block>(&mut self, block: &B) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset;
        let next_block_offset = self.next_block_offset;
        let block_size = block.total_size();

        let block_offset = block.offset();
        if next_block_offset != block_offset {
            return Err(Error::InvalidNextBlock {
                offset: block_offset,
                expected_offset: next_block_offset,
            });
        }

        self.ensure_file_size(block_size)?;
        self.segment_file.write_block(next_file_offset, block)?;

        self.next_file_offset += block_size;
        self.next_block_offset += block_size as BlockOffset;

        Ok(())
    }

    pub fn get_block(&self, offset: BlockOffset) -> Result<DataBlock<ChainData>, Error> {
        let first_block_offset = self.first_block_offset;
        if offset < first_block_offset {
            return Err(Error::OutOfBound(anyhow!(
                "Tried to read block at {}, but first offset was at {}",
                offset,
                first_block_offset
            )));
        }

        if offset >= self.next_block_offset {
            return Err(Error::OutOfBound(anyhow!(
                "Tried to read block at {}, but next offset was at {}",
                offset,
                self.next_block_offset
            )));
        }

        let block_file_offset = (offset - first_block_offset) as usize;
        self.segment_file.get_block(block_file_offset)
    }

    pub fn get_block_from_next_offset(
        &self,
        next_offset: BlockOffset,
    ) -> Result<DataBlock<ChainData>, Error> {
        let first_block_offset = self.first_block_offset;
        if next_offset < first_block_offset {
            return Err(Error::OutOfBound(anyhow!(
                "Tried to read block from next offset {}, but first offset was at {}",
                next_offset,
                first_block_offset
            )));
        }

        if next_offset > self.next_block_offset {
            return Err(Error::OutOfBound(anyhow!(
                "Tried to read block from next offset {}, but next offset was at {}",
                next_offset,
                self.next_block_offset
            )));
        }

        let next_file_offset = (next_offset - first_block_offset) as usize;
        self.segment_file.get_block_from_next(next_file_offset)
    }

    pub fn truncate_from_block_offset(&mut self, block_offset: BlockOffset) -> Result<(), Error> {
        if block_offset < self.first_block_offset {
            return Err(Error::OutOfBound(anyhow!(
                "Offset {} is before first block offset of segment {}",
                block_offset,
                self.first_block_offset
            )));
        }
        self.next_block_offset = block_offset;
        let keep_len = block_offset - self.first_block_offset;
        self.segment_file.set_len(keep_len)
    }

    pub fn open_write(&self) -> Result<(), Error> {
        self.segment_file.maybe_mmap_write()
    }

    pub fn close_write(&self) {
        self.segment_file.close_write();
    }

    pub fn delete(self) -> Result<(), Error> {
        let segment_path = self.segment_path.clone();
        drop(self);
        std::fs::remove_file(&segment_path).map_err(|err| {
            Error::new_io(
                err,
                format!("Couldn't delete segment file {:?}", segment_path),
            )
        })?;
        Ok(())
    }

    pub fn is_segment_file(path: &Path) -> bool {
        path.file_name()
            .and_then(OsStr::to_str)
            .map_or(false, |filename| filename.starts_with("seg_"))
    }

    fn segment_path(directory: &Path, first_offset: BlockOffset) -> PathBuf {
        directory.join(format!("seg_{}.bin", first_offset))
    }

    fn ensure_file_size(&mut self, write_size: usize) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset;

        if self.segment_file.current_size < (next_file_offset + write_size) as u64 {
            let target_size =
                (next_file_offset + write_size) as u64 + self.config.segment_over_allocate_size;
            self.segment_file.set_len(target_size)?;
        }

        Ok(())
    }

    #[cfg(test)]
    fn truncate_extra(&mut self) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset as u64;
        self.segment_file.set_len(next_file_offset)
    }

    pub fn metadata(&self) -> SegmentMetadata {
        let filename = self
            .segment_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        SegmentMetadata {
            filename,
            first_block_offset: self.first_block_offset,
            next_block_offset: self.next_block_offset,
        }
    }
}

/// Metadata of a segment file, such as beginning and end offsets.
///
/// Metadata can be inferred by scanning the file, but can be persisted
/// separately to allow faster opening.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentMetadata {
    pub filename: String,
    pub first_block_offset: BlockOffset,
    pub next_block_offset: BlockOffset,
}

impl SegmentMetadata {
    fn from_segment_file_path(
        path: &Path,
        tracker: SegmentTracker,
    ) -> Result<SegmentMetadata, Error> {
        let segment_file = SegmentFile::open(path, 0, tracker)?;

        Self::from_segment_file(&segment_file)
    }

    fn from_segment_file(segment: &SegmentFile) -> Result<SegmentMetadata, Error> {
        // read first block to validate it has the same offset as segment
        let first_block = segment.get_block(0).map_err(|err| {
            error!(
                "Couldn't read first block from segment file {:?}: {}",
                &segment.path, err
            );
            err
        })?;

        // iterate through segments and find the last block and its offset
        let blocks_iterator = SegmentBlockIterator::new(segment);
        let last_block = blocks_iterator.last().ok_or_else(|| {
            Error::Integrity(anyhow!(
                "Couldn't find last block of segment: no blocks returned by iterator"
            ))
        })?;

        let next_block_offset = last_block.offset + last_block.total_size() as BlockOffset;

        let filename = segment
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(SegmentMetadata {
            filename,
            first_block_offset: first_block.offset,
            next_block_offset,
        })
    }
}

/// Wraps a mmap file stored on disk. As mmap cannot access content that is
/// beyond the file size, the segment is over-allocated so that we can write via
/// mmap. If writing would exceed the size, we re-allocate the file and re-open
/// the mmap.
struct SegmentFile {
    path: PathBuf,
    file: File,
    mmap: RwLock<SegmentMmap>,
    current_size: u64,
    tracker: SegmentTracker,
    registered_segment: RegisteredSegment,
}

enum SegmentMmap {
    Write(memmap2::MmapMut),
    Read(Weak<memmap2::Mmap>),
    Closed,
}

impl SegmentFile {
    fn open(path: &Path, minimum_size: u64, tracker: SegmentTracker) -> Result<SegmentFile, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|err| {
                Error::new_io(
                    err,
                    format!("Error opening/creating segment file {:?}", path),
                )
            })?;

        let mut current_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if current_size < minimum_size {
            current_size = minimum_size;
            file.set_len(minimum_size).map_err(|err| {
                Error::new_io(err, format!("Error setting len of segment file {:?}", path))
            })?;
        }

        let registered_segment = tracker.register(path.to_string_lossy().to_string());

        Ok(SegmentFile {
            path: path.to_path_buf(),
            file,
            mmap: RwLock::new(SegmentMmap::Closed),
            current_size,
            tracker,
            registered_segment,
        })
    }

    fn maybe_mmap_read(&self) -> Result<Option<Arc<memmap2::Mmap>>, Error> {
        self.registered_segment.access();

        {
            // first check if it's already open
            let mmap = self.mmap.read().unwrap();
            match &*mmap {
                SegmentMmap::Read(mmap) => {
                    // make sure mmap is still open
                    if let Some(mmap) = mmap.upgrade() {
                        return Ok(Some(mmap));
                    }
                }
                SegmentMmap::Write(_mmap) => {
                    return Ok(None);
                }
                _ => {}
            }
        }

        let mut mmap = self.mmap.write().unwrap();
        let mmap_arc = unsafe {
            let mmap = memmap2::MmapOptions::new().map(&self.file).map_err(|err| {
                Error::new_io(err, format!("Error mmaping segment file {:?}", self.path))
            })?;
            Arc::new(mmap)
        };

        self.tracker
            .open_read(&self.registered_segment, mmap_arc.clone());

        *mmap = SegmentMmap::Read(Arc::downgrade(&mmap_arc));

        Ok(Some(mmap_arc))
    }

    fn maybe_mmap_write(&self) -> Result<(), Error> {
        self.registered_segment.access();

        {
            // first check if it's already open for write or if it's closed
            let mmap = self.mmap.read().unwrap();
            match &*mmap {
                SegmentMmap::Write(_) => {
                    return Ok(());
                }
                SegmentMmap::Read(mmap) => {
                    // if it's open to read, we try to close it first
                    if mmap.upgrade().is_some() {
                        self.tracker.close(&self.registered_segment);
                    }

                    // then, if it's still open, we're still reading and should bail out
                    if mmap.upgrade().is_some() {
                        return Err(Error::UnexpectedState(anyhow!("Segment is in read-only")));
                    }
                }
                SegmentMmap::Closed => {}
            }
        }

        let mut mmap = self.mmap.write().unwrap();
        *mmap =
            unsafe {
                SegmentMmap::Write(memmap2::MmapOptions::new().map_mut(&self.file).map_err(
                    |err| Error::new_io(err, format!("Mmaping segment file {:?}", self.path)),
                )?)
            };

        self.tracker.open_write(&self.registered_segment);

        Ok(())
    }

    fn close_write(&self) {
        let mut mmap = self.mmap.write().unwrap();
        if !matches!(&*mmap, &SegmentMmap::Write(_)) {
            return;
        }

        *mmap = SegmentMmap::Closed;
    }

    fn get_block(&self, offset: usize) -> Result<DataBlock<ChainData>, Error> {
        let mmap_read = self.maybe_mmap_read()?;

        let mmap = self.mmap.read().unwrap();
        match &*mmap {
            SegmentMmap::Write(mmap) => {
                let data = RefData::new(&mmap[offset..]);
                let block = DataBlock::new(data)?;
                let bytes = block.as_data_vec();
                let data = ChainData::Bytes(bytes);
                Ok(DataBlock::new(data)?)
            }
            SegmentMmap::Read(_mmap) => {
                let mmap = mmap_read.expect("Read mmap, expected it opened");
                let data = MmapData::from_mmap(mmap, self.current_size as usize);
                let data = ChainData::Mmap(data);
                Ok(DataBlock::new(data.view(offset..))?)
            }
            _ => Err(Error::UnexpectedState(anyhow!("Expected map to be open"))),
        }
    }

    fn get_block_from_next(&self, next_offset: usize) -> Result<DataBlock<ChainData>, Error> {
        let mmap_read = self.maybe_mmap_read()?;

        let mmap = self.mmap.read().unwrap();
        match &*mmap {
            SegmentMmap::Write(mmap) => {
                let data = RefData::new(mmap);
                let block = DataBlock::new_from_next_offset(data, next_offset)?;
                let bytes = block.as_data_vec();
                let data = ChainData::Bytes(bytes);
                Ok(DataBlock::new(data)?)
            }
            SegmentMmap::Read(_mmap) => {
                let mmap = mmap_read.expect("Read mmap, expected it opened");
                let data = MmapData::from_mmap(mmap, self.current_size as usize);
                let data = ChainData::Mmap(data);
                Ok(DataBlock::new_from_next_offset(data, next_offset)?)
            }
            _ => Err(Error::UnexpectedState(anyhow!("Expected map to be open"))),
        }
    }

    fn write_block<B: Block>(&mut self, offset: usize, block: &B) -> Result<(), Error> {
        self.maybe_mmap_write()?;

        let mut mmap = self.mmap.write().unwrap();
        let SegmentMmap::Write(mmap) = &mut *mmap else {
            return Err(Error::UnexpectedState(anyhow!(
                "Expected map to be writable"
            )));
        };

        block.copy_data_into(&mut mmap[offset..]);

        Ok(())
    }

    fn set_len(&mut self, new_size: u64) -> Result<(), Error> {
        self.maybe_mmap_write()?;

        {
            // On Windows, we can't resize a file while it's currently being mapped. We
            // close the mmap first by replacing it by an anonymous mmap.
            let mut mmap = self.mmap.write().unwrap();
            *mmap = SegmentMmap::Closed;

            self.tracker.close(&self.registered_segment);

            self.file.set_len(new_size).map_err(|err| {
                Error::new_io(
                    err,
                    format!("Error setting len of segment file {:?}", self.path),
                )
            })?;
        }

        self.maybe_mmap_write()?;
        self.current_size = new_size;

        Ok(())
    }
}

/// Block iterator over a SegmentFile blocks.
struct SegmentBlockIterator<'s> {
    current_offset: usize,
    segment: &'s SegmentFile,
    last_error: Option<Error>,
}

impl<'s> SegmentBlockIterator<'s> {
    fn new(segment: &'s SegmentFile) -> SegmentBlockIterator<'s> {
        SegmentBlockIterator {
            current_offset: 0,
            segment,
            last_error: None,
        }
    }
}

impl<'s> Iterator for SegmentBlockIterator<'s> {
    type Item = DataBlock<ChainData>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset >= self.segment.current_size as usize {
            return None;
        }

        let block_res = self.segment.get_block(self.current_offset);
        match block_res {
            Ok(block) => {
                self.current_offset += block.total_size();
                Some(block)
            }
            Err(other) => {
                self.last_error = Some(other);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use exocore_core::cell::{FullCell, LocalNode};

    use super::{super::tests::create_block, *};

    #[test]
    fn directory_segment_create_and_open() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let dir = tempfile::tempdir()?;
        let tracker = SegmentTracker::new(1);

        let segment_id = 1234;
        let block = create_block(&cell, 1234);

        {
            let segment =
                DirectorySegment::create(Default::default(), dir.path(), &block, tracker.clone())?;
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.next_file_offset, block.total_size());
            assert_eq!(
                segment.next_block_offset as usize,
                1234 + block.total_size()
            );
        }

        {
            let segment = DirectorySegment::open_with_first_offset(
                Default::default(),
                dir.path(),
                segment_id,
                tracker,
            )?;
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.next_file_offset, block.total_size());
            assert_eq!(
                segment.next_block_offset as usize,
                1234 + block.total_size()
            );
        }

        Ok(())
    }

    #[test]
    fn directory_segment_create_already_exist() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let dir = tempfile::tempdir()?;
        let tracker = SegmentTracker::new(1);

        {
            let block = create_block(&cell, 1234);
            let _segment =
                DirectorySegment::create(Default::default(), dir.path(), &block, tracker.clone())?;
        }

        {
            let block = create_block(&cell, 1234);
            assert!(
                DirectorySegment::create(Default::default(), dir.path(), &block, tracker).is_err()
            );
        }

        Ok(())
    }

    #[test]
    fn directory_segment_open_invalid() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;

        {
            let tracker = SegmentTracker::new(1);
            let segment_path = dir.path().join("some_file");
            std::fs::write(&segment_path, "hello")?;
            assert!(DirectorySegment::open(Default::default(), &segment_path, tracker).is_err());
        }

        {
            let tracker = SegmentTracker::new(1);
            let segment_path = dir.path().join("some_file");
            std::fs::write(&segment_path, "hello")?;
            assert!(DirectorySegment::open_with_first_offset(
                Default::default(),
                &segment_path,
                100,
                tracker,
            )
            .is_err());
        }

        Ok(())
    }

    #[test]
    fn directory_segment_append_block() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let dir = tempfile::tempdir()?;
        let tracker = SegmentTracker::new(1);

        let offset1 = 0;
        let block = create_block(&cell, offset1);
        let mut segment =
            DirectorySegment::create(Default::default(), dir.path(), &block, tracker)?;
        {
            let block = segment.get_block(offset1)?;
            assert_eq!(block.offset, offset1);
        }

        let offset2 = offset1 + block.total_size() as BlockOffset;
        assert_eq!(segment.next_block_offset, offset2);
        let block = create_block(&cell, offset2);
        segment.write_block(&block)?;
        {
            let block = segment.get_block(offset2)?;
            assert_eq!(block.offset, offset2);
        }

        let offset3 = offset2 + block.total_size() as BlockOffset;
        assert_eq!(segment.next_block_offset, offset3);
        let block = create_block(&cell, offset3);
        segment.write_block(&block)?;
        {
            let block = segment.get_block(offset3)?;
            assert_eq!(block.offset, offset3);
        }

        assert!(segment.get_block(10).is_err());
        assert!(segment.get_block(offset3 + 10).is_err());

        let last_block = segment.get_block_from_next_offset(segment.next_block_offset)?;
        assert_eq!(last_block.offset, offset3);

        assert!(segment
            .get_block_from_next_offset(segment.next_block_offset + 42)
            .is_err());

        Ok(())
    }

    #[test]
    fn directory_segment_non_zero_offset_write() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let dir = tempfile::tempdir()?;
        let config = Default::default();
        let segment_first_block_offset = 1234;
        let tracker = SegmentTracker::new(1);

        {
            let first_block = create_block(&cell, segment_first_block_offset);
            let mut segment =
                DirectorySegment::create(config, dir.path(), &first_block, tracker.clone())?;
            let next_block_offset = segment.next_block_offset;
            assert_eq!(
                next_block_offset,
                segment_first_block_offset + first_block.total_size() as BlockOffset
            );
            append_blocks_to_segment(&cell, &mut segment, next_block_offset, 999);
        }

        {
            let segment = DirectorySegment::open_with_first_offset(
                config,
                dir.path(),
                segment_first_block_offset,
                tracker,
            )?;
            assert!(segment.get_block(0).is_err());
            assert!(segment.get_block(1234).is_ok());
            assert!(segment.get_block(segment.next_block_offset).is_err());
            assert!(segment
                .get_block_from_next_offset(segment.next_block_offset)
                .is_ok());

            let iter = SegmentBlockIterator::new(&segment.segment_file);
            assert_eq!(iter.count(), 1000);
        }

        Ok(())
    }

    #[test]
    fn directory_segment_grow_and_truncate() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let config = DirectoryChainStoreConfig {
            segment_over_allocate_size: 100_000,
            ..Default::default()
        };
        let tracker = SegmentTracker::new(1);

        let dir = tempfile::tempdir()?;
        let mut next_offset = 0;

        let block = create_block(&cell, next_offset);
        let mut segment = DirectorySegment::create(config, dir.path(), &block, tracker)?;
        next_offset += block.total_size() as u64;
        assert_eq!(segment.next_block_offset, next_offset);
        assert_eq!(segment.next_file_offset, block.total_size());

        let init_segment_size = segment.segment_file.current_size;
        append_blocks_to_segment(&cell, &mut segment, next_offset, 999);
        let end_segment_size = segment.segment_file.current_size;
        let next_file_offset = segment.next_file_offset;

        assert_eq!(init_segment_size, 100_000);
        assert!(end_segment_size >= 200_000);

        segment.truncate_extra()?;

        let truncated_segment_size = segment.segment_file.current_size;
        assert_eq!(truncated_segment_size, next_file_offset as u64);

        let iter = SegmentBlockIterator::new(&segment.segment_file);
        assert_eq!(iter.count(), 1000);

        Ok(())
    }

    #[test]
    fn directory_segment_truncate_from_segment() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let dir = tempfile::tempdir()?;
        let config = DirectoryChainStoreConfig {
            segment_over_allocate_size: 100_000,
            ..Default::default()
        };
        let tracker = SegmentTracker::new(1);

        let mut next_offset = 1000;

        let block = create_block(&cell, next_offset);
        let mut segment = DirectorySegment::create(config, dir.path(), &block, tracker)?;
        next_offset += block.total_size() as u64;
        append_blocks_to_segment(&cell, &mut segment, next_offset, 999);

        // should not remove any blocks, only remove over allocated space
        segment.truncate_from_block_offset(segment.next_block_offset)?;
        let iter = SegmentBlockIterator::new(&segment.segment_file);
        assert_eq!(iter.count(), 1000);

        // truncating before beginning of file is impossible
        assert!(segment.truncate_from_block_offset(900).is_err());

        // truncating at 10th should result in only 10 blocks left
        let mut iter = SegmentBlockIterator::new(&segment.segment_file);
        let nth_offset = iter.nth(10).unwrap().offset;
        segment.truncate_from_block_offset(nth_offset)?;
        let iter = SegmentBlockIterator::new(&segment.segment_file);
        assert_eq!(iter.count(), 10);

        Ok(())
    }

    #[test]
    fn segment_file_create() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let segment_path = dir.path().join("segment_0.seg");
        let tracker = SegmentTracker::new(1);

        let segment_file = SegmentFile::open(&segment_path, 1000, tracker.clone())?;
        assert_eq!(segment_file.current_size, 1000);
        drop(segment_file);

        let mut segment_file = SegmentFile::open(&segment_path, 10, tracker)?;
        assert_eq!(segment_file.current_size, 1000);

        segment_file.set_len(2000)?;
        assert_eq!(segment_file.current_size, 2000);

        Ok(())
    }

    #[test]
    fn segment_file_mmap_transition() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node)?;
        let config = DirectoryChainStoreConfig {
            segment_over_allocate_size: 100_000,
            ..Default::default()
        };
        let dir = tempfile::tempdir()?;
        let tracker = SegmentTracker::new(1);

        let mut next_offset = 0;
        let block = create_block(&cell, next_offset);
        let mut segment = DirectorySegment::create(config, dir.path(), &block, tracker)?;
        next_offset += block.total_size() as u64;

        let block = create_block(&cell, next_offset);
        segment.write_block(&block)?;

        {
            let mmap = segment.segment_file.mmap.read().unwrap();
            assert!(matches!(&*mmap, SegmentMmap::Write(_)));
        }

        segment.close_write();

        {
            let mmap = segment.segment_file.mmap.read().unwrap();
            assert!(matches!(&*mmap, SegmentMmap::Closed));
        }

        let block = segment.get_block(0)?;

        {
            let mmap = segment.segment_file.mmap.read().unwrap();
            assert!(matches!(&*mmap, SegmentMmap::Read(_)));
        }

        // cannot open for write since we have a block referencing the data
        assert!(segment.open_write().is_err());

        drop(block);

        assert!(segment.open_write().is_ok());

        Ok(())
    }

    fn append_blocks_to_segment(
        cell: &FullCell,
        segment: &mut DirectorySegment,
        first_block_offset: BlockOffset,
        nb_blocks: usize,
    ) {
        let mut next_offset = first_block_offset;
        for _i in 0..nb_blocks {
            assert_eq!(next_offset, segment.next_block_offset);
            let block = create_block(cell, next_offset);
            segment.write_block(&block).unwrap();
            next_offset += block.total_size() as u64;
        }
    }
}
