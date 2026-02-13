//! Integration tests for the high-level client API.

use std::collections::HashMap;
use std::net::SocketAddr;

use gql_wire_protocol::client::GqlConnection;
use gql_wire_protocol::proto;
use gql_wire_protocol::server::{
    GqlServiceImpl, SessionManager, SessionServiceImpl, TransactionManager,
};
use gql_wire_protocol::server::mock_backend::MockBackend;
use gql_wire_protocol::types::Value;

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
        let gql_svc = GqlServiceImpl::new(backend, sessions, transactions);

        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

        tonic::transport::Server::builder()
            .add_service(
                proto::session_service_server::SessionServiceServer::new(session_svc),
            )
            .add_service(proto::gql_service_server::GqlServiceServer::new(gql_svc))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

#[tokio::test]
async fn client_session_lifecycle() {
    let addr = start_server().await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let mut session = conn.create_session().await.unwrap();
    assert!(!session.session_id().is_empty());

    // Ping
    let ts = session.ping().await.unwrap();
    assert!(ts > 0);

    // Configure
    session.set_graph("test_graph").await.unwrap();
    session.set_schema("test_schema").await.unwrap();
    session.set_time_zone(60).await.unwrap();

    // Reset
    session.reset().await.unwrap();

    // Close
    session.close().await.unwrap();
}

#[tokio::test]
async fn client_execute_query() {
    let addr = start_server().await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let mut session = conn.create_session().await.unwrap();

    let mut cursor = session
        .execute("MATCH (p:Person) RETURN p.name, p.age", HashMap::new())
        .await
        .unwrap();

    // Check columns
    let cols = cursor.column_names().await.unwrap();
    assert_eq!(cols, vec!["name", "age"]);

    // Collect rows
    let rows = cursor.collect_rows().await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], Value::String("Alice".to_owned()));
    assert_eq!(rows[0][1], Value::Integer(30));
    assert_eq!(rows[1][0], Value::String("Bob".to_owned()));
    assert_eq!(rows[1][1], Value::Integer(25));

    // Check summary
    assert!(cursor.is_success().await.unwrap());
}

#[tokio::test]
async fn client_transaction() {
    let addr = start_server().await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let mut session = conn.create_session().await.unwrap();

    // Begin transaction
    let mut tx = session.begin_transaction().await.unwrap();
    assert!(!tx.transaction_id().is_empty());

    // Execute within transaction
    let mut cursor = tx
        .execute("INSERT (:Person {name: 'Carol'})", HashMap::new())
        .await
        .unwrap();

    // Consume and check
    let _ = cursor.collect_rows().await.unwrap();
    assert!(cursor.is_success().await.unwrap());

    // Commit
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn client_transaction_rollback() {
    let addr = start_server().await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let mut session = conn.create_session().await.unwrap();

    let tx = session.begin_transaction().await.unwrap();
    tx.rollback().await.unwrap();
}
