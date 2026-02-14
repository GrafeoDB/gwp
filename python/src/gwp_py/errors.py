"""Exception hierarchy for GWP."""

from __future__ import annotations

from dataclasses import dataclass


class GqlError(Exception):
    """Base exception for all GWP errors."""


class ConnectionError(GqlError):
    """Failed to connect to the server."""


class SessionError(GqlError):
    """Session-level error (not found, expired)."""


class TransactionError(GqlError):
    """Transaction state error."""


@dataclass
class DiagnosticRecord:
    """Diagnostic context from a GQLSTATUS error."""
    operation: str
    operation_code: int
    current_schema: str = ""


class GqlStatusError(GqlError):
    """GQL-domain error carrying a GQLSTATUS code."""

    def __init__(
        self,
        code: str,
        message: str,
        diagnostic: DiagnosticRecord | None = None,
    ):
        super().__init__(f"[{code}] {message}")
        self.code = code
        self.gql_message = message
        self.diagnostic = diagnostic

    def is_success(self) -> bool:
        return self.code[:2] == "00"

    def is_warning(self) -> bool:
        return self.code[:2] == "01"

    def is_exception(self) -> bool:
        c = self.code[:2]
        if c[0].isalpha():
            return True
        return c >= "08"
