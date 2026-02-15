//! The core GQL value type - a discriminated union of all GQL value types.

use std::fmt;

use crate::proto;

use super::{
    Date, Duration, Edge, LocalDateTime, LocalTime, Node, Path, Record, ZonedDateTime, ZonedTime,
};

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
    /// Arbitrary-precision decimal (unscaled big-endian two's complement + scale).
    Decimal {
        /// Big-endian two's complement of the unscaled value.
        unscaled: Vec<u8>,
        /// Number of digits after the decimal point.
        scale: i32,
    },
    /// Extended-precision integer (INT128/256, UINT128/256).
    BigInteger {
        /// Big-endian two's complement encoding.
        value: Vec<u8>,
        /// Whether this is a signed integer type.
        is_signed: bool,
    },
    /// Extended-precision float (FLOAT128/256).
    BigFloat {
        /// IEEE 754 encoding.
        value: Vec<u8>,
        /// Bit width (128 or 256).
        width: u32,
    },
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
            Some(proto::value::Kind::DecimalValue(v)) => Self::Decimal {
                unscaled: v.unscaled,
                scale: v.scale,
            },
            Some(proto::value::Kind::BigIntegerValue(v)) => Self::BigInteger {
                value: v.value,
                is_signed: v.is_signed,
            },
            Some(proto::value::Kind::BigFloatValue(v)) => Self::BigFloat {
                value: v.value,
                width: v.width,
            },
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
            Value::Decimal { unscaled, scale } => {
                Some(proto::value::Kind::DecimalValue(proto::Decimal {
                    unscaled,
                    scale,
                }))
            }
            Value::BigInteger { value, is_signed } => {
                Some(proto::value::Kind::BigIntegerValue(proto::BigInteger {
                    value,
                    is_signed,
                }))
            }
            Value::BigFloat { value, width } => {
                Some(proto::value::Kind::BigFloatValue(proto::BigFloat {
                    value,
                    width,
                }))
            }
        };
        proto::Value { kind }
    }
}

// ============================================================================
// Display
// ============================================================================

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::UnsignedInteger(u) => write!(f, "{u}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Bytes(b) => write!(f, "0x{}", hex_encode(b)),
            Self::Date(d) => write!(f, "{:04}-{:02}-{:02}", d.year, d.month, d.day),
            Self::LocalTime(t) => write_time(f, t, None),
            Self::ZonedTime(t) => write_time(f, &t.time, Some(t.offset_minutes)),
            Self::LocalDateTime(dt) => write_datetime(f, &dt.date, &dt.time, None),
            Self::ZonedDateTime(dt) => {
                write_datetime(f, &dt.date, &dt.time, Some(dt.offset_minutes))
            }
            Self::Duration(d) => write_duration(f, d),
            Self::List(elems) => write_list(f, elems),
            Self::Record(r) => write_record(f, r),
            Self::Node(n) => write_node(f, n),
            Self::Edge(e) => write_edge(f, e),
            Self::Path(p) => write_path(f, p),
            Self::Decimal { unscaled, scale } => {
                write!(f, "Decimal(0x{}, scale={scale})", hex_encode(unscaled))
            }
            Self::BigInteger { value, is_signed } => {
                let sign = if *is_signed { "signed" } else { "unsigned" };
                write!(f, "BigInteger(0x{}, {sign})", hex_encode(value))
            }
            Self::BigFloat { value, width } => {
                write!(f, "BigFloat(0x{}, {width}bit)", hex_encode(value))
            }
        }
    }
}

fn write_time(
    f: &mut fmt::Formatter<'_>,
    t: &super::LocalTime,
    offset: Option<i32>,
) -> fmt::Result {
    write!(f, "{:02}:{:02}:{:02}", t.hour, t.minute, t.second)?;
    if t.nanosecond > 0 {
        write!(f, ".{:09}", t.nanosecond)?;
    }
    if let Some(off) = offset {
        write_offset(f, off)?;
    }
    Ok(())
}

