//! Hover provider for Tekton YAML files.
//!
//! Provides documentation on hover for:
//! - Tekton resource kinds (Pipeline, Task, etc.)
//! - Tekton field names (tasks, steps, params, etc.)
//! - Common metadata fields

pub mod docs;
pub mod provider;

pub use provider::HoverProvider;
