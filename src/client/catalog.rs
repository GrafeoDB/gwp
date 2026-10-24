//! Client-side wrapper for the `CatalogService` gRPC service.

use tonic::transport::Channel;

use crate::error::GqlError;
use crate::proto;
use crate::proto::catalog_service_client::CatalogServiceClient;
use crate::server::{CreateGraphConfig, GraphInfo, GraphTypeInfo, GraphTypeSpec, SchemaInfo};

/// A client for managing the catalog (schemas, graphs, graph types) on a GQL server.
///
/// Wraps the raw `CatalogServiceClient` gRPC stub with ergonomic
/// methods that return domain types instead of proto messages.
///
/// A client from [`GqlSession::catalog`](crate::client::GqlSession::catalog)
/// names its session on every request, so the server works in that
/// session's catalog; one from [`new`](Self::new) names no session.
pub struct CatalogClient {
    client: CatalogServiceClient<Channel>,
    /// Session named on every request; empty for none.
    session_id: String,
}

impl CatalogClient {
    /// Create a new catalog client from an existing tonic channel.
    ///
    /// Its requests name no session.
    #[must_use]
    pub fn new(channel: Channel) -> Self {
        Self::for_session(channel, String::new())
    }

    /// A catalog client whose requests name `session_id`.
    pub(crate) fn for_session(channel: Channel, session_id: String) -> Self {
        Self {
            client: CatalogServiceClient::new(channel),
            session_id,
        }
    }

