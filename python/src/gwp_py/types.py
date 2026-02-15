"""GQL type wrappers for Python."""

from __future__ import annotations

import datetime
from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class GqlDate:
    """A GQL date value."""

    year: int
    month: int
    day: int

    def to_date(self) -> datetime.date:
        """Convert to a Python date."""
        return datetime.date(self.year, self.month, self.day)


@dataclass(frozen=True)
class GqlLocalTime:
    """A GQL local time value (no timezone)."""

    hour: int
    minute: int
    second: int
    nanosecond: int = 0

    def to_time(self) -> datetime.time:
        """Convert to a Python time (microsecond precision)."""
        return datetime.time(
            self.hour, self.minute, self.second, self.nanosecond // 1000
        )


@dataclass(frozen=True)
class GqlZonedTime:
    """A GQL time value with timezone offset."""

    time: GqlLocalTime
    offset_minutes: int

    def to_time(self) -> datetime.time:
        """Convert to a Python time with timezone."""
        tz = datetime.timezone(datetime.timedelta(minutes=self.offset_minutes))
        return self.time.to_time().replace(tzinfo=tz)


@dataclass(frozen=True)
class GqlLocalDateTime:
    """A GQL local datetime value (no timezone)."""

    date: GqlDate
    time: GqlLocalTime

    def to_datetime(self) -> datetime.datetime:
        """Convert to a Python datetime."""
        return datetime.datetime.combine(self.date.to_date(), self.time.to_time())


@dataclass(frozen=True)
class GqlZonedDateTime:
    """A GQL datetime value with timezone offset."""

    date: GqlDate
    time: GqlLocalTime
    offset_minutes: int

    def to_datetime(self) -> datetime.datetime:
        """Convert to a Python datetime with timezone."""
        tz = datetime.timezone(datetime.timedelta(minutes=self.offset_minutes))
        return datetime.datetime.combine(
            self.date.to_date(), self.time.to_time(), tzinfo=tz
        )


@dataclass(frozen=True)
class GqlDuration:
    """A GQL duration value (months + nanoseconds)."""

    months: int
    nanoseconds: int


@dataclass(frozen=True)
class Field:
    """A named field in a Record."""

    name: str
    value: Any


@dataclass(frozen=True)
class Record:
    """An ordered collection of named fields."""

    fields: tuple[Field, ...] = ()

    def get(self, name: str) -> Any | None:
        """Get a field value by name."""
        for f in self.fields:
            if f.name == name:
                return f.value
        return None

    def __getitem__(self, name: str) -> Any:
        for f in self.fields:
            if f.name == name:
                return f.value
        raise KeyError(name)

    def __len__(self) -> int:
        return len(self.fields)


@dataclass(frozen=True)
class Node:
    """A property graph node."""

    id: bytes
    labels: tuple[str, ...] = ()
    properties: dict[str, Any] = field(default_factory=dict)

    def has_label(self, label: str) -> bool:
        """Check if this node has the given label."""
        return label in self.labels

    def __getitem__(self, key: str) -> Any:
        return self.properties[key]


@dataclass(frozen=True)
class Edge:
    """A property graph edge."""

    id: bytes
    labels: tuple[str, ...] = ()
    source_node_id: bytes = b""
    target_node_id: bytes = b""
    undirected: bool = False
    properties: dict[str, Any] = field(default_factory=dict)

    def has_label(self, label: str) -> bool:
        """Check if this edge has the given label."""
        return label in self.labels

    def __getitem__(self, key: str) -> Any:
        return self.properties[key]


@dataclass(frozen=True)
class Path:
    """A path through a property graph."""

    nodes: tuple[Node, ...] = ()
    edges: tuple[Edge, ...] = ()

    def __len__(self) -> int:
        """Number of edges in the path."""
        return len(self.edges)

    @property
    def start(self) -> Node | None:
        """The start node."""
        return self.nodes[0] if self.nodes else None

    @property
    def end(self) -> Node | None:
        """The end node."""
        return self.nodes[-1] if self.nodes else None
