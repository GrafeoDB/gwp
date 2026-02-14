"""Convert between protobuf messages and native Python types."""

from __future__ import annotations

from typing import Any

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


def value_from_proto(proto_value: Any) -> Any:
    """Convert a protobuf Value to a native Python value."""
    kind = proto_value.WhichOneof("kind")
    if kind is None or kind == "null_value":
        return None
    if kind == "boolean_value":
        return proto_value.boolean_value
    if kind == "integer_value":
        return proto_value.integer_value
    if kind == "unsigned_integer_value":
        return proto_value.unsigned_integer_value
    if kind == "float_value":
        return proto_value.float_value
    if kind == "string_value":
        return proto_value.string_value
    if kind == "bytes_value":
        return proto_value.bytes_value
    if kind == "date_value":
        d = proto_value.date_value
        return GqlDate(year=d.year, month=d.month, day=d.day)
    if kind == "local_time_value":
        t = proto_value.local_time_value
        return GqlLocalTime(
            hour=t.hour, minute=t.minute, second=t.second, nanosecond=t.nanosecond
        )
    if kind == "zoned_time_value":
        zt = proto_value.zoned_time_value
        t = zt.time
        return GqlZonedTime(
            time=GqlLocalTime(
                hour=t.hour, minute=t.minute, second=t.second, nanosecond=t.nanosecond
            ),
            offset_minutes=zt.offset_minutes,
        )
    if kind == "local_datetime_value":
        ldt = proto_value.local_datetime_value
        d = ldt.date
        t = ldt.time
        return GqlLocalDateTime(
            date=GqlDate(year=d.year, month=d.month, day=d.day),
            time=GqlLocalTime(
                hour=t.hour, minute=t.minute, second=t.second, nanosecond=t.nanosecond
            ),
        )
    if kind == "zoned_datetime_value":
        zdt = proto_value.zoned_datetime_value
        d = zdt.date
        t = zdt.time
        return GqlZonedDateTime(
            date=GqlDate(year=d.year, month=d.month, day=d.day),
            time=GqlLocalTime(
                hour=t.hour, minute=t.minute, second=t.second, nanosecond=t.nanosecond
            ),
            offset_minutes=zdt.offset_minutes,
        )
    if kind == "duration_value":
        dur = proto_value.duration_value
        return GqlDuration(months=dur.months, nanoseconds=dur.nanoseconds)
    if kind == "list_value":
        return [value_from_proto(v) for v in proto_value.list_value.elements]
    if kind == "record_value":
        fields = tuple(
            Field(name=f.name, value=value_from_proto(f.value))
            for f in proto_value.record_value.fields
        )
        return Record(fields=fields)
    if kind == "node_value":
        n = proto_value.node_value
        props = {k: value_from_proto(v) for k, v in n.properties.items()}
        return Node(id=n.id, labels=tuple(n.labels), properties=props)
    if kind == "edge_value":
        e = proto_value.edge_value
        props = {k: value_from_proto(v) for k, v in e.properties.items()}
        return Edge(
            id=e.id,
            labels=tuple(e.labels),
            source_node_id=e.source_node_id,
            target_node_id=e.target_node_id,
            undirected=e.undirected,
            properties=props,
        )
    if kind == "path_value":
        p = proto_value.path_value
        nodes = tuple(
            Node(
                id=n.id,
                labels=tuple(n.labels),
                properties={k: value_from_proto(v) for k, v in n.properties.items()},
            )
            for n in p.nodes
        )
        edges = tuple(
            Edge(
                id=e.id,
                labels=tuple(e.labels),
                source_node_id=e.source_node_id,
                target_node_id=e.target_node_id,
                undirected=e.undirected,
                properties={k: value_from_proto(v) for k, v in e.properties.items()},
            )
            for e in p.edges
        )
        return Path(nodes=nodes, edges=edges)
    # BigInteger, BigFloat, Decimal - not supported in v0.1
    return None


def value_to_proto(value: Any, pb2_module: Any) -> Any:
    """Convert a native Python value to a protobuf Value."""
    if value is None:
        return pb2_module.Value(null_value=pb2_module.NullValue())
    if isinstance(value, bool):
        return pb2_module.Value(boolean_value=value)
    if isinstance(value, int):
        return pb2_module.Value(integer_value=value)
    if isinstance(value, float):
        return pb2_module.Value(float_value=value)
    if isinstance(value, str):
        return pb2_module.Value(string_value=value)
    if isinstance(value, bytes):
        return pb2_module.Value(bytes_value=value)
    if isinstance(value, GqlDate):
        return pb2_module.Value(
            date_value=pb2_module.Date(
                year=value.year, month=value.month, day=value.day
            )
        )
    if isinstance(value, GqlDuration):
        return pb2_module.Value(
            duration_value=pb2_module.Duration(
                months=value.months, nanoseconds=value.nanoseconds
            )
        )
    if isinstance(value, list):
        elements = [value_to_proto(v, pb2_module) for v in value]
        return pb2_module.Value(list_value=pb2_module.GqlList(elements=elements))
    # Fallback: null
    return pb2_module.Value(null_value=pb2_module.NullValue())
