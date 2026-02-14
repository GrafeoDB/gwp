package dev.grafeodb.gwp;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Unit tests for {@link GqlStatus} constants and helpers.
 */
class GqlStatusTest {

    // ========================================================================
    // isSuccess
    // ========================================================================

    @Test
    void successCodeIsSuccess() {
        assertTrue(GqlStatus.isSuccess(GqlStatus.SUCCESS));
    }

    @Test
    void successCodeIsNotException() {
        assertFalse(GqlStatus.isException(GqlStatus.SUCCESS));
    }

    @Test
    void omittedResultIsSuccess() {
        assertTrue(GqlStatus.isSuccess(GqlStatus.OMITTED_RESULT));
    }

    // ========================================================================
    // isWarning
    // ========================================================================

    @Test
    void warningCodeIsWarning() {
        assertTrue(GqlStatus.isWarning(GqlStatus.WARNING));
    }

    @Test
    void warningCodeIsNotSuccess() {
        assertFalse(GqlStatus.isSuccess(GqlStatus.WARNING));
    }

    @Test
    void warningCodeIsNotException() {
        assertFalse(GqlStatus.isException(GqlStatus.WARNING));
    }

    @Test
    void warningStringTruncationIsWarning() {
        assertTrue(GqlStatus.isWarning(GqlStatus.WARNING_STRING_TRUNCATION));
    }

    @Test
    void warningNullEliminatedIsWarning() {
        assertTrue(GqlStatus.isWarning(GqlStatus.WARNING_NULL_ELIMINATED));
    }

    // ========================================================================
    // isNoData
    // ========================================================================

    @Test
    void noDataCodeIsNoData() {
        assertTrue(GqlStatus.isNoData(GqlStatus.NO_DATA));
    }

    @Test
    void noDataCodeIsNotSuccess() {
        assertFalse(GqlStatus.isSuccess(GqlStatus.NO_DATA));
    }

    @Test
    void noDataCodeIsNotException() {
        assertFalse(GqlStatus.isException(GqlStatus.NO_DATA));
    }

    // ========================================================================
    // isException
    // ========================================================================

    @Test
    void invalidSyntaxIsException() {
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_SYNTAX));
    }

    @Test
    void invalidSyntaxIsNotSuccess() {
        assertFalse(GqlStatus.isSuccess(GqlStatus.INVALID_SYNTAX));
    }

    @Test
    void dataExceptionIsException() {
        assertTrue(GqlStatus.isException(GqlStatus.DATA_EXCEPTION));
    }

    @Test
    void divisionByZeroIsException() {
        assertTrue(GqlStatus.isException(GqlStatus.DIVISION_BY_ZERO));
    }

    @Test
    void invalidTransactionStateIsException() {
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_TRANSACTION_STATE));
    }

    @Test
    void transactionRollbackIsException() {
        assertTrue(GqlStatus.isException(GqlStatus.TRANSACTION_ROLLBACK));
    }

    @Test
    void graphTypeViolationIsException() {
        assertTrue(GqlStatus.isException(GqlStatus.GRAPH_TYPE_VIOLATION));
    }

    @Test
    void graphTypeViolationStartsWithLetter() {
        // G2 starts with a letter, so it should be an exception
        char first = GqlStatus.GRAPH_TYPE_VIOLATION.charAt(0);
        assertTrue(Character.isLetter(first));
        assertTrue(GqlStatus.isException(GqlStatus.GRAPH_TYPE_VIOLATION));
    }

    // ========================================================================
    // statusClass
    // ========================================================================

    @Test
    void statusClassExtractsFirstTwoChars() {
        assertEquals("00", GqlStatus.statusClass("00000"));
        assertEquals("42", GqlStatus.statusClass("42001"));
        assertEquals("G2", GqlStatus.statusClass("G2000"));
        assertEquals("01", GqlStatus.statusClass("01000"));
        assertEquals("02", GqlStatus.statusClass("02000"));
        assertEquals("22", GqlStatus.statusClass("22G03"));
        assertEquals("2D", GqlStatus.statusClass("2D000"));
        assertEquals("40", GqlStatus.statusClass("40003"));
        assertEquals("25", GqlStatus.statusClass("25G01"));
    }

    @Test
    void statusClassHandlesNull() {
        assertEquals("", GqlStatus.statusClass(null));
    }

    @Test
    void statusClassHandlesShortString() {
        assertEquals("A", GqlStatus.statusClass("A"));
        assertEquals("", GqlStatus.statusClass(""));
    }

    // ========================================================================
    // Edge cases
    // ========================================================================

    @Test
    void allDataExceptionCodesAreExceptions() {
        assertTrue(GqlStatus.isException(GqlStatus.DATA_EXCEPTION));
        assertTrue(GqlStatus.isException(GqlStatus.STRING_TRUNCATION));
        assertTrue(GqlStatus.isException(GqlStatus.NUMERIC_OUT_OF_RANGE));
        assertTrue(GqlStatus.isException(GqlStatus.NULL_NOT_ALLOWED));
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_DATETIME_FORMAT));
        assertTrue(GqlStatus.isException(GqlStatus.DATETIME_OVERFLOW));
        assertTrue(GqlStatus.isException(GqlStatus.DIVISION_BY_ZERO));
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_VALUE_TYPE));
        assertTrue(GqlStatus.isException(GqlStatus.NOT_COMPARABLE));
        assertTrue(GqlStatus.isException(GqlStatus.RECORD_MISMATCH));
        assertTrue(GqlStatus.isException(GqlStatus.MALFORMED_PATH));
    }

    @Test
    void allTransactionCodesAreExceptions() {
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_TRANSACTION_STATE));
        assertTrue(GqlStatus.isException(GqlStatus.ACTIVE_TRANSACTION));
        assertTrue(GqlStatus.isException(GqlStatus.READ_ONLY_TRANSACTION));
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_TRANSACTION_TERMINATION));
        assertTrue(GqlStatus.isException(GqlStatus.TRANSACTION_ROLLBACK));
        assertTrue(GqlStatus.isException(GqlStatus.COMPLETION_UNKNOWN));
    }

    @Test
    void syntaxCodesAreExceptions() {
        assertTrue(GqlStatus.isException(GqlStatus.SYNTAX_OR_ACCESS_ERROR));
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_SYNTAX));
        assertTrue(GqlStatus.isException(GqlStatus.INVALID_REFERENCE));
    }
}
