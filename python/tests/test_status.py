"""Tests for GQLSTATUS helpers."""

from gwp_py.status import (
    CATALOG_AND_DATA_STATEMENT_MIXING,
    DUPLICATE_DEFINITION,
    EDGES_STILL_EXIST,
    GRAPH_DEPENDS_ON_SCHEMA,
    GRAPH_TYPE_VIOLATION,
    INVALID_ELEMENT_ID,
    INVALID_SYNTAX,
    LIST_DATA_RIGHT_TRUNCATION,
    MULTIPLE_GRAPHS_NOT_SUPPORTED,
    NEGATIVE_LIMIT,
    NO_ACTIVE_TRANSACTION,
    NO_DATA,
    OMITTED_RESULT,
    PROCEDURE_NOT_FOUND,
    SUCCESS,
    TRANSACTION_FAILED_STATE,
    VISUALLY_CONFUSABLE_IDENTIFIERS,
    WARNING,
    is_exception,
    is_no_data,
    is_success,
    is_warning,
    status_class,
)


def test_success():
    assert is_success(SUCCESS)
    assert not is_exception(SUCCESS)


def test_omitted():
    assert is_success(OMITTED_RESULT)


def test_warning():
    assert is_warning(WARNING)
    assert not is_success(WARNING)
    assert not is_exception(WARNING)


def test_no_data():
    assert is_no_data(NO_DATA)
    assert not is_success(NO_DATA)


def test_exception():
    assert is_exception(INVALID_SYNTAX)
    assert not is_success(INVALID_SYNTAX)


def test_graph_type_violation():
    assert is_exception(GRAPH_TYPE_VIOLATION)


def test_class_extraction():
    assert status_class("00000") == "00"
    assert status_class("42001") == "42"
    assert status_class("G2000") == "G2"


def test_codes_follow_iso_table_8():
    assert NEGATIVE_LIMIT == "22G02"
    assert LIST_DATA_RIGHT_TRUNCATION == "22G0B"
    assert CATALOG_AND_DATA_STATEMENT_MIXING == "25G02"
    assert MULTIPLE_GRAPHS_NOT_SUPPORTED == "25G04"
    assert VISUALLY_CONFUSABLE_IDENTIFIERS == "42004"
    assert EDGES_STILL_EXIST == "G1001"


def test_non_iso_conditions_use_implementation_defined_subclasses():
    for code in (
        INVALID_ELEMENT_ID,
        NO_ACTIVE_TRANSACTION,
        TRANSACTION_FAILED_STATE,
        DUPLICATE_DEFINITION,
        PROCEDURE_NOT_FOUND,
        GRAPH_DEPENDS_ON_SCHEMA,
    ):
        assert code[2] == "I", code
