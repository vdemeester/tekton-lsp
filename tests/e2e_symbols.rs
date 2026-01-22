//! End-to-end tests for document symbols functionality.
//!
//! These tests verify that the symbols provider returns a proper
//! hierarchical outline of Tekton resources.

use tekton_lsp::{parser, symbols::SymbolsProvider};
use tower_lsp::lsp_types::SymbolKind;

#[test]
fn test_pipeline_symbols_structure() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  params:
    - name: version
    - name: environment
  tasks:
    - name: build
      taskRef:
        name: build-task
    - name: test
      taskRef:
        name: test-task
  finally:
    - name: cleanup
      taskRef:
        name: cleanup-task"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content).expect("Failed to parse YAML");
    let provider = SymbolsProvider::new();

    let symbols = provider.provide_symbols(&yaml_doc);

    // Should have one root symbol
    assert_eq!(symbols.len(), 1, "Should have one root symbol");

    let root = &symbols[0];
    assert!(
        root.name.contains("Pipeline"),
        "Root should be Pipeline, got: {}",
        root.name
    );
    assert!(
        root.name.contains("main-pipeline"),
        "Root should contain name"
    );
    assert_eq!(root.kind, SymbolKind::CLASS);

    // Check children exist
    let children = root.children.as_ref().expect("Root should have children");
    assert!(children.len() >= 2, "Should have metadata and spec");

    // Find spec
    let spec = children
        .iter()
        .find(|c| c.name == "spec")
        .expect("Should have spec");
    let spec_children = spec.children.as_ref().expect("Spec should have children");

    // Check for params, tasks, finally
    let child_names: Vec<&str> = spec_children.iter().map(|c| c.name.as_str()).collect();
    assert!(
        child_names.iter().any(|n| n.contains("params")),
        "Should have params"
    );
    assert!(
        child_names.iter().any(|n| n.contains("tasks")),
        "Should have tasks"
    );
    assert!(
        child_names.iter().any(|n| n.contains("finally")),
        "Should have finally"
    );
}

#[test]
fn test_task_symbols_with_steps() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  params:
    - name: source-url
  steps:
    - name: clone
      image: git
    - name: build
      image: golang
    - name: test
      image: golang"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content).expect("Failed to parse YAML");
    let provider = SymbolsProvider::new();

    let symbols = provider.provide_symbols(&yaml_doc);

    let root = &symbols[0];
    assert!(root.name.contains("Task"), "Root should be Task");
    assert!(root.name.contains("build-task"), "Root should contain name");

    let children = root.children.as_ref().unwrap();
    let spec = children.iter().find(|c| c.name == "spec").unwrap();
    let spec_children = spec.children.as_ref().unwrap();

    // Find steps
    let steps = spec_children
        .iter()
        .find(|c| c.name.contains("steps"))
        .expect("Should have steps");
    assert!(steps.name.contains("(3)"), "Steps should show count of 3");

    // Check step children
    let step_children = steps.children.as_ref().expect("Steps should have children");
    assert_eq!(step_children.len(), 3);

    let step_names: Vec<&str> = step_children.iter().map(|c| c.name.as_str()).collect();
    assert!(step_names.contains(&"clone"));
    assert!(step_names.contains(&"build"));
    assert!(step_names.contains(&"test"));

    // Steps should have FUNCTION kind
    assert_eq!(step_children[0].kind, SymbolKind::FUNCTION);
}

#[test]
fn test_pipeline_run_symbols() {
    let content = r#"apiVersion: tekton.dev/v1
kind: PipelineRun
metadata:
  name: my-pipeline-run
spec:
  pipelineRef:
    name: main-pipeline
  params:
    - name: version
      value: "1.0.0""#;

    let yaml_doc = parser::parse_yaml("test.yaml", content).expect("Failed to parse YAML");
    let provider = SymbolsProvider::new();

    let symbols = provider.provide_symbols(&yaml_doc);

    let root = &symbols[0];
    assert!(
        root.name.contains("PipelineRun"),
        "Root should be PipelineRun"
    );
    assert_eq!(root.kind, SymbolKind::OBJECT);
}

#[test]
fn test_task_run_symbols() {
    let content = r#"apiVersion: tekton.dev/v1
kind: TaskRun
metadata:
  name: my-task-run
spec:
  taskRef:
    name: build-task
  params:
    - name: source
      value: "https://github.com/example/repo""#;

    let yaml_doc = parser::parse_yaml("test.yaml", content).expect("Failed to parse YAML");
    let provider = SymbolsProvider::new();

    let symbols = provider.provide_symbols(&yaml_doc);

    let root = &symbols[0];
    assert!(root.name.contains("TaskRun"), "Root should be TaskRun");
    assert_eq!(root.kind, SymbolKind::OBJECT);
}

#[test]
fn test_empty_spec_symbols() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: empty-pipeline
spec: {}"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content).expect("Failed to parse YAML");
    let provider = SymbolsProvider::new();

    let symbols = provider.provide_symbols(&yaml_doc);

    assert_eq!(symbols.len(), 1);
    assert!(symbols[0].name.contains("empty-pipeline"));
}

#[test]
fn test_symbols_with_workspaces() {
    let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: pipeline-with-workspaces
spec:
  workspaces:
    - name: source
    - name: output
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

    let yaml_doc = parser::parse_yaml("test.yaml", content).expect("Failed to parse YAML");
    let provider = SymbolsProvider::new();

    let symbols = provider.provide_symbols(&yaml_doc);

    let root = &symbols[0];
    let children = root.children.as_ref().unwrap();
    let spec = children.iter().find(|c| c.name == "spec").unwrap();
    let spec_children = spec.children.as_ref().unwrap();

    // Find workspaces
    let workspaces = spec_children
        .iter()
        .find(|c| c.name.contains("workspaces"))
        .expect("Should have workspaces");
    assert!(
        workspaces.name.contains("(2)"),
        "Workspaces should show count of 2"
    );
}
