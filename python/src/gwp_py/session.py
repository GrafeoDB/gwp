"""GQL session management."""

from __future__ import annotations

from typing import Any

import grpc

from gwp_py._convert import value_to_proto
from gwp_py._generated import gql_service_pb2 as gql_pb2
from gwp_py._generated import gql_service_pb2_grpc as gql_grpc
from gwp_py._generated import gql_types_pb2 as types_pb2
from gwp_py.errors import SessionError
from gwp_py.result import ResultCursor
from gwp_py.transaction import Transaction


class GqlSession:
    """An active session with a GWP server."""

    def __init__(
        self,
        session_id: str,
        session_stub: gql_grpc.SessionServiceStub,
        gql_stub: gql_grpc.GqlServiceStub,
    ):
        self._session_id = session_id
        self._session_stub = session_stub
        self._gql_stub = gql_stub
        self._closed = False

    @classmethod
    async def create(cls, channel: grpc.aio.Channel) -> GqlSession:
        """Create a new session via handshake."""
        session_stub = gql_grpc.SessionServiceStub(channel)
        gql_stub = gql_grpc.GqlServiceStub(channel)

        resp = await session_stub.Handshake(
            gql_pb2.HandshakeRequest(protocol_version=1)
        )

        if not resp.session_id:
            raise SessionError("server returned empty session ID")

        return cls(resp.session_id, session_stub, gql_stub)

    @property
    def session_id(self) -> str:
        """The session identifier."""
        return self._session_id

    async def execute(
        self,
        statement: str,
        parameters: dict[str, Any] | None = None,
    ) -> ResultCursor:
        """Execute a GQL statement."""
        proto_params = {}
        if parameters:
            for k, v in parameters.items():
                proto_params[k] = value_to_proto(v, types_pb2)

        stream = self._gql_stub.Execute(
            gql_pb2.ExecuteRequest(
                session_id=self._session_id,
                statement=statement,
                parameters=proto_params,
            )
        )

        return ResultCursor(stream)

    async def begin_transaction(self, *, read_only: bool = False) -> Transaction:
        """Begin a new transaction."""
        mode = gql_pb2.READ_ONLY if read_only else gql_pb2.READ_WRITE
        return await Transaction.begin(self._session_id, self._gql_stub, mode)

    async def set_graph(self, name: str) -> None:
        """Set the current graph."""
        await self._session_stub.Configure(
            gql_pb2.ConfigureRequest(session_id=self._session_id, graph=name)
        )

    async def set_schema(self, name: str) -> None:
        """Set the current schema."""
        await self._session_stub.Configure(
            gql_pb2.ConfigureRequest(session_id=self._session_id, schema=name)
        )

    async def set_time_zone(self, offset_minutes: int) -> None:
        """Set the session timezone."""
        await self._session_stub.Configure(
            gql_pb2.ConfigureRequest(
                session_id=self._session_id,
                time_zone_offset_minutes=offset_minutes,
            )
        )

    async def reset(self) -> None:
        """Reset session state to defaults."""
        await self._session_stub.Reset(
            gql_pb2.ResetRequest(session_id=self._session_id, target=gql_pb2.RESET_ALL)
        )

    async def ping(self) -> int:
        """Ping the server. Returns a timestamp."""
        resp = await self._session_stub.Ping(
            gql_pb2.PingRequest(session_id=self._session_id)
        )
        return resp.timestamp

    async def close(self) -> None:
        """Close the session."""
        if not self._closed:
            await self._session_stub.Close(
                gql_pb2.CloseRequest(session_id=self._session_id)
            )
            self._closed = True

    async def __aenter__(self) -> GqlSession:
        return self

    async def __aexit__(self, *args: object) -> None:
        await self.close()
