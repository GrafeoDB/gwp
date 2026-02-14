"""GQL connection management."""

from __future__ import annotations

import grpc

from gwp_py.errors import ConnectionError
from gwp_py.session import GqlSession


class GqlConnection:
    """A connection to a GWP server."""

    def __init__(self, channel: grpc.aio.Channel):
        self._channel = channel

    @classmethod
    async def connect(
        cls,
        endpoint: str,
        *,
        credentials: grpc.ChannelCredentials | None = None,
    ) -> GqlConnection:
        """Connect to a GWP server.

        Args:
            endpoint: Server address (e.g. "localhost:50051").
            credentials: Optional TLS credentials.

        Returns:
            A connected GqlConnection.

        Raises:
            ConnectionError: If the connection fails.
        """
        try:
            if credentials is not None:
                channel = grpc.aio.secure_channel(endpoint, credentials)
            else:
                channel = grpc.aio.insecure_channel(endpoint)
            return cls(channel)
        except Exception as e:
            raise ConnectionError(f"failed to connect to {endpoint}: {e}") from e

    async def create_session(self) -> GqlSession:
        """Perform handshake and return a session."""
        return await GqlSession.create(self._channel)

    async def close(self) -> None:
        """Close the underlying gRPC channel."""
        await self._channel.close()

    async def __aenter__(self) -> GqlConnection:
        return self

    async def __aexit__(self, *args: object) -> None:
        await self.close()
