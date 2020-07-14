use std::fs::{File, OpenOptions};
use std::ops::Range;
use std::path::{Path, PathBuf};

use super::Error;
use crate::block::{Block, BlockOffset, BlockRef, ChainBlockIterator};

use super::DirectoryChainStoreConfig;
use std::ffi::OsStr;

/// A segment of the chain, stored in its own file (`segment_file`) and that
/// should not exceed a size specified by configuration.
///
/// As mmap can only accessed allocated space in the file, we need to
/// pre-allocate space in the file. When a block would exceed this pre-allocated
/// space, we re-size and re-open the file.
pub struct DirectorySegment {
    config: DirectoryChainStoreConfig,
    first_block_offset: BlockOffset,
    segment_path: PathBuf,
    segment_file: SegmentFile,
    next_block_offset: BlockOffset,
    next_file_offset: usize,
}

impl DirectorySegment {
    pub fn create<B: Block>(
        config: DirectoryChainStoreConfig,
        directory: &Path,
        block: &B,
    ) -> Result<DirectorySegment, Error> {
        let block_header_reader = block.header().get_reader().unwrap();
        let first_block_offset = block_header_reader.get_offset();

        let segment_path = Self::segment_path(directory, first_block_offset);
        if segment_path.exists() {
            return Err(Error::UnexpectedState(format!(
                "Tried to create a new segment at path {:?}, but already existed",
                segment_path
            )));
        }

        info!(
            "Creating new segment at {:?} for offset {}",
            directory, first_block_offset
        );
        let mut segment_file = SegmentFile::open(&segment_path, config.segment_over_allocate_size)?;
        block.copy_data_into(&mut segment_file.mmap[0..]);
        let written_data_size = block.total_size();

        Ok(DirectorySegment {
            config,
            first_block_offset,
            segment_path,
            segment_file,
            next_block_offset: first_block_offset + written_data_size as BlockOffset,
            next_file_offset: written_data_size,
        })
    }

    #[cfg(test)]
    pub fn open_with_first_offset(
        config: DirectoryChainStoreConfig,
        directory: &Path,
        first_offset: BlockOffset,
    ) -> Result<DirectorySegment, Error> {
        let segment_path = Self::segment_path(directory, first_offset);
        let segment = Self::open(config, &segment_path)?;

        if segment.first_block_offset != first_offset {
            return Err(Error::Integrity(format!(
                "First block offset != segment first_offset ({} != {})",
                segment.first_block_offset, first_offset
            )));
        }

        Ok(segment)
    }

    pub fn open(
        config: DirectoryChainStoreConfig,
        segment_path: &Path,
    ) -> Result<DirectorySegment, Error> {
        info!("Opening segment at {:?}", segment_path);

        let segment_file = SegmentFile::open(&segment_path, 0)?;

        // read first block to validate it has the same offset as segment
        let first_block = BlockRef::new(&segment_file.mmap[..]).map_err(|err| {
            error!(
                "Couldn't read first block from segment file {:?}: {}",
                segment_path, err
            );
            err
        })?;

        // iterate through segments and find the last block and its offset
        let blocks_iterator = ChainBlockIterator::new(&segment_file.mmap[..]);
        let last_block = blocks_iterator.last().ok_or_else(|| {
            Error::Integrity(
                "Couldn't find last block of segment: no blocks returned by iterator".to_string(),
            )
        })?;

        let next_block_offset = last_block.offset + last_block.total_size() as BlockOffset;
        let next_file_offset = (next_block_offset - first_block.offset) as usize;

        Ok(DirectorySegment {
            config,
            first_block_offset: first_block.offset,
            segment_path: segment_path.to_path_buf(),
            segment_file,
            next_block_offset,
            next_file_offset,
        })
    }

    pub fn offset_range(&self) -> Range<BlockOffset> {
        self.first_block_offset..self.next_block_offset
    }

    #[inline]
    pub fn first_block_offset(&self) -> BlockOffset {
        self.first_block_offset
    }

    #[inline]
    pub fn next_block_offset(&self) -> BlockOffset {
        self.next_block_offset
    }

    #[inline]
    pub fn next_file_offset(&self) -> usize {
        self.next_file_offset
    }

    pub fn write_block<B: Block>(&mut self, block: &B) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset;
        let next_block_offset = self.next_block_offset;
        let block_size = block.total_size();

