# Task 4: Completion Implementation Plan

## Goal
Implement context-aware completion for Tekton YAML files using schema-based suggestions.

## TDD Approach

### Test-Driven Cycles

**Cycle 1: Basic metadata completion**
- Test: Suggest `name` when completing under `metadata:`
- Implementation: Simple field completion based on parent key

**Cycle 2: Pipeline spec.tasks completion**
- Test: Suggest `tasks` when completing under Pipeline `spec:`
- Implementation: Kind-aware field suggestions

**Cycle 3: PipelineTask fields completion**
- Test: Suggest `taskRef`, `taskSpec`, `params`, etc. under `spec.tasks[*]`
- Implementation: Context-aware completions based on path

**Cycle 4: Completion trigger points**
- Test: Trigger completion after `:`, at line start, after `-`
- Implementation: Position-aware completion triggering

**Cycle 5: Task spec.steps completion**
- Test: Suggest Task-specific fields (steps, params, workspaces)
- Implementation: Task kind completions

**Cycle 6: Step fields completion**
- Test: Suggest step fields (name, image, script, command, args)
- Implementation: Nested context completion

## Architecture

```
LSP Client (Editor)
    ↓
textDocument/completion request
    ↓
Backend::completion() handler
    ↓
CompletionProvider::provide_completions()
    ↓
1. Parse document (already cached)
2. Find node at cursor position
3. Determine context (Pipeline/Task, field path)
4. Get schema for context
5. Return completion items
```

## Data Structures

### CompletionProvider
```rust
pub struct CompletionProvider {
    schemas: TektonSchemas,
}

impl CompletionProvider {
    pub fn provide_completions(
        &self,
        yaml_doc: &YamlDocument,
        position: Position,
    ) -> Vec<CompletionItem> {
        // 1. Find node at position
        // 2. Determine context (parent path)
        // 3. Get valid fields for context
        // 4. Return completion items
    }
}
```

### TektonSchemas
```rust
pub struct TektonSchemas {
    pipeline_fields: HashMap<String, FieldSchema>,
    task_fields: HashMap<String, FieldSchema>,
    // ... more schemas
}

pub struct FieldSchema {
    name: String,
    description: String,
    field_type: FieldType,
    required: bool,
}

pub enum FieldType {
    String,
    Array,
    Object,
    Boolean,
}
```

## Completion Contexts

### 1. Metadata (all resources)
```yaml
metadata:
  |  # Complete: name, namespace, labels, annotations
```

### 2. Pipeline spec
```yaml
spec:
  |  # Complete: tasks, finally, params, workspaces, results
```

### 3. PipelineTask
```yaml
tasks:
  - |  # Complete: name, taskRef, taskSpec, params, workspaces, runAfter
```

### 4. Task spec
```yaml
spec:
  |  # Complete: steps, params, workspaces, results, volumes
```

### 5. Step
```yaml
steps:
  - |  # Complete: name, image, script, command, args, env
```

## Implementation Steps

### Step 1: Create completion module
- `src/completion/mod.rs`
- `src/completion/provider.rs`
- `src/completion/schemas.rs`

### Step 2: Create e2e tests
- `tests/e2e_completion.rs`
- Test infrastructure similar to diagnostics tests

### Step 3: Implement basic completion
- Start with metadata.name completion
- Add to LSP server capabilities
- Implement completion handler

### Step 4: Add schema-based completion
- Define Tekton field schemas
- Context-aware completion logic
- Filter by what's already present

### Step 5: Integration
- Wire up completion handler in main.rs
- Update ServerCapabilities
- Test with real editors

## Expected Test Structure

```rust
#[test]
fn test_complete_metadata_name() {
    let lsp = create_test_lsp();
    
    let doc = r#"
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  |
"#;
    
    let completions = lsp.complete(doc, Position { line: 3, character: 2 });
    
    assert!(completions.contains_label("name"));
    assert!(completions.contains_label("namespace"));
    assert!(completions.contains_label("labels"));
}
```

## Success Criteria

- [ ] Metadata fields completion works
- [ ] Pipeline-specific completions work
- [ ] Task-specific completions work
- [ ] PipelineTask field completion works
- [ ] Step field completion works
- [ ] All tests pass (TDD verified)
- [ ] Integration with LSP server works
- [ ] Editor shows completions in real-time

## File Changes

### New Files
- `src/completion/mod.rs`
- `src/completion/provider.rs`
- `src/completion/schemas.rs`
- `tests/e2e_completion.rs`

### Modified Files
- `src/main.rs` - Add completion handler
- `src/lib.rs` - Export completion module

## Timeline

Following TDD strictly:
- Cycle 1-2: ~30 min (basic fields)
- Cycle 3-4: ~30 min (context-aware)
- Cycle 5-6: ~30 min (nested completion)
- Integration: ~20 min
- **Total: ~2 hours**

## Notes

- Use existing `YamlDocument.find_node_at_position()` for context
- Keep schemas simple initially (hardcoded HashMap)
- Future: Load schemas from Tekton CRD JSON schemas
- Future: Add snippet support (insert complex structures)
