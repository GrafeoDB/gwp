//! Transaction lifecycle: begin, commit, rollback, failures, and races.

use gwp::client::GqlConnection;
use gwp::proto;
use gwp::status;

use crate::common::{self, begin, commit, execute_frames, final_status, rollback};

#[tokio::test]
async fn begin_execute_commit() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    assert!(server.transactions.validate(&tx, &id).await.is_ok());
    assert_eq!(
        server.sessions.active_transaction(&id).await,
        Some(tx.clone())
    );

    let frames = execute_frames(&mut gql, &id, "WRITE", Some(&tx))
        .await
        .unwrap();
    assert_eq!(final_status(&frames).code, status::SUCCESS);

    assert_eq!(commit(&mut gql, &id, &tx).await, status::SUCCESS);
    assert!(server.transactions.validate(&tx, &id).await.is_err());
    assert_eq!(server.sessions.active_transaction(&id).await, None);

    assert_eq!(
        server.backend.events_with(&format!("execute {id}")),
        vec![format!("execute {id} tx={tx} WRITE")]
    );
    assert_eq!(
        server.backend.events_with("commit"),
        vec![format!("commit {id} {tx}")]
    );

    // A new transaction can start after the commit.
    let next = begin(&mut gql, &id).await;
    assert_ne!(next, tx);
}

#[tokio::test]
async fn rollback_ends_the_transaction() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    assert_eq!(rollback(&mut gql, &id, &tx).await, status::SUCCESS);
    assert_eq!(server.backend.transaction_of(&id), None);

    // The transaction is gone: commit, rollback and statements reject it.
    assert_eq!(
        commit(&mut gql, &id, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );
    assert_eq!(
        rollback(&mut gql, &id, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );
    let err = execute_frames(&mut gql, &id, "ECHO", Some(&tx))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);

    begin(&mut gql, &id).await;
}

#[tokio::test]
async fn read_only_mode_reaches_the_backend() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    let tx = session.begin_read_only_transaction().await.unwrap();
    let tx_id = tx.transaction_id().to_owned();
    tx.rollback().await.unwrap();

    let id = session.session_id();
    assert_eq!(
        server.backend.events_with("begin"),
        vec![format!("begin {id} ReadOnly {tx_id}")]
    );
}

#[tokio::test]
async fn unknown_transaction_mode_is_rejected() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    for mode in [2, 7, -1, i32::MAX] {
        let err = gql
            .begin_transaction(proto::BeginRequest {
                session_id: id.clone(),
                mode,
            })
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument, "mode {mode}");
    }
    // Regression: an unknown mode used to start a READ WRITE transaction.
    assert!(server.backend.events_with("begin").is_empty());
    assert!(!server.transactions.has_transaction(&id).await);
}

#[tokio::test]
async fn failure_inside_a_transaction_keeps_it_usable() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    let mut tx = session.begin_transaction().await.unwrap();

    let mut cursor = tx.execute_simple("FAIL").await.unwrap();
    assert!(cursor.collect_rows().await.unwrap().is_empty());
    assert!(!cursor.is_success().await.unwrap());
    let summary = cursor.summary().await.unwrap().unwrap().clone();
    assert_eq!(summary.status.unwrap().code, status::INVALID_SYNTAX);

    // The transaction is still active: statements and rollback work.
    let mut cursor = tx.execute_simple("ECHO").await.unwrap();
    assert!(cursor.is_success().await.unwrap());
    tx.rollback().await.unwrap();

    // And the session can start the next one.
    let tx = session.begin_transaction().await.unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn failed_commit_ends_the_transaction() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    server.backend.fail_next_commit();

    // A failed commit reports a transaction rollback (sec 8.4)...
    assert_eq!(
        commit(&mut gql, &id, &tx).await,
        status::TRANSACTION_ROLLBACK
    );

    // ...and ends the transaction. Regression: it stayed registered, so the
    // session could never begin another transaction (25G01 forever).
    assert!(!server.transactions.has_transaction(&id).await);
    assert_eq!(server.sessions.active_transaction(&id).await, None);
    assert_eq!(
        commit(&mut gql, &id, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );

    // The backend was asked to release it, then a new transaction starts.
    assert_eq!(
        server.backend.events_with("rollback"),
        vec![format!("rollback {id} {tx}")]
    );
    let next = begin(&mut gql, &id).await;
    assert_eq!(commit(&mut gql, &id, &next).await, status::SUCCESS);
}

