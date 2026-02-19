//! `SearchService` gRPC implementation.
//!
//! Vector, text, and hybrid search operations.
//! All errors are returned as gRPC status codes.

use std::collections::HashMap;
use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto;
use crate::proto::search_service_server::SearchService;
use crate::types::Value;

use super::backend::{GqlBackend, HybridSearchParams, TextSearchParams, VectorSearchParams};

/// Implementation of the `SearchService` gRPC service.
pub struct SearchServiceImpl<B: GqlBackend> {
    backend: Arc<B>,
}

impl<B: GqlBackend> SearchServiceImpl<B> {
    /// Create a new search service.
    pub fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }
}

/// Convert a `SearchHit` into a proto `SearchHit`.
fn to_proto_hit(hit: &super::backend::SearchHit) -> proto::SearchHit {
    proto::SearchHit {
        node_id: hit.node_id,
        score: hit.score,
        properties: hit
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), proto::Value::from(v.clone())))
            .collect(),
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> SearchService for SearchServiceImpl<B> {
    #[tracing::instrument(skip(self, request), fields(database, label, property))]
    async fn vector_search(
        &self,
        request: Request<proto::VectorSearchRequest>,
    ) -> Result<Response<proto::VectorSearchResponse>, Status> {
        let req = request.into_inner();
        let span = tracing::Span::current();
        span.record("database", &req.database);
        span.record("label", &req.label);
        span.record("property", &req.property);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }
        if req.query_vector.is_empty() {
            return Err(Status::invalid_argument("query_vector is required"));
        }

        let filters: HashMap<String, Value> = req
            .filters
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect();

        let hits = self
            .backend
            .vector_search(VectorSearchParams {
                database: req.database,
                label: req.label,
                property: req.property,
                query_vector: req.query_vector,
                k: req.k,
                ef: req.ef,
                filters,
            })
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::VectorSearchResponse {
            hits: hits.iter().map(to_proto_hit).collect(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(database, label, property))]
    async fn text_search(
        &self,
        request: Request<proto::TextSearchRequest>,
    ) -> Result<Response<proto::TextSearchResponse>, Status> {
        let req = request.into_inner();
        let span = tracing::Span::current();
        span.record("database", &req.database);
        span.record("label", &req.label);
        span.record("property", &req.property);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }
        if req.query.is_empty() {
            return Err(Status::invalid_argument("query text is required"));
        }

        let hits = self
            .backend
            .text_search(TextSearchParams {
                database: req.database,
                label: req.label,
                property: req.property,
                query: req.query,
                k: req.k,
            })
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::TextSearchResponse {
            hits: hits.iter().map(to_proto_hit).collect(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(database, label))]
    async fn hybrid_search(
        &self,
        request: Request<proto::HybridSearchRequest>,
    ) -> Result<Response<proto::HybridSearchResponse>, Status> {
        let req = request.into_inner();
        let span = tracing::Span::current();
        span.record("database", &req.database);
        span.record("label", &req.label);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }
        if req.query_text.is_empty() {
            return Err(Status::invalid_argument("query_text is required"));
        }

        let hits = self
            .backend
            .hybrid_search(HybridSearchParams {
                database: req.database,
                label: req.label,
                text_property: req.text_property,
                vector_property: req.vector_property,
                query_text: req.query_text,
                query_vector: req.query_vector,
                k: req.k,
            })
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::HybridSearchResponse {
            hits: hits.iter().map(to_proto_hit).collect(),
        }))
    }
}
