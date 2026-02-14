"""Transaction management."""

from __future__ import annotations

from typing import Any

from gwp_py._convert import value_to_proto
from gwp_py._generated import gql_service_pb2 as gql_pb2
from gwp_py._generated import gql_types_pb2 as types_pb2
from gwp_py.errors import GqlStatusError, TransactionError
from gwp_py.result import ResultCursor
from gwp_py.status import is_exception


class Transaction:
    """An explicit transaction within a session."""

    def __init__(
        self,
        session_id: str,
        transaction_id: str,
        gql_stub: Any,
    ):
        self._session_id = session_id
        self._transaction_id = transaction_id
        self._gql_stub = gql_stub
        self._committed = False
        self._rolled_back = False

    @classmethod
    async def begin(
        cls,
        session_id: str,
        gql_stub: Any,
        mode: int,
    ) -> Transaction:
        """Begin a new transaction."""
        resp = await gql_stub.BeginTransaction(
            gql_pb2.BeginRequest(session_id=session_id, mode=mode)
        )

        if resp.status and is_exception(resp.status.code):
            raise GqlStatusError(resp.status.code, resp.status.message)

        if not resp.transaction_id:
            raise TransactionError("server returned empty transaction ID")

        return cls(session_id, resp.transaction_id, gql_stub)

    @property
    def transaction_id(self) -> str:
        """The transaction identifier."""
        return self._transaction_id

    async def execute(
        self,
        statement: str,
        parameters: dict[str, Any] | None = None,
    ) -> ResultCursor:
        """Execute a statement within this transaction."""
        proto_params = {}
        if parameters:
            for k, v in parameters.items():
                proto_params[k] = value_to_proto(v, types_pb2)

        stream = self._gql_stub.Execute(
            gql_pb2.ExecuteRequest(
                session_id=self._session_id,
                statement=statement,
                parameters=proto_params,
                transaction_id=self._transaction_id,
            )
        )

        return ResultCursor(stream)

    async def commit(self) -> None:
        """Commit the transaction."""
        resp = await self._gql_stub.Commit(
            gql_pb2.CommitRequest(
                session_id=self._session_id,
                transaction_id=self._transaction_id,
            )
        )
        self._committed = True

        if resp.status and is_exception(resp.status.code):
            raise GqlStatusError(resp.status.code, resp.status.message)

    async def rollback(self) -> None:
        """Roll back the transaction."""
        if self._committed or self._rolled_back:
            return

        resp = await self._gql_stub.Rollback(
            gql_pb2.RollbackRequest(
                session_id=self._session_id,
                transaction_id=self._transaction_id,
            )
        )
        self._rolled_back = True

        if resp.status and is_exception(resp.status.code):
            raise GqlStatusError(resp.status.code, resp.status.message)

    async def __aenter__(self) -> Transaction:
        return self

    async def __aexit__(self, exc_type: type | None, *args: object) -> None:
        if exc_type is not None:
            await self.rollback()
        elif not self._committed:
            await self.commit()
