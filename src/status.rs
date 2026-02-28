//! GQLSTATUS code constants and helpers (ISO/IEC 39075 Chapter 23).
//!
//! Every GQL operation produces a GQLSTATUS code. This module provides
//! well-known code constants and helper methods for constructing and
//! inspecting status values.

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

/// Invalid datetime format.
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

/// Invalid value type.
pub const INVALID_VALUE_TYPE: &str = "22G03";

/// Values not comparable.
pub const NOT_COMPARABLE: &str = "22G04";

/// Negative limit value.
pub const NEGATIVE_LIMIT: &str = "22G05";

/// Invalid element ID.
pub const INVALID_ELEMENT_ID: &str = "22G06";

/// Duplicate node in path.
pub const DUPLICATE_NODE_IN_PATH: &str = "22G07";

/// Duplicate edge in path.
pub const DUPLICATE_EDGE_IN_PATH: &str = "22G08";

/// List data, right truncation.
pub const LIST_DATA_RIGHT_TRUNCATION: &str = "22G09";

/// Incompatible list element types.
pub const INCOMPATIBLE_LIST_ELEMENT_TYPES: &str = "22G0A";

/// Invalid property reference.
pub const INVALID_PROPERTY_REFERENCE: &str = "22G0B";

/// Property not found.
pub const PROPERTY_NOT_FOUND: &str = "22G0C";

/// Invalid label value.
pub const INVALID_LABEL_VALUE: &str = "22G0D";

/// Invalid element type.
pub const INVALID_ELEMENT_TYPE: &str = "22G0E";

/// Incompatible record field types.
pub const INCOMPATIBLE_RECORD_FIELD_TYPES: &str = "22G0F";

/// Record fields do not match.
pub const RECORD_MISMATCH: &str = "22G0U";

/// Malformed path.
pub const MALFORMED_PATH: &str = "22G0Z";

// ============================================================================
// Transaction state (class 25)
// ============================================================================

/// Invalid transaction state (no subclass).
pub const INVALID_TRANSACTION_STATE: &str = "25000";

/// Active GQL-transaction already exists.
pub const ACTIVE_TRANSACTION: &str = "25G01";

/// No active GQL-transaction.
pub const NO_ACTIVE_TRANSACTION: &str = "25G02";

/// Read-only GQL-transaction.
pub const READ_ONLY_TRANSACTION: &str = "25G03";

/// GQL-transaction in failed state.
pub const TRANSACTION_FAILED_STATE: &str = "25G04";

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

/// Duplicate definition.
pub const DUPLICATE_DEFINITION: &str = "42004";

/// Ambiguous reference.
pub const AMBIGUOUS_REFERENCE: &str = "42005";

/// Unsupported feature.
pub const UNSUPPORTED_FEATURE: &str = "42006";

/// Duplicate label.
pub const DUPLICATE_LABEL: &str = "42007";

/// Invalid number of arguments.
pub const INVALID_ARGUMENT_COUNT: &str = "42008";

/// Incompatible types.
pub const INCOMPATIBLE_TYPES: &str = "42009";

/// Invalid pattern.
pub const INVALID_PATTERN: &str = "42010";

/// Invalid operand for aggregation.
pub const INVALID_AGGREGATION_OPERAND: &str = "42011";

/// Invalid ordering specification.
pub const INVALID_ORDERING: &str = "42012";

/// Missing mandatory property.
pub const MISSING_MANDATORY_PROPERTY: &str = "42013";

/// Invalid graph modification.
pub const INVALID_GRAPH_MODIFICATION: &str = "42014";

/// Procedure not found.
pub const PROCEDURE_NOT_FOUND: &str = "42015";

// ============================================================================
// Dependent object errors (class G1)
// ============================================================================

/// Dependent objects still exist (no subclass).
pub const DEPENDENT_OBJECTS_EXIST: &str = "G1000";

/// Graph depends on schema.
pub const GRAPH_DEPENDS_ON_SCHEMA: &str = "G1001";

/// Graph type depends on schema.
pub const GRAPH_TYPE_DEPENDS_ON_SCHEMA: &str = "G1002";

/// Graph depends on graph type.
pub const GRAPH_DEPENDS_ON_GRAPH_TYPE: &str = "G1003";

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
#[must_use]
pub fn class(code: &str) -> &str {
    if code.len() >= 2 { &code[..2] } else { code }
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
}
