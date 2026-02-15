"""Database management client."""

from __future__ import annotations

from dataclasses import dataclass

import grpc

from gwp_py._generated import gql_service_pb2 as gql_pb2
from gwp_py._generated import gql_service_pb2_grpc as gql_grpc


@dataclass
class DatabaseInfo:
    """Summary information about a database."""

    name: str
    node_count: int
    edge_count: int
    persistent: bool
    database_type: str
    storage_mode: str = ""
    memory_limit_bytes: int | None = None
    backward_edges: bool | None = None
    threads: int | None = None


@dataclass
class CreateDatabaseConfig:
    """Configuration for creating a new database."""

    name: str
    database_type: str = "Lpg"
    storage_mode: str = "InMemory"
    memory_limit_bytes: int | None = None
    backward_edges: bool | None = None
    threads: int | None = None
    wal_enabled: bool | None = None
    wal_durability: str | None = None


class DatabaseClient:
    """A client for managing databases on a GWP server."""

    def __init__(self, channel: grpc.aio.Channel):
        self._stub = gql_grpc.DatabaseServiceStub(channel)

    async def list(self) -> list[DatabaseInfo]:
        """List all databases on the server."""
        resp = await self._stub.ListDatabases(gql_pb2.ListDatabasesRequest())
        return [
            DatabaseInfo(
                name=db.name,
                node_count=db.node_count,
                edge_count=db.edge_count,
                persistent=db.persistent,
                database_type=db.database_type,
            )
            for db in resp.databases
        ]

    async def create(self, config: CreateDatabaseConfig) -> DatabaseInfo:
        """Create a new database."""
        options = gql_pb2.DatabaseOptions()
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

        resp = await self._stub.CreateDatabase(
            gql_pb2.CreateDatabaseRequest(
                name=config.name,
                database_type=config.database_type,
                storage_mode=config.storage_mode,
                options=options,
            )
        )
        db = resp.database
        return DatabaseInfo(
            name=db.name,
            node_count=db.node_count,
            edge_count=db.edge_count,
            persistent=db.persistent,
            database_type=db.database_type,
        )

    async def delete(self, name: str) -> str:
        """Delete a database by name. Returns the deleted database name."""
        resp = await self._stub.DeleteDatabase(
            gql_pb2.DeleteDatabaseRequest(name=name)
        )
        return resp.deleted

    async def get_info(self, name: str) -> DatabaseInfo:
        """Get detailed information about a database."""
        resp = await self._stub.GetDatabaseInfo(
            gql_pb2.GetDatabaseInfoRequest(name=name)
        )
        return DatabaseInfo(
            name=resp.name,
            node_count=resp.node_count,
            edge_count=resp.edge_count,
            persistent=resp.persistent,
            database_type=resp.database_type,
            storage_mode=resp.storage_mode,
            memory_limit_bytes=resp.memory_limit_bytes or None,
            backward_edges=resp.backward_edges if resp.backward_edges else None,
            threads=resp.threads or None,
        )
