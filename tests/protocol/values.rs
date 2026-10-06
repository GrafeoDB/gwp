//! Every GQL value type round-trips client to backend to client.
//!
//! The values go out as statement parameters, the test backend echoes them
//! back as a result row, and the client decodes them again: one trip
//! through both directions of the proto conversion and the wire.

use std::collections::HashMap;

use gwp::client::GqlConnection;
use gwp::proto;
use gwp::types::{
    Date, Duration, Edge, LocalDateTime, LocalTime, Node, Path, Record, Value, ZonedDateTime,
    ZonedTime,
};

use crate::common;

/// Send `values` through the server and return what comes back, in order.
async fn echo(values: &[Value]) -> Vec<Value> {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    // Zero-padded names keep the echoed columns in input order.
    let parameters: HashMap<String, Value> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (format!("p{i:04}"), v.clone()))
        .collect();

    let mut cursor = session.execute("ECHO", parameters).await.unwrap();
    let mut rows = cursor.collect_rows().await.unwrap();
    assert!(cursor.is_success().await.unwrap());
    assert_eq!(rows.len(), 1);
    rows.pop().unwrap()
}

/// Round-trip `values` and assert each one comes back unchanged.
async fn assert_round_trip(values: Vec<Value>) {
    let back = echo(&values).await;
    assert_eq!(back.len(), values.len());
    for (sent, received) in values.iter().zip(&back) {
        assert_eq!(sent, received);
    }
}

fn time(hour: u32, minute: u32, second: u32, nanosecond: u32) -> LocalTime {
    LocalTime {
        hour,
        minute,
        second,
        nanosecond,
    }
}

fn date(year: i32, month: u32, day: u32) -> Date {
    Date { year, month, day }
}

#[tokio::test]
async fn null_and_booleans() {
    assert_round_trip(vec![
        Value::Null,
        Value::Boolean(true),
        Value::Boolean(false),
    ])
    .await;
}

#[tokio::test]
async fn signed_integer_boundaries() {
    assert_round_trip(
        [
            0,
            1,
            -1,
            i64::from(i8::MIN),
            i64::from(i8::MAX),
            i64::from(i16::MIN),
            i64::from(i16::MAX),
            i64::from(i32::MIN),
            i64::from(i32::MAX),
            i64::from(u32::MAX),
            i64::MIN,
            i64::MIN + 1,
            i64::MAX,
        ]
        .into_iter()
        .map(Value::Integer)
        .collect(),
    )
    .await;
}

#[tokio::test]
async fn unsigned_integer_boundaries() {
    assert_round_trip(
        [
            0,
            1,
            u64::from(u32::MAX),
            u64::from(u32::MAX) + 1,
            i64::MAX.unsigned_abs(),
            i64::MAX.unsigned_abs() + 1,
            u64::MAX,
        ]
        .into_iter()
        .map(Value::UnsignedInteger)
        .collect(),
    )
    .await;
}

#[tokio::test]
async fn float_boundaries_and_special_values() {
    let floats = [
        0.0,
        -0.0,
        1.5,
        -1.5,
        f64::MIN,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::from_bits(1), // smallest subnormal
        f64::EPSILON,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::from(0.1_f32),
    ];
    let back = echo(&floats.map(Value::Float)).await;
    for (sent, received) in floats.iter().zip(&back) {
        let received = received.as_float().expect("float comes back as a float");
        // Bit equality: keeps the sign of -0.0 and compares NaN to NaN.
        assert_eq!(
            sent.to_bits(),
            received.to_bits(),
            "{sent} became {received}"
        );
    }
    assert_eq!(Value::from(0.1_f32), Value::Float(f64::from(0.1_f32)));
}

#[tokio::test]
async fn strings_and_bytes() {
    assert_round_trip(vec![
        Value::String(String::new()),
        Value::String("plain ascii".to_owned()),
        Value::String("\u{1f600} emoji, \u{4e2d}\u{6587}, \u{5d0}\u{5d1}\u{5d2}".to_owned()),
        Value::String("embedded\0nul and\nnewline\ttab".to_owned()),
        Value::String("x".repeat(512 * 1024)),
        Value::Bytes(Vec::new()),
        Value::Bytes((0..=255).collect()),
        Value::Bytes(vec![0xAB; 512 * 1024]),
    ])
    .await;
}

