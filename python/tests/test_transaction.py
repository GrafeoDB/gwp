"""Integration tests for transactions."""

import pytest

from gwp_py import GqlConnection


@pytest.mark.asyncio
async def test_begin_and_commit(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            tx = await session.begin_transaction()
            cursor = await tx.execute("INSERT INTO t VALUES (1)")
            await cursor.collect_rows()
            await tx.commit()


@pytest.mark.asyncio
async def test_begin_and_rollback(test_server):
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            tx = await session.begin_transaction()
            cursor = await tx.execute("INSERT INTO t VALUES (1)")
            await cursor.collect_rows()
            await tx.rollback()


@pytest.mark.asyncio
async def test_transaction_context_manager_commit(test_server):
    """Transaction auto-commits on clean exit."""
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            async with await session.begin_transaction() as tx:
                cursor = await tx.execute("MATCH (n) RETURN n")
                rows = await cursor.collect_rows()
                assert len(rows) == 2


@pytest.mark.asyncio
async def test_transaction_context_manager_rollback(test_server):
    """Transaction auto-rollbacks on exception."""
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            with pytest.raises(ValueError):
                async with await session.begin_transaction() as tx:
                    await tx.execute("INSERT INTO t VALUES (1)")
                    raise ValueError("test error")


@pytest.mark.asyncio
async def test_transaction_execute_match(test_server):
    """Execute a MATCH within a transaction."""
    async with await GqlConnection.connect(test_server) as conn:
        async with await conn.create_session() as session:
            async with await session.begin_transaction() as tx:
                cursor = await tx.execute("MATCH (n:Person) RETURN n.name")
                cols = await cursor.column_names()
                assert "name" in cols
                rows = await cursor.collect_rows()
                assert len(rows) == 2
