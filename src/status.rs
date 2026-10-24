//! GQLSTATUS code constants and helpers (ISO/IEC 39075 Chapter 23).
//!
//! Every GQL operation produces a GQLSTATUS code. This module provides
//! well-known code constants and helper methods for constructing and
//! inspecting status values.
//!
//! The constants cover every condition in ISO/IEC 39075 Table 8,
//! "GQLSTATUS class and subclass codes". Conditions that gwp names but the
//! standard does not define use implementation-defined subclasses (ISO
//! 23.1): their subclass starts with `I`, as in `42I01`.

use crate::proto;

// ============================================================================
// Success codes (class 00)
// ============================================================================

/// Successful completion.
pub const SUCCESS: &str = "00000";

/// Successful completion with omitted result (DDL, session commands).
pub const OMITTED_RESULT: &str = "00001";

// ============================================================================
// Warning codes (class 01)
// ============================================================================

/// Warning (no subclass).
pub const WARNING: &str = "01000";

/// String data, right truncation.
pub const WARNING_STRING_TRUNCATION: &str = "01004";

/// Graph does not exist.
pub const WARNING_GRAPH_NOT_FOUND: &str = "01G03";

/// Graph type does not exist.
pub const WARNING_GRAPH_TYPE_NOT_FOUND: &str = "01G04";

/// Null value eliminated in set function.
pub const WARNING_NULL_ELIMINATED: &str = "01G11";

// ============================================================================
// No data (class 02)
// ============================================================================

/// No data - query matched nothing.
pub const NO_DATA: &str = "02000";

// ============================================================================
// Informational (class 03)
// ============================================================================

/// Informational (no subclass).
pub const INFORMATIONAL: &str = "03000";

// ============================================================================
// Connection exceptions (class 08)
// ============================================================================

/// Connection exception (no subclass).
pub const CONNECTION_EXCEPTION: &str = "08000";

/// Transaction resolution unknown.
pub const TRANSACTION_RESOLUTION_UNKNOWN: &str = "08007";

// ============================================================================
// Data exceptions (class 22)
// ============================================================================

/// Data exception (no subclass).
pub const DATA_EXCEPTION: &str = "22000";

/// String data, right truncation.
pub const STRING_TRUNCATION: &str = "22001";

/// Numeric value out of range.
pub const NUMERIC_OUT_OF_RANGE: &str = "22003";

/// Null value not allowed.
pub const NULL_NOT_ALLOWED: &str = "22004";

/// Invalid date, time, or datetime format.
pub const INVALID_DATETIME_FORMAT: &str = "22007";

/// Datetime field overflow.
pub const DATETIME_OVERFLOW: &str = "22008";

/// Substring error.
pub const SUBSTRING_ERROR: &str = "22011";

/// Division by zero.
pub const DIVISION_BY_ZERO: &str = "22012";

/// Interval field overflow.
pub const INTERVAL_FIELD_OVERFLOW: &str = "22015";

/// Invalid character value for cast.
pub const INVALID_CHARACTER_VALUE_FOR_CAST: &str = "22018";

/// Invalid argument for natural logarithm.
pub const INVALID_LOGARITHM_ARGUMENT: &str = "2201E";

/// Invalid argument for power function.
pub const INVALID_POWER_ARGUMENT: &str = "2201F";

/// Trim error.
pub const TRIM_ERROR: &str = "22027";

/// Array data, right truncation.
pub const ARRAY_DATA_RIGHT_TRUNCATION: &str = "2202F";

/// Negative limit value.
pub const NEGATIVE_LIMIT: &str = "22G02";

/// Invalid value type.
pub const INVALID_VALUE_TYPE: &str = "22G03";

/// Values not comparable.
pub const NOT_COMPARABLE: &str = "22G04";

/// Invalid date, time, or datetime function field name.
pub const INVALID_DATETIME_FUNCTION_FIELD_NAME: &str = "22G05";

/// Invalid datetime function value.
pub const INVALID_DATETIME_FUNCTION_VALUE: &str = "22G06";

