//! `CatalogService` gRPC implementation.
//!
//! Manages the catalog hierarchy: schemas, graphs, and graph types.
//! Replaces the flat `DatabaseService` with a spec-aligned catalog model (sec 12).
//! All errors are returned as gRPC status codes.
//!
//! A request may name the session it runs in (`session_id`); the backend
//! then gets that session in the [`CatalogContext`] of the `*_in` catalog
//! methods.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::error::GqlError;
use crate::proto;
use crate::proto::catalog_service_server::CatalogService;

use super::SessionManager;
use super::backend::{CatalogContext, CreateGraphConfig, GqlBackend, GraphTypeSpec, SessionHandle};

/// Implementation of the `CatalogService` gRPC service.
pub struct CatalogServiceImpl<B: GqlBackend> {
    backend: Arc<B>,
    sessions: Option<SessionManager>,
}

impl<B: GqlBackend> CatalogServiceImpl<B> {
    /// Create a new catalog service.
    ///
    /// Without a session manager (see [`with_sessions`](Self::with_sessions))
    /// a request that names a session is rejected with `NOT_FOUND`, since
    /// the session cannot be checked.
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            backend,
            sessions: None,
        }
    }

    /// Check the sessions that catalog requests name against `sessions`.
    #[must_use]
    pub fn with_sessions(mut self, sessions: SessionManager) -> Self {
        self.sessions = Some(sessions);
        self
    }

    /// The context of a request that names `session_id` (empty for none).
    ///
    /// A named session must exist; using it counts as activity.
    async fn context(&self, session_id: &str) -> Result<CatalogContext, Status> {
        tracing::Span::current().record("session_id", session_id);
        if session_id.is_empty() {
            return Ok(CatalogContext::default());
        }
        match &self.sessions {
            Some(sessions) if sessions.exists(session_id).await => {
                sessions.touch(session_id).await;
                Ok(CatalogContext::for_session(SessionHandle(
                    session_id.to_owned(),
                )))
            }
            _ => Err(Status::not_found(format!("session {session_id} not found"))),
        }
    }
}

