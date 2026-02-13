//! Property graph node type.

use std::collections::HashMap;

use super::Value;
use crate::proto;

/// A property graph node with an opaque ID, labels, and properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Opaque element identifier.
    pub id: Vec<u8>,
    /// Label set (unordered).
    pub labels: Vec<String>,
    /// Property map.
    pub properties: HashMap<String, Value>,
}

impl Node {
    /// Create a new node with the given ID.
    #[must_use]
    pub fn new(id: impl Into<Vec<u8>>) -> Self {
        Self {
            id: id.into(),
            labels: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a label to the node.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Add a property to the node.
    #[must_use]
    pub fn with_property(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.properties.insert(name.into(), value.into());
        self
    }

    /// Get a property value by name.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&Value> {
        self.properties.get(name)
    }

    /// Check if the node has a specific label.
    #[must_use]
    pub fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|l| l == label)
    }
}

// ============================================================================
// Proto conversions
// ============================================================================

impl From<proto::Node> for Node {
    fn from(p: proto::Node) -> Self {
        Self {
            id: p.id,
            labels: p.labels,
            properties: p
                .properties
                .into_iter()
                .map(|(k, v)| (k, Value::from(v)))
                .collect(),
        }
    }
}

impl From<Node> for proto::Node {
    fn from(n: Node) -> Self {
        Self {
            id: n.id,
            labels: n.labels,
            properties: n
                .properties
                .into_iter()
                .map(|(k, v)| (k, proto::Value::from(v)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_pattern() {
        let node = Node::new(1_i64.to_be_bytes())
            .with_label("Person")
            .with_label("Employee")
            .with_property("name", "Alice")
            .with_property("age", 30_i64);

        assert!(node.has_label("Person"));
        assert!(node.has_label("Employee"));
        assert!(!node.has_label("Company"));
        assert_eq!(
            node.property("name"),
            Some(&Value::String("Alice".to_owned()))
        );
        assert_eq!(node.property("age"), Some(&Value::Integer(30)));
        assert_eq!(node.property("missing"), None);
    }

    #[test]
    fn round_trip() {
        let node = Node::new(vec![0x01, 0x02])
            .with_label("Person")
            .with_property("name", "Bob");

        let proto_node: proto::Node = node.clone().into();
        let back: Node = proto_node.into();
        assert_eq!(node, back);
    }
}
