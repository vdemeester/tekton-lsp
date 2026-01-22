//! Workspace index for Tekton resources.
//!
//! Maintains a workspace-wide index of all Tekton resources
//! for cross-file navigation and reference finding.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_lsp::lsp_types::{Location, Url};

use crate::parser::{self, NodeValue, YamlDocument};

/// A Tekton resource definition in the workspace.
#[derive(Debug, Clone)]
pub struct ResourceDefinition {
    /// URI of the document containing this resource
    pub uri: Url,
    /// Resource kind (Pipeline, Task, etc.)
    pub kind: String,
    /// Resource name from metadata.name
    pub name: String,
    /// API version
    pub api_version: Option<String>,
    /// Location of the resource name in the document
    pub location: Location,
}

/// A reference to a Tekton resource.
#[derive(Debug, Clone)]
pub struct ResourceReference {
    /// URI of the document containing this reference
    pub uri: Url,
    /// Kind of the referenced resource (Task, Pipeline, etc.)
    pub ref_kind: String,
    /// Name of the referenced resource
    pub ref_name: String,
    /// Location of the reference in the document
    pub location: Location,
}

/// Thread-safe workspace index for Tekton resources.
#[derive(Debug, Clone)]
pub struct WorkspaceIndex {
    /// Resources indexed by "Kind/Name"
    resources: Arc<RwLock<HashMap<String, ResourceDefinition>>>,
    /// References indexed by "Kind/Name" (what they point to)
    references: Arc<RwLock<HashMap<String, Vec<ResourceReference>>>>,
    /// Track which resources/references came from which document
    document_resources: Arc<RwLock<HashMap<Url, Vec<String>>>>,
}

impl WorkspaceIndex {
    /// Create a new empty workspace index.
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            references: Arc::new(RwLock::new(HashMap::new())),
            document_resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Index a document and extract resources and references.
    pub fn index_document(&self, uri: &Url, content: &str) -> Result<(), String> {
        // First remove any existing entries from this document
        self.remove_document(uri);

        // Parse the document
        let yaml_doc = parser::parse_yaml(&uri.to_string(), content)?;

        // Index the resource definition
        self.index_resource_definition(uri, &yaml_doc);

        // Index references (e.g., taskRef in Pipelines)
        self.index_references(uri, &yaml_doc);

        Ok(())
    }

    /// Index a resource definition from a document.
    fn index_resource_definition(&self, uri: &Url, yaml_doc: &YamlDocument) {
        let kind = match &yaml_doc.kind {
            Some(k) => k.clone(),
            None => return,
        };

        // Get metadata.name
        let metadata = match yaml_doc.root.get("metadata") {
            Some(m) => m,
            None => return,
        };

        let name_node = match metadata.get("name") {
            Some(n) => n,
            None => return,
        };

        let name = match name_node.as_scalar() {
            Some(s) => s.to_string(),
            None => return,
        };

        let key = format!("{}/{}", kind, name);

        let resource = ResourceDefinition {
            uri: uri.clone(),
            kind: kind.clone(),
            name: name.clone(),
            api_version: yaml_doc.api_version.clone(),
            location: Location {
                uri: uri.clone(),
                range: name_node.range,
            },
        };

        // Add to resources
        {
            let mut resources = self.resources.write().unwrap();
            resources.insert(key.clone(), resource);
        }

        // Track which resources came from this document
        {
            let mut doc_resources = self.document_resources.write().unwrap();
            doc_resources
                .entry(uri.clone())
                .or_insert_with(Vec::new)
                .push(key);
        }
    }

    /// Index references from a document (e.g., taskRef in Pipelines).
    fn index_references(&self, uri: &Url, yaml_doc: &YamlDocument) {
        let kind = match &yaml_doc.kind {
            Some(k) => k.as_str(),
            None => return,
        };

        match kind {
            "Pipeline" => self.index_pipeline_references(uri, yaml_doc),
            "PipelineRun" => self.index_pipeline_run_references(uri, yaml_doc),
            _ => {}
        }
    }

