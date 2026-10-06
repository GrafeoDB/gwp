"""Typed write counters carried in ``ResultSummary.counters``."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import astuple, dataclass

#: Write counter keys in the summary's counter map. A missing key counts as 0.
COUNTER_KEYS: tuple[str, ...] = (
    "nodes_created",
    "nodes_deleted",
    "edges_created",
    "edges_deleted",
    "properties_set",
    "labels_added",
    "labels_removed",
)


@dataclass(frozen=True)
class Counters:
    """The write counters of a statement, read from the summary's counter map.

    Other entries of that map (such as ``execution_time_ms``) are not write
    counters and are left out; they stay available on
    ``ResultSummary.counters``.
    """

    nodes_created: int = 0
    nodes_deleted: int = 0
    edges_created: int = 0
    edges_deleted: int = 0
    properties_set: int = 0
    labels_added: int = 0
    labels_removed: int = 0

    @classmethod
    def from_map(cls, counters: Mapping[str, int]) -> Counters:
        """Read the write counters from a summary's counter map.

        Missing keys read as 0, and so does a negative value.
        """
        return cls(**{key: max(int(counters.get(key, 0)), 0) for key in COUNTER_KEYS})

    def contains_updates(self) -> bool:
        """Whether any counter is non-zero, that is, whether the statement changed the graph."""
        return any(value > 0 for value in astuple(self))
