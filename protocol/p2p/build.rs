fn main() {
    if std::env::var("PROTOC").is_err() {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    }
    let proto_files = &["./proto/messages.proto", "./proto/p2p.proto"];
    let dirs = &["./proto"];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&proto_files[0..1], dirs)
        .unwrap_or_else(|e| panic!("protobuf compilation failed, error: {e}"));
    // recompile protobufs only if any of the proto files changes.
    for file in proto_files {
        println!("cargo:rerun-if-changed={file}");
    }
}
