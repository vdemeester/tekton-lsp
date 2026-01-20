// End-to-End LSP Tests for Diagnostics
//
// These tests demonstrate the full LSP workflow:
// 1. Client opens a document
// 2. Server parses and validates
// 3. Server publishes diagnostics
// 4. Client displays errors to user
//
// Status: 🚧 Coming in Task 3 (Diagnostics)

#[cfg(test)]
mod e2e_tests {
    use tower_lsp::lsp_types::*;

    // TODO: Implement test helper to start LSP server
    // async fn create_test_lsp() -> (TestClient, TestServer) { ... }

    /// Test Case 1: Valid Pipeline - No Diagnostics
    ///
    /// Given: A valid Tekton Pipeline YAML
    /// When: Client opens the document
    /// Then: Server returns empty diagnostics (no errors)
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_valid_pipeline_no_diagnostics() {
        // let (client, server) = create_test_lsp().await;

        // let valid_pipeline = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   name: build-pipeline
        //   namespace: default
        // spec:
        //   tasks:
        //     - name: fetch-source
        //       taskRef:
        //         name: git-clone
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", valid_pipeline).await;

        // let diagnostics = server.receive_diagnostics().await;
        // assert_eq!(diagnostics.len(), 0, "Valid pipeline should have no errors");
    }

    /// Test Case 2: Missing Required Field
    ///
    /// Given: A Pipeline missing `metadata.name`
    /// When: Client opens the document
    /// Then: Server publishes diagnostic pointing to metadata
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_missing_required_field() {
        // let (client, server) = create_test_lsp().await;

        // let invalid_pipeline = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   namespace: default
        //   # ERROR: Missing required 'name' field
        // spec:
        //   tasks: []
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", invalid_pipeline).await;

        // let diagnostics = server.receive_diagnostics().await;
        // assert_eq!(diagnostics.len(), 1);

        // let error = &diagnostics[0];
        // assert_eq!(error.severity, Some(DiagnosticSeverity::ERROR));
        // assert!(error.message.contains("metadata.name"));
        // assert_eq!(error.range.start.line, 3); // Line with 'metadata:'

        // // Verify position is accurate (from tree-sitter)
        // assert_eq!(error.range.start.character, 0);
    }

    /// Test Case 3: Empty Tasks Array
    ///
    /// Given: A Pipeline with empty tasks array
    /// When: Client opens the document
    /// Then: Server publishes diagnostic for empty tasks
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_empty_tasks_array() {
        // let (client, server) = create_test_lsp().await;

        // let invalid_pipeline = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   name: test
        // spec:
        //   tasks: []  # ERROR: Must have at least one task
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", invalid_pipeline).await;

        // let diagnostics = server.receive_diagnostics().await;
        // assert_eq!(diagnostics.len(), 1);

        // let error = &diagnostics[0];
        // assert!(error.message.contains("at least one task"));
        // assert_eq!(error.range.start.line, 6); // Line with 'tasks: []'
    }

    /// Test Case 4: Type Mismatch
    ///
    /// Given: tasks field is a string instead of array
    /// When: Client opens the document
    /// Then: Server publishes type error diagnostic
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_type_mismatch() {
        // let (client, server) = create_test_lsp().await;

        // let invalid_pipeline = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   name: test
        // spec:
        //   tasks: "should-be-array"  # ERROR: Wrong type
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", invalid_pipeline).await;

        // let diagnostics = server.receive_diagnostics().await;
        // assert!(diagnostics.len() > 0);

        // let error = &diagnostics[0];
        // assert!(error.message.contains("type") || error.message.contains("array"));
    }

    /// Test Case 5: Incremental Update Clears Error
    ///
    /// Given: Document with error
    /// When: User fixes the error via incremental edit
    /// Then: Diagnostics are updated and error is cleared
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_incremental_fix_clears_diagnostic() {
        // let (client, server) = create_test_lsp().await;

        // // Initial invalid pipeline
        // let invalid = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   namespace: default
        // spec:
        //   tasks: []
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", invalid).await;

        // let diagnostics = server.receive_diagnostics().await;
        // assert!(diagnostics.len() > 0, "Should have error initially");

        // // User adds the missing 'name' field
        // client.did_change(vec![
        //     TextDocumentContentChangeEvent {
        //         range: Some(Range {
        //             start: Position { line: 4, character: 0 },
        //             end: Position { line: 4, character: 0 },
        //         }),
        //         text: "  name: fixed-pipeline\n".to_string(),
        //         range_length: None,
        //     }
        // ]).await;

        // let updated_diagnostics = server.receive_diagnostics().await;
        // // Should still have empty tasks error, but not missing name
        // assert!(updated_diagnostics.iter().all(|d| !d.message.contains("name")));
    }

