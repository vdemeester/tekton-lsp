//! Document formatting provider for Tekton YAML files.
//!
//! Provides consistent YAML formatting with Tekton-specific conventions:
//! - 2-space indentation
//! - Consistent key ordering for common sections
//! - Proper list formatting

pub mod provider;

pub use provider::FormattingProvider;
