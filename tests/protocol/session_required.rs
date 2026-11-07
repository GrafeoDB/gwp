//! A server that requires authentication serves nothing without a live
//! session: every RPC of every service, taken from the compiled service
//! descriptors so that a new RPC is covered without a change here, is
//! refused without a session and with an unknown one, before the backend
//! sees it. Without authentication, catalog, admin and search calls without
//! a session still work, and a named session reaches the backend.

use std::collections::{BTreeSet, HashMap};

use bytes::{Buf, BufMut};
use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorSet};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::codegen::http::uri::PathAndQuery;
use tonic::transport::Channel;
use tonic::{Code, Request, Status};

use gwp::client::GqlConnection;
use gwp::error::GqlError;
use gwp::server::VectorSearchParams;

use crate::common::{self, TestServer};

// ============================================================================
// RPCs from the service descriptors
// ============================================================================

/// One RPC of the protocol.
struct Rpc {
    /// `/package.Service/Method`.
    path: String,
    server_streaming: bool,
    /// Field number of `session_id` in the request, if it has one.
    session_field: Option<u32>,
}

/// Every RPC of every service in gwp's protocol.
fn rpcs() -> Vec<Rpc> {
    let set = FileDescriptorSet::decode(gwp::proto::FILE_DESCRIPTOR_SET).unwrap();
    let mut messages: HashMap<String, &DescriptorProto> = HashMap::new();
    for file in &set.file {
        for message in &file.message_type {
            messages.insert(format!(".{}.{}", file.package(), message.name()), message);
        }
    }
    let mut rpcs = Vec::new();
    for file in &set.file {
        for service in &file.service {
            for method in &service.method {
                assert!(
                    !method.client_streaming(),
                    "{}.{} streams requests: extend this test to call it",
                    service.name(),
                    method.name()
                );
                let input = messages[method.input_type()];
                rpcs.push(Rpc {
                    path: format!("/{}.{}/{}", file.package(), service.name(), method.name()),
                    server_streaming: method.server_streaming(),
                    session_field: input
                        .field
                        .iter()
                        .find(|f| f.name() == "session_id")
                        .map(|f| u32::try_from(f.number()).unwrap()),
                });
            }
        }
    }
    rpcs
}

/// A request that sets only `session_id` (field `field`) to `session_id`.
fn request_naming(field: u32, session_id: &str) -> Vec<u8> {
    let mut body = Vec::new();
    prost::encoding::string::encode(field, &session_id.to_owned(), &mut body);
    body
}

/// Passes encoded messages through unchanged, so any RPC can be called.
#[derive(Clone, Copy, Default)]
struct RawCodec;

impl Codec for RawCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;
    type Encoder = Self;
    type Decoder = Self;

    fn encoder(&mut self) -> Self {
        Self
    }

    fn decoder(&mut self) -> Self {
        Self
    }
}

impl Encoder for RawCodec {
    type Item = Vec<u8>;
    type Error = Status;

    fn encode(&mut self, item: Vec<u8>, dst: &mut EncodeBuf<'_>) -> Result<(), Status> {
        dst.put_slice(&item);
        Ok(())
    }
}

impl Decoder for RawCodec {
    type Item = Vec<u8>;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Vec<u8>>, Status> {
        let mut item = vec![0; src.remaining()];
        src.copy_to_slice(&mut item);
        Ok(Some(item))
    }
}

/// Call `rpc` with an encoded request and read the whole response.
async fn call(channel: &Channel, rpc: &Rpc, body: Vec<u8>) -> Result<(), Status> {
    let mut grpc = tonic::client::Grpc::new(channel.clone());
    grpc.ready()
        .await
        .map_err(|e| Status::unavailable(e.to_string()))?;
    let path = PathAndQuery::try_from(rpc.path.as_str()).unwrap();
    if rpc.server_streaming {
        let mut stream = grpc
            .server_streaming(Request::new(body), path, RawCodec)
            .await?
            .into_inner();
        while stream.message().await?.is_some() {}
        Ok(())
    } else {
        grpc.unary(Request::new(body), path, RawCodec).await?;
        Ok(())
    }
}

/// Call `rpc` and require that it is refused for want of a session (or of
/// credentials, for the handshake) without reaching the backend.
async fn assert_refused(server: &TestServer, rpc: &Rpc, body: Vec<u8>, case: &str) {
    let before = server.backend.events();
    let result = call(&server.channel().await, rpc, body).await;
    let code = result.as_ref().map_err(Status::code);
    assert!(
        matches!(code, Err(Code::Unauthenticated | Code::NotFound)),
        "{} {case}: expected UNAUTHENTICATED or NOT_FOUND, got {result:?}",
        rpc.path
    );
    assert_eq!(
        server.backend.events(),
        before,
        "{} {case}: reached the backend",
        rpc.path
    );
}

// ============================================================================
// With authentication
// ============================================================================

