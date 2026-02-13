//! Path through a property graph - alternating sequence of nodes and edges.

use super::{Edge, Node};
use crate::proto;

/// A path through a property graph.
///
/// Consists of an alternating sequence of nodes and edges where
/// `edges[i]` connects `nodes[i]` and `nodes[i+1]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// Nodes in the path (length = `edges.len()` + 1 for non-empty paths).
    pub nodes: Vec<Node>,
    /// Edges connecting consecutive nodes.
    pub edges: Vec<Edge>,
}

impl Path {
    /// Create a path from a single node (length-0 path).
    #[must_use]
    pub fn from_node(node: Node) -> Self {
        Self {
            nodes: vec![node],
            edges: Vec::new(),
        }
    }

    /// Extend the path with an edge and a destination node.
    #[must_use]
    pub fn with_step(mut self, edge: Edge, node: Node) -> Self {
        self.edges.push(edge);
        self.nodes.push(node);
        self
    }

    /// Returns the number of edges in the path.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Returns true if the path has no edges (single-node path).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the start node of the path, if any.
    #[must_use]
    pub fn start(&self) -> Option<&Node> {
        self.nodes.first()
    }

    /// Returns the end node of the path, if any.
    #[must_use]
    pub fn end(&self) -> Option<&Node> {
        self.nodes.last()
    }
}

// ============================================================================
// Proto conversions
// ============================================================================

impl From<proto::Path> for Path {
    fn from(p: proto::Path) -> Self {
        Self {
            nodes: p.nodes.into_iter().map(Node::from).collect(),
            edges: p.edges.into_iter().map(Edge::from).collect(),
        }
    }
}

impl From<Path> for proto::Path {
    fn from(p: Path) -> Self {
        Self {
            nodes: p.nodes.into_iter().map(proto::Node::from).collect(),
            edges: p.edges.into_iter().map(proto::Edge::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_node_path() {
        let path = Path::from_node(Node::new(vec![0x01]).with_label("Person"));
        assert_eq!(path.len(), 0);
        assert!(path.is_empty());
        assert!(path.start().is_some());
        assert_eq!(path.start(), path.end());
    }

    #[test]
    fn multi_step_path() {
        let a = Node::new(vec![0x01]).with_label("Person");
        let b = Node::new(vec![0x02]).with_label("Person");
        let c = Node::new(vec![0x03]).with_label("Company");
        let e1 = Edge::directed(vec![0x10], vec![0x01], vec![0x02]).with_label("knows");
        let e2 = Edge::directed(vec![0x11], vec![0x02], vec![0x03]).with_label("worksAt");

        let path = Path::from_node(a).with_step(e1, b).with_step(e2, c);

        assert_eq!(path.len(), 2);
        assert!(!path.is_empty());
        assert_eq!(path.nodes.len(), 3);
        assert!(path.start().unwrap().has_label("Person"));
        assert!(path.end().unwrap().has_label("Company"));
    }

    #[test]
    fn round_trip() {
        let path = Path::from_node(Node::new(vec![0x01]).with_label("A")).with_step(
            Edge::directed(vec![0x10], vec![0x01], vec![0x02]).with_label("to"),
            Node::new(vec![0x02]).with_label("B"),
        );

        let proto_path: proto::Path = path.clone().into();
        let back: Path = proto_path.into();
        assert_eq!(path, back);
    }
}
