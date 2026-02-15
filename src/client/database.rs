//! Client-side wrapper for the `DatabaseService` gRPC service.

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;
use crate::proto::database_service_client::DatabaseServiceClient;
use crate::server::{CreateDatabaseConfig, DatabaseInfo};

/// A client for managing databases on a GQL server.
///
/// Wraps the raw `DatabaseServiceClient` gRPC stub with ergonomic
/// methods that return domain types instead of proto messages.
pub struct DatabaseClient {
    client: DatabaseServiceClient<Channel>,
}

impl DatabaseClient {
    /// Create a new database client from an existing tonic channel.
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self {
            client: DatabaseServiceClient::new(channel),
        }
    }

    /// List all databases on the server.
    ///
    /// # Errors
    ///
    /// Returns an error if the server does not support database management
    /// or the request fails.
    pub async fn list(&mut self) -> Result<Vec<DatabaseInfo>, GqlError> {
        let resp = self
            .client
            .list_databases(proto::ListDatabasesRequest {})
            .await?
            .into_inner();

        Ok(resp.databases.into_iter().map(into_info).collect())
    }

    /// Create a new database with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the database already exists or the request fails.
    pub async fn create(&mut self, config: CreateDatabaseConfig) -> Result<DatabaseInfo, GqlError> {
        let resp = self
            .client
            .create_database(proto::CreateDatabaseRequest {
                name: config.name,
                database_type: config.database_type,
                storage_mode: config.storage_mode,
                options: Some(proto::DatabaseOptions {
                    memory_limit_bytes: config.memory_limit_bytes,
                    backward_edges: config.backward_edges,
                    threads: config.threads,
                    wal_enabled: config.wal_enabled,
                    wal_durability: config.wal_durability,
                }),
            })
            .await?
            .into_inner();

        resp.database
            .map(into_info)
            .ok_or_else(|| GqlError::Protocol("server returned empty response".into()))
    }

    /// Delete a database by name.
    ///
    /// Returns the name of the deleted database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database is not found or cannot be deleted.
    pub async fn delete(&mut self, name: &str) -> Result<String, GqlError> {
        let resp = self
            .client
            .delete_database(proto::DeleteDatabaseRequest {
                name: name.to_owned(),
            })
            .await?
            .into_inner();

        Ok(resp.deleted)
    }

    /// Get detailed information about a specific database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database is not found.
    pub async fn get_info(&mut self, name: &str) -> Result<DatabaseInfo, GqlError> {
        let resp = self
            .client
            .get_database_info(proto::GetDatabaseInfoRequest {
                name: name.to_owned(),
            })
            .await?
            .into_inner();

        Ok(DatabaseInfo {
            name: resp.name,
            node_count: resp.node_count,
            edge_count: resp.edge_count,
            persistent: resp.persistent,
            database_type: resp.database_type,
            storage_mode: resp.storage_mode,
            memory_limit_bytes: if resp.memory_limit_bytes > 0 {
                Some(resp.memory_limit_bytes)
            } else {
                None
            },
            backward_edges: Some(resp.backward_edges),
            threads: if resp.threads > 0 {
                Some(resp.threads)
            } else {
                None
            },
        })
    }
}

/// Convert a proto `DatabaseSummary` to a domain `DatabaseInfo`.
fn into_info(summary: proto::DatabaseSummary) -> DatabaseInfo {
    DatabaseInfo {
        name: summary.name,
        node_count: summary.node_count,
        edge_count: summary.edge_count,
        persistent: summary.persistent,
        database_type: summary.database_type,
        // Summary doesn't include these extended fields
        storage_mode: String::new(),
        memory_limit_bytes: None,
        backward_edges: None,
        threads: None,
    }
}
