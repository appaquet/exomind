#[macro_use]
extern crate criterion_bencher_compat;

use criterion_bencher_compat::{black_box, Bencher};
use exocore_core::framing::{CapnpFrameBuilder, FrameBuilder, TypedCapnpFrame};
use exocore_protos::generated::data_chain_capnp::block_header;

fn bench_build_message(b: &mut Bencher) {
    let mut data = vec![0; 1000];

    b.iter(|| {
        let mut builder = CapnpFrameBuilder::<block_header::Owned>::new();
        build_test_block(&mut builder);
        let _ = builder.write_into(&mut data);
    });
}

fn bench_read_message_from_slice_with_parsing(b: &mut Bencher) {
    let mut builder = CapnpFrameBuilder::<block_header::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.as_bytes();

    b.iter(|| {
        let message = TypedCapnpFrame::<_, block_header::Owned>::new(data.as_ref()).unwrap();
        let header_reader = message.get_reader().unwrap();
        let _ = black_box(header_reader.get_previous_hash());
    });
}

fn bench_read_message_from_slice_no_parsing(b: &mut Bencher) {
    let mut builder = CapnpFrameBuilder::<block_header::Owned>::new();
    build_test_block(&mut builder);
    let data = builder.as_bytes();

    b.iter(|| {
        let message = TypedCapnpFrame::<_, block_header::Owned>::new(data.as_ref()).unwrap();
        let _ = black_box(message);
    });
}

fn build_test_block(block_msg_builder: &mut CapnpFrameBuilder<block_header::Owned>) {
    let mut block_builder: block_header::Builder = block_msg_builder.get_builder();
    block_builder.set_previous_hash(b"prev hash");
}

benchmark_group!(
    benches,
    bench_build_message,
    bench_read_message_from_slice_with_parsing,
    bench_read_message_from_slice_no_parsing,
);
benchmark_main!(benches);
