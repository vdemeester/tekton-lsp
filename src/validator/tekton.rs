// Tekton resource validator

use crate::parser::YamlDocument;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

/// Validator for Tekton resources
pub struct TektonValidator;

impl TektonValidator {
    /// Create a new Tekton validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a parsed YAML document and return diagnostics
    pub fn validate(&self, doc: &YamlDocument) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Validate metadata.name exists (required for all Tekton resources)
        if let Some(metadata_node) = doc.root.get("metadata") {
            if metadata_node.get("name").is_none() {
                // Missing metadata.name
                diagnostics.push(Diagnostic {
                    range: metadata_node.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("tekton-lsp".to_string()),
                    message: "Required field 'metadata.name' is missing".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        // Validate Pipeline-specific rules
        if doc.kind.as_deref() == Some("Pipeline") {
            self.validate_pipeline(doc, &mut diagnostics);
        }

        diagnostics
    }

    /// Validate Pipeline-specific rules
    fn validate_pipeline(&self, doc: &YamlDocument, diagnostics: &mut Vec<Diagnostic>) {
        if let Some(spec_node) = doc.root.get("spec") {
            if let Some(tasks_node) = spec_node.get("tasks") {
                use crate::parser::NodeValue;

                // Check if tasks has the correct type (should be sequence/array)
                match &tasks_node.value {
                    NodeValue::Sequence(ref tasks) => {
                        // It's a sequence - check if it's empty
                        if tasks.is_empty() {
                            diagnostics.push(Diagnostic {
                                range: tasks_node.range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: None,
                                code_description: None,
                                source: Some("tekton-lsp".to_string()),
                                message: "Pipeline must have at least one task".to_string(),
                                related_information: None,
                                tags: None,
                                data: None,
                            });
                        }
                    }
                    _ => {
                        // Wrong type - should be an array/sequence
                        diagnostics.push(Diagnostic {
                            range: tasks_node.range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            code_description: None,
                            source: Some("tekton-lsp".to_string()),
                            message: "Field 'tasks' must be an array".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    }
                }
            }
        }
    }
}

impl Default for TektonValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn test_valid_pipeline_no_errors() {
        let yaml = r#"
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline
spec:
  tasks: []
"#;

        let doc = parse_yaml("test.yaml", yaml).unwrap();
        let validator = TektonValidator::new();
        let diagnostics = validator.validate(&doc);

        assert_eq!(diagnostics.len(), 0, "Valid pipeline should have no errors");
    }

    #[test]
    fn test_missing_metadata_name() {
        let yaml = r#"
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  namespace: default
spec:
  tasks: []
"#;

        let doc = parse_yaml("test.yaml", yaml).unwrap();
        let validator = TektonValidator::new();
        let diagnostics = validator.validate(&doc);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert!(diagnostics[0].message.contains("metadata.name"));
    }
}
