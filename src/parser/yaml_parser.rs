use super::ast::{Node, NodeValue, YamlDocument};
use serde_yaml::Value;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range};

/// Parse YAML content into a document with position tracking
pub fn parse_yaml(filename: &str, content: &str) -> Result<YamlDocument, serde_yaml::Error> {
    let value: Value = serde_yaml::from_str(content)?;

    // Build AST with position estimation
    // Note: serde_yaml doesn't provide position info, so we estimate based on content
    let root = build_node_from_value(&value, 0, content);

    Ok(YamlDocument::new(filename.to_string(), root))
}

/// Build a node from a serde_yaml Value
///
/// Note: This is a simplified implementation. For production, we'd want to use
/// a YAML parser that provides accurate position information (like yaml-rust2)
fn build_node_from_value(value: &Value, line: u32, _content: &str) -> Node {
    let range = Range {
        start: Position {
            line,
            character: 0,
        },
        end: Position {
            line,
            character: 100, // Placeholder - accurate tracking would need proper parser
        },
    };

    let node_value = match value {
        Value::Null => NodeValue::Null,

        Value::Bool(b) => NodeValue::Scalar(b.to_string()),

        Value::Number(n) => NodeValue::Scalar(n.to_string()),

        Value::String(s) => NodeValue::Scalar(s.clone()),

        Value::Sequence(seq) => {
            let items: Vec<Node> = seq
                .iter()
                .enumerate()
                .map(|(i, v)| build_node_from_value(v, line + i as u32 + 1, _content))
                .collect();
            NodeValue::Sequence(items)
        }

        Value::Mapping(map) => {
            let mut nodes = HashMap::new();
            let mut current_line = line;

            for (key, val) in map {
                if let Value::String(key_str) = key {
                    current_line += 1;
                    let child = build_node_from_value(val, current_line, _content);
                    nodes.insert(key_str.clone(), child);
                }
            }

            NodeValue::Mapping(nodes)
        }

        Value::Tagged(tagged) => {
            // Handle tagged values by recursing on the inner value
            build_node_from_value(&tagged.value, line, _content).value
        }
    };

    Node::new(None, node_value, range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline
"#;

        let doc = parse_yaml("test.yaml", yaml).unwrap();

        assert_eq!(doc.api_version, Some("tekton.dev/v1".to_string()));
        assert_eq!(doc.kind, Some("Pipeline".to_string()));

        let metadata = doc.root.get("metadata").unwrap();
        assert!(metadata.is_mapping());

        let name = metadata.get("name").unwrap();
        assert_eq!(name.as_scalar(), Some("test-pipeline"));
    }

    #[test]
    fn test_parse_yaml_with_sequence() {
        let yaml = r#"
kind: Pipeline
spec:
  tasks:
    - name: task1
      taskRef:
        name: build
    - name: task2
      taskRef:
        name: test
"#;

        let doc = parse_yaml("test.yaml", yaml).unwrap();

        let spec = doc.root.get("spec").unwrap();
        let tasks = spec.get("tasks").unwrap();

        assert!(tasks.is_sequence());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "invalid: yaml: content:";
        let result = parse_yaml("test.yaml", yaml);
        assert!(result.is_err());
    }
}
