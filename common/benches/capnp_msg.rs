#![feature(test)]
extern crate test;

use test::Bencher;

use tempdir;

use exocore_common::chain_block_capnp::block;
use exocore_common::serialization::msg::{FramedMessage, FramedSliceMessage, MessageBuilder};

#[bench]
fn bench_build_message(b: &mut Bencher) {
    let mut data = vec![0; 1000];

    b.iter(|| {
        let mut builder = MessageBuilder::<block::Owned>::new();
        build_test_block(&mut builder);
        let _ = builder.write_into(&mut data);
    });
}

#[bench]
fn bench_read_message(b: &mut Bencher) {
    let mut builder = MessageBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_framed_vec().unwrap();

    b.iter(|| {
        let message = FramedSliceMessage::new(&data).unwrap();
        let block_reader = message.get_typed_reader::<block::Owned>().unwrap();
        let _ = test::black_box(block_reader.get_hash());
    });
}

#[bench]
fn bench_write_file_sequential(b: &mut Bencher) {
    let (_tempdir, _file, mut mmap) = create_test_file(1024 * 1024 * 1024);

    let mut next_offset = 0;
    b.iter(|| {
        let mut builder = MessageBuilder::<block::Owned>::new();
        build_test_block(&mut builder);

        let data_size = builder.write_into(&mut mmap[next_offset..]).unwrap();
        next_offset += data_size;
    });
}

#[bench]
fn bench_read_file_sequential(b: &mut Bencher) {
    let (_tempdir, _file, mut mmap) = create_test_file(1024 * 1024 * 1024);

    let mut next_offset = 0;
    for _i in 0..100_000 {
        let mut builder = MessageBuilder::<block::Owned>::new();
        build_test_block(&mut builder);

        let data_size = builder.write_into(&mut mmap[next_offset..]).unwrap();
        next_offset += data_size;
    }

    let max_offset = next_offset;
    let mut next_offset = 0;
    b.iter(|| {
        let framed_message = FramedSliceMessage::new(&mmap[next_offset..]).unwrap();
        let block_reader = framed_message.get_typed_reader::<block::Owned>().unwrap();
        let _ = test::black_box(block_reader.get_hash());

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

pub fn build_test_block(builder: &mut MessageBuilder<block::Owned>) {
    let mut block = builder.get_builder_typed();
    block.set_hash("block_hash");
    let mut entries = block.init_entries(1);
    {
        let mut entry = entries.reborrow().get(0);
        entry.set_hash("entry_hash");
    }
}
