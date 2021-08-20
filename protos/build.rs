fn main() {
    if std::env::var("GENERATE_PROTOS").is_ok() {
        let prost_protos_file = vec!["./protobuf/exomind/base.proto"];
        prost_build::Config::new()
            .out_dir("./src/generated")
            .compile_protos(&prost_protos_file, &["./protobuf"])
            .expect("prost error");
    }
}
