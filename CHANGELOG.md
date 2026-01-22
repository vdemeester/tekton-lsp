# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-22

### Added

- **LSP Server Foundation**
  - tower-lsp based Language Server with stdio transport
  - Full document synchronization with incremental updates
  - Tree-sitter YAML parsing with accurate position tracking

- **Diagnostics**
  - Real-time validation of Tekton resources
  - Missing required field detection
  - Empty array validation
  - Unknown field warnings
  - Type mismatch detection

- **Completion**
  - Context-aware autocomplete for Tekton fields
  - Schema-based suggestions for Pipeline, Task, and related resources
  - Trigger characters: `:`, ` `, `-`
  - Field descriptions and type information

- **Hover Documentation**
  - Inline documentation for 20+ Tekton fields
  - Documentation for resource kinds (Pipeline, Task, etc.)
  - Metadata field documentation

- **Go-to-Definition**
  - Navigate from taskRef to Task definition
  - Navigate from pipelineRef to Pipeline definition
  - Cross-file navigation within workspace

- **Document Symbols**
  - Hierarchical outline for Tekton resources
  - Shows params, tasks, steps with item counts
  - Supports Pipeline, Task, PipelineRun, TaskRun

- **Formatting**
  - YAML formatting with consistent 2-space indentation
  - Structure preservation
  - Uses serde_yaml for normalization

- **Code Actions**
  - Quick fix for missing required fields (add with template)
  - Quick fix for unknown fields (remove)

- **VS Code Extension**
  - Extension scaffold in `editors/vscode/`
  - Auto-detection of tekton-lsp binary
  - Configuration for server path and tracing

- **Test Coverage**
  - 82 tests total
  - Unit tests for all providers
  - E2E tests for diagnostics, completion, hover, definition, symbols, formatting, code actions
  - Protocol-level Python tests

### Supported Resources

- Pipeline
- Task
- ClusterTask
- PipelineRun
- TaskRun
- TriggerTemplate
- TriggerBinding
- EventListener

### Documentation

- README with feature overview and quick start
- LSP Usage Guide with feature details
- Editor Setup Guide for VS Code, Emacs, Neovim
- Implementation plan and architecture decisions

## [Unreleased]

### Planned

- Find references (workspace-wide)
- Integration with Tekton Hub
- Remote task resolution
- Additional editor plugins (IntelliJ, Sublime)
