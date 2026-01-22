//! Code actions provider for Tekton YAML files.
//!
//! Provides quick fixes for common issues:
//! - Add missing required fields
//! - Remove unknown fields
//! - Fix common mistakes

pub mod provider;

pub use provider::CodeActionsProvider;