    /// Index taskRef references in a Pipeline.
    fn index_pipeline_references(&self, uri: &Url, yaml_doc: &YamlDocument) {
        let spec = match yaml_doc.root.get("spec") {
            Some(s) => s,
            None => return,
        };

        // Index tasks array
        if let Some(tasks) = spec.get("tasks") {
            self.index_pipeline_tasks(uri, tasks);
        }

        // Index finally array
        if let Some(finally) = spec.get("finally") {
            self.index_pipeline_tasks(uri, finally);
        }
    }

    /// Index taskRef references in a tasks/finally array.
    fn index_pipeline_tasks(&self, uri: &Url, tasks_node: &crate::parser::Node) {
        let tasks = match &tasks_node.value {
            NodeValue::Sequence(items) => items,
            _ => return,
        };

        for task in tasks {
            // Check for taskRef
            if let Some(task_ref) = task.get("taskRef") {
                self.index_task_ref(uri, task_ref, "Task");
            }
        }
    }

    /// Index a taskRef reference.
    fn index_task_ref(&self, uri: &Url, task_ref: &crate::parser::Node, default_kind: &str) {
        // Get the name
        let name_node = match task_ref.get("name") {
            Some(n) => n,
            None => return,
        };

        let name = match name_node.as_scalar() {
            Some(s) => s.to_string(),
            None => return,
        };

        // Get kind (default to Task)
        let kind = task_ref
            .get("kind")
            .and_then(|k| k.as_scalar())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default_kind.to_string());

        let key = format!("{}/{}", kind, name);

        let reference = ResourceReference {
            uri: uri.clone(),
            ref_kind: kind,
            ref_name: name,
            location: Location {
                uri: uri.clone(),
                range: name_node.range,
            },
        };

        // Add to references
        {
            let mut references = self.references.write().unwrap();
            references
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(reference);
        }

        // Track which references came from this document
        {
            let mut doc_resources = self.document_resources.write().unwrap();
            doc_resources
                .entry(uri.clone())
                .or_insert_with(Vec::new)
                .push(format!("ref:{}", key));
        }
    }

    /// Index pipelineRef references in a PipelineRun.
    fn index_pipeline_run_references(&self, uri: &Url, yaml_doc: &YamlDocument) {
        let spec = match yaml_doc.root.get("spec") {
            Some(s) => s,
            None => return,
        };

        if let Some(pipeline_ref) = spec.get("pipelineRef") {
            if let Some(name_node) = pipeline_ref.get("name") {
                if let Some(name) = name_node.as_scalar() {
                    let key = format!("Pipeline/{}", name);

                    let reference = ResourceReference {
                        uri: uri.clone(),
                        ref_kind: "Pipeline".to_string(),
                        ref_name: name.to_string(),
                        location: Location {
                            uri: uri.clone(),
                            range: name_node.range,
                        },
                    };

                    {
                        let mut references = self.references.write().unwrap();
                        references
                            .entry(key.clone())
                            .or_insert_with(Vec::new)
                            .push(reference);
                    }

                    {
                        let mut doc_resources = self.document_resources.write().unwrap();
                        doc_resources
                            .entry(uri.clone())
                            .or_insert_with(Vec::new)
                            .push(format!("ref:{}", key));
                    }
                }
            }
        }
    }

    /// Find a resource definition by kind and name.
    pub fn find_resource(&self, kind: &str, name: &str) -> Option<ResourceDefinition> {
        let key = format!("{}/{}", kind, name);
        let resources = self.resources.read().unwrap();
        resources.get(&key).cloned()
    }

    /// Find all references to a resource.
    pub fn find_references(&self, kind: &str, name: &str) -> Vec<ResourceReference> {
        let key = format!("{}/{}", kind, name);
        let references = self.references.read().unwrap();
        references.get(&key).cloned().unwrap_or_default()
    }

    /// Remove a document from the index.
    pub fn remove_document(&self, uri: &Url) {
        let keys_to_remove: Vec<String>;

        // Get keys associated with this document
        {
            let doc_resources = self.document_resources.read().unwrap();
            keys_to_remove = doc_resources.get(uri).cloned().unwrap_or_default();
        }

        // Remove resources
        {
            let mut resources = self.resources.write().unwrap();
            for key in &keys_to_remove {
                if !key.starts_with("ref:") {
                    resources.remove(key);
                }
            }
        }

        // Remove references
        {
            let mut references = self.references.write().unwrap();
            for key in &keys_to_remove {
                if let Some(ref_key) = key.strip_prefix("ref:") {
                    if let Some(refs) = references.get_mut(ref_key) {
                        refs.retain(|r| &r.uri != uri);
                        if refs.is_empty() {
                            references.remove(ref_key);
                        }
                    }
                }
            }
        }

        // Remove document tracking
        {
            let mut doc_resources = self.document_resources.write().unwrap();
            doc_resources.remove(uri);
        }
    }

    /// Get all indexed resources.
    #[allow(dead_code)]
    pub fn all_resources(&self) -> Vec<ResourceDefinition> {
        let resources = self.resources.read().unwrap();
        resources.values().cloned().collect()
    }
}

