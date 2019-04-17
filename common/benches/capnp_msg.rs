#[macro_use]
extern crate criterion_bencher_compat;

use criterion_bencher_compat::{black_box, Bencher};

use exocore_common::data_chain_capnp::block;
use exocore_common::serialization::framed::{Frame, FrameBuilder, OwnedFrame, SliceFrame};

fn bench_build_message(b: &mut Bencher) {
    let mut data = vec![0; 1000];

    b.iter(|| {
        let mut builder = FrameBuilder::<block::Owned>::new();
        build_test_block(&mut builder);
        let _ = builder.write_into_unsigned(&mut data);
    });
}

fn bench_read_message_from_slice_with_parsing(b: &mut Bencher) {
    let mut builder = FrameBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_unsigned_framed_bytes().unwrap();

    b.iter(|| {
        let message = SliceFrame::new(&data).unwrap();
        let block_reader: block::Reader = message.get_typed_reader::<block::Owned>().unwrap();
        let _ = black_box(block_reader.get_previous_hash());
    });
}

fn bench_read_message_from_slice_no_parsing(b: &mut Bencher) {
    let mut builder = FrameBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_unsigned_framed_bytes().unwrap();

    b.iter(|| {
        let message = SliceFrame::new(&data).unwrap();
        let _ = black_box(message);
    });
}

fn bench_read_message_from_owned_with_parsing(b: &mut Bencher) {
    let mut builder = FrameBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_unsigned_framed_bytes().unwrap();

    b.iter(|| {
        let message = OwnedFrame::new(data.clone()).unwrap();
        let block_reader = message.get_typed_reader::<block::Owned>().unwrap();
        let _ = black_box(block_reader.get_previous_hash());
    });
}

fn bench_read_message_from_owned_no_parsing(b: &mut Bencher) {
    let mut builder = FrameBuilder::<block::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.into_unsigned_framed_bytes().unwrap();

    b.iter(|| {
        let message = OwnedFrame::new(data.clone()).unwrap();
        let _ = black_box(message);
    });
}

fn build_test_block(block_msg_builder: &mut FrameBuilder<block::Owned>) {
    let mut block_builder: block::Builder = block_msg_builder.get_builder_typed();
    block_builder.set_previous_hash(b"prev hash");
}

benchmark_group!(
    benches,
    bench_build_message,
    bench_read_message_from_slice_with_parsing,
    bench_read_message_from_slice_no_parsing,
    bench_read_message_from_owned_with_parsing,
    bench_read_message_from_owned_no_parsing,
);
benchmark_main!(benches);
