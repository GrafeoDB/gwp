//! Integration tests for the `CatalogService` gRPC service.

use std::net::SocketAddr;

use gwp::proto;
use gwp::proto::catalog_service_client::CatalogServiceClient;
use gwp::server::mock_backend::MockBackend;
use gwp::server::{
    CatalogServiceImpl, GqlServiceImpl, SessionManager, SessionServiceImpl, TransactionManager,
};

/// Start a server with all services on a random port.
async fn start_server() -> SocketAddr {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let backend = std::sync::Arc::new(MockBackend::new());
        let sessions = SessionManager::new();
        let transactions = TransactionManager::new();

        let session_svc = SessionServiceImpl::new(
            std::sync::Arc::clone(&backend),
            sessions.clone(),
            transactions.clone(),
            None,
        );
        let gql_svc = GqlServiceImpl::new(std::sync::Arc::clone(&backend), sessions, transactions);
        let catalog_svc = CatalogServiceImpl::new(std::sync::Arc::clone(&backend));

        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

        tonic::transport::Server::builder()
            .add_service(proto::session_service_server::SessionServiceServer::new(
                session_svc,
            ))
            .add_service(proto::gql_service_server::GqlServiceServer::new(gql_svc))
            .add_service(proto::catalog_service_server::CatalogServiceServer::new(
                catalog_svc,
            ))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

async fn connect(addr: SocketAddr) -> CatalogServiceClient<tonic::transport::Channel> {
    let channel = tonic::transport::Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();

    CatalogServiceClient::new(channel)
}

// =========================================================================
// Schema tests
// =========================================================================

#[tokio::test]
async fn list_schemas() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .list_schemas(proto::ListSchemasRequest {})
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.schemas.len(), 1);
    assert_eq!(resp.schemas[0].name, "default");
    assert_eq!(resp.schemas[0].graph_count, 2);
}

#[tokio::test]
async fn create_schema() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    client
        .create_schema(proto::CreateSchemaRequest {
            name: "analytics".to_owned(),
            if_not_exists: false,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn create_schema_already_exists() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .create_schema(proto::CreateSchemaRequest {
            name: "default".to_owned(),
            if_not_exists: false,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn create_schema_if_not_exists() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    // Should succeed even though "default" already exists
    client
        .create_schema(proto::CreateSchemaRequest {
            name: "default".to_owned(),
            if_not_exists: true,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn drop_schema_default_fails() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .drop_schema(proto::DropSchemaRequest {
            name: "default".to_owned(),
            if_exists: false,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

// =========================================================================
// Graph tests
// =========================================================================

#[tokio::test]
async fn list_graphs() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .list_graphs(proto::ListGraphsRequest {
            schema: "default".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.graphs.len(), 2);
    assert_eq!(resp.graphs[0].name, "default");
    assert_eq!(resp.graphs[0].node_count, 100);
    assert_eq!(resp.graphs[0].edge_count, 50);
    assert_eq!(resp.graphs[1].name, "test");
    assert_eq!(resp.graphs[1].node_count, 10);
    assert_eq!(resp.graphs[1].edge_count, 5);
}

#[tokio::test]
async fn create_graph() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .create_graph(proto::CreateGraphRequest {
            schema: "default".to_owned(),
            name: "bench".to_owned(),
            if_not_exists: false,
            or_replace: false,
            type_spec: None,
            copy_of: None,
            storage_mode: "InMemory".to_owned(),
            options: None,
        })
        .await
        .unwrap()
        .into_inner();

    let graph = resp.graph.unwrap();
    assert_eq!(graph.schema, "default");
    assert_eq!(graph.name, "bench");
    assert_eq!(graph.node_count, 0);
    assert_eq!(graph.edge_count, 0);
}

#[tokio::test]
async fn create_graph_with_options() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .create_graph(proto::CreateGraphRequest {
            schema: "default".to_owned(),
            name: "mydb".to_owned(),
            if_not_exists: false,
            or_replace: false,
            type_spec: Some(proto::create_graph_request::TypeSpec::OpenType(true)),
            copy_of: None,
            storage_mode: "Persistent".to_owned(),
            options: Some(proto::GraphOptions {
                memory_limit_bytes: Some(1024 * 1024),
                backward_edges: Some(true),
                threads: Some(4),
                wal_enabled: Some(true),
                wal_durability: Some("fsync".to_owned()),
            }),
        })
        .await
        .unwrap()
        .into_inner();

    let graph = resp.graph.unwrap();
    assert_eq!(graph.name, "mydb");
}

#[tokio::test]
async fn create_graph_already_exists() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .create_graph(proto::CreateGraphRequest {
            schema: "default".to_owned(),
            name: "default".to_owned(),
            if_not_exists: false,
            or_replace: false,
            type_spec: None,
            copy_of: None,
            storage_mode: "InMemory".to_owned(),
            options: None,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn create_graph_empty_name() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .create_graph(proto::CreateGraphRequest {
            schema: "default".to_owned(),
            name: String::new(),
            if_not_exists: false,
            or_replace: false,
            type_spec: None,
            copy_of: None,
            storage_mode: "InMemory".to_owned(),
            options: None,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn drop_graph() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .drop_graph(proto::DropGraphRequest {
            schema: "default".to_owned(),
            name: "test".to_owned(),
            if_exists: false,
        })
        .await
        .unwrap()
        .into_inner();

    assert!(resp.existed);
}

#[tokio::test]
async fn drop_default_graph_fails() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .drop_graph(proto::DropGraphRequest {
            schema: "default".to_owned(),
            name: "default".to_owned(),
            if_exists: false,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn get_graph_info() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .get_graph_info(proto::GetGraphInfoRequest {
            schema: "default".to_owned(),
            name: "default".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.name, "default");
    assert_eq!(resp.node_count, 100);
    assert_eq!(resp.edge_count, 50);
    assert_eq!(resp.storage_mode, "InMemory");
}

#[tokio::test]
async fn get_graph_info_not_found() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .get_graph_info(proto::GetGraphInfoRequest {
            schema: "default".to_owned(),
            name: "nonexistent".to_owned(),
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

// =========================================================================
// Graph type tests
// =========================================================================

#[tokio::test]
async fn list_graph_types() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .list_graph_types(proto::ListGraphTypesRequest {
            schema: "default".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.graph_types.len(), 1);
    assert_eq!(resp.graph_types[0].name, "PersonGraph");
}

#[tokio::test]
async fn create_graph_type() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    client
        .create_graph_type(proto::CreateGraphTypeRequest {
            schema: "default".to_owned(),
            name: "SocialGraph".to_owned(),
            if_not_exists: false,
            or_replace: false,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn create_graph_type_already_exists() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .create_graph_type(proto::CreateGraphTypeRequest {
            schema: "default".to_owned(),
            name: "PersonGraph".to_owned(),
            if_not_exists: false,
            or_replace: false,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn drop_graph_type() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .drop_graph_type(proto::DropGraphTypeRequest {
            schema: "default".to_owned(),
            name: "PersonGraph".to_owned(),
            if_exists: false,
        })
        .await
        .unwrap()
        .into_inner();

    assert!(resp.existed);
}
