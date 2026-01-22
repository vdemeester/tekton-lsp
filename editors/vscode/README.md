# Tekton Language Support for VS Code

Language Server Protocol (LSP) support for Tekton Pipelines YAML files.

## Features

- **Diagnostics** - Real-time validation of Tekton resources
- **Completion** - Context-aware autocomplete for Tekton fields
- **Hover** - Documentation for Tekton fields and resources
- **Go-to-Definition** - Navigate to Task and Pipeline definitions
- **Document Symbols** - Outline view of Tekton resources
- **Formatting** - YAML formatting with consistent indentation
- **Code Actions** - Quick fixes for common issues

## Requirements

This extension requires the `tekton-lsp` binary to be installed.

### Installation Options

1. **From PATH**: Install `tekton-lsp` and ensure it's in your PATH:
   ```bash
   cargo install --path /path/to/tekton-lsp
   ```

2. **Manual Configuration**: Set the path in VS Code settings:
   ```json
   {
     "tekton-lsp.serverPath": "/path/to/tekton-lsp"
   }
   ```

## Extension Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `tekton-lsp.serverPath` | Path to the tekton-lsp binary | `""` (auto-detect) |
| `tekton-lsp.trace.server` | Trace communication with server | `"off"` |

## Supported Resources

- Pipeline
- Task
- ClusterTask
- PipelineRun
- TaskRun
- TriggerTemplate
- TriggerBinding
- EventListener

## Development

### Building the Extension

```bash
cd editors/vscode
npm install
npm run compile
```

### Packaging

```bash
npm run package
```

This creates a `.vsix` file that can be installed in VS Code.

### Installing from VSIX

```bash
code --install-extension tekton-lsp-0.1.0.vsix
```

## License

Apache License 2.0

## Links

- [Tekton Documentation](https://tekton.dev/)
- [GitHub Repository](https://github.com/tektoncd/tekton-lsp)