/// Invalid duration function field name.
pub const INVALID_DURATION_FUNCTION_FIELD_NAME: &str = "22G07";

/// List data, right truncation.
pub const LIST_DATA_RIGHT_TRUNCATION: &str = "22G0B";

/// List element error.
pub const LIST_ELEMENT_ERROR: &str = "22G0C";

/// Invalid number of paths or groups.
pub const INVALID_NUMBER_OF_PATHS_OR_GROUPS: &str = "22G0F";

/// Invalid duration format.
pub const INVALID_DURATION_FORMAT: &str = "22G0H";

/// Multiple assignments to a graph element property.
pub const MULTIPLE_PROPERTY_ASSIGNMENTS: &str = "22G0M";

/// Number of node labels below supported minimum.
pub const NODE_LABELS_BELOW_MINIMUM: &str = "22G0N";

/// Number of node labels exceeds supported maximum.
pub const NODE_LABELS_ABOVE_MAXIMUM: &str = "22G0P";

/// Number of edge labels below supported minimum.
pub const EDGE_LABELS_BELOW_MINIMUM: &str = "22G0Q";

/// Number of edge labels exceeds supported maximum.
pub const EDGE_LABELS_ABOVE_MAXIMUM: &str = "22G0R";

/// Number of node properties exceeds supported maximum.
pub const NODE_PROPERTIES_ABOVE_MAXIMUM: &str = "22G0S";

/// Number of edge properties exceeds supported maximum.
pub const EDGE_PROPERTIES_ABOVE_MAXIMUM: &str = "22G0T";

/// Record fields do not match.
pub const RECORD_MISMATCH: &str = "22G0U";

/// Reference value, invalid base type.
pub const REFERENCE_INVALID_BASE_TYPE: &str = "22G0V";

/// Reference value, invalid constrained type.
pub const REFERENCE_INVALID_CONSTRAINED_TYPE: &str = "22G0W";

/// Record data, field unassignable.
pub const RECORD_FIELD_UNASSIGNABLE: &str = "22G0X";

/// Record data, field missing.
pub const RECORD_FIELD_MISSING: &str = "22G0Y";

/// Malformed path.
pub const MALFORMED_PATH: &str = "22G0Z";

/// Path data, right truncation.
pub const PATH_DATA_RIGHT_TRUNCATION: &str = "22G10";

/// Reference value, referent deleted.
pub const REFERENT_DELETED: &str = "22G11";

/// Invalid value type of a property or field reference.
///
/// ISO names this condition "invalid value type", like
/// [`INVALID_VALUE_TYPE`] (22G03); the standard raises it when a property
/// or field reference yields a value outside its declared type.
pub const INVALID_DYNAMIC_VALUE_TYPE: &str = "22G12";

/// Invalid group variable value.
pub const INVALID_GROUP_VARIABLE_VALUE: &str = "22G13";

/// Incompatible temporal instant unit groups.
pub const INCOMPATIBLE_TEMPORAL_UNIT_GROUPS: &str = "22G14";

/// Invalid element ID (implementation-defined).
pub const INVALID_ELEMENT_ID: &str = "22I01";

/// Duplicate node in path (implementation-defined).
pub const DUPLICATE_NODE_IN_PATH: &str = "22I02";

/// Duplicate edge in path (implementation-defined).
pub const DUPLICATE_EDGE_IN_PATH: &str = "22I03";

/// Incompatible list element types (implementation-defined).
pub const INCOMPATIBLE_LIST_ELEMENT_TYPES: &str = "22I04";

/// Invalid property reference (implementation-defined).
pub const INVALID_PROPERTY_REFERENCE: &str = "22I05";

/// Property not found (implementation-defined).
pub const PROPERTY_NOT_FOUND: &str = "22I06";

/// Invalid label value (implementation-defined).
pub const INVALID_LABEL_VALUE: &str = "22I07";

/// Invalid element type (implementation-defined).
pub const INVALID_ELEMENT_TYPE: &str = "22I08";

/// Incompatible record field types (implementation-defined).
pub const INCOMPATIBLE_RECORD_FIELD_TYPES: &str = "22I09";

