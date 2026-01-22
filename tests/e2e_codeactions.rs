//! End-to-end tests for code actions functionality.
//!
//! These tests verify that the code actions provider returns appropriate
//! quick fixes for diagnostics.

use tekton_lsp::actions::CodeActionsProvider;
use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOrCommand, Diagnostic, DiagnosticSeverity, Position, Range, Url,
};

fn create_diagnostic(message: &str, line: u32, start_char: u32, end_char: u32) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line,
                character: start_char,
            },
            end: Position {
                line,
                character: end_char,
            },
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
fn test_quick_fix_for_missing_metadata() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostic = create_diagnostic("Missing required field 'metadata'", 0, 0, 10);
    let actions = provider.provide_actions(&uri, &[diagnostic]);

    assert_eq!(actions.len(), 1, "Should return one action");

    if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
        assert!(
            action.title.contains("Add missing field"),
            "Title should mention adding field"
        );
        assert!(
            action.title.contains("metadata"),
            "Title should mention metadata"
        );
        assert_eq!(
            action.kind,
            Some(CodeActionKind::QUICKFIX),
            "Should be a quickfix"
        );

        let edit = action.edit.as_ref().expect("Should have edit");
        let changes = edit.changes.as_ref().expect("Should have changes");
        let text_edits = changes.get(&uri).expect("Should have edits for URI");

        assert!(!text_edits.is_empty(), "Should have at least one edit");
        assert!(
            text_edits[0].new_text.contains("metadata:"),
            "Edit should add metadata"
        );
        assert!(
            text_edits[0].new_text.contains("name:"),
            "Edit should add name under metadata"
        );
    } else {
        panic!("Expected CodeAction");
    }
}

#[test]
fn test_quick_fix_for_missing_spec() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostic = create_diagnostic("Missing required field 'spec'", 3, 0, 10);
    let actions = provider.provide_actions(&uri, &[diagnostic]);

    assert_eq!(actions.len(), 1);

    if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
        assert!(action.title.contains("spec"));

        let edit = action.edit.as_ref().unwrap();
        let changes = edit.changes.as_ref().unwrap();
        let text_edits = &changes[&uri];

        assert!(
            text_edits[0].new_text.contains("spec:"),
            "Edit should add spec"
        );
        assert!(
            text_edits[0].new_text.contains("steps:"),
            "Edit should add steps template"
        );
    } else {
        panic!("Expected CodeAction");
    }
}

#[test]
fn test_quick_fix_for_unknown_field() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostic = create_diagnostic("Unknown field 'unknownField' in Task spec", 5, 2, 15);
    let actions = provider.provide_actions(&uri, &[diagnostic]);

    assert_eq!(actions.len(), 1);

    if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
        assert!(
            action.title.contains("Remove unknown field"),
            "Title should mention removing field"
        );
        assert!(
            action.title.contains("unknownField"),
            "Title should mention field name"
        );
        assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));

        let edit = action.edit.as_ref().unwrap();
        let changes = edit.changes.as_ref().unwrap();
        let text_edits = &changes[&uri];

        // Remove action should have empty new_text
        assert!(
            text_edits[0].new_text.is_empty(),
            "Remove action should delete the line"
        );
        // Should remove the entire line
        assert_eq!(text_edits[0].range.start.line, 5);
        assert_eq!(text_edits[0].range.end.line, 6);
    } else {
        panic!("Expected CodeAction");
    }
}

#[test]
fn test_multiple_diagnostics_multiple_actions() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostics = vec![
        create_diagnostic("Missing required field 'metadata'", 0, 0, 10),
        create_diagnostic("Unknown field 'foo'", 5, 2, 5),
        create_diagnostic("Missing required field 'steps'", 8, 0, 10),
    ];

    let actions = provider.provide_actions(&uri, &diagnostics);

    assert_eq!(actions.len(), 3, "Should return action for each diagnostic");
}

#[test]
fn test_no_action_for_unhandled_diagnostic() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostic = create_diagnostic("Type mismatch: expected string", 5, 10, 20);
    let actions = provider.provide_actions(&uri, &[diagnostic]);

    assert!(
        actions.is_empty(),
        "Should not provide action for unhandled diagnostic"
    );
}

#[test]
fn test_action_has_correct_diagnostics_reference() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostic = create_diagnostic("Missing required field 'name'", 3, 0, 10);
    let actions = provider.provide_actions(&uri, &[diagnostic.clone()]);

    assert_eq!(actions.len(), 1);

    if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
        let action_diagnostics = action.diagnostics.as_ref().expect("Should have diagnostics");
        assert_eq!(action_diagnostics.len(), 1);
        assert_eq!(action_diagnostics[0].message, diagnostic.message);
    } else {
        panic!("Expected CodeAction");
    }
}

#[test]
fn test_quick_fix_for_missing_steps() {
    let provider = CodeActionsProvider::new();
    let uri = Url::parse("file:///tmp/task.yaml").unwrap();

    let diagnostic = create_diagnostic("Missing required field 'steps'", 5, 0, 10);
    let actions = provider.provide_actions(&uri, &[diagnostic]);

    assert_eq!(actions.len(), 1);

    if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
        let edit = action.edit.as_ref().unwrap();
        let changes = edit.changes.as_ref().unwrap();
        let text_edits = &changes[&uri];

        assert!(
            text_edits[0].new_text.contains("steps:"),
            "Edit should add steps"
        );
        assert!(
            text_edits[0].new_text.contains("name:"),
            "Edit should add step with name"
        );
        assert!(
            text_edits[0].new_text.contains("image:"),
            "Edit should add step with image"
        );
    } else {
        panic!("Expected CodeAction");
    }
}
