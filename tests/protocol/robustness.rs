//! The server must neither panic nor grow without bound on untrusted input,
//! and must keep its bookkeeping straight under concurrency and disconnects.

use std::collections::HashMap;
use std::time::Duration;

use gwp::client::GqlConnection;
use gwp::proto;
use gwp::status;
use gwp::types::Value;

use crate::common::{self, execute_frames, final_status};

/// The server still answers, so the previous request did not take it down.
async fn assert_alive(server: &common::TestServer) {
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;
    let frames = execute_frames(&mut gql, &id, "ECHO", None).await.unwrap();
    assert_eq!(final_status(&frames).code, status::SUCCESS);
}

/// Regression: the statement was cut at byte 100 for the tracing span, which
/// panicked when byte 100 fell inside a multi-byte character.
#[tokio::test]
async fn long_multibyte_statement_does_not_panic() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    for prefix in 97..=100 {
        let statement = format!("{}\u{20ac}\u{1f600} tail", "a".repeat(prefix));
        let frames = execute_frames(&mut gql, &id, &statement, None)
            .await
            .expect("the request must not fail");
        assert_eq!(final_status(&frames).code, status::OMITTED_RESULT);
    }
    // The full statement reached the backend.
    assert!(
        server
            .backend
            .events()
            .iter()
            .any(|e| e.ends_with("\u{20ac}\u{1f600} tail"))
    );
    assert_alive(&server).await;
}

// Multi-threaded, so that the server decodes on worker threads and a stack
// overflow there would not be masked by the test thread's own encoding.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deeply_nested_parameter_is_rejected_without_crashing() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    // Twice protobuf's recursion limit of 100 nested messages, built as
    // proto values directly to keep the test's own recursion shallow.
    let mut value = proto::Value::from(Value::Integer(1));
    for _ in 0..100 {
        value = proto::Value {
            kind: Some(proto::value::Kind::ListValue(proto::GqlList {
                elements: vec![value],
            })),
        };
    }
    let request = proto::ExecuteRequest {
        session_id: id.clone(),
        statement: "ECHO".to_owned(),
        parameters: HashMap::from([("deep".to_owned(), value)]),
        transaction_id: None,
    };
    // Decoding stops at protobuf's recursion limit instead of recursing
    // without bound.
    let err = gql.execute(request).await.unwrap_err();
    assert_ne!(err.code(), tonic::Code::Ok);
    assert!(server.backend.events_with("execute").is_empty());

    assert_alive(&server).await;
}

#[tokio::test]
async fn oversized_request_is_rejected() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    // Larger than the default 4 MiB decoding limit.
    let statement = "x".repeat(5 * 1024 * 1024);
    let err = execute_frames(&mut gql, &id, &statement, None)
        .await
        .unwrap_err();
    assert_ne!(err.code(), tonic::Code::Ok);
    assert!(server.backend.events_with("execute").is_empty());

    assert_alive(&server).await;
}

#[tokio::test]
async fn unknown_session_is_rejected_by_every_rpc() {
    let server = common::start().await;
    let mut sessions = server.session_client().await;
    let mut gql = server.gql_client().await;
    let ghost = "no-such-session".to_owned();

    let mut codes = vec![
        sessions
            .configure(proto::ConfigureRequest {
                session_id: ghost.clone(),
                property: Some(proto::configure_request::Property::TimeZoneOffsetMinutes(0)),
            })
            .await
            .unwrap_err()
            .code(),
        sessions
            .reset(proto::ResetRequest {
                session_id: ghost.clone(),
                target: 0,
            })
            .await
            .unwrap_err()
            .code(),
        sessions
            .ping(proto::PingRequest {
                session_id: ghost.clone(),
            })
            .await
            .unwrap_err()
            .code(),
        sessions
            .close_session(proto::CloseSessionRequest {
                session_id: ghost.clone(),
            })
            .await
            .unwrap_err()
            .code(),
        gql.begin_transaction(proto::BeginRequest {
            session_id: ghost.clone(),
            mode: 0,
        })
        .await
        .unwrap_err()
        .code(),
        gql.commit(proto::CommitRequest {
            session_id: ghost.clone(),
            transaction_id: "tx".to_owned(),
        })
        .await
        .unwrap_err()
        .code(),
        gql.rollback(proto::RollbackRequest {
            session_id: ghost.clone(),
            transaction_id: "tx".to_owned(),
        })
        .await
        .unwrap_err()
        .code(),
        execute_frames(&mut gql, &ghost, "ECHO", None)
            .await
            .unwrap_err()
            .code(),
    ];
    // An empty session id is just another unknown one.
    codes.push(
        execute_frames(&mut gql, "", "ECHO", None)
            .await
            .unwrap_err()
            .code(),
    );

    assert!(
        codes.iter().all(|c| *c == tonic::Code::NotFound),
        "{codes:?}"
    );
    // None of it reached the backend.
    assert!(server.backend.events().is_empty());
}