/// Map a `GqlError` to an appropriate gRPC `Status` for catalog operations.
///
/// Extends the common mapping with `ALREADY_EXISTS` for duplicate resources.
fn map_error(err: GqlError) -> Status {
    match err {
        GqlError::Session(ref msg) if msg.contains("already exists") => {
            Status::already_exists(msg.clone())
        }
        GqlError::Session(ref msg) if msg.contains("not found") => Status::not_found(msg.clone()),
        GqlError::Session(msg) => Status::invalid_argument(msg),
        other => other.to_optional_service_status(),
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> CatalogService for CatalogServiceImpl<B> {
    // =========================================================================
    // Schema operations
    // =========================================================================

    #[tracing::instrument(skip(self, request), fields(session_id))]
    async fn list_schemas(
        &self,
        request: Request<proto::ListSchemasRequest>,
    ) -> Result<Response<proto::ListSchemasResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        let schemas = self
            .backend
            .list_schemas_in(&ctx)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::ListSchemasResponse {
            schemas: schemas
                .into_iter()
                .map(|s| proto::SchemaInfo {
                    name: s.name,
                    graph_count: s.graph_count,
                    graph_type_count: s.graph_type_count,
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema_name))]
    async fn create_schema(
        &self,
        request: Request<proto::CreateSchemaRequest>,
    ) -> Result<Response<proto::CreateSchemaResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("schema name is required"));
        }

        self.backend
            .create_schema_in(&ctx, &req.name, req.if_not_exists)
            .await
            .map_err(map_error)?;

        tracing::info!(schema = %req.name, "schema created");

        Ok(Response::new(proto::CreateSchemaResponse {}))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema_name))]
    async fn drop_schema(
        &self,
        request: Request<proto::DropSchemaRequest>,
    ) -> Result<Response<proto::DropSchemaResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("schema name is required"));
        }

        let existed = self
            .backend
            .drop_schema_in(&ctx, &req.name, req.if_exists)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::DropSchemaResponse { existed }))
    }

    // =========================================================================
    // Graph operations
    // =========================================================================

    #[tracing::instrument(skip(self, request), fields(session_id, schema))]
    async fn list_graphs(
        &self,
        request: Request<proto::ListGraphsRequest>,
    ) -> Result<Response<proto::ListGraphsResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);

        let graphs = self
            .backend
            .list_graphs_in(&ctx, &req.schema)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::ListGraphsResponse {
            graphs: graphs
                .into_iter()
                .map(|g| proto::GraphSummary {
                    schema: g.schema,
                    name: g.name,
                    node_count: g.node_count,
                    edge_count: g.edge_count,
                    graph_type: g.graph_type,
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema, graph_name))]
    async fn create_graph(
        &self,
        request: Request<proto::CreateGraphRequest>,
    ) -> Result<Response<proto::CreateGraphResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);
        tracing::Span::current().record("graph_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("graph name is required"));
        }

        let type_spec = match req.type_spec {
            Some(proto::create_graph_request::TypeSpec::OpenType(true)) => {
                Some(GraphTypeSpec::Open)
            }
            Some(proto::create_graph_request::TypeSpec::GraphTypeRef(name)) => {
                Some(GraphTypeSpec::Named(name))
            }
            _ => None,
        };

        let options = req.options.unwrap_or_default();
        let config = CreateGraphConfig {
            schema: req.schema,
            name: req.name,
            if_not_exists: req.if_not_exists,
            or_replace: req.or_replace,
            type_spec,
            copy_of: req.copy_of,
            storage_mode: req.storage_mode,
            memory_limit_bytes: options.memory_limit_bytes,
            backward_edges: options.backward_edges,
            threads: options.threads,
            wal_enabled: options.wal_enabled,
            wal_durability: options.wal_durability,
        };

        let info = self
            .backend
            .create_graph_in(&ctx, config)
            .await
            .map_err(map_error)?;

        tracing::info!(schema = %info.schema, graph = %info.name, "graph created");

        Ok(Response::new(proto::CreateGraphResponse {
            graph: Some(proto::GraphSummary {
                schema: info.schema,
                name: info.name,
                node_count: info.node_count,
                edge_count: info.edge_count,
                graph_type: info.graph_type,
            }),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema, graph_name))]
    async fn drop_graph(
        &self,
        request: Request<proto::DropGraphRequest>,
    ) -> Result<Response<proto::DropGraphResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);
        tracing::Span::current().record("graph_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("graph name is required"));
        }

        let existed = self
            .backend
            .drop_graph_in(&ctx, &req.schema, &req.name, req.if_exists)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::DropGraphResponse { existed }))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema, graph_name))]
    async fn get_graph_info(
        &self,
        request: Request<proto::GetGraphInfoRequest>,
    ) -> Result<Response<proto::GetGraphInfoResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);
        tracing::Span::current().record("graph_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("graph name is required"));
        }

        let info = self
            .backend
            .get_graph_info_in(&ctx, &req.schema, &req.name)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::GetGraphInfoResponse {
            schema: info.schema,
            name: info.name,
            node_count: info.node_count,
            edge_count: info.edge_count,
            graph_type: info.graph_type,
            storage_mode: info.storage_mode,
            memory_limit_bytes: info.memory_limit_bytes.unwrap_or(0),
            backward_edges: info.backward_edges.unwrap_or(false),
            threads: info.threads.unwrap_or(0),
        }))
    }

    // =========================================================================
    // Graph type operations
    // =========================================================================

    #[tracing::instrument(skip(self, request), fields(session_id, schema))]
    async fn list_graph_types(
        &self,
        request: Request<proto::ListGraphTypesRequest>,
    ) -> Result<Response<proto::ListGraphTypesResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);

        let types = self
            .backend
            .list_graph_types_in(&ctx, &req.schema)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::ListGraphTypesResponse {
            graph_types: types
                .into_iter()
                .map(|t| proto::GraphTypeInfo {
                    schema: t.schema,
                    name: t.name,
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema, type_name))]
    async fn create_graph_type(
        &self,
        request: Request<proto::CreateGraphTypeRequest>,
    ) -> Result<Response<proto::CreateGraphTypeResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);
        tracing::Span::current().record("type_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("graph type name is required"));
        }

        self.backend
            .create_graph_type_in(
                &ctx,
                &req.schema,
                &req.name,
                req.if_not_exists,
                req.or_replace,
            )
            .await
            .map_err(map_error)?;

        tracing::info!(schema = %req.schema, graph_type = %req.name, "graph type created");

        Ok(Response::new(proto::CreateGraphTypeResponse {}))
    }

    #[tracing::instrument(skip(self, request), fields(session_id, schema, type_name))]
    async fn drop_graph_type(
        &self,
        request: Request<proto::DropGraphTypeRequest>,
    ) -> Result<Response<proto::DropGraphTypeResponse>, Status> {
        let req = request.into_inner();
        let ctx = self.context(&req.session_id).await?;
        tracing::Span::current().record("schema", &req.schema);
        tracing::Span::current().record("type_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("graph type name is required"));
        }

        let existed = self
            .backend
            .drop_graph_type_in(&ctx, &req.schema, &req.name, req.if_exists)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::DropGraphTypeResponse { existed }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::mock_backend::MockBackend;

    fn list_schemas(session_id: &str) -> Request<proto::ListSchemasRequest> {
        Request::new(proto::ListSchemasRequest {
            session_id: session_id.to_owned(),
        })
    }

    #[tokio::test]
    async fn without_a_session_manager_a_named_session_is_not_found() {
        let service = CatalogServiceImpl::new(Arc::new(MockBackend::new()));
        let status = service.list_schemas(list_schemas("s1")).await.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
        // A request that names no session still works.
        service.list_schemas(list_schemas("")).await.unwrap();
    }

    #[tokio::test]
    async fn a_live_session_is_passed_on() {
        let sessions = SessionManager::new();
        sessions.register("s1").await.unwrap();
        let service =
            CatalogServiceImpl::new(Arc::new(MockBackend::new())).with_sessions(sessions.clone());

        service.list_schemas(list_schemas("s1")).await.unwrap();
        assert_eq!(
            service.context("s1").await.unwrap().session,
            Some(SessionHandle("s1".to_owned()))
        );
        assert_eq!(service.context("").await.unwrap().session, None);
        assert_eq!(
            service.context("s2").await.unwrap_err().code(),
            tonic::Code::NotFound
        );
    }
}
