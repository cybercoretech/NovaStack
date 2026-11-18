use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the bundled protoc binary unless the caller provides one.
    if std::env::var_os("PROTOC").is_none() {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    tonic_prost_build::configure()
        // Descriptor set for gRPC server reflection (used by grpcurl, Postman, ...)
        .file_descriptor_set_path(out_dir.join("greeter_descriptor.bin"))
        .compile_protos(&["proto/greeter.proto"], &["proto"])?;
    Ok(())
}
