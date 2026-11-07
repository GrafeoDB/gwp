"""Integration tests for the catalog client."""

import grpc
import pytest

from gwp_py import CatalogClient, CreateGraphConfig, GqlConnection


@pytest.mark.asyncio
async def test_session_catalog_names_session(test_server):
    async with (
        await GqlConnection.connect(test_server) as conn,
        await conn.create_session() as session,
    ):
        catalog = session.catalog()
        assert catalog.session_id == session.session_id

        schemas = await catalog.list_schemas()
        assert [s.name for s in schemas] == ["default"]

        graphs = await catalog.list_graphs("default")
        assert graphs
        assert all(g.schema == "default" for g in graphs)


@pytest.mark.asyncio
async def test_connection_catalog_names_no_session(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        catalog = conn.create_catalog_client()
        assert catalog.session_id is None

        schemas = await catalog.list_schemas()
        assert [s.name for s in schemas] == ["default"]


CATALOG_CALLS = {
    "list_schemas": lambda c: c.list_schemas(),
    "create_schema": lambda c: c.create_schema("s"),
    "drop_schema": lambda c: c.drop_schema("s"),
    "list_graphs": lambda c: c.list_graphs("default"),
    "create_graph": lambda c: c.create_graph(
        CreateGraphConfig(schema="default", name="g")
    ),
    "drop_graph": lambda c: c.drop_graph("default", "g"),
    "get_graph_info": lambda c: c.get_graph_info("default", "test"),
    "list_graph_types": lambda c: c.list_graph_types("default"),
    "create_graph_type": lambda c: c.create_graph_type("default", "t"),
    "drop_graph_type": lambda c: c.drop_graph_type("default", "t"),
}


@pytest.mark.asyncio
@pytest.mark.parametrize("call", CATALOG_CALLS.values(), ids=CATALOG_CALLS.keys())
async def test_session_catalog_call_succeeds(test_server, call):
    async with (
        await GqlConnection.connect(test_server) as conn,
        await conn.create_session() as session,
    ):
        await call(session.catalog())


@pytest.mark.asyncio
@pytest.mark.parametrize("call", CATALOG_CALLS.values(), ids=CATALOG_CALLS.keys())
async def test_catalog_unknown_session_is_not_found(test_server, call):
    async with grpc.aio.insecure_channel(test_server) as channel:
        catalog = CatalogClient(channel, session_id="no-such-session")
        assert catalog.session_id == "no-such-session"

        with pytest.raises(grpc.aio.AioRpcError) as excinfo:
            await call(catalog)
        assert excinfo.value.code() == grpc.StatusCode.NOT_FOUND


@pytest.mark.asyncio
async def test_catalog_closed_session_is_not_found(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        session = await conn.create_session()
        catalog = session.catalog()
        await session.close()

        with pytest.raises(grpc.aio.AioRpcError) as excinfo:
            await catalog.list_schemas()
        assert excinfo.value.code() == grpc.StatusCode.NOT_FOUND


@pytest.mark.asyncio
async def test_catalog_empty_session_id_names_no_session():
    async with grpc.aio.insecure_channel("localhost:1") as channel:
        assert CatalogClient(channel).session_id is None
        assert CatalogClient(channel, session_id="").session_id is None
