//! Document symbols provider implementation.

use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind};

use crate::parser::{Node, NodeValue, YamlDocument};

/// Provides document symbols (outline) for Tekton YAML files.
#[derive(Debug, Clone, Default)]
pub struct SymbolsProvider;

impl SymbolsProvider {
    /// Create a new symbols provider.
    pub fn new() -> Self {
        Self
    }

    /// Provide document symbols for a YAML document.
    #[allow(deprecated)] // DocumentSymbol::deprecated field is deprecated but required
    pub fn provide_symbols(&self, yaml_doc: &YamlDocument) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        // Get the resource kind and name for the root symbol
        let kind = yaml_doc.kind.as_deref().unwrap_or("Unknown");
        let name = self.get_resource_name(&yaml_doc.root).unwrap_or_else(|| kind.to_string());

        // Create the root symbol for the resource
        let mut root_symbol = DocumentSymbol {
            name: format!("{}: {}", kind, name),
            detail: yaml_doc.api_version.clone(),
            kind: self.resource_kind_to_symbol_kind(kind),
            tags: None,
            deprecated: None,
            range: yaml_doc.root.range,
            selection_range: yaml_doc.root.range,
            children: Some(Vec::new()),
        };

        // Add child symbols based on resource type
        if let Some(children) = &mut root_symbol.children {
            self.add_resource_children(children, &yaml_doc.root, kind);
        }

