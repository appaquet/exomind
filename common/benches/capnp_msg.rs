#![feature(test)]
extern crate test;

use test::Bencher;

use tempdir;

use exocore_common::chain_block_capnp::block;
use exocore_common::serialization::msg::{
    write_framed_builder_into_buffer, FramedMessage, FramedSliceMessage,
};

#[bench]
fn bench_write_sequential(b: &mut Bencher) {
    let (_tempdir, _file, mut mmap) = create_test_file(1024 * 1024 * 1024);

    let mut next_offset = 0;
    b.iter(|| {
        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);
        let (_written_size, data_size) =
            write_framed_builder_into_buffer(&mut mmap[next_offset..], 123, &message_builder)
                .unwrap();
        next_offset += data_size;
    })
}

#[bench]
fn bench_read_sequential(b: &mut Bencher) {
    let (_tempdir, _file, mut mmap) = create_test_file(1024 * 1024 * 1024);

    let mut next_offset = 0;
    for _i in 0..100_000 {
        let mut message_builder = capnp::message::Builder::new_default();
        build_test_block(&mut message_builder);
        let (_written_size, data_size) =
            write_framed_builder_into_buffer(&mut mmap[next_offset..], 123, &message_builder)
                .unwrap();
        next_offset += data_size;
    }

    let max_offset = next_offset;
    let mut next_offset = 0;
    b.iter(|| {
        let framed_message = FramedSliceMessage::new(&mmap[next_offset..]).unwrap();
        let block_reader = framed_message.get_root::<block::Reader>().unwrap();
        let _ = block_reader.get_hash();

        next_offset += framed_message.data_size();
        if next_offset >= max_offset {
            next_offset = 0;
        }
    })
}

fn create_test_file(size: u64) -> (tempdir::TempDir, std::fs::File, memmap::MmapMut) {
    let dir = tempdir::TempDir::new("test").unwrap();
    let segment_path = dir.path().join("segment");

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(segment_path)
        .unwrap();

    file.set_len(size).unwrap();

    let mmap = unsafe { memmap::MmapOptions::new().map_mut(&file).unwrap() };

    (dir, file, mmap)
}

pub fn build_test_block<A: capnp::message::Allocator>(
    message_builder: &mut capnp::message::Builder<A>,
) {
    let mut block = message_builder.init_root::<block::Builder>();
    block.set_hash("block_hash");
    let mut entries = block.init_entries(1);
    {
        let mut entry = entries.reborrow().get(0);
        entry.set_hash("entry_hash");
    }
}
