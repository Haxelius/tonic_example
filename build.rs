fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tonic_prost_build::compile_protos("proto/helloworld.proto")?;
    //tonic_prost_build::configure().compile_protos(&["./proto/helloworld.proto"], &["./proto/helloworld.proto"])?;
    let proto_path: &std::path::Path = std::path::Path::new("./proto/file_transfer.proto");

    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    tonic_prost_build::configure()
        .out_dir("./proto/output")
        .compile_protos(&[proto_path], &[proto_dir])?;

    Ok(())
}
