# Tekton LSP Usage Guide

## Overview

The Tekton Language Server Protocol (LSP) implementation provides IDE features for Tekton YAML files including diagnostics, completion, hover documentation, navigation, formatting, and code actions.

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
│  ├─ Navigation (goto-definition)    │
│  ├─ Symbols (document outline)      │
│  ├─ Formatting (YAML normalization) │
│  └─ Code Actions (quick fixes)      │
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

### 2. Diagnostics (Validation)

**Status:** ✅ Implemented

Validates Tekton resources against schemas and reports errors.

**Example:**

```yaml
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  # ERROR: missing required field 'name'
spec:
  tasks: []
  # ERROR: empty tasks array not allowed
```

**Editor Behavior:**
- Red squiggly underlines appear at error locations
- Hover shows error message
- Problems panel lists all diagnostics

### 3. Completion (Schema-based)

**Status:** ✅ Implemented

Suggests valid fields based on Tekton schema and context.

**Trigger Characters:** `:`, ` `, `-`

**Example:**
```yaml
spec:
  tasks:
    - name: build
      task|  # <-- completions: taskRef, taskSpec, params, workspaces, runAfter
```

### 4. Hover Documentation

**Status:** ✅ Implemented

Shows documentation when hovering over Tekton fields.

**Supported Elements:**
- Field keys (tasks, steps, params, etc.)
- Resource kinds (Pipeline, Task, etc.)
- Metadata fields (name, labels, annotations)

**Example:**

Hovering over `taskRef` shows:
```markdown
**taskRef**

Reference to a Task resource.

Usage:
  taskRef:
    name: <task-name>
    kind: Task  # optional
```

### 5. Go-to-Definition

**Status:** ✅ Implemented

Jump to Task/Pipeline definition from reference.

**Example:**
```yaml
spec:
  tasks:
    - name: build
      taskRef:
        name: git-clone  # <-- Cmd+Click jumps to task definition
```

**Behavior:**
- Opens the file containing the referenced Task
- Positions cursor at the Task definition
- Works across files in the workspace

### 6. Document Symbols

**Status:** ✅ Implemented

Provides outline view of Tekton resources.

**Example Outline for Pipeline:**
```
Pipeline: my-pipeline
├── metadata
└── spec
    ├── params (2)
    │   ├── version
    │   └── environment
    ├── tasks (3)
    │   ├── build
    │   ├── test
    │   └── deploy
    └── finally (1)
        └── cleanup
```

**Example Outline for Task:**
```
Task: build-task
├── metadata
└── spec
    ├── params (1)
    │   └── source-url
    └── steps (2)
        ├── clone
        └── build
```

### 7. Formatting

**Status:** ✅ Implemented

YAML formatting with consistent indentation.

**Features:**
- Normalizes indentation to 2 spaces
- Preserves document structure
- Handles complex nested structures

**Usage:**
- VS Code: `Shift+Alt+F` or right-click → Format Document
- Neovim: `:lua vim.lsp.buf.format()`

### 8. Code Actions (Quick Fixes)

**Status:** ✅ Implemented

Provides quick fixes for common issues.

**Available Actions:**

| Diagnostic | Quick Fix |
|------------|-----------|
| Missing required field 'X' | Add missing field 'X' with template |
| Unknown field 'X' | Remove unknown field 'X' |

**Example:**
```yaml
apiVersion: tekton.dev/v1
kind: Task
# Diagnostic: Missing required field 'metadata'
# Quick Fix: Add missing field 'metadata'
```

Applying the fix adds:
```yaml
metadata:
  name:
```

## Performance Characteristics

### Parsing Performance

With tree-sitter:
- **Initial parse:** ~1-5ms for typical Tekton YAML (< 500 lines)
- **Incremental parse:** ~0.1-1ms for small edits
- **Memory:** ~10-50KB per document AST

### Response Times

- **didOpen:** < 50ms (parse + cache)
- **didChange:** < 10ms (incremental parse)
- **diagnostics:** < 100ms (validation)
- **completion:** < 50ms (schema lookup)
- **hover:** < 20ms (documentation lookup)
- **definition:** < 50ms (reference resolution)
- **symbols:** < 20ms (outline generation)
- **formatting:** < 50ms (YAML normalization)
- **codeAction:** < 20ms (quick fix generation)

### Scalability

- **Concurrent documents:** 100+ (limited by RAM)
- **Document size:** Tested up to 10,000 lines
- **Workspace size:** 1000+ Tekton YAML files

## Editor Configuration

### VS Code

Use the extension in `editors/vscode/` or configure manually:

```json
{
  "tekton-lsp.serverPath": "/path/to/tekton-lsp",
  "tekton-lsp.trace.server": "verbose"
}
```

### Neovim (with lspconfig)

```lua
require'lspconfig'.tekton_lsp.setup{
  cmd = {"tekton-lsp"},
  filetypes = {"yaml"},
  root_dir = function(fname)
    return lspconfig.util.find_git_ancestor(fname)
  end
}
```

### Emacs (with eglot)

```elisp
(add-to-list 'eglot-server-programs
             '(yaml-mode . ("tekton-lsp")))
```

## Implementation Status

| Feature | Status | Description |
|---------|--------|-------------|
| ✅ LSP Server Scaffold | Done | tower-lsp based server |
| ✅ Document Management | Done | Full sync with incremental updates |
| ✅ Tree-sitter Parser | Done | Accurate position tracking |
| ✅ Diagnostics | Done | Resource validation |
| ✅ Completion | Done | Context-aware suggestions |
| ✅ Hover Documentation | Done | Field documentation |
| ✅ Go-to-Definition | Done | Task/Pipeline navigation |
| ✅ Document Symbols | Done | Outline view |
| ✅ Formatting | Done | YAML normalization |
| ✅ Code Actions | Done | Quick fixes |

## Test Coverage

The LSP implementation has comprehensive test coverage:

| Test Suite | Tests | Description |
|------------|-------|-------------|
| e2e_diagnostics | 8 | Validation scenarios |
| e2e_completion | 5 | Context-aware completion |
| e2e_hover | 8 | Documentation display |
| e2e_definition | 4 | Navigation tests |
| e2e_symbols | 6 | Outline generation |
| e2e_formatting | 6 | YAML formatting |
| e2e_codeactions | 7 | Quick fix actions |
| Unit tests | 38 | Core functionality |
| **Total** | **82** | Full coverage |

## Supported Resources

- Pipeline
- Task
- ClusterTask
- PipelineRun
- TaskRun
- TriggerTemplate
- TriggerBinding
- EventListener

## Future Enhancements

- Find references (workspace-wide)
- Integration with Tekton Hub
- Remote task resolution
- Workspace diagnostics
