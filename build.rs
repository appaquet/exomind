fn main() {
    let prost_protos_file = vec!["./protos/exomind/base.proto"];
    prost_build::compile_protos(&prost_protos_file, &["./protos", "../exocore/protos/"])
        .expect("prost error");
}
