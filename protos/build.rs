fn main() {
    let prost_protos_file = vec!["./protobuf/exomind/base.proto"];
    prost_build::compile_protos(&prost_protos_file, &["./protobuf"]).expect("prost error");
}
