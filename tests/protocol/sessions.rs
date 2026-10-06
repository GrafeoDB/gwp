//! Session properties and resets reach the backend unchanged.

use gwp::client::GqlConnection;
use gwp::proto;
use gwp::types::Value;

use crate::common;

#[tokio::test]
async fn configure_forwards_every_property() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();
    let id = session.session_id().to_owned();

    session.set_schema("analytics").await.unwrap();
    session.set_graph("default").await.unwrap();
    session.set_time_zone(-330).await.unwrap();
    session.set_time_zone(i32::MIN).await.unwrap();
    session.set_parameter("language", "cypher").await.unwrap();
    session
        .set_parameter("limits", Value::List(vec![Value::Integer(1), Value::Null]))
        .await
        .unwrap();

    assert_eq!(
        server.backend.events_with("configure"),
        vec![
            format!("configure {id} Schema(\"analytics\")"),
            format!("configure {id} Graph(\"default\")"),
            format!("configure {id} TimeZone(-330)"),
            format!("configure {id} TimeZone(-2147483648)"),
            format!("configure {id} Parameter {{ name: \"language\", value: String(\"cypher\") }}"),
            format!(
                "configure {id} Parameter {{ name: \"limits\", value: List([Integer(1), Null]) }}"
            ),
        ]
    );
}

#[tokio::test]
async fn parameter_without_value_is_null() {
    let server = common::start().await;
    let mut client = server.session_client().await;
    let id = common::handshake(&mut client).await;

    client
        .configure(proto::ConfigureRequest {
            session_id: id.clone(),
            property: Some(proto::configure_request::Property::Parameter(
                proto::SessionParameter {
                    name: "p".to_owned(),
                    value: None,
                },
            )),
        })
        .await
        .unwrap();

    assert_eq!(
        server.backend.events_with("configure"),
        vec![format!(
            "configure {id} Parameter {{ name: \"p\", value: Null }}"
        )]
    );
}

#[tokio::test]
async fn configure_rejected_by_backend_is_reported() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    let err = session.set_graph("no_such_graph").await.unwrap_err();
    match err {
        gwp::error::GqlError::Grpc(status) => assert_eq!(status.code(), tonic::Code::NotFound),
        other => panic!("unexpected error {other:?}"),
    }
    // The session itself is still usable.
    session.ping().await.unwrap();
    session.set_graph("default").await.unwrap();
}

#[tokio::test]
async fn configure_without_property_is_rejected() {
    let server = common::start().await;
    let mut client = server.session_client().await;
    let id = common::handshake(&mut client).await;

    let status = client
        .configure(proto::ConfigureRequest {
            session_id: id,
            property: None,
        })
        .await
        .unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(server.backend.events_with("configure").is_empty());
}

#[tokio::test]
async fn reset_forwards_each_target() {
    let server = common::start().await;
    let mut client = server.session_client().await;
    let id = common::handshake(&mut client).await;

    for target in [
        proto::ResetTarget::ResetSchema,
        proto::ResetTarget::ResetGraph,
        proto::ResetTarget::ResetTimeZone,
        proto::ResetTarget::ResetParameters,
        proto::ResetTarget::ResetAll,
    ] {
        client
            .reset(proto::ResetRequest {
                session_id: id.clone(),
                target: target.into(),
            })
            .await
            .unwrap();
    }

    assert_eq!(
        server.backend.events_with("reset"),
        vec![
            format!("reset {id} Schema"),
            format!("reset {id} Graph"),
            format!("reset {id} TimeZone"),
            format!("reset {id} Parameters"),
            format!("reset {id} All"),
        ]
    );
}

#[tokio::test]
async fn reset_with_unknown_target_is_rejected() {
    let server = common::start().await;
    let mut client = server.session_client().await;
    let id = common::handshake(&mut client).await;

    for target in [5, 99, -1, i32::MAX] {
        let status = client
            .reset(proto::ResetRequest {
                session_id: id.clone(),
                target,
            })
            .await
            .unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument, "{target}");
    }
    assert!(server.backend.events_with("reset").is_empty());
}

#[tokio::test]
async fn client_reset_resets_everything() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();
    let id = session.session_id().to_owned();

    session.reset().await.unwrap();
    assert_eq!(
        server.backend.events_with("reset"),
        vec![format!("reset {id} All")]
    );
}

#[tokio::test]
async fn handshake_reports_server_identity() {
    let server = common::start().await;
    let mut client = server.session_client().await;
    let response = client
        .handshake(proto::HandshakeRequest {
            protocol_version: 99,
            credentials: None,
            client_info: [("driver".to_owned(), "test".to_owned())].into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.protocol_version, 1);
    assert!(!response.session_id.is_empty());
    let info = response.server_info.unwrap();
    assert_eq!(info.name, "gql-wire-protocol");
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    assert!(server.sessions.exists(&response.session_id).await);
    assert!(server.backend.has_session(&response.session_id));
}
