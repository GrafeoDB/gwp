//! Stress tests for the GQL Wire Protocol server.
//!
//! These tests hammer the server with concurrency, connection churn,
//! session limits, idle timeouts, large payloads, and transaction contention.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Barrier;
use tokio::task::JoinSet;
use tonic::transport::Channel;

use gwp::client::GqlConnection;
use gwp::proto;
use gwp::proto::session_service_client::SessionServiceClient;
use gwp::server::mock_backend::MockBackend;
use gwp::server::{CreateDatabaseConfig, GqlServer};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spin up a server with optional `max_sessions` and `idle_timeout`, return addr.
async fn start_server(
    max_sessions: Option<usize>,
    idle_timeout: Option<Duration>,
) -> SocketAddr {
    let backend = MockBackend::new();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    tokio::spawn(async move {
        let mut builder = GqlServer::builder(backend);
        if let Some(limit) = max_sessions {
            builder = builder.max_sessions(limit);
        }
        if let Some(timeout) = idle_timeout {
            builder = builder.idle_timeout(timeout);
        }
        builder.serve(addr).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    addr
}

/// Create a handshake via raw gRPC and return (client, `session_id`).
async fn handshake(
    addr: SocketAddr,
) -> (SessionServiceClient<Channel>, String) {
    let channel = Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = SessionServiceClient::new(channel);
    let resp = client
        .handshake(proto::HandshakeRequest {
            protocol_version: 1,
            client_info: HashMap::new(),
            credentials: None,
        })
        .await
        .unwrap()
        .into_inner();
    (client, resp.session_id)
}

// ===========================================================================
// 1. CONNECTION CHURN — rapid create/close cycles
// ===========================================================================

#[tokio::test]
async fn stress_connection_churn() {
    let addr = start_server(None, None).await;
    let mut set = JoinSet::new();

    for _ in 0..200 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let session = conn.create_session().await.unwrap();
            session.close().await.unwrap();
        });
    }

    let mut ok = 0;
    while let Some(result) = set.join_next().await {
        result.unwrap();
        ok += 1;
    }
    assert_eq!(ok, 200, "all 200 connect/close cycles should succeed");
}

// ===========================================================================
// 2. CONCURRENCY LIMITS — hit max_sessions wall
// ===========================================================================

#[tokio::test]
async fn stress_session_limit() {
    let limit = 50;
    let addr = start_server(Some(limit), None).await;

    // Open `limit` sessions — keep connections alive
    let mut sessions = Vec::new();
    for _ in 0..limit {
        let conn = GqlConnection::connect(&format!("http://{addr}"))
            .await
            .unwrap();
        let session = conn.create_session().await.unwrap();
        sessions.push((conn, session));
    }

    // Next session should fail with RESOURCE_EXHAUSTED
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let result = conn.create_session().await;
    match result {
        Ok(_session) => panic!("session {limit}+1 should be rejected"),
        Err(e) => {
            let err_msg = format!("{e}");
            assert!(
                err_msg.contains("RESOURCE_EXHAUSTED") || err_msg.contains("resource") || err_msg.contains("capacity"),
                "error should be RESOURCE_EXHAUSTED, got: {err_msg}"
            );
        }
    }

    // Close one, then should succeed again
    let (_, s) = sessions.pop().unwrap();
    s.close().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let conn2 = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let recovered = conn2.create_session().await.unwrap();
    recovered.close().await.unwrap();
}

// ===========================================================================
// 3. CONCURRENT SESSION STORM — many sessions at the exact same instant
// ===========================================================================

#[tokio::test]
async fn stress_concurrent_session_storm() {
    let limit = 100;
    let addr = start_server(Some(limit), None).await;
    let barrier = Arc::new(Barrier::new(150));

    let mut set = JoinSet::new();
    for _ in 0..150 {
        let b = barrier.clone();
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            b.wait().await; // all fire at once
            conn.create_session().await
        });
    }

    let mut succeeded = 0;
    let mut rejected = 0;
    while let Some(result) = set.join_next().await {
        match result.unwrap() {
            Ok(_session) => succeeded += 1,
            Err(_) => rejected += 1,
        }
    }

    assert!(
        succeeded <= limit,
        "should not exceed limit: got {succeeded}/{limit}"
    );
    assert!(
        rejected >= 50,
        "at least 50 should be rejected, got only {rejected}"
    );
    assert_eq!(succeeded + rejected, 150);
}

// ===========================================================================
// 4. IDLE TIMEOUT — sessions get reaped after inactivity
// ===========================================================================

#[tokio::test]
async fn stress_idle_timeout_reaping() {
    let timeout = Duration::from_secs(2);
    let addr = start_server(None, Some(timeout)).await;

    let (mut client, session_id) = handshake(addr).await;

    // Ping should work immediately
    let ping_resp = client
        .ping(proto::PingRequest {
            session_id: session_id.clone(),
        })
        .await;
    assert!(ping_resp.is_ok(), "ping should work on fresh session");

    // Wait for reaper to kick in (timeout + margin)
    tokio::time::sleep(timeout + Duration::from_secs(2)).await;

    // Session should be gone
    let ping_resp = client
        .ping(proto::PingRequest {
            session_id: session_id.clone(),
        })
        .await;
    assert!(ping_resp.is_err(), "session should have been reaped");
}

