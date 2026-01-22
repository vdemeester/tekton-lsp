//! End-to-end tests for document formatting functionality.
//!
//! These tests verify that the formatting provider correctly normalizes
//! YAML structure and indentation.

use tekton_lsp::formatting::FormattingProvider;

#[test]
fn test_format_normalizes_indentation() {
    let provider = FormattingProvider::new();

    // YAML with 4-space indentation
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
    name: my-task
spec:
    steps:
        - name: build
          image: golang:1.21"#;

    let edits = provider.format(content);

    assert!(edits.is_some(), "Should return formatting edits");

    let edits = edits.unwrap();
    assert!(!edits.is_empty(), "Should have edits for inconsistent indentation");

    let formatted = &edits[0].new_text;
    assert!(
        formatted.contains("apiVersion:"),
        "Should preserve apiVersion"
    );
    assert!(formatted.contains("kind: Task"), "Should preserve kind");
    assert!(
        formatted.contains("name: my-task"),
        "Should preserve metadata.name"
    );
}

#[test]
fn test_format_preserves_structure() {
    let provider = FormattingProvider::new();

    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  params:
    - name: version
      type: string
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

    let edits = provider.format(content);

    assert!(edits.is_some(), "Should return edits");

    let edits = edits.unwrap();
    if !edits.is_empty() {
        let formatted = &edits[0].new_text;

        // Verify key elements are preserved
        assert!(formatted.contains("apiVersion:"), "Should preserve apiVersion");
        assert!(formatted.contains("Pipeline"), "Should preserve Pipeline");
        assert!(formatted.contains("main-pipeline"), "Should preserve name");
        assert!(formatted.contains("params:"), "Should preserve params");
        assert!(formatted.contains("tasks:"), "Should preserve tasks");
        assert!(formatted.contains("taskRef:"), "Should preserve taskRef");
    }
}

#[test]
fn test_format_invalid_yaml_returns_none() {
    let provider = FormattingProvider::new();

    let content = r#"
apiVersion: tekton.dev/v1
kind: Task
  invalid: indentation
    here: is wrong"#;

    let edits = provider.format(content);

    // Invalid YAML should return None (can't format)
    assert!(edits.is_none(), "Should return None for invalid YAML");
}

#[test]
fn test_format_already_formatted() {
    let provider = FormattingProvider::new();

    // First, get the canonical format
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: test"#;

    let edits = provider.format(content);
    assert!(edits.is_some());

    // If we format again, it should be stable
    if let Some(edits) = edits {
        if !edits.is_empty() {
            let formatted = &edits[0].new_text;

            // Format the formatted content
            let edits2 = provider.format(formatted);
            assert!(edits2.is_some());

            let edits2 = edits2.unwrap();
            // Should have no changes (already formatted)
            assert!(
                edits2.is_empty(),
                "Formatting should be idempotent"
            );
        }
    }
}

#[test]
fn test_format_complex_pipeline() {
    let provider = FormattingProvider::new();

    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: complex-pipeline
  labels:
    app: myapp
spec:
  params:
    - name: repo-url
      type: string
    - name: revision
      type: string
      default: main
  workspaces:
    - name: shared-workspace
  tasks:
    - name: fetch-source
      taskRef:
        name: git-clone
      workspaces:
        - name: output
          workspace: shared-workspace
      params:
        - name: url
          value: $(params.repo-url)
    - name: build
      taskRef:
        name: build-task
      runAfter:
        - fetch-source"#;

    let edits = provider.format(content);

    assert!(edits.is_some(), "Should format complex pipeline");

    let edits = edits.unwrap();
    if !edits.is_empty() {
        let formatted = &edits[0].new_text;

        // Verify complex structure is preserved
        assert!(formatted.contains("params:"));
        assert!(formatted.contains("workspaces:"));
        assert!(formatted.contains("tasks:"));
        assert!(formatted.contains("runAfter:"));
    }
}

#[test]
fn test_format_task_with_script() {
    let provider = FormattingProvider::new();

    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: script-task
spec:
  steps:
    - name: run-script
      image: bash
      script: |
        #!/bin/bash
        echo "Hello World"
        exit 0"#;

    let edits = provider.format(content);

    assert!(edits.is_some(), "Should format task with script");

    let edits = edits.unwrap();
    if !edits.is_empty() {
        let formatted = &edits[0].new_text;

        // Script should be preserved
        assert!(
            formatted.contains("script:") || formatted.contains("script"),
            "Should preserve script field"
        );
    }
}
