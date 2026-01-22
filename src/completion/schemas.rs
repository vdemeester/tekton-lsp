//! Tekton resource schemas for completion.
//!
//! Defines the fields available for different Tekton resource types.

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub description: String,
    pub field_type: FieldType,
    /// Whether this field is required (for future validation/snippets)
    #[allow(dead_code)]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Array,
    Object,
    /// Boolean type for future use
    #[allow(dead_code)]
    Boolean,
}

#[derive(Debug, Clone)]
pub struct TektonSchemas {
    metadata_fields: Vec<FieldSchema>,
    pipeline_spec_fields: Vec<FieldSchema>,
    pipeline_task_fields: Vec<FieldSchema>,
    task_spec_fields: Vec<FieldSchema>,
    step_fields: Vec<FieldSchema>,
}

impl TektonSchemas {
    pub fn new() -> Self {
        Self {
            metadata_fields: vec![
                FieldSchema {
                    name: "name".to_string(),
                    description: "Resource name (required)".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
                FieldSchema {
                    name: "namespace".to_string(),
                    description: "Resource namespace".to_string(),
                    field_type: FieldType::String,
                    required: false,
                },
                FieldSchema {
                    name: "labels".to_string(),
                    description: "Resource labels".to_string(),
                    field_type: FieldType::Object,
                    required: false,
                },
                FieldSchema {
                    name: "annotations".to_string(),
                    description: "Resource annotations".to_string(),
                    field_type: FieldType::Object,
                    required: false,
                },
            ],
            pipeline_spec_fields: vec![
                FieldSchema {
                    name: "tasks".to_string(),
                    description: "Pipeline tasks to execute".to_string(),
                    field_type: FieldType::Array,
                    required: true,
                },
                FieldSchema {
                    name: "finally".to_string(),
                    description: "Tasks to run after all other tasks".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "params".to_string(),
                    description: "Pipeline parameters".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "workspaces".to_string(),
                    description: "Pipeline workspaces".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "results".to_string(),
                    description: "Pipeline results".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
            ],
            pipeline_task_fields: vec![
                FieldSchema {
                    name: "name".to_string(),
                    description: "Task name (required)".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
                FieldSchema {
                    name: "taskRef".to_string(),
                    description: "Reference to an existing Task".to_string(),
                    field_type: FieldType::Object,
                    required: false,
                },
                FieldSchema {
                    name: "taskSpec".to_string(),
                    description: "Inline Task specification".to_string(),
                    field_type: FieldType::Object,
                    required: false,
                },
                FieldSchema {
                    name: "params".to_string(),
                    description: "Task parameters".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "workspaces".to_string(),
                    description: "Workspace bindings".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "runAfter".to_string(),
                    description: "Tasks that must complete before this task".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
            ],
            task_spec_fields: vec![
                FieldSchema {
                    name: "steps".to_string(),
                    description: "Task steps to execute".to_string(),
                    field_type: FieldType::Array,
                    required: true,
                },
                FieldSchema {
                    name: "params".to_string(),
                    description: "Task parameters".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "workspaces".to_string(),
                    description: "Task workspaces".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "results".to_string(),
                    description: "Task results".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "volumes".to_string(),
                    description: "Kubernetes volumes".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
            ],
            step_fields: vec![
                FieldSchema {
                    name: "name".to_string(),
                    description: "Step name (required)".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
                FieldSchema {
                    name: "image".to_string(),
                    description: "Container image (required)".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
                FieldSchema {
                    name: "script".to_string(),
                    description: "Script to execute".to_string(),
                    field_type: FieldType::String,
                    required: false,
                },
                FieldSchema {
                    name: "command".to_string(),
                    description: "Container entrypoint".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "args".to_string(),
                    description: "Container arguments".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "env".to_string(),
                    description: "Environment variables".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                },
                FieldSchema {
                    name: "workingDir".to_string(),
                    description: "Working directory".to_string(),
                    field_type: FieldType::String,
                    required: false,
                },
            ],
        }
    }

    pub fn get_metadata_fields(&self) -> &[FieldSchema] {
        &self.metadata_fields
    }

    pub fn get_pipeline_spec_fields(&self) -> &[FieldSchema] {
        &self.pipeline_spec_fields
    }

    pub fn get_pipeline_task_fields(&self) -> &[FieldSchema] {
        &self.pipeline_task_fields
    }

    pub fn get_task_spec_fields(&self) -> &[FieldSchema] {
        &self.task_spec_fields
    }

    pub fn get_step_fields(&self) -> &[FieldSchema] {
        &self.step_fields
    }
}

impl Default for TektonSchemas {
    fn default() -> Self {
        Self::new()
    }
}