// ============================================================================
// Transaction state (class 25)
// ============================================================================

/// Invalid transaction state (no subclass).
pub const INVALID_TRANSACTION_STATE: &str = "25000";

/// Active GQL-transaction.
pub const ACTIVE_TRANSACTION: &str = "25G01";

/// Catalog and data statement mixing not supported.
pub const CATALOG_AND_DATA_STATEMENT_MIXING: &str = "25G02";

/// Read-only GQL-transaction.
pub const READ_ONLY_TRANSACTION: &str = "25G03";

/// Accessing multiple graphs not supported.
pub const MULTIPLE_GRAPHS_NOT_SUPPORTED: &str = "25G04";

/// No active GQL-transaction (implementation-defined).
pub const NO_ACTIVE_TRANSACTION: &str = "25I01";

/// GQL-transaction in failed state (implementation-defined).
pub const TRANSACTION_FAILED_STATE: &str = "25I02";

// ============================================================================
// Transaction termination (class 2D)
// ============================================================================

/// Invalid transaction termination.
pub const INVALID_TRANSACTION_TERMINATION: &str = "2D000";

// ============================================================================
// Transaction rollback (class 40)
// ============================================================================

/// Transaction rollback.
pub const TRANSACTION_ROLLBACK: &str = "40000";

/// Statement completion unknown.
pub const COMPLETION_UNKNOWN: &str = "40003";

// ============================================================================
// Syntax / access (class 42)
// ============================================================================

/// Syntax error or access rule violation (no subclass).
pub const SYNTAX_OR_ACCESS_ERROR: &str = "42000";

/// Invalid syntax.
pub const INVALID_SYNTAX: &str = "42001";

/// Invalid reference.
pub const INVALID_REFERENCE: &str = "42002";

/// Use of visually confusable identifiers.
pub const VISUALLY_CONFUSABLE_IDENTIFIERS: &str = "42004";

/// Graph type: number of edge labels below supported minimum.
pub const GRAPH_TYPE_EDGE_LABELS_BELOW_MINIMUM: &str = "42006";

/// Graph type: number of edge labels exceeds supported maximum.
pub const GRAPH_TYPE_EDGE_LABELS_ABOVE_MAXIMUM: &str = "42007";

/// Graph type: number of edge properties exceeds supported maximum.
pub const GRAPH_TYPE_EDGE_PROPERTIES_ABOVE_MAXIMUM: &str = "42008";

/// Graph type: number of node labels below supported minimum.
pub const GRAPH_TYPE_NODE_LABELS_BELOW_MINIMUM: &str = "42009";

/// Graph type: number of node labels exceeds supported maximum.
pub const GRAPH_TYPE_NODE_LABELS_ABOVE_MAXIMUM: &str = "42010";

/// Graph type: number of node properties exceeds supported maximum.
pub const GRAPH_TYPE_NODE_PROPERTIES_ABOVE_MAXIMUM: &str = "42011";

/// Graph type: number of node type key labels below supported minimum.
pub const GRAPH_TYPE_NODE_KEY_LABELS_BELOW_MINIMUM: &str = "42012";

/// Graph type: number of node type key labels exceeds supported maximum.
pub const GRAPH_TYPE_NODE_KEY_LABELS_ABOVE_MAXIMUM: &str = "42013";

/// Graph type: number of edge type key labels below supported minimum.
pub const GRAPH_TYPE_EDGE_KEY_LABELS_BELOW_MINIMUM: &str = "42014";

/// Graph type: number of edge type key labels exceeds supported maximum.
pub const GRAPH_TYPE_EDGE_KEY_LABELS_ABOVE_MAXIMUM: &str = "42015";

/// Duplicate definition (implementation-defined).
pub const DUPLICATE_DEFINITION: &str = "42I01";

/// Ambiguous reference (implementation-defined).
pub const AMBIGUOUS_REFERENCE: &str = "42I02";

/// Unsupported feature (implementation-defined).
pub const UNSUPPORTED_FEATURE: &str = "42I03";

