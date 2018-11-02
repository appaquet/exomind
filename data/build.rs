extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .file("fbs/test.capnp")
        .run()
        .expect("compiling schema");
}