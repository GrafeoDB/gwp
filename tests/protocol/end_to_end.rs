//! End to end over a real socket: the `GqlServer` builder (with
//! `serve_with_listener`) and the high-level Rust client.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use gwp::client::GqlConnection;
use gwp::error::GqlError;
use gwp::proto;
use gwp::proto::auth_credentials::Method;
use gwp::server::{AuthInfo, AuthValidator, CreateGraphConfig, GqlServer};
use gwp::status;
use gwp::types::{Counters, Value};

use crate::common::{self, TestBackend};

/// Start `server` on a fresh listener and return its address.
async fn serve(server: GqlServer<TestBackend>) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { server.serve_with_listener(listener).await.unwrap() });
    addr
}

fn grpc_code(err: GqlError) -> tonic::Code {
    match err {
        GqlError::Grpc(status) => status.code(),
        other => panic!("expected a gRPC error, got {other:?}"),
    }
}

#[tokio::test]
async fn full_client_flow() {
    let backend = TestBackend::new();
    let addr = serve(GqlServer::builder(backend.clone()).max_sessions(8)).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    // Session setup.
    let mut session = conn.create_session().await.unwrap();
    assert!(session.ping().await.unwrap() > 0);
    session.set_schema("default").await.unwrap();
    session.set_graph("default").await.unwrap();
    session.set_time_zone(120).await.unwrap();
    session.set_parameter("language", "gql").await.unwrap();

    // A query with parameters.
    let parameters = HashMap::from([
        ("a".to_owned(), Value::Integer(42)),
        ("b".to_owned(), Value::from("text")),
    ]);
    let mut cursor = session.execute("ECHO", parameters).await.unwrap();
    assert_eq!(cursor.column_names().await.unwrap(), vec!["a", "b"]);
    let rows = cursor.collect_rows().await.unwrap();
    assert_eq!(rows, vec![vec![Value::Integer(42), Value::from("text")]]);
    assert!(cursor.is_success().await.unwrap());

    // Write counters, typed.
    let mut cursor = session.execute_simple("WRITE").await.unwrap();
    let counters = cursor.counters().await.unwrap();
    let mut expected = Counters::default();
    expected.nodes_created = 2;
    expected.properties_set = 5;
    expected.labels_added = 2;
    assert_eq!(counters, expected);
    assert!(counters.contains_updates());
    // Non-counter entries stay in the raw map.
    let summary = cursor.summary().await.unwrap().unwrap();
    assert_eq!(summary.counters["execution_time_ms"], 7);

    // A read has no write counters.
    let mut cursor = session.execute_simple("ROWS 3").await.unwrap();
    assert_eq!(cursor.collect_rows().await.unwrap().len(), 3);
    assert!(!cursor.counters().await.unwrap().contains_updates());

    // An error is reported in the summary, not as a transport error.
    let mut cursor = session.execute_simple("FAIL").await.unwrap();
    assert!(!cursor.is_success().await.unwrap());

    // Transactions.
    let mut tx = session.begin_transaction().await.unwrap();
    let mut cursor = tx.execute_simple("WRITE").await.unwrap();
    assert!(cursor.is_success().await.unwrap());
    tx.commit().await.unwrap();

    let tx = session.begin_transaction().await.unwrap();
    let tx_id = tx.transaction_id().to_owned();
    drop(tx); // rolled back in the background
    let id = session.session_id().to_owned();
    common::eventually("rollback on drop", || {
        backend.events().contains(&format!("rollback {id} {tx_id}"))
    })
    .await;

    // Catalog through the same connection.
    let mut catalog = conn.create_catalog_client();
    catalog.create_schema("e2e", true).await.unwrap();
    let mut config = CreateGraphConfig {
        schema: "e2e".to_owned(),
        name: "g".to_owned(),
        if_not_exists: false,
        or_replace: false,
        type_spec: None,
        copy_of: None,
        storage_mode: String::new(),
        memory_limit_bytes: None,
        backward_edges: None,
        threads: None,
        wal_enabled: None,
        wal_durability: None,
    };
    catalog.create_graph(config.clone()).await.unwrap();
    config.if_not_exists = true;
    catalog.create_graph(config).await.unwrap();
    assert_eq!(catalog.list_graphs("e2e").await.unwrap().len(), 1);

    // Admin and search are optional: this backend does not implement them.
    let mut admin = conn.create_admin_client();
    assert_eq!(
        grpc_code(admin.get_stats("g").await.unwrap_err()),
        tonic::Code::Unimplemented
    );

    // Reset and close.
    session.reset().await.unwrap();
    session.close().await.unwrap();
    assert!(!backend.has_session(&id));

    // A closed session is gone.
    let mut sessions =
        proto::session_service_client::SessionServiceClient::new(conn.channel().clone());
    let err = sessions
        .ping(proto::PingRequest { session_id: id })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn health_service_reports_serving() {
    let addr = serve(GqlServer::builder(TestBackend::new())).await;
    let channel = tonic::transport::Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut health = tonic_health::pb::health_client::HealthClient::new(channel);
    let response = health
        .check(tonic_health::pb::HealthCheckRequest {
            service: "gql.GqlService".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response.status(),
        tonic_health::pb::health_check_response::ServingStatus::Serving
    );
}

#[tokio::test]
async fn session_limit_is_enforced() {
    let backend = TestBackend::new();
    let addr = serve(GqlServer::builder(backend.clone()).max_sessions(2)).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let first = conn.create_session().await.unwrap();
    let _second = conn.create_session().await.unwrap();
    let err = conn.create_session().await.err().unwrap();
    assert_eq!(grpc_code(err), tonic::Code::ResourceExhausted);
    // The rejected backend session was closed again.
    assert_eq!(backend.events_with("close").len(), 1);

    first.close().await.unwrap();
    conn.create_session().await.unwrap();
}

#[tokio::test]
async fn graceful_shutdown_stops_the_server() {
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = GqlServer::builder(TestBackend::new())
        .idle_timeout(Duration::from_secs(60))
        .shutdown(async {
            let _ = stopped.await;
        });
    let handle = tokio::spawn(server.serve_with_listener(listener));

    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    conn.create_session().await.unwrap();

    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), handle)
        .await
        .expect("server did not stop")
        .unwrap()
        .unwrap();
}