/// Duplicate label (implementation-defined).
pub const DUPLICATE_LABEL: &str = "42I04";

/// Invalid number of arguments (implementation-defined).
pub const INVALID_ARGUMENT_COUNT: &str = "42I05";

/// Incompatible types (implementation-defined).
pub const INCOMPATIBLE_TYPES: &str = "42I06";

/// Invalid pattern (implementation-defined).
pub const INVALID_PATTERN: &str = "42I07";

/// Invalid operand for aggregation (implementation-defined).
pub const INVALID_AGGREGATION_OPERAND: &str = "42I08";

/// Invalid ordering specification (implementation-defined).
pub const INVALID_ORDERING: &str = "42I09";

/// Missing mandatory property (implementation-defined).
pub const MISSING_MANDATORY_PROPERTY: &str = "42I10";

/// Invalid graph modification (implementation-defined).
pub const INVALID_GRAPH_MODIFICATION: &str = "42I11";

/// Procedure not found (implementation-defined).
pub const PROCEDURE_NOT_FOUND: &str = "42I12";

// ============================================================================
// Dependent object errors (class G1)
// ============================================================================

/// Dependent object error (no subclass).
pub const DEPENDENT_OBJECTS_EXIST: &str = "G1000";

/// Edges still exist.
pub const EDGES_STILL_EXIST: &str = "G1001";

/// Endpoint node is deleted.
pub const ENDPOINT_NODE_DELETED: &str = "G1002";

/// Endpoint node not in current working graph.
pub const ENDPOINT_NODE_NOT_IN_GRAPH: &str = "G1003";

/// Graph depends on schema (implementation-defined).
pub const GRAPH_DEPENDS_ON_SCHEMA: &str = "G1I01";

/// Graph type depends on schema (implementation-defined).
pub const GRAPH_TYPE_DEPENDS_ON_SCHEMA: &str = "G1I02";

/// Graph depends on graph type (implementation-defined).
pub const GRAPH_DEPENDS_ON_GRAPH_TYPE: &str = "G1I03";

// ============================================================================
// Graph type violation (class G2)
// ============================================================================

/// Graph type violation.
pub const GRAPH_TYPE_VIOLATION: &str = "G2000";

// ============================================================================
// Constructors
// ============================================================================

/// Create a successful `GqlStatus`.
#[must_use]
pub fn success() -> proto::GqlStatus {
    proto::GqlStatus {
        code: SUCCESS.to_owned(),
        message: "successful completion".to_owned(),
        diagnostic: None,
        cause: None,
    }
}

/// Create a successful `GqlStatus` with omitted result.
#[must_use]
pub fn omitted() -> proto::GqlStatus {
    proto::GqlStatus {
        code: OMITTED_RESULT.to_owned(),
        message: "successful completion - omitted result".to_owned(),
        diagnostic: None,
        cause: None,
    }
}

/// Create a no-data `GqlStatus`.
#[must_use]
pub fn no_data() -> proto::GqlStatus {
    proto::GqlStatus {
        code: NO_DATA.to_owned(),
        message: "no data".to_owned(),
        diagnostic: None,
        cause: None,
    }
}

/// Create an error `GqlStatus` with the given code and message.
#[must_use]
pub fn error(code: &str, message: impl Into<String>) -> proto::GqlStatus {
    proto::GqlStatus {
        code: code.to_owned(),
        message: message.into(),
        diagnostic: None,
        cause: None,
    }
}

/// Create a warning `GqlStatus` with the given code and message.
#[must_use]
pub fn warning(code: &str, message: impl Into<String>) -> proto::GqlStatus {
    proto::GqlStatus {
        code: code.to_owned(),
        message: message.into(),
        diagnostic: None,
        cause: None,
    }
}

/// Create an informational `GqlStatus` with the given code and message.
#[must_use]
pub fn informational(code: &str, message: impl Into<String>) -> proto::GqlStatus {
    proto::GqlStatus {
        code: code.to_owned(),
        message: message.into(),
        diagnostic: None,
        cause: None,
    }
}

