fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("gql_descriptor.bin"))
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["proto/gql_types.proto", "proto/gql_service.proto"],
            &["proto"],
        )?;
    Ok(())
}