impl Default for WorkspaceIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_uri(path: &str) -> Url {
        Url::parse(&format!("file://{}", path)).unwrap()
    }

    #[test]
    fn test_index_task_resource() {
        let index = WorkspaceIndex::new();

        let uri = make_test_uri("/workspace/tasks/build.yaml");
        let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  steps:
    - image: golang"#;

        index.index_document(&uri, content).unwrap();

        // Find the task by name
        let resource = index.find_resource("Task", "build-task");
        assert!(resource.is_some(), "Task should be found in index");

        let resource = resource.unwrap();
        assert_eq!(resource.name, "build-task");
        assert_eq!(resource.kind, "Task");
        assert_eq!(resource.uri, uri);
    }

    #[test]
    fn test_index_pipeline_resource() {
        let index = WorkspaceIndex::new();

        let uri = make_test_uri("/workspace/pipelines/main.yaml");
        let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"#;

        index.index_document(&uri, content).unwrap();

        // Find the pipeline by name
        let resource = index.find_resource("Pipeline", "main-pipeline");
        assert!(resource.is_some(), "Pipeline should be found in index");

        let resource = resource.unwrap();
        assert_eq!(resource.name, "main-pipeline");
        assert_eq!(resource.kind, "Pipeline");
    }

    #[test]
    fn test_find_task_references() {
        let index = WorkspaceIndex::new();

        // Index a task
        let task_uri = make_test_uri("/workspace/tasks/build.yaml");
        let task_content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task"#;
        index.index_document(&task_uri, task_content).unwrap();

        // Index a pipeline that references the task
        let pipeline_uri = make_test_uri("/workspace/pipelines/main.yaml");
        let pipeline_content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task"#;
        index.index_document(&pipeline_uri, pipeline_content).unwrap();

        // Find references to build-task
        let refs = index.find_references("Task", "build-task");
        assert!(!refs.is_empty(), "Should find references to build-task");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].uri, pipeline_uri);
        assert_eq!(refs[0].ref_name, "build-task");
    }

    #[test]
    fn test_remove_document() {
        let index = WorkspaceIndex::new();

        let uri = make_test_uri("/workspace/tasks/build.yaml");
        let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task"#;

        index.index_document(&uri, content).unwrap();
        assert!(index.find_resource("Task", "build-task").is_some());

        index.remove_document(&uri);
        assert!(index.find_resource("Task", "build-task").is_none());
    }

    #[test]
    fn test_reindex_document() {
        let index = WorkspaceIndex::new();

        let uri = make_test_uri("/workspace/tasks/build.yaml");

        // Index initial version
        let content1 = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task-v1"#;
        index.index_document(&uri, content1).unwrap();
        assert!(index.find_resource("Task", "build-task-v1").is_some());

        // Re-index with new name
        let content2 = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task-v2"#;
        index.index_document(&uri, content2).unwrap();

        // Old name should be gone, new name should exist
        assert!(index.find_resource("Task", "build-task-v1").is_none());
        assert!(index.find_resource("Task", "build-task-v2").is_some());
    }
}
