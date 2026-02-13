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

/// Null value eliminated in set function.
pub const WARNING_NULL_ELIMINATED: &str = "01G11";

// ============================================================================
// No data (class 02)
// ============================================================================

/// No data - query matched nothing.
pub const NO_DATA: &str = "02000";

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

/// Division by zero.
pub const DIVISION_BY_ZERO: &str = "22012";

/// Invalid value type.
pub const INVALID_VALUE_TYPE: &str = "22G03";

/// Values not comparable.
pub const NOT_COMPARABLE: &str = "22G04";

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

/// Read-only GQL-transaction.
pub const READ_ONLY_TRANSACTION: &str = "25G03";

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
            current_schema: String::new(),
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
