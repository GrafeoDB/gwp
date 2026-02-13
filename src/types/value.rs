//! The core GQL value type - a discriminated union of all GQL value types.

use crate::proto;

use super::{Date, Duration, Edge, LocalDateTime, LocalTime, Node, Path, Record, ZonedDateTime, ZonedTime};

/// A GQL value - the discriminated union of all types that can appear
/// in query results, parameters, or property maps.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// GQL NULL.
    Null,
    /// Boolean value.
    Boolean(bool),
    /// Signed integer (INT8 through INT64).
    Integer(i64),
    /// Unsigned integer (UINT8 through UINT64).
    UnsignedInteger(u64),
    /// Floating point (FLOAT32/64).
    Float(f64),
    /// String value.
    String(String),
    /// Byte string.
    Bytes(Vec<u8>),
    /// Calendar date.
    Date(Date),
    /// Time without timezone.
    LocalTime(LocalTime),
    /// Time with timezone offset.
    ZonedTime(ZonedTime),
    /// Datetime without timezone.
    LocalDateTime(LocalDateTime),
    /// Datetime with timezone offset.
    ZonedDateTime(ZonedDateTime),
    /// Temporal duration.
    Duration(Duration),
    /// Ordered list of values.
    List(Vec<Value>),
    /// Named record with fields.
    Record(Record),
    /// Property graph node.
    Node(Node),
    /// Property graph edge.
    Edge(Edge),
    /// Path through a graph.
    Path(Path),
}

// ============================================================================
// Convenience From implementations for common Rust types
// ============================================================================

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Integer(i64::from(v))
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Self::UnsignedInteger(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::UnsignedInteger(u64::from(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_owned())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Self::List(v)
    }
}

impl From<Node> for Value {
    fn from(v: Node) -> Self {
        Self::Node(v)
    }
}

impl From<Edge> for Value {
    fn from(v: Edge) -> Self {
        Self::Edge(v)
    }
}

impl From<Path> for Value {
    fn from(v: Path) -> Self {
        Self::Path(v)
    }
}

// ============================================================================
// Proto conversions
// ============================================================================

impl From<proto::Value> for Value {
    fn from(pv: proto::Value) -> Self {
        match pv.kind {
            None | Some(proto::value::Kind::NullValue(_)) => Self::Null,
            Some(proto::value::Kind::BooleanValue(v)) => Self::Boolean(v),
            Some(proto::value::Kind::IntegerValue(v)) => Self::Integer(v),
            Some(proto::value::Kind::UnsignedIntegerValue(v)) => Self::UnsignedInteger(v),
            Some(proto::value::Kind::FloatValue(v)) => Self::Float(v),
            Some(proto::value::Kind::StringValue(v)) => Self::String(v),
            Some(proto::value::Kind::BytesValue(v)) => Self::Bytes(v),
            Some(proto::value::Kind::DateValue(v)) => Self::Date(v.into()),
            Some(proto::value::Kind::LocalTimeValue(v)) => Self::LocalTime(v.into()),
            Some(proto::value::Kind::ZonedTimeValue(v)) => Self::ZonedTime(v.into()),
            Some(proto::value::Kind::LocalDatetimeValue(v)) => Self::LocalDateTime(v.into()),
            Some(proto::value::Kind::ZonedDatetimeValue(v)) => Self::ZonedDateTime(v.into()),
            Some(proto::value::Kind::DurationValue(v)) => Self::Duration(v.into()),
            Some(proto::value::Kind::ListValue(v)) => {
                Self::List(v.elements.into_iter().map(Value::from).collect())
            }
            Some(proto::value::Kind::RecordValue(v)) => Self::Record(v.into()),
            Some(proto::value::Kind::NodeValue(v)) => Self::Node(v.into()),
            Some(proto::value::Kind::EdgeValue(v)) => Self::Edge(v.into()),
            Some(proto::value::Kind::PathValue(v)) => Self::Path(v.into()),
            // Extended numeric types - store as-is for now (future: native Rust types)
            Some(proto::value::Kind::BigIntegerValue(_))
            | Some(proto::value::Kind::BigFloatValue(_))
            | Some(proto::value::Kind::DecimalValue(_)) => {
                // TODO: Add native Rust representations for extended numerics
                Self::Null
            }
        }
    }
}

