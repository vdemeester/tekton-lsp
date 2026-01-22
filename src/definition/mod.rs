//! Go-to-definition provider for Tekton YAML files.
//!
//! Provides navigation from:
//! - taskRef.name → Task definition
//! - pipelineRef.name → Pipeline definition

pub mod provider;

pub use provider::DefinitionProvider;
