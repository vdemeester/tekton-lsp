//! Document symbols provider for Tekton YAML files.
//!
//! Provides document outline showing:
//! - Resource kind and name
//! - Spec sections (tasks, steps, params, etc.)
//! - Individual items with their names

pub mod provider;

pub use provider::SymbolsProvider;
