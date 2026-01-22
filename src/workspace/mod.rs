//! Workspace management for Tekton LSP.
//!
//! Provides workspace-wide indexing of Tekton resources for:
//! - Go-to-definition (navigate from taskRef to Task)
//! - Find references (find all uses of a Task/Pipeline)
//! - Cross-file validation

pub mod index;

pub use index::WorkspaceIndex;
