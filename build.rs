fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/gql_types.proto", "proto/gql_service.proto"], &["proto"])?;
    Ok(())
}
