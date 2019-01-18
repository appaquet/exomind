#![feature(test)]
extern crate test;

use test::Bencher;

use tempdir;

use exocore_common::data_chain_capnp::{block, entry_header};
use exocore_common::serialization::msg::{
    FramedMessage, FramedOwnedMessage, FramedSliceMessage, MessageBuilder,
};

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
fn bench_read_message_from_slice_with_parsing(b: &mut Bencher) {
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
fn bench_read_message_from_slice_no_parsing(b: &mut Bencher) {
    let mut builder = MessageBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_framed_vec().unwrap();

    b.iter(|| {
        let message = FramedSliceMessage::new(&data).unwrap();
        let _ = test::black_box(message);
    });
}

#[bench]
fn bench_read_message_from_owned_with_parsing(b: &mut Bencher) {
    let mut builder = MessageBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_framed_vec().unwrap();

    b.iter(|| {
        let message = FramedOwnedMessage::new(data.clone()).unwrap();
        let block_reader = message.get_typed_reader::<block::Owned>().unwrap();
        let _ = test::black_box(block_reader.get_hash());
    });
}

#[bench]
fn bench_read_message_from_owned_no_parsing(b: &mut Bencher) {
    let mut builder = MessageBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_framed_vec().unwrap();

    b.iter(|| {
        let message = FramedOwnedMessage::new(data.clone()).unwrap();
        let _ = test::black_box(message);
    });
}

fn build_test_block(block_msg_builder: &mut MessageBuilder<block::Owned>) {
    let mut block_builder = block_msg_builder.get_builder_typed();
    block_builder.set_hash("block_hash");

    let mut entries = block_builder.init_entries(1);
    {
        let mut entry = entries.reborrow().get(0);

        let mut entry_header_msg_builder = MessageBuilder::<entry_header::Owned>::new();
        let mut header_builder = entry_header_msg_builder.get_builder_typed();
        header_builder.set_hash("entry_hash");

        entry.set_header(header_builder.into_reader()).unwrap();
    }
}
