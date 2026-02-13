//! Property graph edge type (directed or undirected).

use std::collections::HashMap;

use crate::proto;
use super::Value;

/// A property graph edge with an opaque ID, labels, endpoints, and properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Opaque element identifier.
    pub id: Vec<u8>,
    /// Label set.
    pub labels: Vec<String>,
    /// Source node ID (directed) or endpoint A (undirected).
    pub source_node_id: Vec<u8>,
    /// Target node ID (directed) or endpoint B (undirected).
    pub target_node_id: Vec<u8>,
    /// Whether this is an undirected edge.
    pub undirected: bool,
    /// Property map.
    pub properties: HashMap<String, Value>,
}

impl Edge {
    /// Create a new directed edge.
    #[must_use]
    pub fn directed(
        id: impl Into<Vec<u8>>,
        source: impl Into<Vec<u8>>,
        target: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            id: id.into(),
            labels: Vec::new(),
            source_node_id: source.into(),
            target_node_id: target.into(),
            undirected: false,
            properties: HashMap::new(),
        }
    }

    /// Create a new undirected edge.
    #[must_use]
    pub fn undirected(
        id: impl Into<Vec<u8>>,
        endpoint_a: impl Into<Vec<u8>>,
        endpoint_b: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            id: id.into(),
            labels: Vec::new(),
            source_node_id: endpoint_a.into(),
            target_node_id: endpoint_b.into(),
            undirected: true,
            properties: HashMap::new(),
        }
    }

    /// Add a label to the edge.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Add a property to the edge.
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
}

// ============================================================================
// Proto conversions
// ============================================================================

impl From<proto::Edge> for Edge {
    fn from(p: proto::Edge) -> Self {
        Self {
            id: p.id,
            labels: p.labels,
            source_node_id: p.source_node_id,
            target_node_id: p.target_node_id,
            undirected: p.undirected,
            properties: p
                .properties
                .into_iter()
                .map(|(k, v)| (k, Value::from(v)))
                .collect(),
        }
    }
}

impl From<Edge> for proto::Edge {
    fn from(e: Edge) -> Self {
        Self {
            id: e.id,
            labels: e.labels,
            source_node_id: e.source_node_id,
            target_node_id: e.target_node_id,
            undirected: e.undirected,
            properties: e
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
    fn directed_edge() {
        let edge = Edge::directed(vec![0x10], vec![0x01], vec![0x02])
            .with_label("knows")
            .with_property("since", 2020_i64);

        assert!(!edge.undirected);
        assert_eq!(edge.labels, vec!["knows"]);
        assert_eq!(edge.property("since"), Some(&Value::Integer(2020)));
    }

    #[test]
    fn undirected_edge() {
        let edge = Edge::undirected(vec![0x10], vec![0x01], vec![0x02])
            .with_label("friends_with");

        assert!(edge.undirected);
    }

    #[test]
    fn round_trip() {
        let edge = Edge::directed(vec![0x10], vec![0x01], vec![0x02])
            .with_label("knows")
            .with_property("weight", 0.5_f64);

        let proto_edge: proto::Edge = edge.clone().into();
        let back: Edge = proto_edge.into();
        assert_eq!(edge, back);
    }
}