    /// Test Case 6: Multiple Errors in Same Document
    ///
    /// Given: Pipeline with multiple validation errors
    /// When: Client opens the document
    /// Then: All errors are reported with accurate positions
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_multiple_errors() {
        // let (client, server) = create_test_lsp().await;

        // let invalid_pipeline = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata: {}  # ERROR: Missing name
        // spec:
        //   tasks: []   # ERROR: Empty tasks
        //   params:
        //     - value: "x"  # ERROR: Missing 'name' in param
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", invalid_pipeline).await;

        // let diagnostics = server.receive_diagnostics().await;
        // assert!(diagnostics.len() >= 3, "Should report all errors");

        // // Verify each error has unique position
        // let positions: Vec<_> = diagnostics.iter()
        //     .map(|d| (d.range.start.line, d.range.start.character))
        //     .collect();
        // let unique_positions: std::collections::HashSet<_> = positions.iter().collect();
        // assert_eq!(positions.len(), unique_positions.len(), "Each error should have unique position");
    }

    /// Test Case 7: Unknown Field Warning
    ///
    /// Given: Pipeline with unknown/typo field
    /// When: Client opens the document
    /// Then: Server publishes warning (not error) for unknown field
    #[tokio::test]
    #[ignore] // Enable in Task 3
    async fn test_unknown_field_warning() {
        // let (client, server) = create_test_lsp().await;

        // let pipeline_with_typo = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   name: test
        // spec:
        //   taskz: []  # WARN: Unknown field (typo: 'tasks' vs 'taskz')
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", pipeline_with_typo).await;

        // let diagnostics = server.receive_diagnostics().await;
        // let warnings: Vec<_> = diagnostics.iter()
        //     .filter(|d| d.severity == Some(DiagnosticSeverity::WARNING))
        //     .collect();

        // assert!(warnings.len() > 0);
        // assert!(warnings[0].message.contains("taskz") || warnings[0].message.contains("unknown"));
    }

    /// Test Case 8: Task Reference Validation
    ///
    /// Given: Pipeline referencing non-existent Task
    /// When: Client opens the document
    /// Then: Server warns about unresolved reference
    #[tokio::test]
    #[ignore] // Enable in Task 6 (requires cross-file reference resolution)
    async fn test_unresolved_task_reference() {
        // let (client, server) = create_test_lsp().await;

        // let pipeline = r#"
        // apiVersion: tekton.dev/v1
        // kind: Pipeline
        // metadata:
        //   name: test
        // spec:
        //   tasks:
        //     - name: build
        //       taskRef:
        //         name: non-existent-task  # WARN: Task not found
        // "#;

        // client.initialize().await;
        // client.did_open("file:///test/pipeline.yaml", pipeline).await;

        // let diagnostics = server.receive_diagnostics().await;
        // let warnings: Vec<_> = diagnostics.iter()
        //     .filter(|d| d.message.contains("non-existent-task"))
        //     .collect();

        // assert!(warnings.len() > 0);
    }
}

// Notes for Task 3 Implementation:
//
// 1. Test Helper Functions Needed:
//    - create_test_lsp() - Spawn LSP server in test mode
//    - TestClient - Send LSP messages (initialize, didOpen, etc.)
//    - TestServer - Receive LSP responses (diagnostics, etc.)
//
// 2. Validation Logic Needed:
//    - Tekton schema definitions (Pipeline, Task, etc.)
//    - Required field validation
//    - Type checking (array, string, object)
//    - Unknown field detection
//
// 3. Tree-sitter Integration:
//    - Use existing position tracking for accurate error locations
//    - Map AST nodes to Tekton schema fields
//    - Validate structure matches schema
//
// 4. TDD Workflow:
//    - Enable one test at a time (remove #[ignore])
//    - Verify test fails (RED)
//    - Implement minimal validation (GREEN)
//    - Refactor (REFACTOR)
//    - Repeat for next test
