use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range};

/// A node in the YAML AST with position information
#[derive(Debug, Clone)]
pub struct Node {
    /// The key for this node (if it's a map entry)
    pub key: Option<String>,
    /// The value of this node
    pub value: NodeValue,
    /// The range in the document where this node appears
    pub range: Range,
}

/// The different types of values a YAML node can have
#[derive(Debug, Clone)]
pub enum NodeValue {
    /// A scalar value (string, number, boolean, null)
    Scalar(String),
    /// A mapping of keys to nodes
    Mapping(HashMap<String, Node>),
    /// A sequence of nodes
    Sequence(Vec<Node>),
    /// Null value
    Null,
}

impl Node {
    /// Create a new node
    pub fn new(key: Option<String>, value: NodeValue, range: Range) -> Self {
        Self { key, value, range }
    }

    /// Get a child node by key (for mappings)
    pub fn get(&self, key: &str) -> Option<&Node> {
        match &self.value {
            NodeValue::Mapping(map) => map.get(key),
            _ => None,
        }
    }

    /// Get a scalar value as a string
    pub fn as_scalar(&self) -> Option<&str> {
        match &self.value {
            NodeValue::Scalar(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this node is a mapping (used in tests)
    #[allow(dead_code)]
    pub fn is_mapping(&self) -> bool {
        matches!(self.value, NodeValue::Mapping(_))
    }

    /// Check if this node is a sequence (used in tests)
    #[allow(dead_code)]
    pub fn is_sequence(&self) -> bool {
        matches!(self.value, NodeValue::Sequence(_))
    }

    /// Check if this node is a scalar (used in tests)
    #[allow(dead_code)]
    pub fn is_scalar(&self) -> bool {
        matches!(self.value, NodeValue::Scalar(_))
    }
}

/// A parsed YAML document
#[derive(Debug, Clone)]
pub struct YamlDocument {
    /// The filename or URI of the document (used for diagnostics)
    #[allow(dead_code)]
    pub filename: String,
    /// The root node of the document
    pub root: Node,
    /// Tekton-specific fields extracted for quick access
    pub api_version: Option<String>,
    pub kind: Option<String>,
}

impl YamlDocument {
    /// Create a new YAML document
    pub fn new(filename: String, root: Node) -> Self {
        // Extract common Tekton fields
        let api_version = root
            .get("apiVersion")
            .and_then(|n| n.as_scalar())
            .map(String::from);

        let kind = root
            .get("kind")
            .and_then(|n| n.as_scalar())
            .map(String::from);

        Self {
            filename,
            root,
            api_version,
            kind,
        }
    }

    /// Find the node at a specific position in the document (for hover/goto-definition)
    #[allow(dead_code)]
    pub fn find_node_at_position(&self, position: Position) -> Option<&Node> {
        find_node_at_position_recursive(&self.root, position)
    }
}

/// Recursively find the node at a specific position (for hover/goto-definition)
#[allow(dead_code)]
fn find_node_at_position_recursive(node: &Node, position: Position) -> Option<&Node> {
    // Check if position is within this node's range
    if !position_in_range(position, node.range) {
        return None;
    }

    // Check children first (depth-first search for most specific node)
    match &node.value {
        NodeValue::Mapping(map) => {
            for child in map.values() {
                if let Some(found) = find_node_at_position_recursive(child, position) {
                    return Some(found);
                }
            }
        }
        NodeValue::Sequence(items) => {
            for child in items {
                if let Some(found) = find_node_at_position_recursive(child, position) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }

    // This is the most specific node containing the position
    Some(node)
}

/// Check if a position is within a range (for hover/goto-definition)
#[allow(dead_code)]
fn position_in_range(pos: Position, range: Range) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }
    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }
    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_range(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Range {
        Range {
            start: Position {
                line: start_line,
                character: start_char,
            },
            end: Position {
                line: end_line,
                character: end_char,
            },
        }
    }

    #[test]
    fn test_node_creation() {
        let node = Node::new(
            Some("test".to_string()),
            NodeValue::Scalar("value".to_string()),
            make_range(0, 0, 0, 10),
        );

        assert_eq!(node.key, Some("test".to_string()));
        assert!(node.is_scalar());
        assert_eq!(node.as_scalar(), Some("value"));
    }

    #[test]
    fn test_position_in_range() {
        let range = make_range(1, 5, 3, 10);

        // Inside range
        assert!(position_in_range(Position { line: 2, character: 5 }, range));

        // At start
        assert!(position_in_range(Position { line: 1, character: 5 }, range));

        // At end
        assert!(position_in_range(Position { line: 3, character: 10 }, range));

        // Before range
        assert!(!position_in_range(Position { line: 1, character: 4 }, range));

        // After range
        assert!(!position_in_range(Position { line: 3, character: 11 }, range));
    }

    #[test]
    fn test_find_node_at_position() {
        let mut map = HashMap::new();
        map.insert(
            "key1".to_string(),
            Node::new(
                Some("key1".to_string()),
                NodeValue::Scalar("value1".to_string()),
                make_range(1, 0, 1, 12),
            ),
        );

        let root = Node::new(
            None,
            NodeValue::Mapping(map),
            make_range(0, 0, 2, 0),
        );

        let doc = YamlDocument::new("test.yaml".to_string(), root);

        // Find node at position within key1
        let found = doc.find_node_at_position(Position { line: 1, character: 5 });
        assert!(found.is_some());
        assert_eq!(found.unwrap().key, Some("key1".to_string()));
    }
}
