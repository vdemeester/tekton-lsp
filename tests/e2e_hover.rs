//! End-to-end tests for hover functionality.
//!
//! These tests verify that the hover provider returns appropriate
//! documentation based on cursor position and document context.

use tekton_lsp::{hover::HoverProvider, parser};
use tower_lsp::lsp_types::Position;

// TDD Cycle 1: Hover on field keys (tasks, steps, params)
#[test]
fn test_hover_on_tasks_field() {
    // Given: A Pipeline with cursor on "tasks" field
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  tasks:
    - name: build"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    // When: Request hover on "tasks" field (line 5, "tasks:")
    let position = Position {
        line: 5,  // "  tasks:" line
        character: 4,  // On "tasks"
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    // Then: Should return documentation about tasks
    assert!(hover.is_some(), "Should return hover information for 'tasks' field");

    let hover = hover.unwrap();
    let content = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("tasks") || content.contains("Tasks"),
        "Hover should mention tasks. Got: {}", content);
    assert!(content.contains("PipelineTask") || content.contains("Pipeline"),
        "Hover should describe PipelineTask. Got: {}", content);
}

#[test]
fn test_hover_on_steps_field() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  steps:
    - name: compile"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    // Position on "steps" field
    let position = Position {
        line: 5,
        character: 4,
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'steps' field");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("steps") || content.contains("Steps") || content.contains("container"),
        "Hover should describe steps. Got: {}", content);
}

#[test]
fn test_hover_on_params_field() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  params:
    - name: version"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    let position = Position {
        line: 5,
        character: 4,
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'params' field");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("param") || content.contains("Param"),
        "Hover should describe params. Got: {}", content);
}

// TDD Cycle 2: Hover on kind values (Pipeline, Task)
#[test]
fn test_hover_on_pipeline_kind() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    // Position on "Pipeline" value (line 1, character 6+)
    let position = Position {
        line: 1,
        character: 7,  // On "Pipeline"
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'Pipeline' kind value");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("Pipeline"),
        "Hover should describe Pipeline. Got: {}", content);
    assert!(content.contains("Task") || content.contains("collection"),
        "Hover should describe Pipeline as a collection of Tasks. Got: {}", content);
}

#[test]
fn test_hover_on_task_kind() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    let position = Position {
        line: 1,
        character: 7,
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'Task' kind value");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("Task"),
        "Hover should describe Task. Got: {}", content);
    assert!(content.contains("Step") || content.contains("Pod"),
        "Hover should describe Task execution. Got: {}", content);
}

// TDD Cycle 3: Hover on metadata fields
#[test]
fn test_hover_on_metadata_field() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    // Position on "metadata" field
    let position = Position {
        line: 2,
        character: 2,
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'metadata' field");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("metadata") || content.contains("Kubernetes"),
        "Hover should describe metadata. Got: {}", content);
}

#[test]
fn test_hover_on_name_field() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    // Position on "name" field inside metadata
    let position = Position {
        line: 3,
        character: 4,
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'name' field");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("name") || content.contains("Name"),
        "Hover should describe name field. Got: {}", content);
}

#[test]
fn test_hover_on_labels_field() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
  labels:
    app: my-app"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    let provider = HoverProvider::new();

    // Position on "labels" field
    let position = Position {
        line: 4,
        character: 4,
    };

    let hover = provider.provide_hover(&yaml_doc, position);

    assert!(hover.is_some(), "Should return hover for 'labels' field");

    let content = match hover.unwrap().contents {
        tower_lsp::lsp_types::HoverContents::Markup(m) => m.value,
        _ => panic!("Expected Markup content"),
    };

    assert!(content.contains("label") || content.contains("Label"),
        "Hover should describe labels. Got: {}", content);
}
