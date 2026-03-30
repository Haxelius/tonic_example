fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/bi.stream.proto");
    tonic_prost_build::compile_protos("proto/bi_stream.proto")?;
    Ok(())
}
