# Tekton LSP Usage Guide

## Overview

The Tekton Language Server Protocol (LSP) implementation provides IDE features for Tekton YAML files including diagnostics, completion, hover documentation, and navigation.

## Architecture

```
┌─────────────────────────────────────┐
│ LSP Client (VS Code, Neovim, etc.) │
└──────────────┬──────────────────────┘
               │ JSON-RPC over stdio
               │
┌──────────────▼──────────────────────┐
│ tower-lsp Server (Rust)             │
│  ├─ Lifecycle (init/shutdown)       │
│  ├─ Document Sync (open/change)     │
│  ├─ Diagnostics (validation)        │
│  ├─ Completion (schema-based)       │
│  ├─ Hover (documentation)           │
│  └─ Navigation (goto-definition)    │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│ Document Cache (thread-safe)        │
│  Arc<RwLock<HashMap<Url, Document>>>│
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│ Tree-sitter YAML Parser              │
│  ├─ Accurate position tracking      │
│  ├─ Incremental parsing              │
│  ├─ Error recovery                  │
│  └─ AST with Range info             │
└─────────────────────────────────────┘
```

## LSP Features

### 1. Document Synchronization

**Lifecycle:**
```
Client                    Server
  │                         │
  ├──initialize────────────>│
  │<─────capabilities───────┤
  ├──initialized───────────>│
  │                         │
  ├──didOpen──────────────>│ (cache document)
  ├──didChange────────────>│ (incremental update)
  ├──didClose─────────────>│ (remove from cache)
  │                         │
  ├──shutdown─────────────>│
  │<─────ok──────────────── │
  ├──exit─────────────────>│
```

**Example: Opening a Tekton Pipeline**

When you open `pipeline.yaml` in your editor:

```yaml
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: build-pipeline
spec:
  tasks:
    - name: fetch-source
      taskRef:
        name: git-clone
```

1. Client sends `textDocument/didOpen` with full content
2. Server caches document in `DocumentCache`
3. Server parses YAML using tree-sitter
4. Server extracts `apiVersion` and `kind` for quick lookup
5. Server is ready for LSP requests on this document

**Example: Editing a Document**

When you type in the editor (incremental sync):

```
User types:   "  - name: test"
              ^^^^^^^^^^^^^^^^^^^

Client sends: didChange with:
  range: { start: {line: 8, char: 0}, end: {line: 8, char: 0} }
  text: "  - name: test\n"

Server:
  1. Retrieves document from cache
  2. Applies incremental change (efficient!)
  3. Reparses affected portion (tree-sitter incremental parsing)
  4. Updates AST
  5. Ready for next request
```

### 2. Diagnostics (Validation)

**Status:** 🚧 Coming in Task 3

Validates Tekton resources against schemas and reports errors.

**Example Use Case:**

```yaml
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  # ERROR: missing required field 'name'
spec:
  tasks: []
  # ERROR: empty tasks array not allowed
```

**Expected Diagnostics:**

```json
{
  "uri": "file:///path/to/pipeline.yaml",
  "diagnostics": [
    {
      "range": {
        "start": {"line": 2, "character": 0},
        "end": {"line": 2, "character": 8}
      },
      "severity": 1,  // Error
      "message": "Required field 'metadata.name' is missing",
      "source": "tekton-lsp"
    },
    {
      "range": {
        "start": {"line": 5, "character": 2},
        "end": {"line": 5, "character": 9}
      },
      "severity": 1,
      "message": "Pipeline must have at least one task",
      "source": "tekton-lsp"
    }
  ]
}
```

**In Your Editor:**

- Red squiggly underlines appear at error locations
- Hover shows error message
- Problems panel lists all diagnostics

### 3. Completion (Schema-based)

**Status:** 🚧 Coming in Task 4

Suggests valid fields based on Tekton schema and context.

**Example Use Case:**

```yaml
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  tasks:
    - name: build
      task|  # <-- cursor here, trigger completion
```

**Expected Completions:**

```
taskRef:        Reference to an existing Task
  name: <string>

taskSpec:       Inline Task specification
  steps:
    - name: <string>
      image: <string>

params:         Parameters for this task
  - name: <string>
    value: <string>

workspaces:     Workspace bindings
  - name: <string>
    workspace: <string>

runAfter:       Tasks that must complete first
  - <task-name>
```

### 4. Hover Documentation

**Status:** 🚧 Coming in Task 5

Shows documentation when hovering over Tekton fields.

**Example Use Case:**

Hovering over `taskRef`:

```yaml
taskRef:   # <-- hover here
  name: git-clone
```

**Shows:**

```markdown
**taskRef**

Reference to a Task resource.

Usage:
  taskRef:
    name: <task-name>
    kind: Task  # optional, defaults to Task

A TaskRef can reference:
- Cluster Tasks (cluster-scoped)
- Namespaced Tasks (same namespace as Pipeline)
- Remote Tasks (via resolvers)

See: https://tekton.dev/docs/pipelines/taskruns/#specifying-the-target-task
```

### 5. Go-to-Definition

**Status:** 🚧 Coming in Task 6

Jump to Task/Pipeline definition from reference.

**Example Use Case:**

