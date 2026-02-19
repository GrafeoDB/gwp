//! `DatabaseService` gRPC implementation.
//!
//! Manages database lifecycle (list, create, delete, inspect).
//! All errors are returned as gRPC status codes.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::error::GqlError;
use crate::proto;
use crate::proto::database_service_server::DatabaseService;

use super::backend::{CreateDatabaseConfig, DatabaseInfo, GqlBackend};

/// Implementation of the `DatabaseService` gRPC service.
pub struct DatabaseServiceImpl<B: GqlBackend> {
    backend: Arc<B>,
}

impl<B: GqlBackend> DatabaseServiceImpl<B> {
    /// Create a new database service.
    pub fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }
}

/// Convert a `DatabaseInfo` to a proto `DatabaseSummary`.
fn to_summary(info: &DatabaseInfo) -> proto::DatabaseSummary {
    proto::DatabaseSummary {
        name: info.name.clone(),
        node_count: info.node_count,
        edge_count: info.edge_count,
        persistent: info.persistent,
        database_type: info.database_type.clone(),
    }
}

/// Map a `GqlError` to an appropriate gRPC `Status` for database operations.
///
/// Extends the common mapping with `ALREADY_EXISTS` for duplicate databases
/// and a `Session` fallback to `INVALID_ARGUMENT`.
fn map_error(err: GqlError) -> Status {
    match err {
        GqlError::Session(ref msg) if msg.contains("already exists") => {
            Status::already_exists(msg.clone())
        }
        GqlError::Session(ref msg) if msg.contains("not found") => {
            Status::not_found(msg.clone())
        }
        GqlError::Session(msg) => Status::invalid_argument(msg),
        other => other.to_optional_service_status(),
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> DatabaseService for DatabaseServiceImpl<B> {
    #[tracing::instrument(skip(self, _request))]
    async fn list_databases(
        &self,
        _request: Request<proto::ListDatabasesRequest>,
    ) -> Result<Response<proto::ListDatabasesResponse>, Status> {
        let databases = self.backend.list_databases().await.map_err(map_error)?;

        let summaries = databases.iter().map(to_summary).collect();

        Ok(Response::new(proto::ListDatabasesResponse {
            databases: summaries,
        }))
    }

    #[tracing::instrument(skip(self, request), fields(db_name))]
    async fn create_database(
        &self,
        request: Request<proto::CreateDatabaseRequest>,
    ) -> Result<Response<proto::CreateDatabaseResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("db_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let options = req.options.unwrap_or_default();
        let config = CreateDatabaseConfig {
            name: req.name,
            database_type: req.database_type,
            storage_mode: req.storage_mode,
            memory_limit_bytes: options.memory_limit_bytes,
            backward_edges: options.backward_edges,
            threads: options.threads,
            wal_enabled: options.wal_enabled,
            wal_durability: options.wal_durability,
        };

        let info = self
            .backend
            .create_database(config)
            .await
            .map_err(map_error)?;

        tracing::info!(db_name = %info.name, "database created");

        Ok(Response::new(proto::CreateDatabaseResponse {
            database: Some(to_summary(&info)),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(db_name))]
    async fn delete_database(
        &self,
        request: Request<proto::DeleteDatabaseRequest>,
    ) -> Result<Response<proto::DeleteDatabaseResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("db_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let deleted = self
            .backend
            .delete_database(&req.name)
            .await
            .map_err(map_error)?;

        tracing::info!(db_name = %deleted, "database deleted");

        Ok(Response::new(proto::DeleteDatabaseResponse { deleted }))
    }

    #[tracing::instrument(skip(self, request), fields(db_name))]
    async fn get_database_info(
        &self,
        request: Request<proto::GetDatabaseInfoRequest>,
    ) -> Result<Response<proto::GetDatabaseInfoResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("db_name", &req.name);

        if req.name.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let info = self
            .backend
            .get_database_info(&req.name)
            .await
            .map_err(map_error)?;

        Ok(Response::new(proto::GetDatabaseInfoResponse {
            name: info.name,
            node_count: info.node_count,
            edge_count: info.edge_count,
            persistent: info.persistent,
            database_type: info.database_type,
            storage_mode: info.storage_mode,
            memory_limit_bytes: info.memory_limit_bytes.unwrap_or(0),
            backward_edges: info.backward_edges.unwrap_or(false),
            threads: info.threads.unwrap_or(0),
        }))
    }
}
