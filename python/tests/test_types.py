"""Tests for GQL type wrappers."""

import datetime

from gwp_py.types import (
    Edge,
    Field,
    GqlDate,
    GqlDuration,
    GqlLocalDateTime,
    GqlLocalTime,
    GqlZonedDateTime,
    GqlZonedTime,
    Node,
    Path,
    Record,
)


def test_node():
    node = Node(id=b"\x01", labels=("Person",), properties={"name": "Alice"})
    assert node.has_label("Person")
    assert not node.has_label("Company")
    assert node["name"] == "Alice"


def test_edge():
    edge = Edge(
        id=b"\x10",
        labels=("knows",),
        source_node_id=b"\x01",
        target_node_id=b"\x02",
    )
    assert edge.has_label("knows")
    assert not edge.undirected


def test_path():
    a = Node(id=b"\x01", labels=("A",))
    b = Node(id=b"\x02", labels=("B",))
    e = Edge(id=b"\x10", labels=("to",), source_node_id=b"\x01", target_node_id=b"\x02")
    path = Path(nodes=(a, b), edges=(e,))
    assert len(path) == 1
    assert path.start == a
    assert path.end == b


def test_record():
    rec = Record(fields=(Field("x", 1), Field("y", 2)))
    assert rec.get("x") == 1
    assert rec["y"] == 2
    assert rec.get("z") is None
    assert len(rec) == 2


def test_date():
    d = GqlDate(2024, 3, 15)
    assert d.to_date() == datetime.date(2024, 3, 15)


def test_local_time():
    t = GqlLocalTime(14, 30, 0, 500_000_000)
    py_time = t.to_time()
    assert py_time.hour == 14
    assert py_time.minute == 30


def test_zoned_time():
    zt = GqlZonedTime(time=GqlLocalTime(10, 0, 0), offset_minutes=60)
    py_time = zt.to_time()
    assert py_time.tzinfo is not None


def test_local_datetime():
    ldt = GqlLocalDateTime(
        date=GqlDate(2024, 6, 15),
        time=GqlLocalTime(12, 0, 0),
    )
    dt = ldt.to_datetime()
    assert dt.year == 2024
    assert dt.month == 6


def test_zoned_datetime():
    zdt = GqlZonedDateTime(
        date=GqlDate(2024, 6, 15),
        time=GqlLocalTime(12, 0, 0),
        offset_minutes=-300,
    )
    dt = zdt.to_datetime()
    assert dt.tzinfo is not None


def test_duration():
    d = GqlDuration(months=12, nanoseconds=500_000_000)
    assert d.months == 12
    assert d.nanoseconds == 500_000_000
