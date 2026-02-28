"""Catalog management client (schemas, graphs, graph types)."""

from __future__ import annotations

from dataclasses import dataclass

import grpc

from gwp_py._generated import gql_service_pb2 as gql_pb2
from gwp_py._generated import gql_service_pb2_grpc as gql_grpc


@dataclass
class SchemaInfo:
    """Summary information about a schema."""

    name: str
    graph_count: int
    graph_type_count: int


@dataclass
class GraphInfo:
    """Summary information about a graph."""

    schema: str
    name: str
    node_count: int
    edge_count: int
    graph_type: str
    storage_mode: str = ""
    memory_limit_bytes: int | None = None
    backward_edges: bool | None = None
    threads: int | None = None


@dataclass
class GraphTypeInfo:
    """Summary information about a graph type."""

    schema: str
    name: str


@dataclass
class CreateGraphConfig:
    """Configuration for creating a new graph."""

    schema: str
    name: str
    if_not_exists: bool = False
    or_replace: bool = False
    storage_mode: str = "InMemory"
    memory_limit_bytes: int | None = None
    backward_edges: bool | None = None
    threads: int | None = None
    wal_enabled: bool | None = None
    wal_durability: str | None = None


class CatalogClient:
    """A client for managing schemas, graphs, and graph types on a GWP server."""

    def __init__(self, channel: grpc.aio.Channel):
        self._stub = gql_grpc.CatalogServiceStub(channel)

    # ========================================================================
    # Schema operations
    # ========================================================================

    async def list_schemas(self) -> list[SchemaInfo]:
        """List all schemas on the server."""
        resp = await self._stub.ListSchemas(gql_pb2.ListSchemasRequest())
        return [
            SchemaInfo(
                name=s.name,
                graph_count=s.graph_count,
                graph_type_count=s.graph_type_count,
            )
            for s in resp.schemas
        ]

    async def create_schema(self, name: str, *, if_not_exists: bool = False) -> None:
        """Create a new schema."""
        await self._stub.CreateSchema(
            gql_pb2.CreateSchemaRequest(name=name, if_not_exists=if_not_exists)
        )

    async def drop_schema(self, name: str, *, if_exists: bool = False) -> bool:
        """Drop a schema. Returns True if it existed."""
        resp = await self._stub.DropSchema(
            gql_pb2.DropSchemaRequest(name=name, if_exists=if_exists)
        )
        return resp.existed

    # ========================================================================
    # Graph operations
    # ========================================================================

    async def list_graphs(self, schema: str) -> list[GraphInfo]:
        """List all graphs in a schema."""
        resp = await self._stub.ListGraphs(gql_pb2.ListGraphsRequest(schema=schema))
        return [
            GraphInfo(
                schema=g.schema,
                name=g.name,
                node_count=g.node_count,
                edge_count=g.edge_count,
                graph_type=g.graph_type,
            )
            for g in resp.graphs
        ]

    async def create_graph(self, config: CreateGraphConfig) -> GraphInfo:
        """Create a new graph."""
        options = gql_pb2.GraphOptions()
        if config.memory_limit_bytes is not None:
            options.memory_limit_bytes = config.memory_limit_bytes
        if config.backward_edges is not None:
            options.backward_edges = config.backward_edges
        if config.threads is not None:
            options.threads = config.threads
        if config.wal_enabled is not None:
            options.wal_enabled = config.wal_enabled
        if config.wal_durability is not None:
            options.wal_durability = config.wal_durability

        resp = await self._stub.CreateGraph(
            gql_pb2.CreateGraphRequest(
                schema=config.schema,
                name=config.name,
                if_not_exists=config.if_not_exists,
                or_replace=config.or_replace,
                storage_mode=config.storage_mode,
                options=options,
            )
        )
        g = resp.graph
        return GraphInfo(
            schema=g.schema,
            name=g.name,
            node_count=g.node_count,
            edge_count=g.edge_count,
            graph_type=g.graph_type,
        )

    async def drop_graph(
        self, schema: str, name: str, *, if_exists: bool = False
    ) -> bool:
        """Drop a graph. Returns True if it existed."""
        resp = await self._stub.DropGraph(
            gql_pb2.DropGraphRequest(schema=schema, name=name, if_exists=if_exists)
        )
        return resp.existed

    async def get_graph_info(self, schema: str, name: str) -> GraphInfo:
        """Get detailed information about a graph."""
        resp = await self._stub.GetGraphInfo(
            gql_pb2.GetGraphInfoRequest(schema=schema, name=name)
        )
        return GraphInfo(
            schema=resp.schema,
            name=resp.name,
            node_count=resp.node_count,
            edge_count=resp.edge_count,
            graph_type=resp.graph_type,
            storage_mode=resp.storage_mode,
            memory_limit_bytes=resp.memory_limit_bytes or None,
            backward_edges=resp.backward_edges if resp.backward_edges else None,
            threads=resp.threads or None,
        )

    # ========================================================================
    # Graph type operations
    # ========================================================================

    async def list_graph_types(self, schema: str) -> list[GraphTypeInfo]:
        """List all graph types in a schema."""
        resp = await self._stub.ListGraphTypes(
            gql_pb2.ListGraphTypesRequest(schema=schema)
        )
        return [GraphTypeInfo(schema=t.schema, name=t.name) for t in resp.graph_types]

    async def create_graph_type(
        self,
        schema: str,
        name: str,
        *,
        if_not_exists: bool = False,
        or_replace: bool = False,
    ) -> None:
        """Create a new graph type."""
        await self._stub.CreateGraphType(
            gql_pb2.CreateGraphTypeRequest(
                schema=schema,
                name=name,
                if_not_exists=if_not_exists,
                or_replace=or_replace,
            )
        )

    async def drop_graph_type(
        self, schema: str, name: str, *, if_exists: bool = False
    ) -> bool:
        """Drop a graph type. Returns True if it existed."""
        resp = await self._stub.DropGraphType(
            gql_pb2.DropGraphTypeRequest(schema=schema, name=name, if_exists=if_exists)
        )
        return resp.existed
