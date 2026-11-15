//! Search parameters reach the backend as sent: the hybrid search's `ef`
//! (the vector half's beam width) through the Rust client and as a raw
//! proto field, and the constructors and setters of the non-exhaustive
//! parameter structs, used from outside the crate.

use std::collections::HashMap;

use gwp::client::GqlConnection;
use gwp::proto;
use gwp::proto::search_service_client::SearchServiceClient;
use gwp::server::{HybridSearchParams, TextSearchParams, VectorSearchParams};
use gwp::types::Value;
use tonic::transport::Channel;

use crate::common;

#[tokio::test]
async fn hybrid_ef_reaches_the_backend_through_the_client() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut search = conn.create_search_client();

    let params = HybridSearchParams::new("g", "Doc", "body", "embedding", "graph databases", 14)
        .with_query_vector(vec![0.5, 0.5])
        .with_ef(256);
    search.hybrid_search(params).await.unwrap();

    assert_eq!(
        server.backend.events_with("hybrid_search"),
        vec!["hybrid_search g k=14 ef=Some(256) vector=[0.5, 0.5] session=-"]
    );
}

#[tokio::test]
async fn hybrid_without_ef_reaches_the_backend_as_none() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut search = conn.create_search_client();

    let params = HybridSearchParams::new("g", "Doc", "body", "embedding", "q", 5)
        .with_query_vector(vec![1.0, 0.0]);
    search.hybrid_search(params).await.unwrap();

    assert_eq!(
        server.backend.events_with("hybrid_search"),
        vec!["hybrid_search g k=5 ef=None vector=[1.0, 0.0] session=-"]
    );
}

#[tokio::test]
async fn hybrid_text_only_with_ef_is_accepted() {
    // No query vector: the backend decides what `ef` means without a vector
    // half; the protocol passes it on unchanged.
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut search = conn.create_search_client();

    let params = HybridSearchParams::new("g", "Doc", "body", "embedding", "q", 5).with_ef(64);
    search.hybrid_search(params).await.unwrap();

    assert_eq!(
        server.backend.events_with("hybrid_search"),
        vec!["hybrid_search g k=5 ef=Some(64) vector=[] session=-"]
    );
}

#[test]
fn hybrid_ef_set_to_none_clears_it() {
    let params = HybridSearchParams::new("g", "Doc", "body", "embedding", "q", 5)
        .with_ef(256)
        .with_ef(None);
    assert_eq!(params.ef, None);
}

#[tokio::test]
async fn raw_proto_hybrid_ef_reaches_the_backend() {
    // A client on generated stubs (any language) sets the field directly.
    let server = common::start().await;
    let channel = Channel::from_shared(server.endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut search = SearchServiceClient::new(channel);

    search
        .hybrid_search(proto::HybridSearchRequest {
            graph: "g".to_owned(),
            query_text: "q".to_owned(),
            k: 3,
            ef: Some(128),
            ..Default::default()
        })
        .await
        .unwrap();
    search
        .hybrid_search(proto::HybridSearchRequest {
            graph: "g".to_owned(),
            query_text: "q".to_owned(),
            k: 3,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        server.backend.events_with("hybrid_search"),
        vec![
            "hybrid_search g k=3 ef=Some(128) vector=[] session=-",
            "hybrid_search g k=3 ef=None vector=[] session=-",
        ]
    );
}

#[tokio::test]
async fn vector_ef_and_filters_reach_the_backend() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut search = conn.create_search_client();

    let filters = HashMap::from([("lang".to_owned(), Value::String("en".to_owned()))]);
    let params = VectorSearchParams::new("g", "Doc", "embedding", vec![1.0, 0.0], 3)
        .with_ef(Some(200))
        .with_filters(filters.clone());
    assert_eq!(params.filters, filters);
    search.vector_search(params).await.unwrap();

    assert_eq!(
        server.backend.events_with("vector_search"),
        vec!["vector_search g ef=Some(200) session=-"]
    );
}

#[test]
fn constructors_set_required_fields_and_default_the_rest() {
    let v = VectorSearchParams::new("g", "Doc", "embedding", vec![1.0], 3);
    assert_eq!(
        (v.graph.as_str(), v.label.as_str(), v.property.as_str(), v.k),
        ("g", "Doc", "embedding", 3)
    );
    assert_eq!(v.query_vector, vec![1.0]);
    assert_eq!(v.ef, None);
    assert_eq!(v.filters, HashMap::new());

    let t = TextSearchParams::new("g", "Doc", "body", "query", 7);
    assert_eq!(
        (
            t.graph.as_str(),
            t.label.as_str(),
            t.property.as_str(),
            t.query.as_str(),
            t.k
        ),
        ("g", "Doc", "body", "query", 7)
    );

    let h = HybridSearchParams::new("g", "Doc", "body", "embedding", "q", 9);
    assert_eq!(
        (
            h.graph.as_str(),
            h.label.as_str(),
            h.text_property.as_str(),
            h.vector_property.as_str(),
            h.query_text.as_str(),
            h.k
        ),
        ("g", "Doc", "body", "embedding", "q", 9)
    );
    assert_eq!(h.query_vector, Vec::<f32>::new());
    assert_eq!(h.ef, None);
}
