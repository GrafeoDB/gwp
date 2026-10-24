package dev.grafeo.gwp;

/**
 * GQLSTATUS code constants and helpers per ISO/IEC 39075 Chapter 23.
 *
 * <p>Status codes are 5-character strings. The first two characters identify the
 * status class. Class "00" is success, "01" is warning, "02" is no data, and
 * classes >= "08" or starting with a letter are exceptions.</p>
 *
 * <p>The constants cover every condition in ISO/IEC 39075 Table 8. Conditions
 * that gwp names but the standard does not define use implementation-defined
 * subclasses (ISO 23.1): their subclass starts with {@code I}, as in
 * {@code 42I01}.</p>
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

    /** Successful completion with omitted result (DDL, session commands). */
    public static final String OMITTED_RESULT = "00001";

    // ========================================================================
    // Warning codes (class 01)
    // ========================================================================

    /** Warning (no subclass). */
    public static final String WARNING = "01000";

    /** String data, right truncation. */
    public static final String WARNING_STRING_TRUNCATION = "01004";

    /** Graph does not exist. */
    public static final String WARNING_GRAPH_NOT_FOUND = "01G03";

    /** Graph type does not exist. */
    public static final String WARNING_GRAPH_TYPE_NOT_FOUND = "01G04";

    /** Null value eliminated in set function. */
    public static final String WARNING_NULL_ELIMINATED = "01G11";

    // ========================================================================
    // No data (class 02)
    // ========================================================================

    /** No data - query matched nothing. */
    public static final String NO_DATA = "02000";

    // ========================================================================
    // Informational (class 03)
    // ========================================================================

    /** Informational (no subclass). */
    public static final String INFORMATIONAL = "03000";

    // ========================================================================
    // Connection exceptions (class 08)
    // ========================================================================

    /** Connection exception (no subclass). */
    public static final String CONNECTION_EXCEPTION = "08000";

    /** Transaction resolution unknown. */
    public static final String TRANSACTION_RESOLUTION_UNKNOWN = "08007";

    // ========================================================================
    // Data exceptions (class 22)
    // ========================================================================

    /** Data exception (no subclass). */
    public static final String DATA_EXCEPTION = "22000";

    /** String data, right truncation. */
    public static final String STRING_TRUNCATION = "22001";

    /** Numeric value out of range. */
    public static final String NUMERIC_OUT_OF_RANGE = "22003";

    /** Null value not allowed. */
    public static final String NULL_NOT_ALLOWED = "22004";

    /** Invalid date, time, or datetime format. */
    public static final String INVALID_DATETIME_FORMAT = "22007";

    /** Datetime field overflow. */
    public static final String DATETIME_OVERFLOW = "22008";

    /** Substring error. */
    public static final String SUBSTRING_ERROR = "22011";

    /** Division by zero. */
    public static final String DIVISION_BY_ZERO = "22012";

    /** Interval field overflow. */
    public static final String INTERVAL_FIELD_OVERFLOW = "22015";

    /** Invalid character value for cast. */
    public static final String INVALID_CHARACTER_VALUE_FOR_CAST = "22018";

    /** Invalid argument for natural logarithm. */
    public static final String INVALID_LOGARITHM_ARGUMENT = "2201E";

    /** Invalid argument for power function. */
    public static final String INVALID_POWER_ARGUMENT = "2201F";

    /** Trim error. */
    public static final String TRIM_ERROR = "22027";

    /** Array data, right truncation. */
    public static final String ARRAY_DATA_RIGHT_TRUNCATION = "2202F";

    /** Negative limit value. */
    public static final String NEGATIVE_LIMIT = "22G02";

    /** Invalid value type. */
    public static final String INVALID_VALUE_TYPE = "22G03";

    /** Values not comparable. */
    public static final String NOT_COMPARABLE = "22G04";

    /** Invalid date, time, or datetime function field name. */
    public static final String INVALID_DATETIME_FUNCTION_FIELD_NAME = "22G05";

    /** Invalid datetime function value. */
    public static final String INVALID_DATETIME_FUNCTION_VALUE = "22G06";

    /** Invalid duration function field name. */
    public static final String INVALID_DURATION_FUNCTION_FIELD_NAME = "22G07";

    /** List data, right truncation. */
    public static final String LIST_DATA_RIGHT_TRUNCATION = "22G0B";

    /** List element error. */
    public static final String LIST_ELEMENT_ERROR = "22G0C";

    /** Invalid number of paths or groups. */
    public static final String INVALID_NUMBER_OF_PATHS_OR_GROUPS = "22G0F";

    /** Invalid duration format. */
    public static final String INVALID_DURATION_FORMAT = "22G0H";

    /** Multiple assignments to a graph element property. */
    public static final String MULTIPLE_PROPERTY_ASSIGNMENTS = "22G0M";

    /** Number of node labels below supported minimum. */
    public static final String NODE_LABELS_BELOW_MINIMUM = "22G0N";

    /** Number of node labels exceeds supported maximum. */
    public static final String NODE_LABELS_ABOVE_MAXIMUM = "22G0P";

    /** Number of edge labels below supported minimum. */
    public static final String EDGE_LABELS_BELOW_MINIMUM = "22G0Q";

    /** Number of edge labels exceeds supported maximum. */
    public static final String EDGE_LABELS_ABOVE_MAXIMUM = "22G0R";

    /** Number of node properties exceeds supported maximum. */
    public static final String NODE_PROPERTIES_ABOVE_MAXIMUM = "22G0S";

    /** Number of edge properties exceeds supported maximum. */
    public static final String EDGE_PROPERTIES_ABOVE_MAXIMUM = "22G0T";

    /** Record fields do not match. */
    public static final String RECORD_MISMATCH = "22G0U";

    /** Reference value, invalid base type. */
    public static final String REFERENCE_INVALID_BASE_TYPE = "22G0V";

    /** Reference value, invalid constrained type. */
    public static final String REFERENCE_INVALID_CONSTRAINED_TYPE = "22G0W";

    /** Record data, field unassignable. */
    public static final String RECORD_FIELD_UNASSIGNABLE = "22G0X";

    /** Record data, field missing. */
    public static final String RECORD_FIELD_MISSING = "22G0Y";

    /** Malformed path. */
    public static final String MALFORMED_PATH = "22G0Z";

    /** Path data, right truncation. */
    public static final String PATH_DATA_RIGHT_TRUNCATION = "22G10";

    /** Reference value, referent deleted. */
    public static final String REFERENT_DELETED = "22G11";

    /** Invalid value type of a property or field reference. */
    public static final String INVALID_DYNAMIC_VALUE_TYPE = "22G12";

    /** Invalid group variable value. */
    public static final String INVALID_GROUP_VARIABLE_VALUE = "22G13";

    /** Incompatible temporal instant unit groups. */
    public static final String INCOMPATIBLE_TEMPORAL_UNIT_GROUPS = "22G14";

    /** Invalid element ID (implementation-defined). */
    public static final String INVALID_ELEMENT_ID = "22I01";

    /** Duplicate node in path (implementation-defined). */
    public static final String DUPLICATE_NODE_IN_PATH = "22I02";

    /** Duplicate edge in path (implementation-defined). */
    public static final String DUPLICATE_EDGE_IN_PATH = "22I03";

    /** Incompatible list element types (implementation-defined). */
    public static final String INCOMPATIBLE_LIST_ELEMENT_TYPES = "22I04";

    /** Invalid property reference (implementation-defined). */
    public static final String INVALID_PROPERTY_REFERENCE = "22I05";

    /** Property not found (implementation-defined). */
    public static final String PROPERTY_NOT_FOUND = "22I06";

    /** Invalid label value (implementation-defined). */
    public static final String INVALID_LABEL_VALUE = "22I07";

    /** Invalid element type (implementation-defined). */
    public static final String INVALID_ELEMENT_TYPE = "22I08";

    /** Incompatible record field types (implementation-defined). */
    public static final String INCOMPATIBLE_RECORD_FIELD_TYPES = "22I09";

    // ========================================================================
    // Transaction state (class 25)
    // ========================================================================

    /** Invalid transaction state (no subclass). */
    public static final String INVALID_TRANSACTION_STATE = "25000";

    /** Active GQL-transaction. */
    public static final String ACTIVE_TRANSACTION = "25G01";

    /** Catalog and data statement mixing not supported. */
    public static final String CATALOG_AND_DATA_STATEMENT_MIXING = "25G02";

    /** Read-only GQL-transaction. */
    public static final String READ_ONLY_TRANSACTION = "25G03";

    /** Accessing multiple graphs not supported. */
    public static final String MULTIPLE_GRAPHS_NOT_SUPPORTED = "25G04";

    /** No active GQL-transaction (implementation-defined). */
    public static final String NO_ACTIVE_TRANSACTION = "25I01";

    /** GQL-transaction in failed state (implementation-defined). */
    public static final String TRANSACTION_FAILED_STATE = "25I02";

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

    /** Statement completion unknown. */
    public static final String COMPLETION_UNKNOWN = "40003";

    // ========================================================================
    // Syntax / access (class 42)
    // ========================================================================

    /** Syntax error or access rule violation (no subclass). */
    public static final String SYNTAX_OR_ACCESS_ERROR = "42000";

    /** Invalid syntax. */
    public static final String INVALID_SYNTAX = "42001";

    /** Invalid reference. */
    public static final String INVALID_REFERENCE = "42002";

    /** Use of visually confusable identifiers. */
    public static final String VISUALLY_CONFUSABLE_IDENTIFIERS = "42004";

    /** Graph type: number of edge labels below supported minimum. */
    public static final String GRAPH_TYPE_EDGE_LABELS_BELOW_MINIMUM = "42006";

    /** Graph type: number of edge labels exceeds supported maximum. */
    public static final String GRAPH_TYPE_EDGE_LABELS_ABOVE_MAXIMUM = "42007";

    /** Graph type: number of edge properties exceeds supported maximum. */
    public static final String GRAPH_TYPE_EDGE_PROPERTIES_ABOVE_MAXIMUM = "42008";

    /** Graph type: number of node labels below supported minimum. */
    public static final String GRAPH_TYPE_NODE_LABELS_BELOW_MINIMUM = "42009";

    /** Graph type: number of node labels exceeds supported maximum. */
    public static final String GRAPH_TYPE_NODE_LABELS_ABOVE_MAXIMUM = "42010";

    /** Graph type: number of node properties exceeds supported maximum. */
    public static final String GRAPH_TYPE_NODE_PROPERTIES_ABOVE_MAXIMUM = "42011";

    /** Graph type: number of node type key labels below supported minimum. */
    public static final String GRAPH_TYPE_NODE_KEY_LABELS_BELOW_MINIMUM = "42012";

    /** Graph type: number of node type key labels exceeds supported maximum. */
    public static final String GRAPH_TYPE_NODE_KEY_LABELS_ABOVE_MAXIMUM = "42013";

    /** Graph type: number of edge type key labels below supported minimum. */
    public static final String GRAPH_TYPE_EDGE_KEY_LABELS_BELOW_MINIMUM = "42014";

    /** Graph type: number of edge type key labels exceeds supported maximum. */
    public static final String GRAPH_TYPE_EDGE_KEY_LABELS_ABOVE_MAXIMUM = "42015";

    /** Duplicate definition (implementation-defined). */
    public static final String DUPLICATE_DEFINITION = "42I01";

    /** Ambiguous reference (implementation-defined). */
    public static final String AMBIGUOUS_REFERENCE = "42I02";

    /** Unsupported feature (implementation-defined). */
    public static final String UNSUPPORTED_FEATURE = "42I03";

    /** Duplicate label (implementation-defined). */
    public static final String DUPLICATE_LABEL = "42I04";

    /** Invalid number of arguments (implementation-defined). */
    public static final String INVALID_ARGUMENT_COUNT = "42I05";

    /** Incompatible types (implementation-defined). */
    public static final String INCOMPATIBLE_TYPES = "42I06";

    /** Invalid pattern (implementation-defined). */
    public static final String INVALID_PATTERN = "42I07";

    /** Invalid operand for aggregation (implementation-defined). */
    public static final String INVALID_AGGREGATION_OPERAND = "42I08";

    /** Invalid ordering specification (implementation-defined). */
    public static final String INVALID_ORDERING = "42I09";

    /** Missing mandatory property (implementation-defined). */
    public static final String MISSING_MANDATORY_PROPERTY = "42I10";

    /** Invalid graph modification (implementation-defined). */
    public static final String INVALID_GRAPH_MODIFICATION = "42I11";

    /** Procedure not found (implementation-defined). */
    public static final String PROCEDURE_NOT_FOUND = "42I12";

    // ========================================================================
    // Dependent object errors (class G1)
    // ========================================================================

    /** Dependent object error (no subclass). */
    public static final String DEPENDENT_OBJECTS_EXIST = "G1000";

    /** Edges still exist. */
    public static final String EDGES_STILL_EXIST = "G1001";

    /** Endpoint node is deleted. */
    public static final String ENDPOINT_NODE_DELETED = "G1002";

    /** Endpoint node not in current working graph. */
    public static final String ENDPOINT_NODE_NOT_IN_GRAPH = "G1003";

    /** Graph depends on schema (implementation-defined). */
    public static final String GRAPH_DEPENDS_ON_SCHEMA = "G1I01";

    /** Graph type depends on schema (implementation-defined). */
    public static final String GRAPH_TYPE_DEPENDS_ON_SCHEMA = "G1I02";

    /** Graph depends on graph type (implementation-defined). */
    public static final String GRAPH_DEPENDS_ON_GRAPH_TYPE = "G1I03";

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