#[tokio::test]
async fn failed_rollback_still_ends_the_transaction() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    server.backend.fail_rollbacks();

    let code = rollback(&mut gql, &id, &tx).await;
    assert!(status::is_exception(&code), "{code}");
    assert!(!server.transactions.has_transaction(&id).await);

    begin(&mut gql, &id).await;
}

#[tokio::test]
async fn double_begin_never_reaches_the_backend() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    let second = gql
        .begin_transaction(proto::BeginRequest {
            session_id: id.clone(),
            mode: proto::TransactionMode::ReadWrite.into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second.status.unwrap().code, status::ACTIVE_TRANSACTION);
    assert!(second.transaction_id.is_empty());

    // The backend saw one begin and no rollback: the first transaction is
    // untouched (a backend that nests or replaces transactions on a second
    // begin would otherwise be affected).
    assert_eq!(server.backend.events_with("begin").len(), 1);
    assert!(server.backend.events_with("rollback").is_empty());
    assert_eq!(server.backend.transaction_of(&id), Some(tx.clone()));
    assert_eq!(commit(&mut gql, &id, &tx).await, status::SUCCESS);
}

#[tokio::test]
async fn transactions_are_bound_to_their_session() {
    let server = common::start().await;
    let alix = server.handshake().await;
    let gus = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &alix).await;

    assert_eq!(
        commit(&mut gql, &gus, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );
    assert_eq!(
        rollback(&mut gql, &gus, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );
    let err = execute_frames(&mut gql, &gus, "ECHO", Some(&tx))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    assert_eq!(
        commit(&mut gql, &alix, "no-such-transaction").await,
        status::INVALID_TRANSACTION_STATE
    );
    assert_eq!(
        commit(&mut gql, &alix, "").await,
        status::INVALID_TRANSACTION_STATE
    );

    // The owner's transaction was not disturbed.
    assert_eq!(commit(&mut gql, &alix, &tx).await, status::SUCCESS);
}

#[tokio::test]
async fn close_rolls_back_the_active_transaction() {
    let server = common::start().await;
    let mut sessions = server.session_client().await;
    let id = common::handshake(&mut sessions).await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    sessions
        .close_session(proto::CloseSessionRequest {
            session_id: id.clone(),
        })
        .await
        .unwrap();

    let events = server.backend.events();
    let rollback_at = events
        .iter()
        .position(|e| *e == format!("rollback {id} {tx}"))
        .expect("transaction rolled back");
    let close_at = events
        .iter()
        .position(|e| *e == format!("close {id}"))
        .expect("session closed");
    assert!(rollback_at < close_at);
    assert!(!server.transactions.has_transaction(&id).await);
    assert!(!server.sessions.exists(&id).await);
}

#[tokio::test]
async fn transaction_survives_a_session_reset() {
    let server = common::start().await;
    let mut sessions = server.session_client().await;
    let id = common::handshake(&mut sessions).await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    sessions
        .reset(proto::ResetRequest {
            session_id: id.clone(),
            target: proto::ResetTarget::ResetAll.into(),
        })
        .await
        .unwrap();

    assert_eq!(
        server.sessions.active_transaction(&id).await,
        Some(tx.clone())
    );
    assert_eq!(commit(&mut gql, &id, &tx).await, status::SUCCESS);
}

/// While a commit is in flight, the transaction is claimed: a rollback,
/// a second commit, a statement and a new begin on it are all rejected.
#[tokio::test]
async fn commit_in_flight_claims_the_transaction() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let tx = begin(&mut gql, &id).await;
    let gate = server.backend.gate_next_commit();

    let commit_task = {
        let mut gql = gql.clone();
        let (id, tx) = (id.clone(), tx.clone());
        tokio::spawn(async move { commit(&mut gql, &id, &tx).await })
    };
    gate.reached().await;

    assert_eq!(
        rollback(&mut gql, &id, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );
    assert_eq!(
        commit(&mut gql, &id, &tx).await,
        status::INVALID_TRANSACTION_STATE
    );
    let err = execute_frames(&mut gql, &id, "ECHO", Some(&tx))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    let busy = gql
        .begin_transaction(proto::BeginRequest {
            session_id: id.clone(),
            mode: proto::TransactionMode::ReadWrite.into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(busy.status.unwrap().code, status::ACTIVE_TRANSACTION);

    gate.open();
    assert_eq!(commit_task.await.unwrap(), status::SUCCESS);

    // Only the in-flight commit reached the backend.
    assert_eq!(server.backend.events_with("commit").len(), 1);
    assert!(server.backend.events_with("rollback").is_empty());
    begin(&mut gql, &id).await;
}

/// A session closed while the backend is starting a transaction must not
/// leave that transaction behind.
#[tokio::test]
async fn begin_racing_close_does_not_leak_the_transaction() {
    let server = common::start().await;
    let mut sessions = server.session_client().await;
    let id = common::handshake(&mut sessions).await;
    let gql = server.gql_client().await;

    let gate = server.backend.gate_next_begin();
    let begin_task = {
        let mut gql = gql.clone();
        let id = id.clone();
        tokio::spawn(async move {
            gql.begin_transaction(proto::BeginRequest {
                session_id: id,
                mode: proto::TransactionMode::ReadWrite.into(),
            })
            .await
        })
    };
    gate.reached().await;

    sessions
        .close_session(proto::CloseSessionRequest {
            session_id: id.clone(),
        })
        .await
        .unwrap();

    gate.open();
    let result = begin_task.await.unwrap();
    // The backend may fail the begin itself (the session is gone there) or
    // start it; either way the client learns the session is gone or the
    // begin failed, and nothing stays registered.
    match result {
        Err(status) => assert_eq!(status.code(), tonic::Code::NotFound),
        Ok(response) => {
            let response = response.into_inner();
            assert!(status::is_exception(&response.status.unwrap().code));
        }
    }
    assert!(!server.transactions.has_transaction(&id).await);
    assert_eq!(server.backend.transaction_of(&id), None);
}

/// Same race, with a backend that does start the transaction: the begin
/// must roll it back itself.
#[tokio::test]
async fn transaction_started_for_a_closed_session_is_rolled_back() {
    let server = common::start().await;
    let id = server.handshake().await;
    let mut gql = server.gql_client().await;

    let gate = server.backend.gate_next_begin();
    let begin_task = {
        let mut gql = gql.clone();
        let id = id.clone();
        tokio::spawn(async move {
            gql.begin_transaction(proto::BeginRequest {
                session_id: id,
                mode: proto::TransactionMode::ReadWrite.into(),
            })
            .await
        })
    };
    gate.reached().await;

    // Unregister the session only on the protocol side, as a close does
    // first, while the backend still accepts the begin.
    assert!(server.sessions.remove(&id).await);
    gate.open();

    let status = begin_task.await.unwrap().unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);

    let begun = server.backend.events_with("begin");
    assert_eq!(begun.len(), 1);
    let tx = begun[0].rsplit(' ').next().unwrap().to_owned();
    assert_eq!(
        server.backend.events_with("rollback"),
        vec![format!("rollback {id} {tx}")]
    );
    assert!(!server.transactions.has_transaction(&id).await);
    let err = execute_frames(&mut gql, &id, "ECHO", Some(&tx))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}
