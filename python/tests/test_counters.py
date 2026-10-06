"""Tests for typed write counters."""

import pytest

from gwp_py import COUNTER_KEYS, Counters, GqlConnection


def test_missing_keys_read_as_zero():
    counters = Counters.from_map({})
    assert counters == Counters()
    assert not counters.contains_updates()


def test_reads_every_key_and_ignores_others():
    raw = {key: i + 1 for i, key in enumerate(COUNTER_KEYS)}
    raw.update(execution_time_ms=99, rows_scanned=1000)

    counters = Counters.from_map(raw)
    assert counters == Counters(
        nodes_created=1,
        nodes_deleted=2,
        edges_created=3,
        edges_deleted=4,
        properties_set=5,
        labels_added=6,
        labels_removed=7,
    )
    assert counters.contains_updates()


def test_negative_values_read_as_zero():
    assert Counters.from_map({"nodes_created": -5}).nodes_created == 0


@pytest.mark.parametrize("key", COUNTER_KEYS)
def test_single_counter_is_an_update(key):
    assert Counters.from_map({key: 1}).contains_updates()


@pytest.mark.asyncio
async def test_insert_reports_write_counters(test_server):
    async with (
        await GqlConnection.connect(test_server) as conn,
        await conn.create_session() as session,
    ):
        cursor = await session.execute("INSERT (:Person {name: 'Alix'})")
        counters = await cursor.counters()
        assert counters == Counters(nodes_created=3, labels_added=3, properties_set=6)
        assert counters.contains_updates()

        # Other entries stay in the raw map.
        summary = await cursor.summary()
        assert summary is not None
        assert summary.counters["execution_time_ms"] == 1
        assert summary.write_counters == counters


@pytest.mark.asyncio
async def test_read_writes_nothing(test_server):
    async with (
        await GqlConnection.connect(test_server) as conn,
        await conn.create_session() as session,
    ):
        cursor = await session.execute("MATCH (n) RETURN n")
        await cursor.collect_rows()
        assert not (await cursor.counters()).contains_updates()