/// Create an error `GqlStatus` with diagnostic context.
#[must_use]
pub fn error_with_diagnostic(
    code: &str,
    message: impl Into<String>,
    operation: impl Into<String>,
    operation_code: i32,
) -> proto::GqlStatus {
    proto::GqlStatus {
        code: code.to_owned(),
        message: message.into(),
        diagnostic: Some(proto::DiagnosticRecord {
            operation: operation.into(),
            operation_code,
            current_schema: None,
            invalid_reference: None,
        }),
        cause: None,
    }
}

// ============================================================================
// Inspection helpers
// ============================================================================

/// Returns the class (first 2 characters) of a GQLSTATUS code.
///
/// Codes come from the other side of the wire, so a malformed code is
/// tolerated: if its first two bytes are not two whole characters, the
/// code is returned unchanged (and matches no known class).
#[must_use]
pub fn class(code: &str) -> &str {
    code.get(..2).unwrap_or(code)
}

/// Returns true if the code represents a successful completion (class 00).
#[must_use]
pub fn is_success(code: &str) -> bool {
    class(code) == "00"
}

/// Returns true if the code represents a warning (class 01).
#[must_use]
pub fn is_warning(code: &str) -> bool {
    class(code) == "01"
}

/// Returns true if the code represents no data (class 02).
#[must_use]
pub fn is_no_data(code: &str) -> bool {
    class(code) == "02"
}

/// Returns true if the code represents an informational status (class 03).
#[must_use]
pub fn is_informational(code: &str) -> bool {
    class(code) == "03"
}

/// Returns true if the code represents an exception (class >= 08).
#[must_use]
pub fn is_exception(code: &str) -> bool {
    let c = class(code);
    // Exception classes: 08 and above (numeric), or letter-starting (G2, etc.)
    if c.len() < 2 {
        return false;
    }
    let first = c.as_bytes()[0];
    // Letter-starting classes are always exceptions
    if first.is_ascii_alphabetic() {
        return true;
    }
    // Numeric classes >= 08 are exceptions (00=success, 01=warn, 02=nodata, 03=info)
    c >= "08"
}

// ============================================================================
// Operation code constants (ISO/IEC 39075 Table 9)
// ============================================================================

