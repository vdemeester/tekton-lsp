# Tekton LSP Setup Guide

This guide shows you how to set up the Tekton Language Server with various editors and tools.

## Building the LSP Server

First, build the LSP server binary:

```bash
# Clone the repository
git clone https://github.com/tektoncd/tekton-lsp
cd tekton-lsp

# Build release binary
cargo build --release

# Binary will be at: target/release/tekton-lsp
```

The release binary is optimized and small (~5-10MB).

## VS Code Setup

### Option 1: Manual Configuration (Development)

Create `.vscode/settings.json` in your workspace:

```json
{
  "yaml.customTags": [
    "!reference scalar",
    "!reference sequence"
  ],
  "yaml.schemas": {
    "https://raw.githubusercontent.com/tektoncd/pipeline/main/docs/api-spec/spec.json": "*.yaml"
  },
  "[yaml]": {
    "editor.defaultFormatter": "redhat.vscode-yaml"
  }
}
```

Install a generic LSP client extension:
- Install **vscode-lsp-client** or **Custom Local Formatters**
- Configure the LSP server path

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lsp",
      "request": "launch",
      "name": "Tekton LSP",
      "command": "/path/to/tekton-lsp/target/release/tekton-lsp",
      "args": ["--verbose"],
      "filetypes": ["yaml"],
      "rootPatterns": [".git", "kustomization.yaml"]
    }
  ]
}
```

### Option 2: Using vscode-langservers-extracted

If you have `vscode-langservers-extracted`, you can add custom LSP configuration:

Create a VS Code extension or use workspace settings to register the LSP:

```json
{
  "languageserver": {
    "tekton": {
      "command": "/path/to/tekton-lsp",
      "args": ["--verbose"],
      "filetypes": ["yaml"],
      "rootPatterns": [".git"],
      "settings": {}
    }
  }
}
```

### Option 3: Create VS Code Extension (Future)

We plan to publish an official VS Code extension. Stay tuned!

## Emacs Setup (with eglot)

Emacs with **eglot** (built-in Emacs 29+) provides excellent LSP support.

Add to your `init.el` or `.emacs`:

```elisp
;; Add Tekton LSP server to eglot (supports both yaml-mode and yaml-ts-mode)
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '((yaml-mode yaml-ts-mode) . ("/path/to/tekton-lsp/target/release/tekton-lsp"))))

;; Auto-start eglot for YAML files with Tekton resources
(defun my/tekton-lsp-maybe-enable ()
  "Enable Tekton LSP for Tekton YAML files."
  (when (and buffer-file-name
             (or (string-match-p "pipeline.*\\.yaml$" buffer-file-name)
                 (string-match-p "task.*\\.yaml$" buffer-file-name)
                 (string-match-p "tekton.*\\.yaml$" buffer-file-name)))
    (eglot-ensure)))

(add-hook 'yaml-mode-hook #'my/tekton-lsp-maybe-enable)
(add-hook 'yaml-ts-mode-hook #'my/tekton-lsp-maybe-enable)
```

### With Conditional Activation

Only activate for Tekton files:

```elisp
(defun my/is-tekton-file ()
  "Check if current buffer is a Tekton YAML file."
  (and buffer-file-name
       (string-match-p "\\.yaml$" buffer-file-name)
       (save-excursion
         (goto-char (point-min))
         (re-search-forward "apiVersion: tekton\\.dev/" nil t))))

(defun my/tekton-lsp-conditional-enable ()
  "Enable Tekton LSP if current file is a Tekton resource."
  (when (my/is-tekton-file)
    (eglot-ensure)))

(add-hook 'yaml-mode-hook #'my/tekton-lsp-conditional-enable)
(add-hook 'yaml-ts-mode-hook #'my/tekton-lsp-conditional-enable)
```

### Eglot Configuration

Customize eglot for Tekton LSP:

```elisp
;; Show diagnostics in echo area
(setq eglot-report-progress t)

;; Auto-format on save (optional)
(add-hook 'eglot-managed-mode-hook
          (lambda ()
            (when (derived-mode-p 'yaml-mode 'yaml-ts-mode)
              (add-hook 'before-save-hook #'eglot-format-buffer nil t))))