        let block_offset = block.offset();
        if next_block_offset != block_offset {
            return Err(Error::Integrity(format!(
                "Trying to write a block at an offset that wasn't next offset: next_block_offset={} block_offset={}",
                next_block_offset, block_offset
            )));
        }

        {
            self.ensure_file_size(block_size)?;
            block.copy_data_into(&mut self.segment_file.mmap[next_file_offset..]);
        }

        self.next_file_offset += block_size;
        self.next_block_offset += block_size as BlockOffset;

        Ok(())
    }

    pub fn get_block(&self, offset: BlockOffset) -> Result<BlockRef, Error> {
        let first_block_offset = self.first_block_offset;
        if offset < first_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block at {}, but first offset was at {}",
                offset, first_block_offset
            )));
        }

        if offset >= self.next_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block at {}, but next offset was at {}",
                offset, self.next_block_offset
            )));
        }

        let block_file_offset = (offset - first_block_offset) as usize;
        Ok(BlockRef::new(&self.segment_file.mmap[block_file_offset..])?)
    }

    pub fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<BlockRef, Error> {
        let first_block_offset = self.first_block_offset;
        if next_offset < first_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block from next offset {}, but first offset was at {}",
                next_offset, first_block_offset
            )));
        }

        if next_offset > self.next_block_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block from next offset {}, but next offset was at {}",
                next_offset, self.next_block_offset
            )));
        }

        let next_file_offset = (next_offset - first_block_offset) as usize;
        let block = BlockRef::new_from_next_offset(&self.segment_file.mmap[..], next_file_offset)?;

        Ok(block)
    }

    pub fn truncate_from_block_offset(&mut self, block_offset: BlockOffset) -> Result<(), Error> {
        if block_offset < self.first_block_offset {
            return Err(Error::OutOfBound(format!(
                "Offset {} is before first block offset of segment {}",
                block_offset, self.first_block_offset
            )));
        }
        self.next_block_offset = block_offset;
        let keep_len = block_offset - self.first_block_offset;
        self.segment_file.set_len(keep_len)
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
            if target_size > self.config.segment_max_size {
                return Err(Error::SegmentFull);
            }

            self.segment_file.set_len(target_size)?;
        }

        Ok(())
    }

    #[cfg(test)]
    fn truncate_extra(&mut self) -> Result<(), Error> {
        let next_file_offset = self.next_file_offset as u64;
        self.segment_file.set_len(next_file_offset)
    }
}

/// Wraps a mmap'ed file stored on disk. As mmap cannot access content that is
/// beyond the file size, the segment is over-allocated so that we can write via
/// mmap. If writing would exceed the size, we re-allocate the file and re-open
/// the mmap.
struct SegmentFile {
    path: PathBuf,
    file: File,
    mmap: memmap::MmapMut,
    current_size: u64,
}

impl SegmentFile {
    fn open(path: &Path, minimum_size: u64) -> Result<SegmentFile, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
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

        let mmap = unsafe {
            memmap::MmapOptions::new().map_mut(&file).map_err(|err| {
                Error::new_io(err, format!("Error mmaping segment file {:?}", path))
            })?
        };