        symbols.push(root_symbol);
        symbols
    }

    /// Get the resource name from metadata.name.
    fn get_resource_name(&self, root: &Node) -> Option<String> {
        root.get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_scalar())
            .map(String::from)
    }

    /// Map Tekton resource kind to LSP SymbolKind.
    fn resource_kind_to_symbol_kind(&self, kind: &str) -> SymbolKind {
        match kind {
            "Pipeline" | "Task" | "ClusterTask" => SymbolKind::CLASS,
            "PipelineRun" | "TaskRun" => SymbolKind::OBJECT,
            "TriggerTemplate" | "TriggerBinding" | "EventListener" => SymbolKind::INTERFACE,
            _ => SymbolKind::FILE,
        }
    }

    /// Add child symbols based on resource type.
    #[allow(deprecated)]
    fn add_resource_children(&self, children: &mut Vec<DocumentSymbol>, root: &Node, kind: &str) {
        // Add metadata section
        if let Some(metadata) = root.get("metadata") {
            children.push(self.create_section_symbol("metadata", metadata, SymbolKind::NAMESPACE));
        }

        // Add spec section with nested symbols
        if let Some(spec) = root.get("spec") {
            let mut spec_symbol = self.create_section_symbol("spec", spec, SymbolKind::MODULE);

            if let Some(spec_children) = &mut spec_symbol.children {
                match kind {
                    "Pipeline" => self.add_pipeline_spec_children(spec_children, spec),
                    "Task" | "ClusterTask" => self.add_task_spec_children(spec_children, spec),
                    "PipelineRun" => self.add_pipeline_run_spec_children(spec_children, spec),
                    "TaskRun" => self.add_task_run_spec_children(spec_children, spec),
                    _ => {}
                }
            }

            children.push(spec_symbol);
        }
    }

    /// Add Pipeline spec children.
    #[allow(deprecated)]
    fn add_pipeline_spec_children(&self, children: &mut Vec<DocumentSymbol>, spec: &Node) {
        // Add params
        if let Some(params) = spec.get("params") {
            children.push(self.create_array_symbol("params", params, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add workspaces
        if let Some(workspaces) = spec.get("workspaces") {
            children.push(self.create_array_symbol("workspaces", workspaces, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add tasks
        if let Some(tasks) = spec.get("tasks") {
            children.push(self.create_array_symbol("tasks", tasks, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add finally
        if let Some(finally) = spec.get("finally") {
            children.push(self.create_array_symbol("finally", finally, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add results
        if let Some(results) = spec.get("results") {
            children.push(self.create_array_symbol("results", results, |item| {
                self.get_name_from_node(item)
            }));
        }
    }

    /// Add Task spec children.
    #[allow(deprecated)]
    fn add_task_spec_children(&self, children: &mut Vec<DocumentSymbol>, spec: &Node) {
        // Add params
        if let Some(params) = spec.get("params") {
            children.push(self.create_array_symbol("params", params, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add workspaces
        if let Some(workspaces) = spec.get("workspaces") {
            children.push(self.create_array_symbol("workspaces", workspaces, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add steps
        if let Some(steps) = spec.get("steps") {
            children.push(self.create_array_symbol_with_kind("steps", steps, SymbolKind::FUNCTION, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add sidecars
        if let Some(sidecars) = spec.get("sidecars") {
            children.push(self.create_array_symbol_with_kind("sidecars", sidecars, SymbolKind::FUNCTION, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add results
        if let Some(results) = spec.get("results") {
            children.push(self.create_array_symbol("results", results, |item| {
                self.get_name_from_node(item)
            }));
        }
    }

    /// Add PipelineRun spec children.
    #[allow(deprecated)]
    fn add_pipeline_run_spec_children(&self, children: &mut Vec<DocumentSymbol>, spec: &Node) {
        // Add pipelineRef
        if let Some(pipeline_ref) = spec.get("pipelineRef") {
            children.push(self.create_section_symbol("pipelineRef", pipeline_ref, SymbolKind::PROPERTY));
        }

        // Add params
        if let Some(params) = spec.get("params") {
            children.push(self.create_array_symbol("params", params, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add workspaces
        if let Some(workspaces) = spec.get("workspaces") {
            children.push(self.create_array_symbol("workspaces", workspaces, |item| {
                self.get_name_from_node(item)
            }));
        }
    }

    /// Add TaskRun spec children.
    #[allow(deprecated)]
    fn add_task_run_spec_children(&self, children: &mut Vec<DocumentSymbol>, spec: &Node) {
        // Add taskRef
        if let Some(task_ref) = spec.get("taskRef") {
            children.push(self.create_section_symbol("taskRef", task_ref, SymbolKind::PROPERTY));
        }

        // Add params
        if let Some(params) = spec.get("params") {
            children.push(self.create_array_symbol("params", params, |item| {
                self.get_name_from_node(item)
            }));
        }

        // Add workspaces
        if let Some(workspaces) = spec.get("workspaces") {
            children.push(self.create_array_symbol("workspaces", workspaces, |item| {
                self.get_name_from_node(item)
            }));
        }
    }

    /// Create a symbol for a section (like metadata, spec).
    #[allow(deprecated)]
    fn create_section_symbol(&self, name: &str, node: &Node, kind: SymbolKind) -> DocumentSymbol {
        DocumentSymbol {
            name: name.to_string(),
            detail: None,
            kind,
            tags: None,
            deprecated: None,
            range: node.range,
            selection_range: node.range,
            children: Some(Vec::new()),
        }
    }

    /// Create a symbol for an array with item names.
    #[allow(deprecated)]
    fn create_array_symbol<F>(&self, name: &str, node: &Node, get_item_name: F) -> DocumentSymbol
    where
        F: Fn(&Node) -> Option<String>,
    {
        self.create_array_symbol_with_kind(name, node, SymbolKind::VARIABLE, get_item_name)
    }

    /// Create a symbol for an array with item names and custom kind.
    #[allow(deprecated)]
    fn create_array_symbol_with_kind<F>(
        &self,
        name: &str,
        node: &Node,
        item_kind: SymbolKind,
        get_item_name: F,
    ) -> DocumentSymbol
    where
        F: Fn(&Node) -> Option<String>,
    {
        let mut children = Vec::new();

        if let NodeValue::Sequence(items) = &node.value {
            for item in items {
                let item_name = get_item_name(item).unwrap_or_else(|| "unnamed".to_string());
                children.push(DocumentSymbol {
                    name: item_name,
                    detail: None,
                    kind: item_kind,
                    tags: None,
                    deprecated: None,
                    range: item.range,
                    selection_range: item.range,
                    children: None,
                });
            }
        }

        let count = children.len();
        DocumentSymbol {
            name: format!("{} ({})", name, count),
            detail: None,
            kind: SymbolKind::ARRAY,
            tags: None,
            deprecated: None,
            range: node.range,
            selection_range: node.range,
            children: if children.is_empty() { None } else { Some(children) },
        }
    }

    /// Get the name field from a node.
    fn get_name_from_node(&self, node: &Node) -> Option<String> {
        node.get("name")
            .and_then(|n| n.as_scalar())
            .map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_pipeline_symbols() {
        let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  params:
    - name: version
  tasks:
    - name: build
      taskRef:
        name: build-task
    - name: test
      taskRef:
        name: test-task"#;

        let yaml_doc = parser::parse_yaml("test.yaml", content).unwrap();
        let provider = SymbolsProvider::new();

        let symbols = provider.provide_symbols(&yaml_doc);

        assert_eq!(symbols.len(), 1, "Should have one root symbol");

        let root = &symbols[0];
        assert!(root.name.contains("Pipeline"), "Root should be Pipeline");
        assert!(root.name.contains("main-pipeline"), "Root should contain name");
        assert_eq!(root.kind, SymbolKind::CLASS);

        // Check children
        let children = root.children.as_ref().unwrap();
        assert!(children.len() >= 2, "Should have metadata and spec");

        // Find spec
        let spec = children.iter().find(|c| c.name == "spec").unwrap();
        let spec_children = spec.children.as_ref().unwrap();

        // Check tasks array
        let tasks = spec_children.iter().find(|c| c.name.starts_with("tasks")).unwrap();
        assert!(tasks.name.contains("(2)"), "Tasks should show count of 2");

        let task_children = tasks.children.as_ref().unwrap();
        assert_eq!(task_children.len(), 2);
        assert_eq!(task_children[0].name, "build");
        assert_eq!(task_children[1].name, "test");
    }

    #[test]
    fn test_task_symbols() {
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
      image: golang"#;

        let yaml_doc = parser::parse_yaml("test.yaml", content).unwrap();
        let provider = SymbolsProvider::new();

        let symbols = provider.provide_symbols(&yaml_doc);

        let root = &symbols[0];
        assert!(root.name.contains("Task"), "Root should be Task");
        assert!(root.name.contains("build-task"), "Root should contain name");

        let children = root.children.as_ref().unwrap();
        let spec = children.iter().find(|c| c.name == "spec").unwrap();
        let spec_children = spec.children.as_ref().unwrap();

        // Check steps array
        let steps = spec_children.iter().find(|c| c.name.starts_with("steps")).unwrap();
        assert!(steps.name.contains("(2)"), "Steps should show count of 2");

        let step_children = steps.children.as_ref().unwrap();
        assert_eq!(step_children.len(), 2);
        assert_eq!(step_children[0].name, "clone");
        assert_eq!(step_children[1].name, "build");
        assert_eq!(step_children[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_empty_spec_symbols() {
        let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: empty-pipeline
spec: {}"#;

        let yaml_doc = parser::parse_yaml("test.yaml", content).unwrap();
        let provider = SymbolsProvider::new();

        let symbols = provider.provide_symbols(&yaml_doc);

        assert_eq!(symbols.len(), 1);
        assert!(symbols[0].name.contains("empty-pipeline"));
    }
}
