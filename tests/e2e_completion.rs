//! End-to-end tests for completion functionality.
//!
//! These tests verify that the completion provider returns appropriate
//! suggestions based on cursor position and document context.

use tekton_lsp::{parser, completion::CompletionProvider};
use tower_lsp::lsp_types::Position;

// TDD Cycle 1: Basic metadata completion
#[test]
fn test_complete_metadata_fields() {
    // Given: A Pipeline with cursor in metadata section (with placeholder value)
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  namespace: default"#;

    // Parse the document
    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");

    // Create completion provider
    let provider = CompletionProvider::new();

    // When: Request completions in metadata section (at beginning of "namespace" line)
    let position = Position {
        line: 3,  // The "namespace: default" line
        character: 2,  // At start of "namespace"
    };

    let completions = provider.provide_completions(&yaml_doc, position);

    // Then: Should suggest metadata fields
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    
    assert!(labels.contains(&"name".to_string()), 
        "Should suggest 'name' field. Got: {:?}", labels);
    assert!(labels.contains(&"namespace".to_string()),
        "Should suggest 'namespace' field. Got: {:?}", labels);
    assert!(labels.contains(&"labels".to_string()),
        "Should suggest 'labels' field. Got: {:?}", labels);
    assert!(labels.contains(&"annotations".to_string()),
        "Should suggest 'annotations' field. Got: {:?}", labels);
}

// TDD Cycle 2: Pipeline spec completion
#[test]
#[ignore] // Enable after Cycle 1 passes
fn test_complete_pipeline_spec_fields() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  "#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");
    let provider = CompletionProvider::new();

    let position = Position { line: 4, character: 7 };
    let completions = provider.provide_completions(&yaml_doc, position);

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    
    assert!(labels.contains(&"tasks".to_string()));
    assert!(labels.contains(&"params".to_string()));
    assert!(labels.contains(&"workspaces".to_string()));
}

// TDD Cycle 3: PipelineTask fields completion
#[test]
#[ignore] // Enable after Cycle 2 passes
fn test_complete_pipeline_task_fields() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  tasks:
    - "#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");
    let provider = CompletionProvider::new();

    let position = Position { line: 6, character: 6 };
    let completions = provider.provide_completions(&yaml_doc, position);

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    
    assert!(labels.contains(&"name".to_string()));
    assert!(labels.contains(&"taskRef".to_string()));
    assert!(labels.contains(&"taskSpec".to_string()));
}

// TDD Cycle 4: Task spec completion
#[test]
#[ignore] // Enable after Cycle 3 passes
fn test_complete_task_spec_fields() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: test-task
spec:
  "#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");
    let provider = CompletionProvider::new();

    let position = Position { line: 4, character: 7 };
    let completions = provider.provide_completions(&yaml_doc, position);

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    
    assert!(labels.contains(&"steps".to_string()));
    assert!(labels.contains(&"params".to_string()));
    assert!(labels.contains(&"workspaces".to_string()));
}

// TDD Cycle 5: Step fields completion
#[test]
#[ignore] // Enable after Cycle 4 passes
fn test_complete_step_fields() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: test-task
spec:
  steps:
    - "#;

    let yaml_doc = parser::parse_yaml("test.yaml", content)
        .expect("Failed to parse YAML");
    let provider = CompletionProvider::new();

    let position = Position { line: 6, character: 6 };
    let completions = provider.provide_completions(&yaml_doc, position);

    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    
    assert!(labels.contains(&"name".to_string()));
    assert!(labels.contains(&"image".to_string()));
    assert!(labels.contains(&"script".to_string()));
    assert!(labels.contains(&"command".to_string()));
}