fn write_datetime(
    f: &mut fmt::Formatter<'_>,
    d: &super::Date,
    t: &super::LocalTime,
    offset: Option<i32>,
) -> fmt::Result {
    write!(f, "{:04}-{:02}-{:02}T", d.year, d.month, d.day)?;
    write_time(f, t, offset)
}

fn write_duration(f: &mut fmt::Formatter<'_>, d: &super::Duration) -> fmt::Result {
    write!(f, "P")?;
    if d.months != 0 {
        write!(f, "{}M", d.months)?;
    }
    if d.nanoseconds != 0 || d.months == 0 {
        let secs = d.nanoseconds / 1_000_000_000;
        let nanos = d.nanoseconds % 1_000_000_000;
        if nanos == 0 {
            write!(f, "T{secs}S")?;
        } else {
            write!(f, "T{secs}.{:09}S", nanos.unsigned_abs())?;
        }
    }
    Ok(())
}

fn write_list(f: &mut fmt::Formatter<'_>, elems: &[Value]) -> fmt::Result {
    write!(f, "[")?;
    for (i, e) in elems.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{e}")?;
    }
    write!(f, "]")
}

fn write_record(f: &mut fmt::Formatter<'_>, r: &super::Record) -> fmt::Result {
    write!(f, "{{")?;
    for (i, field) in r.fields.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}: {}", field.name, field.value)?;
    }
    write!(f, "}}")
}

fn write_props(
    f: &mut fmt::Formatter<'_>,
    props: &std::collections::HashMap<std::string::String, Value>,
) -> fmt::Result {
    if !props.is_empty() {
        write!(f, " {{")?;
        for (i, (k, v)) in props.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{k}: {v}")?;
        }
        write!(f, "}}")?;
    }
    Ok(())
}

fn write_labels(f: &mut fmt::Formatter<'_>, labels: &[std::string::String]) -> fmt::Result {
    for (i, label) in labels.iter().enumerate() {
        if i > 0 {
            write!(f, ":")?;
        }
        write!(f, "{label}")?;
    }
    Ok(())
}

fn write_node(f: &mut fmt::Formatter<'_>, n: &super::Node) -> fmt::Result {
    write!(f, "(:")?;
    write_labels(f, &n.labels)?;
    write_props(f, &n.properties)?;
    write!(f, ")")
}

fn write_edge(f: &mut fmt::Formatter<'_>, e: &super::Edge) -> fmt::Result {
    let arrow = if e.undirected { "-" } else { "->" };
    write!(f, "[:")?;
    write_labels(f, &e.labels)?;
    write_props(f, &e.properties)?;
    write!(f, "]{arrow}")
}

fn write_path(f: &mut fmt::Formatter<'_>, p: &super::Path) -> fmt::Result {
    for (i, node) in p.nodes.iter().enumerate() {
        if i > 0 {
            if let Some(edge) = p.edges.get(i - 1) {
                write!(f, "-")?;
                write_edge(f, edge)?;
                write!(f, "-")?;
            }
        }
        write_node(f, node)?;
    }
    Ok(())
}

/// Format a UTC offset in +HH:MM or -HH:MM.
fn write_offset(f: &mut fmt::Formatter<'_>, offset_minutes: i32) -> fmt::Result {
    let sign = if offset_minutes >= 0 { '+' } else { '-' };
    let abs = offset_minutes.unsigned_abs();
    write!(f, "{sign}{:02}:{:02}", abs / 60, abs % 60)
}