        Ok(SegmentFile {
            path: path.to_path_buf(),
            file,
            mmap,
            current_size,
        })
    }

    fn set_len(&mut self, new_size: u64) -> Result<(), Error> {
        // On Windows, we can't resize a file while it's currently being mapped. We
        // close the mmap first by replacing it by an anonymous mmmap.
        if cfg!(target_os = "windows") {
            self.mmap = memmap::MmapOptions::new()
                .len(1)
                .map_anon()
                .map_err(|err| Error::new_io(err, "Error creating anonymous mmap"))?;
        }

        self.file.set_len(new_size).map_err(|err| {
            Error::new_io(
                err,
                format!("Error setting len of segment file {:?}", self.path),
            )
        })?;

        self.mmap = unsafe {
            memmap::MmapOptions::new()
                .map_mut(&self.file)
                .map_err(|err| {
                    Error::new_io(err, format!("Error mmaping segment file {:?}", self.path))
                })?
        };

        self.current_size = new_size;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::create_block;
    use super::*;
    use exocore_core::cell::FullCell;
    use exocore_core::cell::LocalNode;

    #[test]
    fn directory_segment_create_and_open() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;

        let segment_id = 1234;
        let block = create_block(&cell, 1234);

        {
            let segment = DirectorySegment::create(Default::default(), dir.path(), &block)?;
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.next_file_offset as usize, block.total_size());
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
            )?;
            assert_eq!(segment.first_block_offset, 1234);
            assert_eq!(segment.next_file_offset as usize, block.total_size());
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
        let cell = FullCell::generate(local_node);

        let dir = tempfile::tempdir()?;

        {
            let block = create_block(&cell, 1234);
            let _segment = DirectorySegment::create(Default::default(), dir.path(), &block)?;
        }

        {
            let block = create_block(&cell, 1234);
            assert!(DirectorySegment::create(Default::default(), dir.path(), &block).is_err());
        }

        Ok(())
    }

    #[test]
    fn directory_segment_open_invalid() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;

        {
            let segment_path = dir.path().join("some_file");
            std::fs::write(&segment_path, "hello")?;
            assert!(DirectorySegment::open(Default::default(), &segment_path).is_err());
        }

        {
            let segment_path = dir.path().join("some_file");
            std::fs::write(&segment_path, "hello")?;
            assert!(DirectorySegment::open_with_first_offset(
                Default::default(),
                &segment_path,
                100,
            )
            .is_err());
        }

        Ok(())
    }

    #[test]
    fn directory_segment_append_block() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;

        let offset1 = 0;
        let block = create_block(&cell, offset1);
        let mut segment = DirectorySegment::create(Default::default(), dir.path(), &block)?;
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
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let config = Default::default();
        let segment_first_block_offset = 1234;

        {
            let first_block = create_block(&cell, segment_first_block_offset);
            let mut segment = DirectorySegment::create(config, dir.path(), &first_block)?;
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
            )?;
            assert!(segment.get_block(0).is_err());
            assert!(segment.get_block(1234).is_ok());
            assert!(segment.get_block(segment.next_block_offset).is_err());
            assert!(segment
                .get_block_from_next_offset(segment.next_block_offset)
                .is_ok());

            let iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
            assert_eq!(iter.count(), 1000);
        }

        Ok(())
    }

    #[test]
    fn directory_segment_grow_and_truncate() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_over_allocate_size = 100_000;

        let dir = tempfile::tempdir()?;
        let mut next_offset = 0;

        let block = create_block(&cell, next_offset);
        let mut segment = DirectorySegment::create(config, dir.path(), &block)?;
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

        let iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
        assert_eq!(iter.count(), 1000);

        Ok(())
    }

    #[test]
    fn directory_segment_truncate_from_segment() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let cell = FullCell::generate(local_node);
        let dir = tempfile::tempdir()?;
        let mut config: DirectoryChainStoreConfig = Default::default();
        config.segment_over_allocate_size = 100_000;

        let mut next_offset = 1000;

        let block = create_block(&cell, next_offset);
        let mut segment = DirectorySegment::create(config, dir.path(), &block)?;
        next_offset += block.total_size() as u64;
        append_blocks_to_segment(&cell, &mut segment, next_offset, 999);

        // should not remove any blocks, only remove over allocated space
        segment.truncate_from_block_offset(segment.next_block_offset)?;
        let iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
        assert_eq!(iter.count(), 1000);

        // truncating before beginning of file is impossible
        assert!(segment.truncate_from_block_offset(900).is_err());

        // truncating at 10th should result in only 10 blocks left
        let mut iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
        let nth_offset = iter.nth(10).unwrap().offset;
        segment.truncate_from_block_offset(nth_offset)?;
        let iter = ChainBlockIterator::new(&segment.segment_file.mmap[0..]);
        assert_eq!(iter.count(), 10);

        Ok(())
    }

    #[test]
    fn segment_file_create() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let segment_path = dir.path().join("segment_0.seg");

        let segment_file = SegmentFile::open(&segment_path, 1000)?;
        assert_eq!(segment_file.current_size, 1000);
        drop(segment_file);

        let mut segment_file = SegmentFile::open(&segment_path, 10)?;
        assert_eq!(segment_file.current_size, 1000);

        segment_file.set_len(2000)?;
        assert_eq!(segment_file.current_size, 2000);

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
            let block = create_block(&cell, next_offset);
            segment.write_block(&block).unwrap();
            next_offset += block.total_size() as u64;
        }
    }
}
