//! Integration tests: start a real gRPC server with `MockBackend`,
//! connect with tonic clients, and verify the full round-trip.

use std::collections::HashMap;
use std::net::SocketAddr;

use gwp::proto;
use gwp::proto::gql_service_client::GqlServiceClient;
use gwp::proto::session_service_client::SessionServiceClient;
use gwp::server::mock_backend::MockBackend;
use gwp::server::{
    GqlServiceImpl, SessionManager, SessionServiceImpl, TransactionManager,
};
use gwp::status;

/// Start a server on a random port and return the address.
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
            .add_service(proto::session_service_server::SessionServiceServer::new(
                session_svc,
            ))
            .add_service(proto::gql_service_server::GqlServiceServer::new(gql_svc))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

/// Helper to connect clients to a running server.
async fn connect(
    addr: SocketAddr,
) -> (
    SessionServiceClient<tonic::transport::Channel>,
    GqlServiceClient<tonic::transport::Channel>,
) {
    let channel = tonic::transport::Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();

    let session_client = SessionServiceClient::new(channel.clone());
    let gql_client = GqlServiceClient::new(channel);

    (session_client, gql_client)
}

/// Perform a handshake and return the `session_id`.
async fn handshake(client: &mut SessionServiceClient<tonic::transport::Channel>) -> String {
    let resp = client
        .handshake(proto::HandshakeRequest {
            protocol_version: 1,
            credentials: None,
            client_info: HashMap::new(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.protocol_version, 1);
    assert!(!resp.session_id.is_empty());
    assert!(resp.server_info.is_some());

    let info = resp.server_info.unwrap();
    assert_eq!(info.name, "gql-wire-protocol");

    resp.session_id
}

#[tokio::test]
async fn handshake_and_close() {
    let addr = start_server().await;
    let (mut session_client, _) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    // Close should succeed
    session_client
        .close(proto::CloseRequest {
            session_id: session_id.clone(),
        })
        .await
        .unwrap();

    // After close, ping should fail with NOT_FOUND
    let result = session_client
        .ping(proto::PingRequest {
            session_id: session_id.clone(),
        })
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn ping() {
    let addr = start_server().await;
    let (mut session_client, _) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    let pong = session_client
        .ping(proto::PingRequest {
            session_id: session_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(pong.timestamp > 0);
}

#[tokio::test]
async fn configure_and_reset() {
    let addr = start_server().await;
    let (mut session_client, _) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    // Configure graph
    session_client
        .configure(proto::ConfigureRequest {
            session_id: session_id.clone(),
            property: Some(proto::configure_request::Property::Graph(
                "my_graph".to_owned(),
            )),
        })
        .await
        .unwrap();

    // Reset
    session_client
        .reset(proto::ResetRequest {
            session_id: session_id.clone(),
            target: proto::ResetTarget::ResetAll.into(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn execute_query_streaming() {
    let addr = start_server().await;
    let (mut session_client, mut gql_client) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    let mut stream = gql_client
        .execute(proto::ExecuteRequest {
            session_id: session_id.clone(),
            statement: "MATCH (p:Person) RETURN p.name, p.age".to_owned(),
            parameters: HashMap::new(),
            transaction_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    // Frame 1: header
    let msg = stream.message().await.unwrap().unwrap();
    let header = match msg.frame {
        Some(proto::execute_response::Frame::Header(h)) => h,
        other => panic!("expected header, got {other:?}"),
    };
    assert_eq!(header.result_type(), proto::ResultType::BindingTable);
    assert_eq!(header.columns.len(), 2);
    assert_eq!(header.columns[0].name, "name");
    assert_eq!(header.columns[1].name, "age");

    // Frame 2: row batch
    let msg = stream.message().await.unwrap().unwrap();
    let batch = match msg.frame {
        Some(proto::execute_response::Frame::RowBatch(b)) => b,
        other => panic!("expected row batch, got {other:?}"),
    };
    assert_eq!(batch.rows.len(), 2);

    // Frame 3: summary
    let msg = stream.message().await.unwrap().unwrap();
    let summary = match msg.frame {
        Some(proto::execute_response::Frame::Summary(s)) => s,
        other => panic!("expected summary, got {other:?}"),
    };
    let code = &summary.status.as_ref().unwrap().code;
    assert!(status::is_success(code));

    // Stream should end
    assert!(stream.message().await.unwrap().is_none());
}

#[tokio::test]
async fn execute_ddl() {
    let addr = start_server().await;
    let (mut session_client, mut gql_client) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    let mut stream = gql_client
        .execute(proto::ExecuteRequest {
            session_id: session_id.clone(),
            statement: "CREATE GRAPH my_graph".to_owned(),
            parameters: HashMap::new(),
            transaction_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    // Header with OMITTED type
    let msg = stream.message().await.unwrap().unwrap();
    let header = match msg.frame {
        Some(proto::execute_response::Frame::Header(h)) => h,
        other => panic!("expected header, got {other:?}"),
    };
    assert_eq!(header.result_type(), proto::ResultType::Omitted);

    // Summary
    let msg = stream.message().await.unwrap().unwrap();
    let summary = match msg.frame {
        Some(proto::execute_response::Frame::Summary(s)) => s,
        other => panic!("expected summary, got {other:?}"),
    };
    assert_eq!(
        summary.status.as_ref().unwrap().code,
        status::OMITTED_RESULT
    );
}

#[tokio::test]
async fn execute_error() {
    let addr = start_server().await;
    let (mut session_client, mut gql_client) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    let mut stream = gql_client
        .execute(proto::ExecuteRequest {
            session_id: session_id.clone(),
            statement: "ERROR this should fail".to_owned(),
            parameters: HashMap::new(),
            transaction_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();

    // Error should come as a summary with GQLSTATUS, not a gRPC error
    let msg = stream.message().await.unwrap().unwrap();
    let summary = match msg.frame {
        Some(proto::execute_response::Frame::Summary(s)) => s,
        other => panic!("expected summary, got {other:?}"),
    };
    let code = &summary.status.as_ref().unwrap().code;
    assert!(status::is_exception(code));
    assert_eq!(code, status::INVALID_SYNTAX);
}

#[tokio::test]
async fn transaction_lifecycle() {
    let addr = start_server().await;
    let (mut session_client, mut gql_client) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    // Begin
    let begin_resp = gql_client
        .begin_transaction(proto::BeginRequest {
            session_id: session_id.clone(),
            mode: proto::TransactionMode::ReadWrite.into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(!begin_resp.transaction_id.is_empty());
    assert!(status::is_success(
        &begin_resp.status.as_ref().unwrap().code
    ));

    let tx_id = begin_resp.transaction_id;

    // Execute within transaction
    let mut stream = gql_client
        .execute(proto::ExecuteRequest {
            session_id: session_id.clone(),
            statement: "INSERT (:Person {name: 'Carol'})".to_owned(),
            parameters: HashMap::new(),
            transaction_id: tx_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    // Consume the stream
    while let Some(_msg) = stream.message().await.unwrap() {}

    // Commit
    let commit_resp = gql_client
        .commit(proto::CommitRequest {
            session_id: session_id.clone(),
            transaction_id: tx_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(status::is_success(
        &commit_resp.status.as_ref().unwrap().code
    ));
}

#[tokio::test]
async fn transaction_rollback() {
    let addr = start_server().await;
    let (mut session_client, mut gql_client) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    // Begin
    let begin_resp = gql_client
        .begin_transaction(proto::BeginRequest {
            session_id: session_id.clone(),
            mode: proto::TransactionMode::ReadWrite.into(),
        })
        .await
        .unwrap()
        .into_inner();

    let tx_id = begin_resp.transaction_id;

    // Rollback
    let rollback_resp = gql_client
        .rollback(proto::RollbackRequest {
            session_id: session_id.clone(),
            transaction_id: tx_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(status::is_success(
        &rollback_resp.status.as_ref().unwrap().code
    ));
}

#[tokio::test]
async fn double_begin_returns_gqlstatus_error() {
    let addr = start_server().await;
    let (mut session_client, mut gql_client) = connect(addr).await;

    let session_id = handshake(&mut session_client).await;

    // First begin
    gql_client
        .begin_transaction(proto::BeginRequest {
            session_id: session_id.clone(),
            mode: proto::TransactionMode::ReadWrite.into(),
        })
        .await
        .unwrap();

    // Second begin should return GQLSTATUS error, not gRPC error
    let begin2 = gql_client
        .begin_transaction(proto::BeginRequest {
            session_id: session_id.clone(),
            mode: proto::TransactionMode::ReadOnly.into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert!(begin2.transaction_id.is_empty());
    assert!(status::is_exception(&begin2.status.as_ref().unwrap().code));
}

#[tokio::test]
async fn invalid_session_returns_grpc_not_found() {
    let addr = start_server().await;
    let (_, mut gql_client) = connect(addr).await;

    let result = gql_client
        .execute(proto::ExecuteRequest {
            session_id: "nonexistent".to_owned(),
            statement: "MATCH (n) RETURN n".to_owned(),
            parameters: HashMap::new(),
            transaction_id: String::new(),
        })
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}