/// Hex-encode a byte slice (lowercase, no prefix).
fn hex_encode(bytes: &[u8]) -> std::string::String {
    use std::fmt::Write;
    bytes.iter().fold(
        std::string::String::with_capacity(bytes.len() * 2),
        |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(value: &Value) {
        let proto_value: proto::Value = value.clone().into();
        let back: Value = proto_value.into();
        assert_eq!(*value, back);
    }

    #[test]
    fn round_trip_null() {
        round_trip(&Value::Null);
    }

    #[test]
    fn round_trip_boolean() {
        round_trip(&Value::Boolean(true));
        round_trip(&Value::Boolean(false));
    }

    #[test]
    fn round_trip_integer() {
        round_trip(&Value::Integer(0));
        round_trip(&Value::Integer(42));
        round_trip(&Value::Integer(-1));
        round_trip(&Value::Integer(i64::MAX));
        round_trip(&Value::Integer(i64::MIN));
    }

    #[test]
    fn round_trip_unsigned() {
        round_trip(&Value::UnsignedInteger(0));
        round_trip(&Value::UnsignedInteger(u64::MAX));
    }

    #[test]
    fn round_trip_float() {
        round_trip(&Value::Float(0.0));
        round_trip(&Value::Float(1.5));
        round_trip(&Value::Float(-1.0));
    }

    #[test]
    fn round_trip_string() {
        round_trip(&Value::String(String::new()));
        round_trip(&Value::String("hello world".to_owned()));
    }

    #[test]
    fn round_trip_bytes() {
        round_trip(&Value::Bytes(vec![]));
        round_trip(&Value::Bytes(vec![0x00, 0xFF, 0x42]));
    }

    #[test]
    fn round_trip_list() {
        round_trip(&Value::List(vec![]));
        round_trip(&Value::List(vec![
            Value::Integer(1),
            Value::String("two".to_owned()),
            Value::Null,
        ]));
    }

    #[test]
    fn round_trip_nested_list() {
        round_trip(&Value::List(vec![
            Value::List(vec![Value::Integer(1), Value::Integer(2)]),
            Value::List(vec![Value::Integer(3)]),
        ]));
    }

    #[test]
    fn round_trip_decimal() {
        // Represents 12.50 (unscaled = 1250, scale = 2)
        round_trip(&Value::Decimal {
            unscaled: vec![0x04, 0xE2], // 1250 big-endian
            scale: 2,
        });
        // Zero
        round_trip(&Value::Decimal {
            unscaled: vec![0x00],
            scale: 0,
        });
    }

    #[test]
    fn round_trip_big_integer() {
        round_trip(&Value::BigInteger {
            value: vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            is_signed: true,
        });
        round_trip(&Value::BigInteger {
            value: vec![0xFF],
            is_signed: false,
        });
    }

    #[test]
    fn round_trip_big_float() {
        round_trip(&Value::BigFloat {
            value: vec![0x40, 0x09, 0x21, 0xFB],
            width: 128,
        });
        round_trip(&Value::BigFloat {
            value: vec![],
            width: 256,
        });
    }

    #[test]
    fn from_conversions() {
        assert_eq!(Value::from(true), Value::Boolean(true));
        assert_eq!(Value::from(42_i64), Value::Integer(42));
        assert_eq!(Value::from(42_i32), Value::Integer(42));
        assert_eq!(Value::from(42_u64), Value::UnsignedInteger(42));
        assert_eq!(Value::from(42_u32), Value::UnsignedInteger(42));
        assert_eq!(Value::from(1.5_f64), Value::Float(1.5));
        assert_eq!(Value::from("hello"), Value::String("hello".to_owned()));
    }

    #[test]
    fn display_primitives() {
        assert_eq!(Value::Null.to_string(), "NULL");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Integer(-42).to_string(), "-42");
        assert_eq!(Value::UnsignedInteger(99).to_string(), "99");
        assert_eq!(Value::Float(1.5).to_string(), "1.5");
        assert_eq!(Value::String("hello".to_owned()).to_string(), "hello");
        assert_eq!(Value::Bytes(vec![0xDE, 0xAD]).to_string(), "0xdead");
    }

    #[test]
    fn display_temporal() {
        use super::{Date, LocalTime};

        assert_eq!(
            Value::Date(Date {
                year: 2026,
                month: 2,
                day: 14
            })
            .to_string(),
            "2026-02-14"
        );
        assert_eq!(
            Value::LocalTime(LocalTime {
                hour: 9,
                minute: 30,
                second: 0,
                nanosecond: 0,
            })
            .to_string(),
            "09:30:00"
        );
    }

    #[test]
    fn display_list() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::String("two".to_owned()),
            Value::Null,
        ]);
        assert_eq!(list.to_string(), "[1, two, NULL]");
    }
}
