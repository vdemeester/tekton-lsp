//! Code actions provider implementation.

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Position, Range, TextEdit, Url,
    WorkspaceEdit,
};
use std::collections::HashMap;

/// Provides code actions (quick fixes) for Tekton YAML files.
#[derive(Debug, Clone, Default)]
pub struct CodeActionsProvider;

impl CodeActionsProvider {
    /// Create a new code actions provider.
    pub fn new() -> Self {
        Self
    }

    /// Provide code actions for the given diagnostics.
    pub fn provide_actions(
        &self,
        uri: &Url,
        diagnostics: &[Diagnostic],
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        for diagnostic in diagnostics {
            if let Some(action) = self.create_action_for_diagnostic(uri, diagnostic) {
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }

        actions
    }

    /// Create a code action for a specific diagnostic.
    fn create_action_for_diagnostic(&self, uri: &Url, diagnostic: &Diagnostic) -> Option<CodeAction> {
        let message = &diagnostic.message;

        // Handle missing required field
        if message.contains("Missing required field") {
            return self.create_add_field_action(uri, diagnostic, message);
        }

        // Handle unknown field
        if message.contains("Unknown field") {
            return self.create_remove_field_action(uri, diagnostic, message);
        }

        None
    }

    /// Create an action to add a missing required field.
    fn create_add_field_action(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        message: &str,
    ) -> Option<CodeAction> {
        // Extract field name from message like "Missing required field 'metadata'"
        let field_name = self.extract_field_name(message, "Missing required field")?;

        // Determine the text to insert based on the field
        let insert_text = self.get_field_template(&field_name);

        // Insert at the end of the diagnostic range (after the current line)
        let insert_position = Position {
            line: diagnostic.range.end.line + 1,
            character: 0,
        };

        let mut changes = HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: insert_position,
                    end: insert_position,
                },
                new_text: insert_text,
            }],
        );

        Some(CodeAction {
            title: format!("Add missing field '{}'", field_name),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// Create an action to remove an unknown field.
    fn create_remove_field_action(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        message: &str,
    ) -> Option<CodeAction> {
        // Extract field name from message like "Unknown field 'foo'"
        let field_name = self.extract_field_name(message, "Unknown field")?;

        // Remove the entire line containing the unknown field
        let remove_range = Range {
            start: Position {
                line: diagnostic.range.start.line,
                character: 0,
            },
            end: Position {
                line: diagnostic.range.start.line + 1,
                character: 0,
            },
        };

        let mut changes = HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: remove_range,
                new_text: String::new(),
            }],
        );

        Some(CodeAction {
            title: format!("Remove unknown field '{}'", field_name),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// Extract a field name from a diagnostic message.
    fn extract_field_name(&self, message: &str, prefix: &str) -> Option<String> {
        // Look for pattern like "prefix 'fieldname'"
        let start = message.find(prefix)? + prefix.len();
        let after_prefix = &message[start..];

        // Find the quoted field name
        let quote_start = after_prefix.find('\'')?;
        let name_start = quote_start + 1;
        let quote_end = after_prefix[name_start..].find('\'')?;

        Some(after_prefix[name_start..name_start + quote_end].to_string())
    }

    /// Get a template for a field.
    fn get_field_template(&self, field_name: &str) -> String {
        match field_name {
            "metadata" => "metadata:\n  name: \n".to_string(),
            "spec" => "spec:\n  steps:\n    - name: step-1\n      image: alpine\n".to_string(),
            "name" => "  name: \n".to_string(),
            "steps" => "  steps:\n    - name: step-1\n      image: alpine\n".to_string(),
            "tasks" => "  tasks:\n    - name: task-1\n      taskRef:\n        name: \n".to_string(),
            "image" => "      image: alpine\n".to_string(),
            _ => format!("  {}: \n", field_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::DiagnosticSeverity;

    fn create_diagnostic(message: &str, line: u32) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position { line, character: 0 },
                end: Position { line, character: 10 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("tekton-lsp".to_string()),
            message: message.to_string(),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    #[test]
    fn test_add_missing_field_action() {
        let provider = CodeActionsProvider::new();
        let uri = Url::parse("file:///tmp/test.yaml").unwrap();

        let diagnostic = create_diagnostic("Missing required field 'metadata'", 0);
        let actions = provider.provide_actions(&uri, &[diagnostic]);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Add missing field"));
            assert!(action.title.contains("metadata"));
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.edit.is_some());

            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            assert!(changes.contains_key(&uri));

            let text_edits = &changes[&uri];
            assert_eq!(text_edits.len(), 1);
            assert!(text_edits[0].new_text.contains("metadata:"));
            assert!(text_edits[0].new_text.contains("name:"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_remove_unknown_field_action() {
        let provider = CodeActionsProvider::new();
        let uri = Url::parse("file:///tmp/test.yaml").unwrap();

        let diagnostic = create_diagnostic("Unknown field 'foo' in Task spec", 5);
        let actions = provider.provide_actions(&uri, &[diagnostic]);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Remove unknown field"));
            assert!(action.title.contains("foo"));
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.edit.is_some());

            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let text_edits = &changes[&uri];

            // Should remove the entire line
            assert_eq!(text_edits.len(), 1);
            assert!(text_edits[0].new_text.is_empty());
            assert_eq!(text_edits[0].range.start.line, 5);
            assert_eq!(text_edits[0].range.end.line, 6);
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_no_action_for_unhandled_diagnostic() {
        let provider = CodeActionsProvider::new();
        let uri = Url::parse("file:///tmp/test.yaml").unwrap();

        let diagnostic = create_diagnostic("Some other error", 0);
        let actions = provider.provide_actions(&uri, &[diagnostic]);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_multiple_diagnostics() {
        let provider = CodeActionsProvider::new();
        let uri = Url::parse("file:///tmp/test.yaml").unwrap();

        let diagnostics = vec![
            create_diagnostic("Missing required field 'spec'", 3),
            create_diagnostic("Unknown field 'bar'", 5),
        ];

        let actions = provider.provide_actions(&uri, &diagnostics);
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_extract_field_name() {
        let provider = CodeActionsProvider::new();

        assert_eq!(
            provider.extract_field_name("Missing required field 'metadata'", "Missing required field"),
            Some("metadata".to_string())
        );

        assert_eq!(
            provider.extract_field_name("Unknown field 'foo' in spec", "Unknown field"),
            Some("foo".to_string())
        );

        assert_eq!(
            provider.extract_field_name("No field here", "Missing"),
            None
        );
    }
}