    /// The session this client's requests name, if any.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        (!self.session_id.is_empty()).then_some(self.session_id.as_str())
    }

    // =========================================================================
    // Schema operations
    // =========================================================================

    /// List all schemas.
    ///
    /// # Errors
    ///
    /// Returns an error if the server does not support catalog management
    /// or the request fails.
    pub async fn list_schemas(&mut self) -> Result<Vec<SchemaInfo>, GqlError> {
        let resp = self
            .client
            .list_schemas(proto::ListSchemasRequest {
                session_id: self.session_id.clone(),
            })
            .await?
            .into_inner();

        Ok(resp
            .schemas
            .into_iter()
            .map(|s| SchemaInfo {
                name: s.name,
                graph_count: s.graph_count,
                graph_type_count: s.graph_type_count,
            })
            .collect())
    }

    /// Create a schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema already exists (and `if_not_exists` is false)
    /// or the request fails.
    pub async fn create_schema(&mut self, name: &str, if_not_exists: bool) -> Result<(), GqlError> {
        self.client
            .create_schema(proto::CreateSchemaRequest {
                session_id: self.session_id.clone(),
                name: name.to_owned(),
                if_not_exists,
            })
            .await?;
        Ok(())
    }

    /// Drop a schema. Returns whether the schema existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema does not exist (and `if_exists` is false)
    /// or the request fails.
    pub async fn drop_schema(&mut self, name: &str, if_exists: bool) -> Result<bool, GqlError> {
        let resp = self
            .client
            .drop_schema(proto::DropSchemaRequest {
                session_id: self.session_id.clone(),
                name: name.to_owned(),
                if_exists,
            })
            .await?
            .into_inner();
        Ok(resp.existed)
    }

    // =========================================================================
    // Graph operations
    // =========================================================================

    /// List all graphs in a schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn list_graphs(&mut self, schema: &str) -> Result<Vec<GraphInfo>, GqlError> {
        let resp = self
            .client
            .list_graphs(proto::ListGraphsRequest {
                session_id: self.session_id.clone(),
                schema: schema.to_owned(),
            })
            .await?
            .into_inner();

        Ok(resp
            .graphs
            .into_iter()
            .map(|g| GraphInfo {
                schema: g.schema,
                name: g.name,
                node_count: g.node_count,
                edge_count: g.edge_count,
                graph_type: g.graph_type,
                storage_mode: String::new(),
                memory_limit_bytes: None,
                backward_edges: None,
                threads: None,
            })
            .collect())
    }

    /// Create a graph with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph already exists or the request fails.
    pub async fn create_graph(&mut self, config: CreateGraphConfig) -> Result<GraphInfo, GqlError> {
        let type_spec = config.type_spec.map(|ts| match ts {
            GraphTypeSpec::Open => proto::create_graph_request::TypeSpec::OpenType(true),
            GraphTypeSpec::Named(name) => proto::create_graph_request::TypeSpec::GraphTypeRef(name),
        });

        let resp = self
            .client
            .create_graph(proto::CreateGraphRequest {
                session_id: self.session_id.clone(),
                schema: config.schema,
                name: config.name,
                if_not_exists: config.if_not_exists,
                or_replace: config.or_replace,
                type_spec,
                copy_of: config.copy_of,
                storage_mode: config.storage_mode,
                options: Some(proto::GraphOptions {
                    memory_limit_bytes: config.memory_limit_bytes,
                    backward_edges: config.backward_edges,
                    threads: config.threads,
                    wal_enabled: config.wal_enabled,
                    wal_durability: config.wal_durability,
                }),
            })
            .await?
            .into_inner();

        resp.graph
            .map(|g| GraphInfo {
                schema: g.schema,
                name: g.name,
                node_count: g.node_count,
                edge_count: g.edge_count,
                graph_type: g.graph_type,
                storage_mode: String::new(),
                memory_limit_bytes: None,
                backward_edges: None,
                threads: None,
            })
            .ok_or_else(|| GqlError::Protocol("server returned empty response".into()))
    }

    /// Drop a graph. Returns whether the graph existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph does not exist (and `if_exists` is false)
    /// or the request fails.
    pub async fn drop_graph(
        &mut self,
        schema: &str,
        name: &str,
        if_exists: bool,
    ) -> Result<bool, GqlError> {
        let resp = self
            .client
            .drop_graph(proto::DropGraphRequest {
                session_id: self.session_id.clone(),
                schema: schema.to_owned(),
                name: name.to_owned(),
                if_exists,
            })
            .await?
            .into_inner();
        Ok(resp.existed)
    }

    /// Get detailed information about a graph.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph is not found.
    pub async fn get_graph_info(
        &mut self,
        schema: &str,
        name: &str,
    ) -> Result<GraphInfo, GqlError> {
        let resp = self
            .client
            .get_graph_info(proto::GetGraphInfoRequest {
                session_id: self.session_id.clone(),
                schema: schema.to_owned(),
                name: name.to_owned(),
            })
            .await?
            .into_inner();

        Ok(GraphInfo {
            schema: resp.schema,
            name: resp.name,
            node_count: resp.node_count,
            edge_count: resp.edge_count,
            graph_type: resp.graph_type,
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

    // =========================================================================
    // Graph type operations
    // =========================================================================

    /// List graph types in a schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn list_graph_types(&mut self, schema: &str) -> Result<Vec<GraphTypeInfo>, GqlError> {
        let resp = self
            .client
            .list_graph_types(proto::ListGraphTypesRequest {
                session_id: self.session_id.clone(),
                schema: schema.to_owned(),
            })
            .await?
            .into_inner();

        Ok(resp
            .graph_types
            .into_iter()
            .map(|t| GraphTypeInfo {
                schema: t.schema,
                name: t.name,
            })
            .collect())
    }

    /// Create a graph type.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph type already exists or the request fails.
    pub async fn create_graph_type(
        &mut self,
        schema: &str,
        name: &str,
        if_not_exists: bool,
        or_replace: bool,
    ) -> Result<(), GqlError> {
        self.client
            .create_graph_type(proto::CreateGraphTypeRequest {
                session_id: self.session_id.clone(),
                schema: schema.to_owned(),
                name: name.to_owned(),
                if_not_exists,
                or_replace,
            })
            .await?;
        Ok(())
    }

    /// Drop a graph type. Returns whether it existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph type does not exist (and `if_exists` is false)
    /// or the request fails.
    pub async fn drop_graph_type(
        &mut self,
        schema: &str,
        name: &str,
        if_exists: bool,
    ) -> Result<bool, GqlError> {
        let resp = self
            .client
            .drop_graph_type(proto::DropGraphTypeRequest {
                session_id: self.session_id.clone(),
                schema: schema.to_owned(),
                name: name.to_owned(),
                if_exists,
            })
            .await?
            .into_inner();
        Ok(resp.existed)
    }
}
