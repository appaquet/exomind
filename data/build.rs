extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .file("proto/chain_block.capnp")
        .run()
        .expect("compiling schema");
}