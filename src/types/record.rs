//! Record type - named collection of fields.

use crate::proto;
use super::Value;

/// A single field within a record.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Field name.
    pub name: String,
    /// Field value.
    pub value: Value,
}

/// A named collection of fields (GQL record type).
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// Fields in order.
    pub fields: Vec<Field>,
}

impl Record {
    /// Create an empty record.
    #[must_use]
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Add a field to the record.
    #[must_use]
    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.push(Field {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Get a field value by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| &f.value)
    }

    /// Returns the number of fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns true if the record has no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Proto conversions
// ============================================================================

impl From<proto::Record> for Record {
    fn from(p: proto::Record) -> Self {
        Self {
            fields: p
                .fields
                .into_iter()
                .map(|f| Field {
                    name: f.name,
                    value: f.value.map_or(Value::Null, Value::from),
                })
                .collect(),
        }
    }
}

impl From<Record> for proto::Record {
    fn from(r: Record) -> Self {
        Self {
            fields: r
                .fields
                .into_iter()
                .map(|f| proto::Field {
                    name: f.name,
                    value: Some(proto::Value::from(f.value)),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_pattern() {
        let rec = Record::new()
            .with_field("name", "Alice")
            .with_field("age", 30_i64);

        assert_eq!(rec.len(), 2);
        assert!(!rec.is_empty());
        assert_eq!(rec.get("name"), Some(&Value::String("Alice".to_owned())));
        assert_eq!(rec.get("age"), Some(&Value::Integer(30)));
        assert_eq!(rec.get("missing"), None);
    }

    #[test]
    fn empty_record() {
        let rec = Record::new();
        assert!(rec.is_empty());
        assert_eq!(rec.len(), 0);
    }

    #[test]
    fn round_trip() {
        let rec = Record::new()
            .with_field("x", 1_i64)
            .with_field("y", "hello");

        let proto_rec: proto::Record = rec.clone().into();
        let back: Record = proto_rec.into();
        assert_eq!(rec, back);
    }
}
