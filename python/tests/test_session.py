"""Integration tests for session configuration."""

import pytest

from gwp_py import GqlConnection


@pytest.mark.asyncio
async def test_set_graph(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            await session.set_graph("mygraph")


@pytest.mark.asyncio
async def test_set_schema(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            await session.set_schema("myschema")


@pytest.mark.asyncio
async def test_set_time_zone(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            await session.set_time_zone(-300)


@pytest.mark.asyncio
async def test_reset(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            await session.set_graph("mygraph")
            await session.reset()
