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

        diagnostics
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
