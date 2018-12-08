extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .file("proto/chain_block.capnp")
        .edition(capnpc::RustEdition::Rust2018)
        .run()
        .expect("compiling schema");
}
