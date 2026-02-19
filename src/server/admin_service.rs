//! `AdminService` gRPC implementation.
//!
//! Database introspection, maintenance, and index management.
//! All errors are returned as gRPC status codes.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto;
use crate::proto::admin_service_server::AdminService;

use super::backend::{GqlBackend, IndexDefinition};

/// Implementation of the `AdminService` gRPC service.
pub struct AdminServiceImpl<B: GqlBackend> {
    backend: Arc<B>,
}

impl<B: GqlBackend> AdminServiceImpl<B> {
    /// Create a new admin service.
    pub fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }
}

#[tonic::async_trait]
impl<B: GqlBackend> AdminService for AdminServiceImpl<B> {
    #[tracing::instrument(skip(self, request), fields(database))]
    async fn get_database_stats(
        &self,
        request: Request<proto::GetDatabaseStatsRequest>,
    ) -> Result<Response<proto::GetDatabaseStatsResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("database", &req.database);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let stats = self
            .backend
            .get_database_stats(&req.database)
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::GetDatabaseStatsResponse {
            node_count: stats.node_count,
            edge_count: stats.edge_count,
            label_count: stats.label_count,
            edge_type_count: stats.edge_type_count,
            property_key_count: stats.property_key_count,
            index_count: stats.index_count,
            memory_bytes: stats.memory_bytes,
            disk_bytes: stats.disk_bytes,
        }))
    }

    #[tracing::instrument(skip(self, request), fields(database))]
    async fn wal_status(
        &self,
        request: Request<proto::WalStatusRequest>,
    ) -> Result<Response<proto::WalStatusResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("database", &req.database);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let status = self
            .backend
            .wal_status(&req.database)
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::WalStatusResponse {
            enabled: status.enabled,
            path: status.path,
            size_bytes: status.size_bytes,
            record_count: status.record_count,
            last_checkpoint: status.last_checkpoint,
            current_epoch: status.current_epoch,
        }))
    }

    #[tracing::instrument(skip(self, request), fields(database))]
    async fn wal_checkpoint(
        &self,
        request: Request<proto::WalCheckpointRequest>,
    ) -> Result<Response<proto::WalCheckpointResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("database", &req.database);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        self.backend
            .wal_checkpoint(&req.database)
            .await
            .map_err(|e| e.to_optional_service_status())?;

        tracing::info!(database = %req.database, "WAL checkpoint completed");

        Ok(Response::new(proto::WalCheckpointResponse {}))
    }

    #[tracing::instrument(skip(self, request), fields(database))]
    async fn validate(
        &self,
        request: Request<proto::ValidateRequest>,
    ) -> Result<Response<proto::ValidateResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("database", &req.database);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let result = self
            .backend
            .validate(&req.database)
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::ValidateResponse {
            valid: result.valid,
            errors: result
                .errors
                .into_iter()
                .map(|e| proto::ValidationError {
                    code: e.code,
                    message: e.message,
                    context: e.context,
                })
                .collect(),
            warnings: result
                .warnings
                .into_iter()
                .map(|w| proto::ValidationWarning {
                    code: w.code,
                    message: w.message,
                    context: w.context,
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(database))]
    async fn create_index(
        &self,
        request: Request<proto::CreateIndexRequest>,
    ) -> Result<Response<proto::CreateIndexResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("database", &req.database);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let index_def = match req.index {
            Some(proto::create_index_request::Index::PropertyIndex(def)) => {
                IndexDefinition::Property {
                    property: def.property,
                }
            }
            Some(proto::create_index_request::Index::VectorIndex(def)) => {
                IndexDefinition::Vector {
                    label: def.label,
                    property: def.property,
                    dimensions: def.dimensions,
                    metric: def.metric,
                    m: def.m,
                    ef_construction: def.ef_construction,
                }
            }
            Some(proto::create_index_request::Index::TextIndex(def)) => IndexDefinition::Text {
                label: def.label,
                property: def.property,
            },
            None => {
                return Err(Status::invalid_argument("index definition is required"));
            }
        };

        self.backend
            .create_index(&req.database, index_def)
            .await
            .map_err(|e| e.to_optional_service_status())?;

        tracing::info!(database = %req.database, "index created");

        Ok(Response::new(proto::CreateIndexResponse {}))
    }

    #[tracing::instrument(skip(self, request), fields(database))]
    async fn drop_index(
        &self,
        request: Request<proto::DropIndexRequest>,
    ) -> Result<Response<proto::DropIndexResponse>, Status> {
        let req = request.into_inner();
        tracing::Span::current().record("database", &req.database);

        if req.database.is_empty() {
            return Err(Status::invalid_argument("database name is required"));
        }

        let index_def = match req.index {
            Some(proto::drop_index_request::Index::PropertyIndex(def)) => {
                IndexDefinition::Property {
                    property: def.property,
                }
            }
            Some(proto::drop_index_request::Index::VectorIndex(def)) => {
                IndexDefinition::Vector {
                    label: def.label,
                    property: def.property,
                    dimensions: None,
                    metric: None,
                    m: None,
                    ef_construction: None,
                }
            }
            Some(proto::drop_index_request::Index::TextIndex(def)) => IndexDefinition::Text {
                label: def.label,
                property: def.property,
            },
            None => {
                return Err(Status::invalid_argument("index definition is required"));
            }
        };

        let existed = self
            .backend
            .drop_index(&req.database, index_def)
            .await
            .map_err(|e| e.to_optional_service_status())?;

        Ok(Response::new(proto::DropIndexResponse { existed }))
    }
}