impl From<Value> for proto::Value {
    fn from(v: Value) -> Self {
        let kind = match v {
            Value::Null => Some(proto::value::Kind::NullValue(proto::NullValue {})),
            Value::Boolean(b) => Some(proto::value::Kind::BooleanValue(b)),
            Value::Integer(i) => Some(proto::value::Kind::IntegerValue(i)),
            Value::UnsignedInteger(u) => Some(proto::value::Kind::UnsignedIntegerValue(u)),
            Value::Float(f) => Some(proto::value::Kind::FloatValue(f)),
            Value::String(s) => Some(proto::value::Kind::StringValue(s)),
            Value::Bytes(b) => Some(proto::value::Kind::BytesValue(b)),
            Value::Date(d) => Some(proto::value::Kind::DateValue(d.into())),
            Value::LocalTime(t) => Some(proto::value::Kind::LocalTimeValue(t.into())),
            Value::ZonedTime(t) => Some(proto::value::Kind::ZonedTimeValue(t.into())),
            Value::LocalDateTime(dt) => Some(proto::value::Kind::LocalDatetimeValue(dt.into())),
            Value::ZonedDateTime(dt) => Some(proto::value::Kind::ZonedDatetimeValue(dt.into())),
            Value::Duration(d) => Some(proto::value::Kind::DurationValue(d.into())),
            Value::List(elems) => Some(proto::value::Kind::ListValue(proto::GqlList {
                elements: elems.into_iter().map(proto::Value::from).collect(),
            })),
            Value::Record(r) => Some(proto::value::Kind::RecordValue(r.into())),
            Value::Node(n) => Some(proto::value::Kind::NodeValue(n.into())),
            Value::Edge(e) => Some(proto::value::Kind::EdgeValue(e.into())),
            Value::Path(p) => Some(proto::value::Kind::PathValue(p.into())),
        };
        proto::Value { kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(value: Value) {
        let proto_value: proto::Value = value.clone().into();
        let back: Value = proto_value.into();
        assert_eq!(value, back);
    }

    #[test]
    fn round_trip_null() {
        round_trip(Value::Null);
    }

    #[test]
    fn round_trip_boolean() {
        round_trip(Value::Boolean(true));
        round_trip(Value::Boolean(false));
    }

    #[test]
    fn round_trip_integer() {
        round_trip(Value::Integer(0));
        round_trip(Value::Integer(42));
        round_trip(Value::Integer(-1));
        round_trip(Value::Integer(i64::MAX));
        round_trip(Value::Integer(i64::MIN));
    }

    #[test]
    fn round_trip_unsigned() {
        round_trip(Value::UnsignedInteger(0));
        round_trip(Value::UnsignedInteger(u64::MAX));
    }

    #[test]
    fn round_trip_float() {
        round_trip(Value::Float(0.0));
        round_trip(Value::Float(3.14));
        round_trip(Value::Float(-1.0));
    }

    #[test]
    fn round_trip_string() {
        round_trip(Value::String(String::new()));
        round_trip(Value::String("hello world".to_owned()));
    }

    #[test]
    fn round_trip_bytes() {
        round_trip(Value::Bytes(vec![]));
        round_trip(Value::Bytes(vec![0x00, 0xFF, 0x42]));
    }

    #[test]
    fn round_trip_list() {
        round_trip(Value::List(vec![]));
        round_trip(Value::List(vec![
            Value::Integer(1),
            Value::String("two".to_owned()),
            Value::Null,
        ]));
    }

    #[test]
    fn round_trip_nested_list() {
        round_trip(Value::List(vec![
            Value::List(vec![Value::Integer(1), Value::Integer(2)]),
            Value::List(vec![Value::Integer(3)]),
        ]));
    }

    #[test]
    fn from_conversions() {
        assert_eq!(Value::from(true), Value::Boolean(true));
        assert_eq!(Value::from(42_i64), Value::Integer(42));
        assert_eq!(Value::from(42_i32), Value::Integer(42));
        assert_eq!(Value::from(42_u64), Value::UnsignedInteger(42));
        assert_eq!(Value::from(42_u32), Value::UnsignedInteger(42));
        assert_eq!(Value::from(3.14_f64), Value::Float(3.14));
        assert_eq!(Value::from("hello"), Value::String("hello".to_owned()));
    }
}
