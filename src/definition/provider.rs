//! Definition provider implementation.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Position};

use crate::parser::{Node, NodeValue, YamlDocument};
use crate::workspace::WorkspaceIndex;

/// Provides go-to-definition functionality for Tekton resources.
#[derive(Debug, Clone)]
pub struct DefinitionProvider {
    index: WorkspaceIndex,
}

impl DefinitionProvider {
    /// Create a new definition provider with the given workspace index.
    pub fn new(index: WorkspaceIndex) -> Self {
        Self { index }
    }

    /// Get the workspace index (for updating).
    pub fn index(&self) -> &WorkspaceIndex {
        &self.index
    }

    /// Provide go-to-definition for a position in a document.
    pub fn provide_definition(
        &self,
        yaml_doc: &YamlDocument,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        // Find what we're hovering over
        let context = self.find_reference_context(&yaml_doc.root, position, yaml_doc)?;

        // Look up the definition in the workspace index
        let definition = self.index.find_resource(&context.kind, &context.name)?;

        Some(GotoDefinitionResponse::Scalar(definition.location))
    }

    /// Find the reference context at a position (what resource is being referenced).
    fn find_reference_context(
        &self,
        node: &Node,
        position: Position,
        yaml_doc: &YamlDocument,
    ) -> Option<ReferenceContext> {
        if !self.position_in_range(position, &node.range) {
            return None;
        }

        // Check if we're in a taskRef or pipelineRef
        if let Some(key) = &node.key {
            match key.as_str() {
                "taskRef" => {
                    // Check if we're on the name field
                    if let Some(name_node) = node.get("name") {
                        if self.position_in_range(position, &name_node.range) {
                            if let Some(name) = name_node.as_scalar() {
                                // Get kind (default to Task)
                                let kind = node
                                    .get("kind")
                                    .and_then(|k| k.as_scalar())
                                    .unwrap_or("Task");
                                return Some(ReferenceContext {
                                    kind: kind.to_string(),
                                    name: name.to_string(),
                                });
                            }
                        }
                    }
                }
                "pipelineRef" => {
                    // Check if we're on the name field
                    if let Some(name_node) = node.get("name") {
                        if self.position_in_range(position, &name_node.range) {
                            if let Some(name) = name_node.as_scalar() {
                                return Some(ReferenceContext {
                                    kind: "Pipeline".to_string(),
                                    name: name.to_string(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Recursively check children
        match &node.value {
            NodeValue::Mapping(map) => {
                for (_key, child) in map {
                    if let Some(ctx) = self.find_reference_context(child, position, yaml_doc) {
                        return Some(ctx);
                    }
                }
            }
            NodeValue::Sequence(items) => {
                for item in items {
                    if let Some(ctx) = self.find_reference_context(item, position, yaml_doc) {
                        return Some(ctx);
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn position_in_range(&self, pos: Position, range: &tower_lsp::lsp_types::Range) -> bool {
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

/// Context for a resource reference.
#[derive(Debug)]
struct ReferenceContext {
    kind: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use tower_lsp::lsp_types::Url;

    fn make_test_uri(path: &str) -> Url {
        Url::parse(&format!("file://{}", path)).unwrap()
    }

    #[test]
    fn test_goto_task_definition() {
        let index = WorkspaceIndex::new();

        // Index a task
        let task_uri = make_test_uri("/workspace/tasks/build.yaml");
        let task_content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  steps:
    - image: golang"#;
        index.index_document(&task_uri, task_content).unwrap();

        // Create a pipeline that references the task
        let pipeline_content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

        let pipeline_doc = parser::parse_yaml("pipeline.yaml", pipeline_content).unwrap();

        let provider = DefinitionProvider::new(index);

        // Position on "build-task" in taskRef.name (line 8, on the name value)
        let position = Position {
            line: 8,
            character: 14, // On "build-task"
        };

        let result = provider.provide_definition(&pipeline_doc, position);

        assert!(result.is_some(), "Should find definition for taskRef");

        let location = match result.unwrap() {
            GotoDefinitionResponse::Scalar(loc) => loc,
            _ => panic!("Expected scalar location"),
        };

        assert_eq!(location.uri, task_uri);
    }

    #[test]
    fn test_goto_definition_not_on_ref() {
        let index = WorkspaceIndex::new();

        let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

        let doc = parser::parse_yaml("pipeline.yaml", content).unwrap();
        let provider = DefinitionProvider::new(index);

        // Position on "main" (not a reference)
        let position = Position {
            line: 3,
            character: 8,
        };

        let result = provider.provide_definition(&doc, position);
        assert!(result.is_none(), "Should not find definition outside of refs");
    }

    #[test]
    fn test_goto_definition_task_not_found() {
        let index = WorkspaceIndex::new();
        // Don't index any tasks

        let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main
spec:
  tasks:
    - name: build
      taskRef:
        name: nonexistent-task"#;

        let doc = parser::parse_yaml("pipeline.yaml", content).unwrap();
        let provider = DefinitionProvider::new(index);

        let position = Position {
            line: 8,
            character: 14,
        };

        let result = provider.provide_definition(&doc, position);
        assert!(result.is_none(), "Should not find definition for nonexistent task");
    }
}
