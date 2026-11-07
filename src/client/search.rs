//! Client-side wrapper for the `SearchService` gRPC service.

use std::collections::HashMap;

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;
use crate::proto::search_service_client::SearchServiceClient;
use crate::server::{HybridSearchParams, SearchHit, TextSearchParams, VectorSearchParams};
use crate::types::Value;

/// A client for search operations (vector, text, hybrid) on a GQL server.
///
/// Wraps the raw `SearchServiceClient` gRPC stub with ergonomic
/// methods that return domain types instead of proto messages.
///
/// A client from [`GqlSession::search`](crate::client::GqlSession::search)
/// names its session on every request, so the server can check who calls;
/// one from [`new`](Self::new) names no session.
pub struct SearchClient {
    client: SearchServiceClient<Channel>,
    /// Session named on every request; empty for none.
    session_id: String,
}

impl SearchClient {
    /// Create a new search client from an existing tonic channel.
    ///
    /// Its requests name no session, which a server that requires
    /// authentication refuses; use
    /// [`GqlSession::search`](crate::client::GqlSession::search) there.
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self::for_session(channel, String::new())
    }

    /// A search client whose requests name `session_id`.
    pub(crate) fn for_session(channel: Channel, session_id: String) -> Self {
        Self {
            client: SearchServiceClient::new(channel),
            session_id,
        }
    }

    /// The session this client's requests name, if any.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        (!self.session_id.is_empty()).then_some(self.session_id.as_str())
    }

    /// Vector similarity search (KNN via HNSW index).
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or search is not supported.
    pub async fn vector_search(
        &mut self,
        params: VectorSearchParams,
    ) -> Result<Vec<SearchHit>, GqlError> {
        let filters: HashMap<String, proto::Value> = params
            .filters
            .into_iter()
            .map(|(k, v)| (k, proto::Value::from(v)))
            .collect();

        let resp = self
            .client
            .vector_search(proto::VectorSearchRequest {
                graph: params.graph,
                label: params.label,
                property: params.property,
                query_vector: params.query_vector,
                k: params.k,
                ef: params.ef,
                filters,
                session_id: self.session_id.clone(),
            })
            .await?
            .into_inner();

        Ok(resp.hits.into_iter().map(into_hit).collect())
    }

    /// Full-text search (BM25 scoring).
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or search is not supported.
    pub async fn text_search(
        &mut self,
        params: TextSearchParams,
    ) -> Result<Vec<SearchHit>, GqlError> {
        let resp = self
            .client
            .text_search(proto::TextSearchRequest {
                graph: params.graph,
                label: params.label,
                property: params.property,
                query: params.query,
                k: params.k,
                session_id: self.session_id.clone(),
            })
            .await?
            .into_inner();

        Ok(resp.hits.into_iter().map(into_hit).collect())
    }

    /// Hybrid search combining text and vector results with rank fusion.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or search is not supported.
    pub async fn hybrid_search(
        &mut self,
        params: HybridSearchParams,
    ) -> Result<Vec<SearchHit>, GqlError> {
        let resp = self
            .client
            .hybrid_search(proto::HybridSearchRequest {
                graph: params.graph,
                label: params.label,
                text_property: params.text_property,
                vector_property: params.vector_property,
                query_text: params.query_text,
                query_vector: params.query_vector,
                k: params.k,
                session_id: self.session_id.clone(),
            })
            .await?
            .into_inner();

        Ok(resp.hits.into_iter().map(into_hit).collect())
    }
}

/// Convert a proto `SearchHit` to a domain `SearchHit`.
fn into_hit(hit: proto::SearchHit) -> SearchHit {
    SearchHit {
        node_id: hit.node_id,
        score: hit.score,
        properties: hit
            .properties
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect(),
    }
}