```

### Test the Setup

1. Open a Tekton YAML file
2. Check mode line for `[eglot ...]` indicator
3. View diagnostics: `M-x flymake-show-buffer-diagnostics`
4. Trigger completion: `M-x completion-at-point` (or `C-M-i`)

## Neovim Setup (with nvim-lspconfig)

Using `nvim-lspconfig`:

```lua
-- Add to your init.lua or after/plugin/lsp.lua

local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define tekton-lsp if not already defined
if not configs.tekton_lsp then
  configs.tekton_lsp = {
    default_config = {
      cmd = {'/path/to/tekton-lsp/target/release/tekton-lsp'},
      filetypes = {'yaml'},
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
      end,
      settings = {},
    },
  }
end

-- Setup tekton-lsp
lspconfig.tekton_lsp.setup({
  -- Auto-start only for Tekton YAML files
  on_attach = function(client, bufnr)
    -- Enable completion triggered by <c-x><c-o>
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

    -- Keybindings
    local opts = { noremap=true, silent=true, buffer=bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)

    print("Tekton LSP attached to buffer " .. bufnr)
  end,

  -- Only activate for Tekton YAML files
  autostart = function()
    local bufname = vim.api.nvim_buf_get_name(0)
    if bufname:match("%.yaml$") then
      -- Read first few lines to check for Tekton apiVersion
      local lines = vim.api.nvim_buf_get_lines(0, 0, 10, false)
      for _, line in ipairs(lines) do
        if line:match("apiVersion: tekton%.dev/") then
          return true
        end
      end
    end
    return false
  end,
})
```

## Claude Code Integration

While Claude Code doesn't currently have built-in LSP client capabilities, you can still leverage the Tekton LSP server for validation and analysis!

### Manual LSP Validation

You can use the LSP server as a standalone validation tool:

```bash
# Build the LSP server
cd /path/to/tekton-lsp
cargo build --release

# Create a simple validation script
cat > check-tekton.py << 'EOF'
#!/usr/bin/env python3
import json, subprocess, sys, time

def validate_tekton_file(filepath):
    with open(filepath) as f:
        content = f.read()

    proc = subprocess.Popen(
        ["./target/release/tekton-lsp"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )

    # Send LSP initialize
    init_msg = {"jsonrpc":"2.0","id":1,"method":"initialize",
                "params":{"capabilities":{},"processId":None,"rootUri":f"file://{filepath}"}}
    content_len = len(json.dumps(init_msg))
    proc.stdin.write(f"Content-Length: {content_len}\r\n\r\n{json.dumps(init_msg)}".encode())
    proc.stdin.flush()
    time.sleep(0.3)

    # Send initialized
    init_notif = {"jsonrpc":"2.0","method":"initialized","params":{}}
    content_len = len(json.dumps(init_notif))
    proc.stdin.write(f"Content-Length: {content_len}\r\n\r\n{json.dumps(init_notif)}".encode())
    proc.stdin.flush()
    time.sleep(0.3)

    # Send didOpen
    didopen = {"jsonrpc":"2.0","method":"textDocument/didOpen",
               "params":{"textDocument":{"uri":f"file://{filepath}","languageId":"yaml",
                                        "version":1,"text":content}}}
    content_len = len(json.dumps(didopen))
    proc.stdin.write(f"Content-Length: {content_len}\r\n\r\n{json.dumps(didopen)}".encode())
    proc.stdin.flush()
    time.sleep(1)

    proc.stdin.close()
    output = proc.stdout.read().decode()

    # Parse diagnostics
    if "publishDiagnostics" in output:
        # Extract diagnostics from LSP response
        import re
        diag_match = re.search(r'\{"jsonrpc.*publishDiagnostics.*?\}\}', output)
        if diag_match:
            diag_data = json.loads(diag_match.group(0))
            diagnostics = diag_data.get("params", {}).get("diagnostics", [])
            return diagnostics
    return []

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: check-tekton.py <file.yaml>")
        sys.exit(1)

    diagnostics = validate_tekton_file(sys.argv[1])
    if diagnostics:
        print(f"Found {len(diagnostics)} issues:")
        for diag in diagnostics:
            line = diag["range"]["start"]["line"] + 1
            print(f"  Line {line}: {diag['message']}")
    else:
        print("✓ No issues found!")
EOF

chmod +x check-tekton.py
```

### Using with Claude Code

When working with Tekton files in Claude Code, you can:

1. **Ask Claude to validate**: "Run the LSP validator on this file"
2. **Claude can execute the validation script**: The script above will report diagnostics
3. **Claude can fix issues**: Based on the diagnostic output

### Example Workflow

```
User: Check pipeline.yaml for Tekton validation errors

Claude: Let me validate pipeline.yaml using the Tekton LSP server...
[Runs validation script]

Found 2 issues:
  Line 3: Required field 'metadata.name' is missing
  Line 6: Pipeline must have at least one task

Would you like me to fix these?

User: Yes

Claude: [Adds metadata.name and adds a sample task]
Done! The file now passes validation.
```

### Future: Native LSP Integration

Future versions of Claude Code may include native LSP client capabilities. Until then, the manual validation approach above provides equivalent functionality for validation and diagnostics.

## Testing the LSP Server

### Manual Test with stdio

You can test the LSP server manually:

```bash
# Run the server
./target/release/tekton-lsp

# Send initialize request (paste this JSON)
Content-Length: 124

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}