#[tokio::test]
async fn stress_idle_timeout_kept_alive_by_ping() {
    let timeout = Duration::from_secs(3);
    let addr = start_server(None, Some(timeout)).await;

    let (mut client, session_id) = handshake(addr).await;

    // Keep pinging every second for 5 seconds (longer than timeout)
    for _ in 0..5 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let resp = client
            .ping(proto::PingRequest {
                session_id: session_id.clone(),
            })
            .await;
        assert!(resp.is_ok(), "session should stay alive with pings");
    }

    // Stop pinging, wait for reaper
    tokio::time::sleep(timeout + Duration::from_secs(2)).await;

    let resp = client
        .ping(proto::PingRequest {
            session_id: session_id.clone(),
        })
        .await;
    assert!(resp.is_err(), "session should be reaped after pings stop");
}

// ===========================================================================
// 5. TRANSACTION CONTENTION — double-begin, concurrent tx on same session
// ===========================================================================

#[tokio::test]
async fn stress_transaction_double_begin() {
    let addr = start_server(None, None).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let mut session = conn.create_session().await.unwrap();

    let _tx = session.begin_transaction().await.unwrap();

    // Second begin on same session should fail
    let err = session.begin_transaction().await;
    assert!(err.is_err(), "double begin should fail");
}

#[tokio::test]
async fn stress_transaction_parallel_sessions() {
    let addr = start_server(None, None).await;
    let mut set = JoinSet::new();

    for _ in 0..50 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            let mut tx = session.begin_transaction().await.unwrap();

            let mut cursor = tx
                .execute("MATCH (n) RETURN n", HashMap::new())
                .await
                .unwrap();
            let rows = cursor.collect_rows().await.unwrap();
            assert!(!rows.is_empty());

            tx.commit().await.unwrap();
            session.close().await.unwrap();
        });
    }

    let mut ok = 0;
    while let Some(result) = set.join_next().await {
        result.unwrap();
        ok += 1;
    }
    assert_eq!(ok, 50);
}

// ===========================================================================
// 6. QUERY FLOOD — rapid-fire execute on one session
// ===========================================================================

#[tokio::test]
async fn stress_query_flood_sequential() {
    let addr = start_server(None, None).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let mut session = conn.create_session().await.unwrap();

    let start = std::time::Instant::now();
    for i in 0..1000 {
        let mut cursor = session
            .execute(&format!("MATCH (n) RETURN n /* {i} */"), HashMap::new())
            .await
            .unwrap();
        let _rows = cursor.collect_rows().await.unwrap();
    }
    let elapsed = start.elapsed();
    eprintln!("1000 sequential queries: {elapsed:?}");

    assert!(
        elapsed < Duration::from_secs(30),
        "1000 queries took too long: {elapsed:?}"
    );

    session.close().await.unwrap();
}

#[tokio::test]
async fn stress_query_flood_parallel_sessions() {
    let addr = start_server(None, None).await;
    let barrier = Arc::new(Barrier::new(20));
    let mut set = JoinSet::new();

    let start = std::time::Instant::now();
    for _ in 0..20 {
        let b = barrier.clone();
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            b.wait().await;

            for i in 0..100 {
                let mut cursor = session
                    .execute(
                        &format!("MATCH (n) RETURN n /* {i} */"),
                        HashMap::new(),
                    )
                    .await
                    .unwrap();
                let _rows = cursor.collect_rows().await.unwrap();
            }
            session.close().await.unwrap();
        });
    }

    while let Some(result) = set.join_next().await {
        result.unwrap();
    }
    let elapsed = start.elapsed();
    eprintln!("20 sessions x 100 queries each (2000 total): {elapsed:?}");

    assert!(
        elapsed < Duration::from_secs(60),
        "parallel flood took too long: {elapsed:?}"
    );
}

// ===========================================================================
// 7. CONFIGURATION CHURN — rapidly set/reset session properties
// ===========================================================================

#[tokio::test]
async fn stress_config_churn() {
    let addr = start_server(None, None).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let mut session = conn.create_session().await.unwrap();

    for i in 0..200i32 {
        session
            .set_graph(&format!("graph_{i}"))
            .await
            .unwrap();
        session
            .set_schema(&format!("schema_{i}"))
            .await
            .unwrap();
        session.set_time_zone(i % 1440 - 720).await.unwrap();
    }

    // Reset and verify it doesn't break
    session.reset().await.unwrap();

    // Should still be able to query
    let mut cursor = session
        .execute("MATCH (n) RETURN n", HashMap::new())
        .await
        .unwrap();
    assert!(cursor.is_success().await.unwrap());

    session.close().await.unwrap();
}

// ===========================================================================
// 8. DATABASE CLIENT STRESS — create/list/delete under load
// ===========================================================================

