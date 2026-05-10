fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/blog.proto");
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&["proto/blog.proto"], &["proto"])?;
    Ok(())
}
