"""Tests for GQLSTATUS helpers."""

from gwp_py.status import (
    GRAPH_TYPE_VIOLATION,
    INVALID_SYNTAX,
    NO_DATA,
    OMITTED_RESULT,
    SUCCESS,
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
