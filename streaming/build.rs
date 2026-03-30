fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_path: &std::path::Path = std::path::Path::new("./proto/number_stream.proto");

    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    tonic_prost_build::configure()
        .out_dir("./proto/output")
        .compile_protos(&[proto_path], &[proto_dir])?;

    Ok(())
}
