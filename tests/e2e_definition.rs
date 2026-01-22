//! End-to-end tests for go-to-definition functionality.
//!
//! These tests verify that the definition provider correctly resolves
//! references to Task and Pipeline definitions.

use tekton_lsp::{definition::DefinitionProvider, parser, workspace::WorkspaceIndex};
use tower_lsp::lsp_types::{Position, Url};

fn create_provider_with_indexed_task() -> (DefinitionProvider, Url) {
    let index = WorkspaceIndex::new();
    let provider = DefinitionProvider::new(index);

    // Index a Task document
    let task_uri = Url::parse("file:///tmp/tasks/build-task.yaml").unwrap();
    let task_content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  steps:
    - name: compile
      image: golang:1.21"#;

    provider
        .index()
        .index_document(&task_uri, task_content)
        .expect("Failed to index task");

    (provider, task_uri)
}

#[test]
fn test_goto_task_definition_from_pipeline() {
    let (provider, _task_uri) = create_provider_with_indexed_task();

    // Pipeline that references the task
    let pipeline_content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

    let pipeline_uri = Url::parse("file:///tmp/pipelines/main.yaml").unwrap();
    let yaml_doc = parser::parse_yaml(&pipeline_uri.to_string(), pipeline_content)
        .expect("Failed to parse pipeline");

    // Position on "build-task" in taskRef.name (line 8)
    let position = Position {
        line: 8,
        character: 14,
    };

    let definition = provider.provide_definition(&yaml_doc, position);

    assert!(
        definition.is_some(),
        "Should find definition for task reference"
    );
}

#[test]
fn test_goto_definition_task_not_indexed() {
    let index = WorkspaceIndex::new();
    let provider = DefinitionProvider::new(index);

    // Pipeline that references a task that isn't indexed
    let pipeline_content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: unknown-task"#;

    let pipeline_uri = Url::parse("file:///tmp/pipelines/main.yaml").unwrap();
    let yaml_doc = parser::parse_yaml(&pipeline_uri.to_string(), pipeline_content)
        .expect("Failed to parse pipeline");

    let position = Position {
        line: 8,
        character: 14,
    };

    let definition = provider.provide_definition(&yaml_doc, position);

    // Should return None when task isn't found
    assert!(
        definition.is_none(),
        "Should not find definition for unknown task"
    );
}

#[test]
fn test_goto_definition_not_on_task_ref() {
    let (provider, _) = create_provider_with_indexed_task();

    let pipeline_content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

    let pipeline_uri = Url::parse("file:///tmp/pipelines/main.yaml").unwrap();
    let yaml_doc = parser::parse_yaml(&pipeline_uri.to_string(), pipeline_content)
        .expect("Failed to parse pipeline");

    // Position on "Pipeline" (not a reference)
    let position = Position {
        line: 1,
        character: 7,
    };

    let definition = provider.provide_definition(&yaml_doc, position);

    assert!(
        definition.is_none(),
        "Should not provide definition when not on a reference"
    );
}

#[test]
fn test_workspace_index_multiple_tasks() {
    let index = WorkspaceIndex::new();
    let provider = DefinitionProvider::new(index);

    // Index multiple tasks
    let task1_uri = Url::parse("file:///tmp/tasks/task1.yaml").unwrap();
    let task1_content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: task-one
spec:
  steps:
    - name: step1
      image: alpine"#;

    let task2_uri = Url::parse("file:///tmp/tasks/task2.yaml").unwrap();
    let task2_content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: task-two
spec:
  steps:
    - name: step1
      image: alpine"#;

    provider
        .index()
        .index_document(&task1_uri, task1_content)
        .expect("Failed to index task1");
    provider
        .index()
        .index_document(&task2_uri, task2_content)
        .expect("Failed to index task2");

    // Pipeline referencing task-two
    let pipeline_content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: first
      taskRef:
        name: task-two"#;

    let pipeline_uri = Url::parse("file:///tmp/pipelines/main.yaml").unwrap();
    let yaml_doc = parser::parse_yaml(&pipeline_uri.to_string(), pipeline_content)
        .expect("Failed to parse pipeline");

    let position = Position {
        line: 8,
        character: 14,
    };

    let definition = provider.provide_definition(&yaml_doc, position);

    assert!(
        definition.is_some(),
        "Should find task-two definition"
    );
}