// ============================================================================
// Idle session reaper
// ============================================================================

/// Regression: a zero idle timeout made the reaper's interval zero, which
/// panics inside the reaper task, so idle sessions were never reaped.
#[tokio::test]
async fn zero_idle_timeout_reaps_instead_of_panicking() {
    let backend = TestBackend::new();
    let addr = serve(GqlServer::builder(backend.clone()).idle_timeout(Duration::ZERO)).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let session = conn.create_session().await.unwrap();
    let id = session.session_id().to_owned();
    common::eventually("idle session reaped", || !backend.has_session(&id)).await;
}

/// The reaper cleans up like `CloseSession`: the active transaction is rolled
/// back before the backend session is closed.
#[tokio::test]
async fn reaper_rolls_back_before_closing() {
    let backend = TestBackend::new();
    let addr =
        serve(GqlServer::builder(backend.clone()).idle_timeout(Duration::from_millis(200))).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let session = conn.create_session().await.unwrap();
    let id = session.session_id().to_owned();
    // A raw begin: the client-side `Transaction` would roll back on drop.
    let mut gql = proto::gql_service_client::GqlServiceClient::new(conn.channel().clone());
    let tx_id = common::begin(&mut gql, &id).await;

    common::eventually("idle session reaped", || !backend.has_session(&id)).await;
    let events = backend.events();
    let rollback_at = events
        .iter()
        .position(|e| *e == format!("rollback {id} {tx_id}"))
        .expect("transaction rolled back");
    let close_at = events
        .iter()
        .position(|e| *e == format!("close {id}"))
        .expect("session closed");
    assert!(rollback_at < close_at);
}

#[tokio::test]
async fn activity_keeps_a_session_alive() {
    let backend = TestBackend::new();
    let addr =
        serve(GqlServer::builder(backend.clone()).idle_timeout(Duration::from_millis(300))).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let mut session = conn.create_session().await.unwrap();
    for _ in 0..8 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        session.ping().await.unwrap();
    }
    assert!(backend.has_session(session.session_id()));
}

// ============================================================================
// Authentication
// ============================================================================

struct TokenValidator;

#[tonic::async_trait]
impl AuthValidator for TokenValidator {
    async fn validate(&self, credentials: &proto::AuthCredentials) -> Result<AuthInfo, GqlError> {
        match &credentials.method {
            Some(Method::BearerToken(token)) if token == "secret" => Ok(AuthInfo {
                principal: "alix".to_owned(),
            }),
            Some(Method::Basic(basic)) if basic.username == "gus" && basic.password == "pw" => {
                Ok(AuthInfo {
                    principal: "gus".to_owned(),
                })
            }
            _ => Err(GqlError::Protocol("rejected".to_owned())),
        }
    }
}

#[tokio::test]
async fn credentials_are_checked_and_passed_to_the_backend() {
    let backend = TestBackend::new();
    let addr = serve(GqlServer::builder(backend.clone()).auth(TokenValidator)).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();

    let missing = conn.create_session().await.err().unwrap();
    assert_eq!(grpc_code(missing), tonic::Code::Unauthenticated);

    let wrong = conn
        .create_session_with_credentials(proto::AuthCredentials {
            method: Some(Method::BearerToken("guess".to_owned())),
        })
        .await
        .err()
        .unwrap();
    assert_eq!(grpc_code(wrong), tonic::Code::Unauthenticated);

    let empty = conn
        .create_session_with_credentials(proto::AuthCredentials { method: None })
        .await
        .err()
        .unwrap();
    assert_eq!(grpc_code(empty), tonic::Code::Unauthenticated);
    assert!(backend.events_with("create").is_empty());

    let mut alix = conn
        .create_session_with_credentials(proto::AuthCredentials {
            method: Some(Method::BearerToken("secret".to_owned())),
        })
        .await
        .unwrap();
    let gus = conn
        .create_session_with_credentials(proto::AuthCredentials {
            method: Some(Method::Basic(proto::BasicAuth {
                username: "gus".to_owned(),
                password: "pw".to_owned(),
            })),
        })
        .await
        .unwrap();

    assert_eq!(
        backend.events_with("create"),
        vec![
            format!("create {} principal=alix", alix.session_id()),
            format!("create {} principal=gus", gus.session_id()),
        ]
    );
    let mut cursor = alix.execute_simple("ECHO").await.unwrap();
    assert_eq!(
        cursor
            .summary()
            .await
            .unwrap()
            .unwrap()
            .status
            .as_ref()
            .unwrap()
            .code,
        status::SUCCESS
    );
}
