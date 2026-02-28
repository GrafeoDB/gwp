//! Client-side wrapper for the `AdminService` gRPC service.

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;
use crate::proto::admin_service_client::AdminServiceClient;
use crate::server::{
    AdminStats, AdminValidationResult, AdminWalStatus, IndexDefinition, ValidationDiagnostic,
};

/// A client for admin operations (stats, WAL, validation, indexes) on a GQL server.
///
/// Wraps the raw `AdminServiceClient` gRPC stub with ergonomic
/// methods that return domain types instead of proto messages.
pub struct AdminClient {
    client: AdminServiceClient<Channel>,
}

impl AdminClient {
    /// Create a new admin client from an existing tonic channel.
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: AdminServiceClient::new(channel),
        }
    }

    /// Get detailed graph statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or admin is not supported.
    pub async fn get_stats(&mut self, graph: &str) -> Result<AdminStats, GqlError> {
        let resp = self
            .client
            .get_graph_stats(proto::GetGraphStatsRequest {
                graph: graph.to_owned(),
            })
            .await?
            .into_inner();

        Ok(AdminStats {
            node_count: resp.node_count,
            edge_count: resp.edge_count,
            label_count: resp.label_count,
            edge_type_count: resp.edge_type_count,
            property_key_count: resp.property_key_count,
            index_count: resp.index_count,
            memory_bytes: resp.memory_bytes,
            disk_bytes: resp.disk_bytes,
        })
    }

    /// Get WAL status for a graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or admin is not supported.
    pub async fn wal_status(&mut self, graph: &str) -> Result<AdminWalStatus, GqlError> {
        let resp = self
            .client
            .wal_status(proto::WalStatusRequest {
                graph: graph.to_owned(),
            })
            .await?
            .into_inner();

        Ok(AdminWalStatus {
            enabled: resp.enabled,
            path: resp.path,
            size_bytes: resp.size_bytes,
            record_count: resp.record_count,
            last_checkpoint: resp.last_checkpoint,
            current_epoch: resp.current_epoch,
        })
    }

    /// Force a WAL checkpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or admin is not supported.
    pub async fn wal_checkpoint(&mut self, graph: &str) -> Result<(), GqlError> {
        self.client
            .wal_checkpoint(proto::WalCheckpointRequest {
                graph: graph.to_owned(),
            })
            .await?;
        Ok(())
    }

    /// Validate graph integrity.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found or admin is not supported.
    pub async fn validate(&mut self, graph: &str) -> Result<AdminValidationResult, GqlError> {
        let resp = self
            .client
            .validate(proto::ValidateRequest {
                graph: graph.to_owned(),
            })
            .await?
            .into_inner();

        Ok(AdminValidationResult {
            valid: resp.valid,
            errors: resp
                .errors
                .into_iter()
                .map(|e| ValidationDiagnostic {
                    code: e.code,
                    message: e.message,
                    context: e.context,
                })
                .collect(),
            warnings: resp
                .warnings
                .into_iter()
                .map(|w| ValidationDiagnostic {
                    code: w.code,
                    message: w.message,
                    context: w.context,
                })
                .collect(),
        })
    }

    /// Create an index on a graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn create_index(
        &mut self,
        graph: &str,
        index: IndexDefinition,
    ) -> Result<(), GqlError> {
        let index_proto = match index {
            IndexDefinition::Property { property } => {
                proto::create_index_request::Index::PropertyIndex(proto::PropertyIndexDef {
                    property,
                })
            }
            IndexDefinition::Vector {
                label,
                property,
                dimensions,
                metric,
                m,
                ef_construction,
            } => proto::create_index_request::Index::VectorIndex(proto::VectorIndexDef {
                label,
                property,
                dimensions,
                metric,
                m,
                ef_construction,
            }),
            IndexDefinition::Text { label, property } => {
                proto::create_index_request::Index::TextIndex(proto::TextIndexDef {
                    label,
                    property,
                })
            }
        };

        self.client
            .create_index(proto::CreateIndexRequest {
                graph: graph.to_owned(),
                index: Some(index_proto),
            })
            .await?;
        Ok(())
    }

    /// Drop an index from a graph. Returns whether the index existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn drop_index(
        &mut self,
        graph: &str,
        index: IndexDefinition,
    ) -> Result<bool, GqlError> {
        let index_proto = match index {
            IndexDefinition::Property { property } => {
                proto::drop_index_request::Index::PropertyIndex(proto::PropertyIndexDef {
                    property,
                })
            }
            IndexDefinition::Vector {
                label, property, ..
            } => proto::drop_index_request::Index::VectorIndex(proto::VectorIndexDef {
                label,
                property,
                dimensions: None,
                metric: None,
                m: None,
                ef_construction: None,
            }),
            IndexDefinition::Text { label, property } => {
                proto::drop_index_request::Index::TextIndex(proto::TextIndexDef { label, property })
            }
        };

        let resp = self
            .client
            .drop_index(proto::DropIndexRequest {
                graph: graph.to_owned(),
                index: Some(index_proto),
            })
            .await?
            .into_inner();
        Ok(resp.existed)
    }
}