#[tokio::test]
async fn extended_numerics_are_opaque_and_exact() {
    assert_round_trip(vec![
        // 12.50
        Value::Decimal {
            unscaled: vec![0x04, 0xE2],
            scale: 2,
        },
        // -1 (two's complement), negative and extreme scales
        Value::Decimal {
            unscaled: vec![0xFF],
            scale: -3,
        },
        Value::Decimal {
            unscaled: vec![0x7F; 32],
            scale: i32::MAX,
        },
        Value::Decimal {
            unscaled: Vec::new(),
            scale: i32::MIN,
        },
        Value::BigInteger {
            value: vec![0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            is_signed: true,
        },
        Value::BigInteger {
            value: vec![0xFF; 32],
            is_signed: false,
        },
        Value::BigFloat {
            value: vec![0x3F, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            width: 128,
        },
        Value::BigFloat {
            value: vec![0; 32],
            width: 256,
        },
    ])
    .await;
}

#[tokio::test]
async fn dates_and_times() {
    assert_round_trip(vec![
        Value::Date(date(2026, 10, 5)),
        Value::Date(date(0, 1, 1)),
        Value::Date(date(-44, 3, 15)),
        Value::Date(date(i32::MIN, 1, 1)),
        Value::Date(date(i32::MAX, 12, 31)),
        Value::LocalTime(time(0, 0, 0, 0)),
        Value::LocalTime(time(23, 59, 59, 999_999_999)),
        Value::LocalDateTime(LocalDateTime {
            date: date(2024, 2, 29),
            time: time(12, 30, 0, 1),
        }),
    ])
    .await;
}

#[tokio::test]
async fn temporal_values_keep_their_offsets() {
    let mut values = Vec::new();
    // UTC, whole hours, half and quarter hours, and the +-18:00 extremes.
    for offset in [0, 60, -60, 330, 345, -570, 1080, -1080] {
        values.push(Value::ZonedTime(ZonedTime {
            time: time(9, 15, 30, 500),
            offset_minutes: offset,
        }));
        values.push(Value::ZonedDateTime(ZonedDateTime {
            date: date(2026, 3, 29),
            time: time(2, 30, 0, 0),
            offset_minutes: offset,
        }));
    }
    assert_round_trip(values).await;
}

#[tokio::test]
async fn durations() {
    assert_round_trip(
        [
            (0, 0),
            (14, 0),
            (0, 86_400_000_000_000),
            (-1, -1),
            (12, -1),
            (-12, 1),
            (i64::MAX, i64::MAX),
            (i64::MIN, i64::MIN),
        ]
        .into_iter()
        .map(|(months, nanoseconds)| {
            Value::Duration(Duration {
                months,
                nanoseconds,
            })
        })
        .collect(),
    )
    .await;
}

#[tokio::test]
async fn lists() {
    let mut nested = Value::Integer(7);
    for _ in 0..20 {
        nested = Value::List(vec![nested]);
    }
    assert_round_trip(vec![
        Value::List(Vec::new()),
        Value::List(vec![Value::List(Vec::new()), Value::List(Vec::new())]),
        Value::List(vec![Value::Null, Value::Null]),
        Value::List(vec![
            Value::Integer(1),
            Value::String("two".to_owned()),
            Value::Float(3.0),
            Value::Null,
            Value::List(vec![Value::Boolean(true)]),
        ]),
        Value::List((0..10_000).map(Value::Integer).collect()),
        nested,
    ])
    .await;
}

#[tokio::test]
async fn records() {
    let inner = Record::new().with_field("z", Value::Null);
    assert_round_trip(vec![
        Value::Record(Record::new()),
        Value::Record(
            Record::new()
                .with_field("name", "Alix")
                .with_field("age", 30_i64)
                .with_field("missing", Value::Null),
        ),
        // Field order is kept, and duplicate names are not merged.
        Value::Record(
            Record::new()
                .with_field("b", 1_i64)
                .with_field("a", 2_i64)
                .with_field("b", 3_i64),
        ),
        Value::Record(
            Record::new()
                .with_field("", "empty name")
                .with_field("nested", inner),
        ),
    ])
    .await;
}

#[tokio::test]
async fn nodes_edges_and_paths() {
    let alix = Node::new(vec![0x01])
        .with_label("Person")
        .with_label("Employee")
        .with_property("name", "Alix")
        .with_property(
            "tags",
            Value::List(vec![Value::from("a"), Value::from("b")]),
        )
        .with_property("score", Value::Float(-0.5))
        .with_property("", Value::Null);
    let gus = Node::new(vec![0x02]).with_label("Person");
    let knows = Edge::directed(vec![0x10], vec![0x01], vec![0x02])
        .with_label("KNOWS")
        .with_property("since", 2020_i64);
    let near = Edge::undirected(vec![0x11], vec![0x02], vec![0x01]).with_label("NEAR");

    assert_round_trip(vec![
        Value::Node(Node::new(Vec::<u8>::new())),
        Value::Node(alix.clone()),
        Value::Edge(Edge::directed(
            Vec::<u8>::new(),
            Vec::<u8>::new(),
            Vec::<u8>::new(),
        )),
        Value::Edge(knows.clone()),
        Value::Edge(near.clone()),
        Value::Path(Path::from_node(alix.clone())),
        Value::Path(
            Path::from_node(alix.clone())
                .with_step(knows, gus.clone())
                .with_step(near, alix),
        ),
        // The protocol does not validate path shape: an empty path passes.
        Value::Path(Path {
            nodes: Vec::new(),
            edges: Vec::new(),
        }),
    ])
    .await;
}

#[tokio::test]
async fn out_of_range_temporal_fields_pass_through_unchanged() {
    // The wire protocol carries what the backend sends; validating calendar
    // values is the backend's job. Nothing is clamped or rejected.
    assert_round_trip(vec![
        Value::Date(date(2026, 13, 0)),
        Value::LocalTime(time(24, 60, 61, 2_000_000_000)),
        Value::ZonedTime(ZonedTime {
            time: time(0, 0, 0, 0),
            offset_minutes: i32::MIN,
        }),
    ])
    .await;
}

#[tokio::test]
async fn unicode_parameter_names() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    let parameters = HashMap::from([
        ("\u{e9}t\u{e9}".to_owned(), Value::Integer(1)),
        ("\u{1f600}".to_owned(), Value::Integer(2)),
    ]);
    let mut cursor = session.execute("ECHO", parameters).await.unwrap();
    let mut names = cursor.column_names().await.unwrap();
    names.sort();
    assert_eq!(names, vec!["\u{e9}t\u{e9}", "\u{1f600}"]);
    let rows = cursor.collect_rows().await.unwrap();
    assert_eq!(rows.len(), 1);
}

// ============================================================================
// Lenient decoding of incomplete messages (no wire needed)
// ============================================================================

#[test]
fn missing_value_kind_decodes_as_null() {
    assert_eq!(Value::from(proto::Value { kind: None }), Value::Null);
    let explicit = proto::Value::from(Value::Null);
    assert!(matches!(
        explicit.kind,
        Some(proto::value::Kind::NullValue(_))
    ));
}

#[test]
fn missing_temporal_parts_decode_as_zero() {
    let midnight = time(0, 0, 0, 0);
    let zoned = Value::from(proto::Value {
        kind: Some(proto::value::Kind::ZonedTimeValue(proto::ZonedTime {
            time: None,
            offset_minutes: 60,
        })),
    });
    assert_eq!(
        zoned,
        Value::ZonedTime(ZonedTime {
            time: midnight,
            offset_minutes: 60
        })
    );

    let local = Value::from(proto::Value {
        kind: Some(proto::value::Kind::LocalDatetimeValue(
            proto::LocalDateTime {
                date: None,
                time: None,
            },
        )),
    });
    assert_eq!(
        local,
        Value::LocalDateTime(LocalDateTime {
            date: date(0, 0, 0),
            time: midnight,
        })
    );

    let zoned_datetime = Value::from(proto::Value {
        kind: Some(proto::value::Kind::ZonedDatetimeValue(
            proto::ZonedDateTime {
                date: None,
                time: None,
                offset_minutes: -60,
            },
        )),
    });
    assert_eq!(
        zoned_datetime,
        Value::ZonedDateTime(ZonedDateTime {
            date: date(0, 0, 0),
            time: midnight,
            offset_minutes: -60,
        })
    );
}

#[test]
fn missing_record_field_value_decodes_as_null() {
    let record = proto::Record {
        fields: vec![proto::Field {
            name: "x".to_owned(),
            value: None,
        }],
    };
    assert_eq!(
        Value::from(proto::Value {
            kind: Some(proto::value::Kind::RecordValue(record)),
        }),
        Value::Record(Record::new().with_field("x", Value::Null))
    );
}

#[test]
fn property_without_kind_decodes_as_null() {
    let node = proto::Node {
        id: vec![1],
        labels: vec!["L".to_owned()],
        properties: HashMap::from([("p".to_owned(), proto::Value { kind: None })]),
    };
    let decoded = gwp::types::Node::from(node);
    assert_eq!(decoded.property("p"), Some(&Value::Null));
}
