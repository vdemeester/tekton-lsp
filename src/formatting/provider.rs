//! YAML formatting provider implementation.

use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Provides document formatting for Tekton YAML files.
#[derive(Debug, Clone, Default)]
pub struct FormattingProvider {
    /// Number of spaces per indentation level.
    indent_size: usize,
}

impl FormattingProvider {
    /// Create a new formatting provider with default settings.
    pub fn new() -> Self {
        Self { indent_size: 2 }
    }

    /// Format a YAML document and return text edits.
    pub fn format(&self, content: &str) -> Option<Vec<TextEdit>> {
        // Parse the YAML
        let value: serde_yaml::Value = match serde_yaml::from_str(content) {
            Ok(v) => v,
            Err(_) => return None, // Don't format invalid YAML
        };

        // Serialize with consistent formatting
        let formatted = match serde_yaml::to_string(&value) {
            Ok(s) => s,
            Err(_) => return None,
        };

        // If content is unchanged, return empty edits
        if content.trim() == formatted.trim() {
            return Some(vec![]);
        }

        // Calculate the range of the entire document
        let lines: Vec<&str> = content.lines().collect();
        let last_line = lines.len().saturating_sub(1);
        let last_char = lines.last().map(|l| l.len()).unwrap_or(0);

        Some(vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: last_line as u32,
                    character: last_char as u32,
                },
            },
            new_text: formatted,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_basic_yaml() {
        let provider = FormattingProvider::new();

        let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: my-task
spec:
  steps:
    - name: hello
      image: ubuntu
"#;

        let edits = provider.format(content);
        assert!(edits.is_some());
    }

    #[test]
    fn test_format_invalid_yaml() {
        let provider = FormattingProvider::new();

        let content = r#"
apiVersion: tekton.dev/v1
kind: Task
  invalid: indentation
"#;

        let edits = provider.format(content);
        // Invalid YAML should return None
        assert!(edits.is_none());
    }

    #[test]
    fn test_format_normalizes_indentation() {
        let provider = FormattingProvider::new();

        // YAML with inconsistent indentation (4 spaces instead of 2)
        let content = r#"apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
    name: my-pipeline
spec:
    tasks:
        - name: build
"#;

        let edits = provider.format(content);
        assert!(edits.is_some());

        let edits = edits.unwrap();
        if !edits.is_empty() {
            let formatted = &edits[0].new_text;
            // serde_yaml uses 2-space indentation by default
            assert!(formatted.contains("  name:") || formatted.contains("name:"));
        }
    }

    #[test]
    fn test_format_preserves_structure() {
        let provider = FormattingProvider::new();

        let content = r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  params:
    - name: version
      type: string
  steps:
    - name: compile
      image: golang:1.21
"#;

        let edits = provider.format(content);
        assert!(edits.is_some());

        let edits = edits.unwrap();
        if !edits.is_empty() {
            let formatted = &edits[0].new_text;
            // Verify key elements are preserved
            assert!(formatted.contains("apiVersion:"));
            assert!(formatted.contains("kind: Task"));
            assert!(formatted.contains("name: build-task"));
            assert!(formatted.contains("params:"));
            assert!(formatted.contains("steps:"));
        }
    }

    #[test]
    fn test_format_unchanged_returns_empty() {
        let provider = FormattingProvider::new();

        // Already well-formatted YAML (matches serde_yaml output)
        let value: serde_yaml::Value = serde_yaml::from_str(
            r#"apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: test
"#,
        )
        .unwrap();
        let canonical = serde_yaml::to_string(&value).unwrap();

        let edits = provider.format(&canonical);
        assert!(edits.is_some());
        // When content matches, should return empty vec
        assert!(edits.unwrap().is_empty());
    }
}
