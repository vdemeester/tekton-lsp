//! Completion provider implementation.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};

use crate::parser::{YamlDocument, Node, NodeValue};
use super::schemas::{TektonSchemas, FieldSchema};

#[derive(Debug, Clone)]
pub struct CompletionProvider {
    schemas: TektonSchemas,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            schemas: TektonSchemas::new(),
        }
    }

    /// Provide completion suggestions for a given position in a YAML document.
    pub fn provide_completions(
        &self,
        yaml_doc: &YamlDocument,
        position: Position,
    ) -> Vec<CompletionItem> {
        // Find the context at the cursor position
        let context = self.determine_context(yaml_doc, position);

        // Get appropriate fields for the context
        let fields = self.get_fields_for_context(&context, yaml_doc);

        // Convert to completion items
        fields
            .iter()
            .map(|field| self.field_to_completion_item(field))
            .collect()
    }

    fn determine_context(&self, yaml_doc: &YamlDocument, position: Position) -> CompletionContext {
        // Walk the document tree to find the context
        self.find_completion_context(&yaml_doc.root, position, yaml_doc)
    }

    fn find_completion_context(
        &self,
        node: &Node,
        position: Position,
        yaml_doc: &YamlDocument,
    ) -> CompletionContext {
        // Check if position is within this node's range
        if !self.position_in_range(position, &node.range) {
            return CompletionContext::Unknown;
        }

        // If we have a key, check what context this represents
        if let Some(key) = &node.key {
            match key.as_str() {
                "metadata" => {
                    // Position is in metadata - return Metadata context for completion
                    return CompletionContext::Metadata;
                }
                "spec" => {
                    // First check if we're inside a child array (tasks/steps)
                    if let NodeValue::Mapping(children) = &node.value {
                        for (child_key, child) in children {
                            if self.position_in_range(position, &child.range) {
                                // We're inside a specific child - check what it is
                                match child_key.as_str() {
                                    "tasks" | "finally" => return CompletionContext::PipelineTask,
                                    "steps" => return CompletionContext::Step,
                                    _ => {}
                                }
                            }
                        }
                    }

                    // Not in a child array - return spec context based on kind
                    if let Some(kind) = &yaml_doc.kind {
                        match kind.as_str() {
                            "Pipeline" => return CompletionContext::PipelineSpec,
                            "Task" => return CompletionContext::TaskSpec,
                            _ => {}
                        }
                    }
                }
                "tasks" | "finally" => {
                    // We're in a tasks array - completions are for PipelineTask
                    return CompletionContext::PipelineTask;
                }
                "steps" => {
                    // We're in a steps array - completions are for Step
                    return CompletionContext::Step;
                }
                _ => {}
            }
        }

        // Recursively check children
        if let NodeValue::Mapping(children) = &node.value {
            for (_key, child) in children {
                let child_context = self.find_completion_context(child, position, yaml_doc);
                if child_context != CompletionContext::Unknown {
                    return child_context;
                }
            }
        } else if let NodeValue::Sequence(items) = &node.value {
            for item in items {
                let item_context = self.find_completion_context(item, position, yaml_doc);
                if item_context != CompletionContext::Unknown {
                    return item_context;
                }
            }
        }

        CompletionContext::Unknown
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

    fn get_fields_for_context(
        &self,
        context: &CompletionContext,
        _yaml_doc: &YamlDocument,
    ) -> Vec<FieldSchema> {
        match context {
            CompletionContext::Metadata => self.schemas.get_metadata_fields().to_vec(),
            CompletionContext::PipelineSpec => self.schemas.get_pipeline_spec_fields().to_vec(),
            CompletionContext::PipelineTask => self.schemas.get_pipeline_task_fields().to_vec(),
            CompletionContext::TaskSpec => self.schemas.get_task_spec_fields().to_vec(),
            CompletionContext::Step => self.schemas.get_step_fields().to_vec(),
            CompletionContext::Unknown => vec![],
        }
    }

    fn field_to_completion_item(&self, field: &FieldSchema) -> CompletionItem {
        use super::schemas::FieldType;

        let kind = match field.field_type {
            FieldType::String => CompletionItemKind::FIELD,
            FieldType::Array => CompletionItemKind::VALUE,
            FieldType::Object => CompletionItemKind::STRUCT,
            FieldType::Boolean => CompletionItemKind::VALUE,
        };

        CompletionItem {
            label: field.name.clone(),
            kind: Some(kind),
            detail: Some(field.description.clone()),
            documentation: None,
            ..Default::default()
        }
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
enum CompletionContext {
    Metadata,
    PipelineSpec,
    PipelineTask,
    TaskSpec,
    Step,
    Unknown,
}