/// Unrecognized statement or command.
pub const OP_UNRECOGNIZED: i32 = 0;
/// SESSION SET SCHEMA COMMAND.
pub const OP_SESSION_SET_SCHEMA: i32 = 1;
/// SESSION SET PROPERTY GRAPH COMMAND.
pub const OP_SESSION_SET_GRAPH: i32 = 2;
/// SESSION SET TIME ZONE COMMAND.
pub const OP_SESSION_SET_TIME_ZONE: i32 = 3;
/// SESSION SET PROPERTY GRAPH PARAMETER COMMAND.
pub const OP_SESSION_SET_GRAPH_PARAMETER: i32 = 4;
/// SESSION SET BINDING TABLE PARAMETER COMMAND.
pub const OP_SESSION_SET_BINDING_TABLE_PARAMETER: i32 = 5;
/// SESSION SET VALUE PARAMETER COMMAND.
pub const OP_SESSION_SET_VALUE_PARAMETER: i32 = 6;
/// SESSION RESET COMMAND.
pub const OP_SESSION_RESET: i32 = 7;
/// SESSION CLOSE COMMAND.
pub const OP_SESSION_CLOSE: i32 = 8;
/// START TRANSACTION COMMAND.
pub const OP_START_TRANSACTION: i32 = 50;
/// ROLLBACK COMMAND.
pub const OP_ROLLBACK: i32 = 51;
/// COMMIT COMMAND.
pub const OP_COMMIT: i32 = 52;
/// CREATE SCHEMA STATEMENT.
pub const OP_CREATE_SCHEMA: i32 = 100;
/// DROP SCHEMA STATEMENT.
pub const OP_DROP_SCHEMA: i32 = 101;
/// CREATE GRAPH STATEMENT.
pub const OP_CREATE_GRAPH: i32 = 200;
/// DROP GRAPH STATEMENT.
pub const OP_DROP_GRAPH: i32 = 201;
/// CREATE GRAPH TYPE STATEMENT.
pub const OP_CREATE_GRAPH_TYPE: i32 = 300;
/// DROP GRAPH TYPE STATEMENT.
pub const OP_DROP_GRAPH_TYPE: i32 = 301;
/// INSERT STATEMENT.
pub const OP_INSERT_STATEMENT: i32 = 500;
/// SET STATEMENT.
pub const OP_SET_STATEMENT: i32 = 501;
/// REMOVE STATEMENT.
pub const OP_REMOVE_STATEMENT: i32 = 502;
/// DELETE STATEMENT.
pub const OP_DELETE_STATEMENT: i32 = 503;
/// MATCH STATEMENT.
pub const OP_MATCH_STATEMENT: i32 = 600;
/// FILTER STATEMENT.
pub const OP_FILTER_STATEMENT: i32 = 601;
/// LET STATEMENT.
pub const OP_LET_STATEMENT: i32 = 602;
/// FOR STATEMENT.
pub const OP_FOR_STATEMENT: i32 = 603;
/// ORDER BY AND PAGE STATEMENT.
pub const OP_ORDER_BY: i32 = 604;
/// RETURN STATEMENT.
pub const OP_RETURN_STATEMENT: i32 = 605;
/// SELECT STATEMENT.
pub const OP_SELECT_STATEMENT: i32 = 606;
/// CALL PROCEDURE STATEMENT.
pub const OP_CALL_PROCEDURE: i32 = 800;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_status() {
        let s = success();
        assert_eq!(s.code, "00000");
        assert!(is_success(&s.code));
        assert!(!is_exception(&s.code));
    }

    #[test]
    fn omitted_status() {
        let s = omitted();
        assert_eq!(s.code, "00001");
        assert!(is_success(&s.code));
    }

    #[test]
    fn no_data_status() {
        let s = no_data();
        assert_eq!(s.code, "02000");
        assert!(is_no_data(&s.code));
        assert!(!is_success(&s.code));
        assert!(!is_exception(&s.code));
    }

    #[test]
    fn error_status() {
        let s = error(INVALID_SYNTAX, "unexpected token");
        assert_eq!(s.code, "42001");
        assert!(is_exception(&s.code));
        assert!(!is_success(&s.code));
    }

    #[test]
    fn error_with_diagnostic_status() {
        let s = error_with_diagnostic(
            NUMERIC_OUT_OF_RANGE,
            "value 999 exceeds INT8 range",
            "MATCH STATEMENT",
            600,
        );
        assert_eq!(s.code, "22003");
        assert!(is_exception(&s.code));
        let d = s.diagnostic.unwrap();
        assert_eq!(d.operation, "MATCH STATEMENT");
        assert_eq!(d.operation_code, 600);
    }

    #[test]
    fn warning_classification() {
        assert!(is_warning(WARNING));
        assert!(!is_exception(WARNING));
        assert!(!is_success(WARNING));
    }

    #[test]
    fn graph_type_violation_is_exception() {
        assert!(is_exception(GRAPH_TYPE_VIOLATION));
    }

    #[test]
    fn class_extraction() {
        assert_eq!(class("00000"), "00");
        assert_eq!(class("42001"), "42");
        assert_eq!(class("G2000"), "G2");
    }

    #[test]
    fn malformed_codes_do_not_panic() {
        // Regression: slicing at byte 2 panicked inside a multi-byte character.
        for code in [
            "\u{20ac}",
            "\u{20ac}0000",
            "0\u{e9}000",
            "\u{1f600}",
            "",
            "0",
        ] {
            let _ = class(code);
            assert!(!is_success(code), "{code:?}");
            let _ = is_warning(code);
            let _ = is_no_data(code);
            let _ = is_informational(code);
            let _ = is_exception(code);
        }
        assert_eq!(class("\u{20ac}0000"), "\u{20ac}0000");
        assert_eq!(class(""), "");
        // A code with an unknown, non-ASCII class counts as an exception.
        assert!(is_exception("\u{e9}0000"));
        assert!(is_exception("\u{20ac}0000"));
    }

    #[test]
    fn warning_constructor() {
        let s = warning(WARNING_GRAPH_NOT_FOUND, "graph 'test' does not exist");
        assert_eq!(s.code, "01G03");
        assert!(is_warning(&s.code));
        assert!(!is_exception(&s.code));
    }

    #[test]
    fn informational_constructor() {
        let s = informational(INFORMATIONAL, "operation completed with information");
        assert_eq!(s.code, "03000");
        assert!(is_informational(&s.code));
        assert!(!is_exception(&s.code));
    }

    #[test]
    fn connection_exception_is_exception() {
        assert!(is_exception(CONNECTION_EXCEPTION));
        assert!(is_exception(TRANSACTION_RESOLUTION_UNKNOWN));
    }

    #[test]
    fn dependent_objects_is_exception() {
        assert!(is_exception(DEPENDENT_OBJECTS_EXIST));
        assert!(is_exception(GRAPH_DEPENDS_ON_SCHEMA));
    }

    #[test]
    fn codes_follow_iso_table_8() {
        // Regression: these used codes ISO gives other conditions.
        assert_eq!(NEGATIVE_LIMIT, "22G02");
        assert_eq!(LIST_DATA_RIGHT_TRUNCATION, "22G0B");
        assert_eq!(CATALOG_AND_DATA_STATEMENT_MIXING, "25G02");
        assert_eq!(MULTIPLE_GRAPHS_NOT_SUPPORTED, "25G04");
        assert_eq!(VISUALLY_CONFUSABLE_IDENTIFIERS, "42004");
        assert_eq!(EDGES_STILL_EXIST, "G1001");
    }

    #[test]
    fn non_iso_conditions_use_implementation_defined_subclasses() {
        // ISO 23.1: an implementation-defined subclass starts with 5 to 9
        // or I to Z.
        for code in [
            INVALID_ELEMENT_ID,
            DUPLICATE_NODE_IN_PATH,
            DUPLICATE_EDGE_IN_PATH,
            INCOMPATIBLE_LIST_ELEMENT_TYPES,
            INVALID_PROPERTY_REFERENCE,
            PROPERTY_NOT_FOUND,
            INVALID_LABEL_VALUE,
            INVALID_ELEMENT_TYPE,
            INCOMPATIBLE_RECORD_FIELD_TYPES,
            NO_ACTIVE_TRANSACTION,
            TRANSACTION_FAILED_STATE,
            DUPLICATE_DEFINITION,
            AMBIGUOUS_REFERENCE,
            UNSUPPORTED_FEATURE,
            DUPLICATE_LABEL,
            INVALID_ARGUMENT_COUNT,
            INCOMPATIBLE_TYPES,
            INVALID_PATTERN,
            INVALID_AGGREGATION_OPERAND,
            INVALID_ORDERING,
            MISSING_MANDATORY_PROPERTY,
            INVALID_GRAPH_MODIFICATION,
            PROCEDURE_NOT_FOUND,
            GRAPH_DEPENDS_ON_SCHEMA,
            GRAPH_TYPE_DEPENDS_ON_SCHEMA,
            GRAPH_DEPENDS_ON_GRAPH_TYPE,
        ] {
            assert_eq!(code.len(), 5, "{code}");
            assert_eq!(&code[2..3], "I", "{code}");
        }
    }

    #[test]
    fn operation_codes_follow_iso_table_9() {
        assert_eq!(OP_SESSION_RESET, 7);
        assert_eq!(OP_SESSION_CLOSE, 8);
        assert_eq!(OP_START_TRANSACTION, 50);
        assert_eq!(OP_ROLLBACK, 51);
        assert_eq!(OP_COMMIT, 52);
        assert_eq!(OP_CREATE_GRAPH, 200);
        assert_eq!(OP_DROP_GRAPH_TYPE, 301);
        assert_eq!(OP_RETURN_STATEMENT, 605);
        assert_eq!(OP_SELECT_STATEMENT, 606);
    }
}
