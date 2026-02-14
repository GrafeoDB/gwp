package dev.grafeodb.gwp;

/**
 * GQLSTATUS code constants and helpers per ISO/IEC 39075 Chapter 23.
 *
 * <p>Status codes are 5-character strings. The first two characters identify the
 * status class. Class "00" is success, "01" is warning, "02" is no data, and
 * classes >= "08" or starting with a letter are exceptions.</p>
 */
public final class GqlStatus {

    private GqlStatus() {
        // utility class
    }

    // ========================================================================
    // Success codes (class 00)
    // ========================================================================

    /** Successful completion. */
    public static final String SUCCESS = "00000";

    /** Successful completion - omitted result. */
    public static final String OMITTED_RESULT = "00001";

    // ========================================================================
    // Warning codes (class 01)
    // ========================================================================

    /** Warning. */
    public static final String WARNING = "01000";

    /** Warning - string data, right truncation. */
    public static final String WARNING_STRING_TRUNCATION = "01004";

    /** Warning - null value eliminated in set function. */
    public static final String WARNING_NULL_ELIMINATED = "01G11";

    // ========================================================================
    // No data (class 02)
    // ========================================================================

    /** No data. */
    public static final String NO_DATA = "02000";

    // ========================================================================
    // Data exceptions (class 22)
    // ========================================================================

    /** Data exception. */
    public static final String DATA_EXCEPTION = "22000";

    /** Data exception - string data, right truncation. */
    public static final String STRING_TRUNCATION = "22001";

    /** Data exception - numeric value out of range. */
    public static final String NUMERIC_OUT_OF_RANGE = "22003";

    /** Data exception - null value not allowed. */
    public static final String NULL_NOT_ALLOWED = "22004";

    /** Data exception - invalid datetime format. */
    public static final String INVALID_DATETIME_FORMAT = "22007";

    /** Data exception - datetime field overflow. */
    public static final String DATETIME_OVERFLOW = "22008";

    /** Data exception - division by zero. */
    public static final String DIVISION_BY_ZERO = "22012";

    /** Data exception - invalid value type. */
    public static final String INVALID_VALUE_TYPE = "22G03";

    /** Data exception - values not comparable. */
    public static final String NOT_COMPARABLE = "22G04";

    /** Data exception - record field mismatch. */
    public static final String RECORD_MISMATCH = "22G0U";

    /** Data exception - malformed path value. */
    public static final String MALFORMED_PATH = "22G0Z";

    // ========================================================================
    // Transaction state (class 25)
    // ========================================================================

    /** Invalid transaction state. */
    public static final String INVALID_TRANSACTION_STATE = "25000";

    /** Invalid transaction state - active transaction exists. */
    public static final String ACTIVE_TRANSACTION = "25G01";

    /** Invalid transaction state - read-only transaction. */
    public static final String READ_ONLY_TRANSACTION = "25G03";

    // ========================================================================
    // Transaction termination (class 2D)
    // ========================================================================

    /** Invalid transaction termination. */
    public static final String INVALID_TRANSACTION_TERMINATION = "2D000";

    // ========================================================================
    // Transaction rollback (class 40)
    // ========================================================================

    /** Transaction rollback. */
    public static final String TRANSACTION_ROLLBACK = "40000";

    /** Transaction rollback - completion unknown. */
    public static final String COMPLETION_UNKNOWN = "40003";

    // ========================================================================
    // Syntax / access (class 42)
    // ========================================================================

    /** Syntax error or access rule violation. */
    public static final String SYNTAX_OR_ACCESS_ERROR = "42000";

    /** Syntax error or access rule violation - invalid syntax. */
    public static final String INVALID_SYNTAX = "42001";

    /** Syntax error or access rule violation - invalid reference. */
    public static final String INVALID_REFERENCE = "42002";

    // ========================================================================
    // Graph type violation (class G2)
    // ========================================================================

    /** Graph type violation. */
    public static final String GRAPH_TYPE_VIOLATION = "G2000";

    // ========================================================================
    // Helper methods
    // ========================================================================

    /**
     * Return the class (first 2 characters) of a GQLSTATUS code.
     *
     * @param code the 5-character GQLSTATUS code
     * @return the 2-character status class
     */
    public static String statusClass(String code) {
        if (code == null || code.length() < 2) {
            return code == null ? "" : code;
        }
        return code.substring(0, 2);
    }

    /**
     * Check if a GQLSTATUS code represents success (class "00").
     *
     * @param code the 5-character GQLSTATUS code
     * @return true if the code is a success status
     */
    public static boolean isSuccess(String code) {
        return "00".equals(statusClass(code));
    }

    /**
     * Check if a GQLSTATUS code represents a warning (class "01").
     *
     * @param code the 5-character GQLSTATUS code
     * @return true if the code is a warning status
     */
    public static boolean isWarning(String code) {
        return "01".equals(statusClass(code));
    }

    /**
     * Check if a GQLSTATUS code represents no data (class "02").
     *
     * @param code the 5-character GQLSTATUS code
     * @return true if the code is a no-data status
     */
    public static boolean isNoData(String code) {
        return "02".equals(statusClass(code));
    }

    /**
     * Check if a GQLSTATUS code represents an exception.
     *
     * <p>An exception is any status class that starts with a letter, or any
     * numeric class >= "08".</p>
     *
     * @param code the 5-character GQLSTATUS code
     * @return true if the code is an exception status
     */
    public static boolean isException(String code) {
        String cls = statusClass(code);
        if (cls.length() < 2) {
            return false;
        }
        char first = cls.charAt(0);
        if (Character.isLetter(first)) {
            return true;
        }
        return cls.compareTo("08") >= 0;
    }
}