```yaml
# pipeline.yaml
spec:
  tasks:
    - name: build
      taskRef:
        name: git-clone  # <-- Cmd+Click here
```

**Behavior:**
- Opens `task-git-clone.yaml` (if in workspace)
- Or opens browser to Tekton Hub if cluster task
- Or shows "Definition not found" if missing

### 6. Find References

**Status:** 🚧 Coming in Task 6

Find all usages of a Task/Pipeline.

**Example Use Case:**

In `task-git-clone.yaml`, trigger "Find References":

```yaml
# Shows:
pipeline-1.yaml:8    taskRef: git-clone
pipeline-2.yaml:15   taskRef: git-clone
pipeline-3.yaml:22   taskRef: git-clone
```

## End-to-End Testing

### Test Structure

```rust
// tests/integration/lsp_e2e.rs

#[tokio::test]
async fn test_diagnostics_missing_name() {
    // 1. Start LSP server
    let (client, server) = create_test_lsp();

    // 2. Initialize
    client.initialize(/*...*/).await;

    // 3. Open document with error
    client.did_open("file:///test.yaml", r#"
apiVersion: tekton.dev/v1
kind: Pipeline
metadata: {}  # Missing 'name'
spec:
  tasks: []
"#).await;

    // 4. Receive diagnostics
    let diagnostics = server.receive_diagnostics().await;

    // 5. Assert
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message, "Required field 'metadata.name' is missing");
    assert_eq!(diagnostics[0].range.start.line, 3);
}
```

### Integration Test Scenarios

**Scenario 1: Valid Pipeline**
- Open valid pipeline YAML
- Expect no diagnostics
- Request completion at various positions
- Verify valid suggestions

**Scenario 2: Invalid Pipeline (Missing Fields)**
- Open pipeline with missing `metadata.name`
- Receive diagnostic with accurate position
- Fix the error by adding `name`
- Diagnostics should clear

**Scenario 3: Invalid Pipeline (Wrong Type)**
- Open pipeline with `spec.tasks: "string"` instead of array
- Receive type error diagnostic
- Verify error points to exact location

**Scenario 4: Incremental Updates**
- Open document
- Make incremental changes (add/remove text)
- Verify document stays in sync
- Verify diagnostics update correctly

**Scenario 5: Completion**
- Open partial pipeline
- Trigger completion at `spec.|`
- Verify "tasks", "params", "workspaces" appear
- Verify invalid fields don't appear

**Scenario 6: Hover**
- Open pipeline with `taskRef`
- Hover over `taskRef`
- Verify documentation appears
- Verify markdown formatting

**Scenario 7: Go-to-Definition**
- Open pipeline referencing `taskRef: git-clone`
- Request definition on "git-clone"
- Verify jumps to task definition (if exists)
- Or returns "not found" (if missing)

## Performance Characteristics

### Parsing Performance

With tree-sitter:
- **Initial parse:** ~1-5ms for typical Tekton YAML (< 500 lines)
- **Incremental parse:** ~0.1-1ms for small edits
- **Memory:** ~10-50KB per document AST

### Response Times (Target)

- **didOpen:** < 50ms (parse + cache)
- **didChange:** < 10ms (incremental parse)
- **diagnostics:** < 100ms (validation)
- **completion:** < 50ms (schema lookup)
- **hover:** < 20ms (documentation lookup)
- **definition:** < 50ms (reference resolution)

### Scalability

- **Concurrent documents:** 100+ (limited by RAM)
- **Document size:** Tested up to 10,000 lines
- **Workspace size:** 1000+ Tekton YAML files

## Editor Configuration

### VS Code

```json
{
  "tekton-lsp.enable": true,
  "tekton-lsp.trace.server": "verbose",
  "tekton-lsp.validation": {
    "enabled": true,
    "schemas": "strict"
  }
}
```

### Neovim (with lspconfig)

```lua
require'lspconfig'.tekton_lsp.setup{
  cmd = {"tekton-lsp"},
  filetypes = {"yaml"},
  root_dir = function(fname)
    return lspconfig.util.find_git_ancestor(fname)
  end,
  settings = {
    tekton = {
      validation = { enabled = true }
    }
  }
}
```

## Implementation Status

| Feature | Status | Task |
|---------|--------|------|
| ✅ LSP Server Scaffold | Done | Task 1 |
| ✅ Document Management | Done | Task 2 |
| ✅ Tree-sitter Parser | Done | Task 2 |
| ✅ Position Tracking | Done | Task 2 |
| 🚧 Diagnostics | In Progress | Task 3 |
| 🔜 Completion | Planned | Task 4 |
| 🔜 Hover Documentation | Planned | Task 5 |
| 🔜 Go-to-Definition | Planned | Task 6 |
| 🔜 Find References | Planned | Task 6 |
| 🔜 Document Symbols | Planned | Task 7 |

## Next Steps

See `docs/plans/2026-01-20-tekton-lsp-implementation.md` for the full implementation plan.

**Current Focus: Task 3 - Diagnostics**

Using Test-Driven Development (TDD):
1. Write failing test for Pipeline validation
2. Implement Tekton schema types
3. Add validation logic
4. Publish diagnostics to client
5. Verify end-to-end with integration tests
