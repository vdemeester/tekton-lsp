//! Completion provider for Tekton YAML files.
//!
//! Provides context-aware completion suggestions based on:
//! - Document kind (Pipeline, Task, etc.)
//! - Cursor position and context
//! - Tekton resource schemas

pub mod provider;
pub mod schemas;

pub use provider::CompletionProvider;
