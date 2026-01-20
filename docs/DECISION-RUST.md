# Architectural Decision: Rust Implementation

**Date**: 2026-01-20
**Status**: Accepted
**Decision**: Implement Tekton LSP in Rust instead of Go

## Context

Initially started implementation in Go to leverage existing Tekton Go libraries (`github.com/tektoncd/pipeline`). After completing the initial scaffold, we reconsidered the technology choice.

## Decision

Switch to **Rust** for the LSP implementation.

## Rationale

### Primary Reasons

1. **Superior LSP Ecosystem**
   - `tower-lsp`: Production-ready, mature LSP framework
   - Many successful LSPs written in Rust (rust-analyzer, taplo, yaml-language-server)
   - Better ergonomics for LSP protocol handling

2. **Performance & Resource Efficiency**
   - LSPs run as long-lived daemon processes
   - Rust's memory efficiency critical for IDE responsiveness
   - Smaller binary size: ~5-10MB (Rust) vs ~50-100MB (Go)
   - Better resource usage for developer machines

3. **CRD Schema Validation**
   - Can validate against Kubernetes CRD schemas directly
   - Don't need full Tekton Go dependency tree
   - Lighter dependencies, faster builds

4. **Opportunity for Rust Ecosystem**
   - Create reusable Tekton library for Rust
   - Fill gap in Tekton ecosystem (currently Go/Python focused)
   - Benefit broader Rust + Kubernetes community

### Technical Comparison

| Aspect | Rust | Go |
|--------|------|-----|
| LSP Framework | `tower-lsp` (⭐⭐⭐⭐⭐) | `go.lsp.dev` (⭐⭐⭐) |
| Binary Size | ~5-10MB | ~50-100MB |
| Memory Usage | Excellent | Good |
| YAML Parsing | `serde_yaml` + position tracking | `gopkg.in/yaml.v3` |
| Type Safety | Stronger | Strong |
| Async/Concurrency | `tokio` async runtime | goroutines |
| Dependency Management | Cargo | Go modules |

## Consequences

### Positive

- ✅ Better long-term performance and resource usage
- ✅ Smaller binary for easier distribution
- ✅ More maintainable LSP code
- ✅ Creates Tekton Rust library as byproduct
- ✅ Fills ecosystem gap

### Negative

- ❌ Restart implementation (lost ~1 hour of Go work)
- ❌ Less direct integration with Tekton Go codebase
- ❌ Must implement CRD validation ourselves
- ❌ Potentially less familiar to Tekton contributors

### Neutral

- ⚖️ Different error handling paradigm (Result types)
- ⚖️ Learning curve if team is Go-focused

## Implementation Plan Changes

### Rust Tech Stack

- **LSP Framework**: `tower-lsp` (async LSP server)
- **Runtime**: `tokio` (async runtime)
- **YAML Parsing**: `serde_yaml` (with `serde`)
- **JSON Schema**: `schemars` or `jsonschema` for CRD validation
- **Error Handling**: `anyhow` / `thiserror`
- **Logging**: `tracing` / `tracing-subscriber`

### New Architecture

```
tekton-lsp/
├── Cargo.toml
├── src/
│   ├── main.rs              # Binary entry point
│   ├── server.rs            # LSP server implementation
│   ├── handlers/            # LSP request handlers
│   │   ├── mod.rs
│   │   ├── completion.rs
│   │   ├── hover.rs
│   │   ├── diagnostics.rs
│   │   └── ...
│   ├── parser/              # YAML parsing with positions
│   │   ├── mod.rs
│   │   └── ast.rs
│   ├── workspace/           # Workspace indexing
│   │   ├── mod.rs
│   │   └── index.rs
│   └── tekton/              # Tekton types & validation (lib)
│       ├── mod.rs
│       ├── pipeline.rs
│       ├── task.rs
│       └── validation.rs
└── tekton-types/            # Separate crate for reusable library
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── v1/              # Tekton v1 API
    │   └── schemas/         # CRD schemas
    └── examples/
```

### Phases Remain Similar

1. **Foundation**: Server scaffold + YAML parsing
2. **Diagnostics**: CRD validation
3. **Completion**: Schema-based suggestions
4. **Hover**: Documentation
5. **Navigation**: Workspace indexing
6. **Additional Features**: Symbols, formatting, code actions
7. **Integration**: Testing + VS Code extension

## References

- Go implementation: `git tag v0.0.1-go` / `git branch go-initial-attempt`
- Original plan: `docs/plans/2026-01-20-tekton-lsp-implementation.md` (Go-based)
- tower-lsp: https://github.com/ebkalderon/tower-lsp
- Rust LSP examples: rust-analyzer, taplo-lsp, yaml-language-server

## Next Steps

1. Initialize Rust project with Cargo
2. Add tower-lsp and core dependencies
3. Implement basic LSP server scaffold
4. Create YAML parser with position tracking
5. Design Tekton types for validation
6. Consider publishing `tekton-types` crate separately