# Server should respond with capabilities
```

### Test with a Client

Create a test file `test-pipeline.yaml`:

```yaml
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  # Missing name - should show error
  namespace: default
spec:
  tasks: []  # Empty tasks - should show error
```

Open this file in your configured editor and verify:
1. Two diagnostics appear (missing name, empty tasks)
2. Diagnostics have correct line numbers
3. Severity is ERROR for both

## Troubleshooting

### LSP Server Not Starting

**Check the binary works:**
```bash
./target/release/tekton-lsp --version
```

**Enable verbose logging:**
```bash
RUST_LOG=debug ./target/release/tekton-lsp
```

**Check stderr output:**
The LSP server logs to stderr. In VS Code, check **Output** > **Tekton LSP**.

### No Diagnostics Appearing

**Verify file is recognized as YAML:**
- File extension should be `.yaml` or `.yml`
- Check editor's language mode

**Check LSP server is running:**
- VS Code: View > Output > Select "Tekton LSP"
- Emacs: `M-x eglot-events-buffer`
- Neovim: `:LspInfo`

**Verify file is valid Tekton YAML:**
The LSP activates for files with `apiVersion: tekton.dev/*`

### Diagnostics Not Updating

**Check incremental sync is working:**
- Make an edit and save
- Diagnostics should update within 100ms

**If diagnostics are stale:**
- Close and reopen the file
- Restart the LSP server

### Performance Issues

**Large files (>10,000 lines):**
- Tree-sitter should still parse in <50ms
- If slow, check RUST_LOG isn't set to trace

**Many open files:**
- LSP server uses ~10-50KB per document
- Should handle 100+ files easily

## Advanced Configuration

### Custom Validation Rules

Future: Configure which validations to enable:

```json
{
  "tekton.validation": {
    "requiredFields": true,
    "emptyArrays": true,
    "unknownFields": "warning",
    "typeChecking": true
  }
}
```

### Workspace-Specific Settings

Create `.tekton-lsp.json` in your project root:

```json
{
  "validation": {
    "strictMode": true,
    "allowedUnknownFields": ["customField1", "customField2"]
  }
}
```

## Next Steps

- **Task Completion**: Implement autocomplete for Tekton fields
- **Hover Documentation**: Show field documentation on hover
- **Go-to-Definition**: Jump to Task/Pipeline definitions
- **VS Code Extension**: Official extension with one-click install

## Getting Help

- **Issues**: https://github.com/tektoncd/tekton-lsp/issues
- **Documentation**: See `docs/LSP_USAGE.md` for LSP feature details
- **Tekton Community**: https://tektoncd.slack.com

## Contributing

Help us improve the LSP server:
1. Test with your editor
2. Report bugs and UX issues
3. Contribute editor-specific configurations
4. Share your setup in discussions

---

**Status**: Active Development | **Version**: 0.1.0 | **Last Updated**: 2026-01-20
