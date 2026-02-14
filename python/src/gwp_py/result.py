"""Streaming result cursor."""

from __future__ import annotations

from typing import Any

import grpc

from gwp_py._convert import value_from_proto
from gwp_py.status import is_success


class ResultSummary:
    """Summary of a completed query execution."""

    def __init__(self, proto_summary: Any):
        self._proto = proto_summary

    @property
    def status_code(self) -> str:
        """The GQLSTATUS code."""
        return self._proto.status.code if self._proto.status else ""

    @property
    def message(self) -> str:
        """The status message."""
        return self._proto.status.message if self._proto.status else ""

    @property
    def rows_affected(self) -> int:
        """Number of rows affected."""
        return self._proto.rows_affected

    @property
    def counters(self) -> dict[str, int]:
        """Operation counters."""
        return dict(self._proto.counters)

    def is_success(self) -> bool:
        """Check if the execution was successful."""
        return is_success(self.status_code)


class ResultCursor:
    """Cursor over streaming result frames from an Execute RPC."""

    def __init__(self, stream: Any):
        self._stream = stream
        self._header: Any | None = None
        self._summary: Any | None = None
        self._buffered_rows: list[list[Any]] = []
        self._row_index: int = 0
        self._done: bool = False

    async def _consume_until_rows_or_done(self) -> None:
        """Read frames until we have buffered rows or reach the end."""
        while not self._done and self._row_index >= len(self._buffered_rows):
            try:
                response = await self._stream.read()
                if response is grpc.aio.EOF:
                    self._done = True
                    return
            except Exception:
                self._done = True
                return

            frame = response.WhichOneof("frame")
            if frame == "header":
                self._header = response.header
            elif frame == "row_batch":
                for row in response.row_batch.rows:
                    self._buffered_rows.append(
                        [value_from_proto(v) for v in row.values]
                    )
            elif frame == "summary":
                self._summary = response.summary
                self._done = True

    async def column_names(self) -> list[str]:
        """Get the column names from the result header."""
        if self._header is None:
            await self._consume_until_rows_or_done()
        if self._header is None:
            return []
        return [col.name for col in self._header.columns]

    async def next_row(self) -> list[Any] | None:
        """Get the next row. Returns None when done."""
        if self._row_index < len(self._buffered_rows):
            row = self._buffered_rows[self._row_index]
            self._row_index += 1
            return row

        await self._consume_until_rows_or_done()

        if self._row_index < len(self._buffered_rows):
            row = self._buffered_rows[self._row_index]
            self._row_index += 1
            return row

        return None

    async def collect_rows(self) -> list[list[Any]]:
        """Collect all remaining rows."""
        rows: list[list[Any]] = []
        while True:
            row = await self.next_row()
            if row is None:
                break
            rows.append(row)
        return rows

    async def summary(self) -> ResultSummary | None:
        """Get the result summary. Consumes remaining frames if needed."""
        while not self._done:
            # Skip past any buffered rows so _consume can read more frames
            self._row_index = len(self._buffered_rows)
            await self._consume_until_rows_or_done()
        if self._summary is not None:
            return ResultSummary(self._summary)
        return None

    async def is_success(self) -> bool:
        """Check if the execution was successful."""
        s = await self.summary()
        return s.is_success() if s else False

    async def rows_affected(self) -> int:
        """Get the number of rows affected."""
        s = await self.summary()
        return s.rows_affected if s else 0

    def __aiter__(self) -> ResultCursor:
        return self

    async def __anext__(self) -> list[Any]:
        row = await self.next_row()
        if row is None:
            raise StopAsyncIteration
        return row
