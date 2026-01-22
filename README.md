# Tekton Language Server Protocol (LSP)

> A Rust-based Language Server Protocol implementation for Tekton YAML files providing intelligent IDE features like diagnostics, completion, hover documentation, and navigation.

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![LSP](https://img.shields.io/badge/LSP-3.17-green.svg)](https://microsoft.github.io/language-server-protocol/)

## Features

- ✅ **Document Management** - Full document synchronization with incremental updates
- ✅ **YAML Parsing** - Tree-sitter based parsing with accurate position tracking
- ✅ **Diagnostics** - Real-time validation of Tekton resources
- ✅ **Completion** - Context-aware autocomplete for Tekton fields
- ✅ **Hover Documentation** - Inline documentation for Tekton resources
- ✅ **Go-to-Definition** - Jump to Task/Pipeline definitions
- ✅ **Document Symbols** - Outline view of Tekton resources
- ✅ **Formatting** - YAML formatting with consistent indentation
- ✅ **Code Actions** - Quick fixes for common issues

## Quick Start

### Prerequisites

- Rust 1.70+ (or use provided Nix environment)
- A compatible editor (VS Code, Neovim, Emacs, Claude Code, etc.)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/tektoncd/tekton-lsp
cd tekton-lsp

# Build the LSP server
cargo build --release

# The binary will be at target/release/tekton-lsp
./target/release/tekton-lsp --version
```

### Using Nix (Recommended for Development)

```bash
# Enter development shell with all dependencies
nix-shell

# Build and run
cargo build
cargo test
```

### Editor Setup

See **[docs/SETUP.md](docs/SETUP.md)** for detailed setup instructions for:
- **VS Code** - Extension available in `editors/vscode/`
- **Emacs** - Using eglot (built-in Emacs 29+)
- **Neovim** - Using nvim-lspconfig
- **Claude Code** - LSP integration via MCP

Quick editor test:
1. Build the LSP server (`cargo build --release`)
2. Configure your editor (see SETUP.md)
3. Open a Tekton YAML file with errors
4. See diagnostics appear automatically!

## Development

### Running Tests

```bash
# Run all unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test suites
cargo test --test e2e_diagnostics
cargo test --test e2e_completion
cargo test --test e2e_hover
cargo test --test e2e_definition
cargo test --test e2e_symbols
cargo test --test e2e_formatting
cargo test --test e2e_codeactions
```

### Building

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (optimized, ~5-10MB binary)
cargo build --release

# Check without building
cargo check
```

### Logging

Enable verbose logging for development:

```bash
# Run with trace-level logging
RUST_LOG=trace ./target/debug/tekton-lsp
```

## Implementation Status

| Phase | Task | Status | Description |
|-------|------|--------|-------------|
| **Phase 1: Foundation** | | | |
| | Task 1 | ✅ Done | LSP server scaffold with tower-lsp |
| | Task 2 | ✅ Done | Document management & tree-sitter parsing |
| **Phase 2: Diagnostics** | | | |
| | Task 3 | ✅ Done | Tekton resource validation |
| **Phase 3: Completion** | | | |
| | Task 4 | ✅ Done | Context-aware completion |
| **Phase 4: Documentation** | | | |
| | Task 5 | ✅ Done | Hover documentation provider |
| **Phase 5: Navigation** | | | |
| | Task 6 | ✅ Done | Go-to-definition |
| **Phase 6: Advanced** | | | |
| | Task 7 | ✅ Done | Document symbols & outline |
| | Task 8 | ✅ Done | YAML formatting |
| | Task 9 | ✅ Done | Code actions & quick fixes |
| **Phase 7: Integration** | | | |
| | Task 10 | ✅ Done | Integration tests (81 tests) |
| | Task 11 | ✅ Done | VS Code extension |
| | Task 12 | ✅ Done | Documentation & release |

## Documentation

- [LSP Usage Guide](docs/LSP_USAGE.md) - Comprehensive guide on LSP features and usage
- [Editor Setup](docs/SETUP.md) - Detailed editor configuration instructions
- [Implementation Plan](docs/plans/2026-01-20-tekton-lsp-implementation.md) - Detailed implementation roadmap
- [Architectural Decision: Rust](docs/DECISION-RUST.md) - Why we chose Rust over Go

## VS Code Extension

A VS Code extension is available in `editors/vscode/`. See the [extension README](editors/vscode/README.md) for installation instructions.

## Contributing

Contributions are welcome! All planned features are now implemented with comprehensive test coverage.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Write tests first
4. Implement features to pass tests
5. Run `cargo test` and `cargo clippy`
6. Submit a pull request

## Performance

### Parsing Performance

With tree-sitter:
- **Initial parse**: ~1-5ms for typical Tekton YAML (< 500 lines)
- **Incremental parse**: ~0.1-1ms for small edits
- **Memory**: ~10-50KB per document AST

### Target Response Times

- **didOpen**: < 50ms (parse + cache)
- **didChange**: < 10ms (incremental parse)
- **diagnostics**: < 100ms (validation)
- **completion**: < 50ms (schema lookup)
- **hover**: < 20ms (documentation lookup)
- **definition**: < 50ms (reference resolution)

## Supported Resources

- Pipeline
- Task
- ClusterTask
- PipelineRun
- TaskRun
- TriggerTemplate
- TriggerBinding
- EventListener

## Roadmap

### Completed
- ✅ LSP server scaffold
- ✅ Document synchronization
- ✅ Tree-sitter YAML parsing
- ✅ Diagnostics & validation
- ✅ Context-aware completion
- ✅ Hover documentation
- ✅ Go-to-definition
- ✅ Document symbols
- ✅ YAML formatting
- ✅ Code actions
- ✅ VS Code extension

### Future Enhancements
- Find references (workspace-wide)
- Integration with Tekton Hub
- Extract reusable `tekton-types` crate
- Performance optimizations
- Additional editor plugins (IntelliJ, Sublime)

## License

Apache License 2.0 - See [LICENSE](LICENSE) for details.

## References

- [Tekton Documentation](https://tekton.dev/)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp](https://github.com/ebkalderon/tower-lsp)
- [tree-sitter](https://tree-sitter.github.io/)
- [tree-sitter-yaml](https://github.com/tree-sitter-grammars/tree-sitter-yaml)

## Contact

- Issues: [GitHub Issues](https://github.com/tektoncd/tekton-lsp/issues)
- Discussions: [Tekton Slack](https://tektoncd.slack.com/)

---

**Status**: Feature Complete | **Last Updated**: 2026-01-22
