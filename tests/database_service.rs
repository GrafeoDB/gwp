//! Integration tests for the `DatabaseService` gRPC service.

use std::net::SocketAddr;

use gwp::proto;
use gwp::proto::database_service_client::DatabaseServiceClient;
use gwp::server::mock_backend::MockBackend;
use gwp::server::{
    DatabaseServiceImpl, GqlServiceImpl, SessionManager, SessionServiceImpl, TransactionManager,
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
        );
        let gql_svc =
            GqlServiceImpl::new(std::sync::Arc::clone(&backend), sessions, transactions);
        let db_svc = DatabaseServiceImpl::new(std::sync::Arc::clone(&backend));

        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

        tonic::transport::Server::builder()
            .add_service(proto::session_service_server::SessionServiceServer::new(
                session_svc,
            ))
            .add_service(proto::gql_service_server::GqlServiceServer::new(gql_svc))
            .add_service(proto::database_service_server::DatabaseServiceServer::new(
                db_svc,
            ))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

async fn connect(addr: SocketAddr) -> DatabaseServiceClient<tonic::transport::Channel> {
    let channel = tonic::transport::Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();

    DatabaseServiceClient::new(channel)
}

#[tokio::test]
async fn list_databases() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .list_databases(proto::ListDatabasesRequest {})
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.databases.len(), 2);
    assert_eq!(resp.databases[0].name, "default");
    assert_eq!(resp.databases[0].node_count, 100);
    assert_eq!(resp.databases[0].edge_count, 50);
    assert_eq!(resp.databases[0].database_type, "Lpg");
    assert_eq!(resp.databases[1].name, "test");
    assert_eq!(resp.databases[1].node_count, 10);
    assert_eq!(resp.databases[1].edge_count, 5);
}

#[tokio::test]
async fn create_database() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .create_database(proto::CreateDatabaseRequest {
            name: "bench".to_owned(),
            database_type: "Lpg".to_owned(),
            storage_mode: "InMemory".to_owned(),
            options: None,
        })
        .await
        .unwrap()
        .into_inner();

    let db = resp.database.unwrap();
    assert_eq!(db.name, "bench");
    assert_eq!(db.node_count, 0);
    assert_eq!(db.edge_count, 0);
    assert_eq!(db.database_type, "Lpg");
    assert!(!db.persistent);
}

#[tokio::test]
async fn create_database_with_options() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .create_database(proto::CreateDatabaseRequest {
            name: "mydb".to_owned(),
            database_type: "Lpg".to_owned(),
            storage_mode: "Persistent".to_owned(),
            options: Some(proto::DatabaseOptions {
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

    let db = resp.database.unwrap();
    assert_eq!(db.name, "mydb");
    assert!(db.persistent);
}

#[tokio::test]
async fn create_database_already_exists() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .create_database(proto::CreateDatabaseRequest {
            name: "default".to_owned(),
            database_type: "Lpg".to_owned(),
            storage_mode: "InMemory".to_owned(),
            options: None,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn create_database_empty_name() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .create_database(proto::CreateDatabaseRequest {
            name: String::new(),
            database_type: "Lpg".to_owned(),
            storage_mode: "InMemory".to_owned(),
            options: None,
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn delete_database() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .delete_database(proto::DeleteDatabaseRequest {
            name: "test".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.deleted, "test");
}

#[tokio::test]
async fn delete_default_database_fails() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .delete_database(proto::DeleteDatabaseRequest {
            name: "default".to_owned(),
        })
        .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code(),
        tonic::Code::InvalidArgument
    );
}

#[tokio::test]
async fn get_database_info() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let resp = client
        .get_database_info(proto::GetDatabaseInfoRequest {
            name: "default".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.name, "default");
    assert_eq!(resp.node_count, 100);
    assert_eq!(resp.edge_count, 50);
    assert_eq!(resp.database_type, "Lpg");
    assert_eq!(resp.storage_mode, "InMemory");
    assert!(!resp.persistent);
}

#[tokio::test]
async fn get_database_info_not_found() {
    let addr = start_server().await;
    let mut client = connect(addr).await;

    let result = client
        .get_database_info(proto::GetDatabaseInfoRequest {
            name: "nonexistent".to_owned(),
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}