#[tokio::test]
async fn operations_after_close_are_rejected() {
    let server = common::start().await;
    let mut sessions = server.session_client().await;
    let id = common::handshake(&mut sessions).await;
    let mut gql = server.gql_client().await;
    let tx = common::begin(&mut gql, &id).await;

    sessions
        .close_session(proto::CloseSessionRequest {
            session_id: id.clone(),
        })
        .await
        .unwrap();
    let events_after_close = server.backend.events().len();

    assert_eq!(
        execute_frames(&mut gql, &id, "ECHO", None)
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    assert_eq!(
        execute_frames(&mut gql, &id, "ECHO", Some(&tx))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::NotFound
    );
    for code in [
        gql.commit(proto::CommitRequest {
            session_id: id.clone(),
            transaction_id: tx.clone(),
        })
        .await
        .unwrap_err()
        .code(),
        gql.begin_transaction(proto::BeginRequest {
            session_id: id.clone(),
            mode: 0,
        })
        .await
        .unwrap_err()
        .code(),
        sessions
            .ping(proto::PingRequest {
                session_id: id.clone(),
            })
            .await
            .unwrap_err()
            .code(),
        sessions
            .close_session(proto::CloseSessionRequest {
                session_id: id.clone(),
            })
            .await
            .unwrap_err()
            .code(),
    ] {
        assert_eq!(code, tonic::Code::NotFound);
    }

    assert_eq!(server.backend.events().len(), events_after_close);
    assert!(!server.backend.has_session(&id));
    assert!(!server.transactions.has_transaction(&id).await);
}

// Multi-threaded, so that the closes really run in parallel.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_closes_close_once() {
    let server = common::start().await;
    let id = server.handshake().await;
    let channel = server.channel().await;

    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..16 {
        let mut client = proto::session_service_client::SessionServiceClient::new(channel.clone());
        let id = id.clone();
        tasks.spawn(async move {
            client
                .close_session(proto::CloseSessionRequest { session_id: id })
                .await
                .map(|_| ())
        });
    }
    let mut ok = 0;
    while let Some(result) = tasks.join_next().await {
        match result.unwrap() {
            Ok(()) => ok += 1,
            Err(status) => assert_eq!(status.code(), tonic::Code::NotFound),
        }
    }
    assert_eq!(ok, 1);
    assert_eq!(server.backend.events_with("close").len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_statements_on_one_session() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let session_id = conn.create_session().await.unwrap().session_id().to_owned();
    let channel = server.channel().await;

    let mut tasks = tokio::task::JoinSet::new();
    for n in 0..64_i64 {
        let mut gql = proto::gql_service_client::GqlServiceClient::new(channel.clone());
        let session_id = session_id.clone();
        tasks.spawn(async move {
            let mut stream = gql
                .execute(proto::ExecuteRequest {
                    session_id,
                    statement: "ECHO".to_owned(),
                    parameters: HashMap::from([(
                        "n".to_owned(),
                        proto::Value::from(Value::Integer(n)),
                    )]),
                    transaction_id: None,
                })
                .await
                .unwrap()
                .into_inner();
            let mut echoed = None;
            while let Some(response) = stream.message().await.unwrap() {
                if let Some(proto::execute_response::Frame::RowBatch(batch)) = response.frame {
                    echoed = Some(Value::from(batch.rows[0].values[0].clone()));
                }
            }
            (n, echoed)
        });
    }
    while let Some(result) = tasks.join_next().await {
        let (n, echoed) = result.unwrap();
        assert_eq!(echoed, Some(Value::Integer(n)));
    }
}

#[tokio::test]
async fn missing_index_definition_and_empty_search_are_rejected() {
    let server = common::start().await;
    let channel = server.channel().await;
    let mut admin = proto::admin_service_client::AdminServiceClient::new(channel.clone());
    let mut search = proto::search_service_client::SearchServiceClient::new(channel);

    let codes = [
        admin
            .create_index(proto::CreateIndexRequest {
                graph: "g".to_owned(),
                index: None,
            })
            .await
            .unwrap_err()
            .code(),
        admin
            .drop_index(proto::DropIndexRequest {
                graph: "g".to_owned(),
                index: None,
            })
            .await
            .unwrap_err()
            .code(),
        admin
            .get_graph_stats(proto::GetGraphStatsRequest {
                graph: String::new(),
            })
            .await
            .unwrap_err()
            .code(),
        search
            .vector_search(proto::VectorSearchRequest {
                graph: "g".to_owned(),
                query_vector: Vec::new(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        search
            .text_search(proto::TextSearchRequest {
                graph: "g".to_owned(),
                query: String::new(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        search
            .hybrid_search(proto::HybridSearchRequest {
                graph: String::new(),
                query_text: "q".to_owned(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
    ];
    assert!(
        codes.iter().all(|c| *c == tonic::Code::InvalidArgument),
        "{codes:?}"
    );

    // A backend without admin or search support answers UNIMPLEMENTED.
    let unsupported = admin
        .wal_status(proto::WalStatusRequest {
            graph: "g".to_owned(),
        })
        .await
        .unwrap_err();
    assert_eq!(unsupported.code(), tonic::Code::Unimplemented);
    let unsupported = search
        .text_search(proto::TextSearchRequest {
            graph: "g".to_owned(),
            query: "q".to_owned(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(unsupported.code(), tonic::Code::Unimplemented);
}

// ============================================================================
// Streaming
// ============================================================================

/// Regression: after a backend error mid-stream, the frames the backend
/// produced next were still sent after the error summary.
#[tokio::test]
async fn error_mid_stream_ends_with_its_summary() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let frames = execute_frames(&mut gql, &id, "FAIL_MIDSTREAM", None)
        .await
        .unwrap();
    assert_eq!(frames.len(), 3, "{frames:?}");
    assert!(matches!(
        frames[0],
        proto::execute_response::Frame::Header(_)
    ));
    assert!(matches!(
        frames[1],
        proto::execute_response::Frame::RowBatch(_)
    ));
    assert_eq!(final_status(&frames).code, status::DIVISION_BY_ZERO);
}

#[tokio::test]
async fn nothing_follows_the_summary() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let frames = execute_frames(&mut gql, &id, "TWO_SUMMARIES", None)
        .await
        .unwrap();
    assert_eq!(frames.len(), 2, "{frames:?}");
    assert_eq!(final_status(&frames).code, status::SUCCESS);
}

#[tokio::test]
async fn large_result_streams_arrive_complete_and_in_order() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    let mut cursor = session.execute_simple("ROWS 100000").await.unwrap();
    let mut expected = 0_i64;
    while let Some(row) = cursor.next_row().await.unwrap() {
        assert_eq!(row, vec![Value::Integer(expected)]);
        expected += 1;
    }
    assert_eq!(expected, 100_000);
    assert_eq!(cursor.rows_affected().await.unwrap(), 100_000);
    assert!(cursor.is_success().await.unwrap());
}

/// A client that stops reading must stop the server from pulling frames out
/// of the backend: memory stays bounded by the HTTP/2 flow-control window.
#[tokio::test]
async fn slow_reader_applies_backpressure() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let mut stream = gql
        .execute(proto::ExecuteRequest {
            session_id: id,
            statement: "STREAM_FOREVER".to_owned(),
            parameters: HashMap::new(),
            transaction_id: None,
        })
        .await
        .unwrap()
        .into_inner();
    stream.message().await.unwrap().unwrap();

    // Wait until the server stops producing while nobody reads.
    let mut last = server.backend.frames_polled();
    loop {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let now = server.backend.frames_polled();
        if now == last {
            break;
        }
        last = now;
    }
    // Each frame carries 1 KiB: a few hundred frames fill the windows and
    // buffers; an unbounded producer would be far past this by now.
    assert!(
        last < 5_000,
        "server buffered {last} frames for a stalled client"
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(server.backend.frames_polled(), last);

    // Reading again resumes production.
    for _ in 0..(last + 100) {
        stream.message().await.unwrap().unwrap();
    }
    assert!(server.backend.frames_polled() > last);
    assert!(!server.backend.stream_dropped());
}

/// When the client goes away mid-stream, the backend stream is dropped, so
/// a backend can stop its work in `Drop`.
#[tokio::test]
async fn client_disconnect_drops_the_backend_stream() {
    let server = common::start().await;
    let id = server.handshake().await;

    {
        let mut gql = server.gql_client().await;
        let mut stream = gql
            .execute(proto::ExecuteRequest {
                session_id: id.clone(),
                statement: "STREAM_FOREVER".to_owned(),
                parameters: HashMap::new(),
                transaction_id: None,
            })
            .await
            .unwrap()
            .into_inner();
        for _ in 0..10 {
            stream.message().await.unwrap().unwrap();
        }
        // The stream and its connection are dropped here.
    }

    common::eventually("backend stream dropped", || server.backend.stream_dropped()).await;
    // The session itself is unaffected.
    assert!(server.sessions.exists(&id).await);
    assert_alive(&server).await;
}

/// Same, through the high-level client: dropping a cursor cancels the call.
#[tokio::test]
async fn dropping_a_cursor_cancels_the_stream() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    let mut cursor = session.execute_simple("STREAM_FOREVER").await.unwrap();
    for _ in 0..5 {
        cursor.next_row().await.unwrap().unwrap();
    }
    drop(cursor);

    common::eventually("backend stream dropped", || server.backend.stream_dropped()).await;
    // The same connection and session keep working.
    let mut cursor = session.execute_simple("ECHO").await.unwrap();
    assert!(cursor.is_success().await.unwrap());
}

/// Regression: dropping a `Transaction` outside a tokio runtime panicked,
/// because its rollback-on-drop called `tokio::spawn`.
#[test]
fn dropping_a_transaction_outside_a_runtime_does_not_panic() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let transaction = runtime.block_on(async {
        let server = common::start().await;
        let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
        let mut session = conn.create_session().await.unwrap();
        session.begin_transaction().await.unwrap()
    });
    drop(runtime);
    drop(transaction);
}
