"""GQLSTATUS code constants and helpers."""

# Success codes (class 00)
SUCCESS = "00000"
OMITTED_RESULT = "00001"

# Warning codes (class 01)
WARNING = "01000"
WARNING_STRING_TRUNCATION = "01004"
WARNING_NULL_ELIMINATED = "01G11"

# No data (class 02)
NO_DATA = "02000"

# Data exceptions (class 22)
DATA_EXCEPTION = "22000"
STRING_TRUNCATION = "22001"
NUMERIC_OUT_OF_RANGE = "22003"
NULL_NOT_ALLOWED = "22004"
INVALID_DATETIME_FORMAT = "22007"
DATETIME_OVERFLOW = "22008"
DIVISION_BY_ZERO = "22012"
INVALID_VALUE_TYPE = "22G03"
NOT_COMPARABLE = "22G04"
RECORD_MISMATCH = "22G0U"
MALFORMED_PATH = "22G0Z"

# Transaction state (class 25)
INVALID_TRANSACTION_STATE = "25000"
ACTIVE_TRANSACTION = "25G01"
READ_ONLY_TRANSACTION = "25G03"

# Transaction termination (class 2D)
INVALID_TRANSACTION_TERMINATION = "2D000"

# Transaction rollback (class 40)
TRANSACTION_ROLLBACK = "40000"
COMPLETION_UNKNOWN = "40003"

# Syntax / access (class 42)
SYNTAX_OR_ACCESS_ERROR = "42000"
INVALID_SYNTAX = "42001"
INVALID_REFERENCE = "42002"

# Graph type violation (class G2)
GRAPH_TYPE_VIOLATION = "G2000"


def status_class(code: str) -> str:
    """Return the class (first 2 chars) of a GQLSTATUS code."""
    return code[:2] if len(code) >= 2 else code


def is_success(code: str) -> bool:
    """Check if a GQLSTATUS code represents success."""
    return status_class(code) == "00"


def is_warning(code: str) -> bool:
    """Check if a GQLSTATUS code represents a warning."""
    return status_class(code) == "01"


def is_no_data(code: str) -> bool:
    """Check if a GQLSTATUS code represents no data."""
    return status_class(code) == "02"


def is_exception(code: str) -> bool:
    """Check if a GQLSTATUS code represents an exception."""
    c = status_class(code)
    if len(c) < 2:
        return False
    if c[0].isalpha():
        return True
    return c >= "08"