#[tokio::test]
async fn stress_database_operations() {
    let addr = start_server(None, None).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let mut db = conn.create_database_client();

    // Create many databases
    for i in 0..50 {
        let config = CreateDatabaseConfig {
            name: format!("stress_db_{i}"),
            database_type: String::new(),
            storage_mode: String::new(),
            memory_limit_bytes: None,
            backward_edges: None,
            threads: None,
            wal_enabled: None,
            wal_durability: None,
        };
        db.create(config).await.unwrap();
    }

    // List should return them
    let list = db.list().await.unwrap();
    assert!(!list.is_empty());

    // Delete them all
    for i in 0..50 {
        db.delete(&format!("stress_db_{i}")).await.unwrap();
    }
}

// ===========================================================================
// 9. MIXED WORKLOAD — concurrent sessions doing different things
// ===========================================================================

#[tokio::test]
async fn stress_mixed_workload() {
    let addr = start_server(None, None).await;
    let mut set = JoinSet::new();

    // Readers
    for _ in 0..10 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            for _ in 0..50 {
                let mut cursor = session
                    .execute("MATCH (n) RETURN n", HashMap::new())
                    .await
                    .unwrap();
                let _rows = cursor.collect_rows().await.unwrap();
            }
            session.close().await.unwrap();
        });
    }

    // Writers (DML)
    for _ in 0..10 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            for _ in 0..50 {
                let mut cursor = session
                    .execute("INSERT (n:Person {name: 'x'})", HashMap::new())
                    .await
                    .unwrap();
                let _ = cursor.rows_affected().await;
            }
            session.close().await.unwrap();
        });
    }

    // DDL
    for _ in 0..5 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            for _ in 0..20 {
                let mut cursor = session
                    .execute("CREATE GRAPH TYPE mytype", HashMap::new())
                    .await
                    .unwrap();
                let _ = cursor.summary().await;
            }
            session.close().await.unwrap();
        });
    }

    // Pingers
    for _ in 0..5 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            for _ in 0..100 {
                session.ping().await.unwrap();
            }
            session.close().await.unwrap();
        });
    }

    // Transaction workers
    for _ in 0..10 {
        set.spawn(async move {
            let conn = GqlConnection::connect(&format!("http://{addr}"))
                .await
                .unwrap();
            let mut session = conn.create_session().await.unwrap();
            for _ in 0..10 {
                let mut tx = session.begin_transaction().await.unwrap();
                let mut cursor = tx
                    .execute("MATCH (n) RETURN n", HashMap::new())
                    .await
                    .unwrap();
                let _rows = cursor.collect_rows().await.unwrap();
                tx.commit().await.unwrap();
            }
            session.close().await.unwrap();
        });
    }

    let mut ok = 0;
    while let Some(result) = set.join_next().await {
        result.unwrap();
        ok += 1;
    }
    assert_eq!(ok, 40, "all 40 workers should complete");
}

// ===========================================================================
// 10. SESSION CLOSE DURING ACTIVE TRANSACTION — cleanup test
// ===========================================================================

#[tokio::test]
async fn stress_close_with_active_transaction() {
    let addr = start_server(None, None).await;

    for _ in 0..50 {
        let conn = GqlConnection::connect(&format!("http://{addr}"))
            .await
            .unwrap();
        let mut session = conn.create_session().await.unwrap();
        let _tx = session.begin_transaction().await.unwrap();
        // Close session without committing — should auto-rollback
        session.close().await.unwrap();
    }

    // Server should still be healthy — create a new session
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let session = conn.create_session().await.unwrap();
    session.close().await.unwrap();
}

// ===========================================================================
// 11. RAPID TRANSACTION CYCLE — begin/commit in tight loop
// ===========================================================================

#[tokio::test]
async fn stress_rapid_transaction_cycle() {
    let addr = start_server(None, None).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let mut session = conn.create_session().await.unwrap();

    let start = std::time::Instant::now();
    for _ in 0..500 {
        let tx = session.begin_transaction().await.unwrap();
        tx.commit().await.unwrap();
    }
    let elapsed = start.elapsed();
    eprintln!("500 begin/commit cycles: {elapsed:?}");

    assert!(
        elapsed < Duration::from_secs(30),
        "500 tx cycles took too long: {elapsed:?}"
    );

    session.close().await.unwrap();
}

// ===========================================================================
// 12. PING FLOOD — raw ping throughput
// ===========================================================================

#[tokio::test]
async fn stress_ping_flood() {
    let addr = start_server(None, None).await;
    let conn = GqlConnection::connect(&format!("http://{addr}"))
        .await
        .unwrap();
    let mut session = conn.create_session().await.unwrap();

    let start = std::time::Instant::now();
    for _ in 0..5000 {
        session.ping().await.unwrap();
    }
    let elapsed = start.elapsed();
    eprintln!("5000 pings: {elapsed:?}");

    let per_ping = elapsed / 5000;
    eprintln!("per ping: {per_ping:?}");

    session.close().await.unwrap();
}
