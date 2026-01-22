//! Hover provider implementation.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

use crate::parser::{Node, NodeValue, YamlDocument};
use super::docs::get_documentation;

/// Provides hover documentation for Tekton YAML files.
#[derive(Debug, Clone)]
pub struct HoverProvider;

impl HoverProvider {
    /// Create a new hover provider.
    pub fn new() -> Self {
        Self
    }

    /// Provide hover information for a given position in a YAML document.
    pub fn provide_hover(&self, yaml_doc: &YamlDocument, position: Position) -> Option<Hover> {
        // Find the node at the cursor position
        let (node, key) = self.find_node_with_key_at_position(&yaml_doc.root, position)?;

        // Try to get documentation
        let documentation = self.get_hover_documentation(node, key.as_deref(), yaml_doc)?;

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: documentation,
            }),
            range: Some(node.range),
        })
    }

    /// Find the node at a position, along with its key if it's a mapping entry.
    fn find_node_with_key_at_position<'a>(
        &self,
        node: &'a Node,
        position: Position,
    ) -> Option<(&'a Node, Option<String>)> {
        if !self.position_in_range(position, &node.range) {
            return None;
        }

        // Check children first for more specific matches
        match &node.value {
            NodeValue::Mapping(map) => {
                for (key, child) in map {
                    if let Some(result) = self.find_node_with_key_at_position(child, position) {
                        return Some(result);
                    }
                    // If we're in the child's range but didn't find a more specific match,
                    // return the child with its key
                    if self.position_in_range(position, &child.range) {
                        return Some((child, Some(key.clone())));
                    }
                }
            }
            NodeValue::Sequence(items) => {
                for item in items {
                    if let Some(result) = self.find_node_with_key_at_position(item, position) {
                        return Some(result);
                    }
                }
            }
            _ => {}
        }

        // Return this node with its key
        Some((node, node.key.clone()))
    }

    /// Get hover documentation for a node.
    fn get_hover_documentation(
        &self,
        node: &Node,
        key: Option<&str>,
        yaml_doc: &YamlDocument,
    ) -> Option<String> {
        // First, try to get documentation for the key (field name)
        if let Some(key) = key {
            if let Some(doc) = get_documentation(key) {
                return Some(doc.to_string());
            }
        }

        // If the node is a scalar value, check if it's a known kind
        if let NodeValue::Scalar(value) = &node.value {
            if let Some(doc) = get_documentation(value) {
                return Some(doc.to_string());
            }
        }

        // Check if the key is a well-known value
        if let Some(key) = &node.key {
            if let Some(doc) = get_documentation(key) {
                return Some(doc.to_string());
            }
        }

        // For document-level kind, provide context
        if key == Some("kind") {
            if let Some(kind) = &yaml_doc.kind {
                if let Some(doc) = get_documentation(kind) {
                    return Some(doc.to_string());
                }
            }
        }

        None
    }

    fn position_in_range(&self, pos: Position, range: &Range) -> bool {
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
}

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}
