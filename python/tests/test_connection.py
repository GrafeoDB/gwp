"""Integration tests for connection and session lifecycle."""

import pytest
import pytest_asyncio

from gwp_py import GqlConnection


@pytest.mark.asyncio
async def test_connect_and_create_session(test_server):
    conn = await GqlConnection.connect(test_server)
    session = await conn.create_session()
    assert session.session_id.startswith("mock-session-")
    await session.close()
    await conn.close()


@pytest.mark.asyncio
async def test_connection_context_manager(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            assert session.session_id


@pytest.mark.asyncio
async def test_ping(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            ts = await session.ping()
            assert isinstance(ts, int)
            assert ts > 0