#[test]
fn the_descriptors_cover_every_service() {
    let services: BTreeSet<String> = rpcs()
        .iter()
        .map(|rpc| rpc.path.split('/').nth(1).unwrap().to_owned())
        .collect();
    assert_eq!(
        services,
        BTreeSet::from(
            [
                "gql.AdminService",
                "gql.CatalogService",
                "gql.GqlService",
                "gql.SearchService",
                "gql.SessionService",
            ]
            .map(str::to_owned)
        )
    );
    // Only the handshake, which opens a session, has no session to name.
    let without: Vec<String> = rpcs()
        .into_iter()
        .filter(|rpc| rpc.session_field.is_none())
        .map(|rpc| rpc.path)
        .collect();
    assert_eq!(without, vec!["/gql.SessionService/Handshake"]);
}

#[tokio::test]
async fn with_auth_every_rpc_is_refused_without_a_session() {
    let server = common::start_with_auth().await;
    for rpc in rpcs() {
        assert_refused(&server, &rpc, Vec::new(), "without a session").await;
    }
}

#[tokio::test]
async fn with_auth_every_rpc_is_refused_for_an_unknown_session() {
    let server = common::start_with_auth().await;
    for rpc in rpcs() {
        if let Some(field) = rpc.session_field {
            let body = request_naming(field, "no-such-session");
            assert_refused(&server, &rpc, body, "with an unknown session").await;
        }
    }
}

#[tokio::test]
async fn with_auth_a_call_without_a_session_is_unauthenticated() {
    let server = common::start_with_auth().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let code = |result: Result<(), GqlError>| match result {
        Err(GqlError::Grpc(status)) => status.code(),
        other => panic!("expected a gRPC error, got {other:?}"),
    };
    assert_eq!(
        code(conn.create_catalog_client().list_schemas().await.map(drop)),
        Code::Unauthenticated
    );
    assert_eq!(
        code(conn.create_admin_client().get_stats("g").await.map(drop)),
        Code::Unauthenticated
    );
    assert_eq!(
        code(
            conn.create_search_client()
                .vector_search(vector_search("g"))
                .await
                .map(drop)
        ),
        Code::Unauthenticated
    );
}

#[tokio::test]
async fn with_auth_an_authenticated_session_is_served() {
    let server = common::start_with_auth().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let session = conn
        .create_session_with_credentials(common::credentials())
        .await
        .unwrap();
    let id = session.session_id().to_owned();

    session.catalog().list_schemas().await.unwrap();
    session.admin().get_stats("g").await.unwrap();
    session
        .search()
        .vector_search(vector_search("g"))
        .await
        .unwrap();

    assert_eq!(
        server.backend.events_with("stats"),
        vec![format!("stats g session={id}")]
    );
    assert_eq!(
        server.backend.events_with("vector_search"),
        vec![format!("vector_search g session={id}")]
    );
}

// ============================================================================
// Without authentication
// ============================================================================

fn vector_search(graph: &str) -> VectorSearchParams {
    VectorSearchParams {
        graph: graph.to_owned(),
        label: "Doc".to_owned(),
        property: "embedding".to_owned(),
        query_vector: vec![1.0, 0.0],
        k: 3,
        ef: None,
        filters: HashMap::new(),
    }
}

#[tokio::test]
async fn without_auth_admin_and_search_work_without_a_session() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();

    let mut admin = conn.create_admin_client();
    assert_eq!(admin.session_id(), None);
    admin.get_stats("g").await.unwrap();
    let mut search = conn.create_search_client();
    assert_eq!(search.session_id(), None);
    search.vector_search(vector_search("g")).await.unwrap();

    assert_eq!(
        server.backend.events_with("stats"),
        vec!["stats g session=-"]
    );
    assert_eq!(
        server.backend.events_with("vector_search"),
        vec!["vector_search g session=-"]
    );
}

#[tokio::test]
async fn without_auth_a_session_bound_client_names_its_session() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let session = conn.create_session().await.unwrap();
    let id = session.session_id().to_owned();

    let mut admin = session.admin();
    assert_eq!(admin.session_id(), Some(id.as_str()));
    admin.get_stats("g").await.unwrap();
    let mut search = session.search();
    assert_eq!(search.session_id(), Some(id.as_str()));
    search.vector_search(vector_search("g")).await.unwrap();

    assert_eq!(
        server.backend.events_with("stats"),
        vec![format!("stats g session={id}")]
    );
    assert_eq!(
        server.backend.events_with("vector_search"),
        vec![format!("vector_search g session={id}")]
    );
}

#[tokio::test]
async fn without_auth_an_unknown_session_is_not_found() {
    let server = common::start().await;
    let channel = server.channel().await;
    for rpc in rpcs() {
        let Some(field) = rpc.session_field else {
            continue;
        };
        let result = call(&channel, &rpc, request_naming(field, "no-such-session")).await;
        assert_eq!(
            result.as_ref().map_err(Status::code),
            Err(Code::NotFound),
            "{}: {result:?}",
            rpc.path
        );
    }
}
