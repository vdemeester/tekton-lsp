//! LSP server implementation for Tekton.
//!
//! Contains the Backend struct and LanguageServer trait implementation.

use crate::cache::DocumentCache;
use crate::completion::CompletionProvider;
use crate::definition::DefinitionProvider;
use crate::formatting::FormattingProvider;
use crate::hover::HoverProvider;
use crate::parser;
use crate::symbols::SymbolsProvider;
use crate::validator::TektonValidator;
use crate::workspace::WorkspaceIndex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Backend state for the Tekton LSP server.
#[derive(Debug, Clone)]
pub struct Backend {
    client: Client,
    cache: DocumentCache,
    validator: TektonValidator,
    completion_provider: CompletionProvider,
    hover_provider: HoverProvider,
    definition_provider: DefinitionProvider,
    symbols_provider: SymbolsProvider,
    formatting_provider: FormattingProvider,
}

impl Backend {
    /// Create a new Backend instance with the given client.
    pub fn new(client: Client) -> Self {
        let workspace_index = WorkspaceIndex::new();
        Self {
            client,
            cache: DocumentCache::new(),
            validator: TektonValidator::new(),
            completion_provider: CompletionProvider::new(),
            hover_provider: HoverProvider::new(),
            definition_provider: DefinitionProvider::new(workspace_index),
            symbols_provider: SymbolsProvider::new(),
            formatting_provider: FormattingProvider::new(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "tekton-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".to_string(), " ".to_string(), "-".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Tekton LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document from cache
        if let Some(doc) = self.cache.get(uri) {
            // Parse the document
            match parser::parse_yaml(&uri.to_string(), &doc.content) {
                Ok(yaml_doc) => {
                    // Get completions from provider
                    let completions = self.completion_provider.provide_completions(&yaml_doc, position);

                    tracing::debug!(
                        "Providing {} completions at {}:{}",
                        completions.len(),
                        position.line,
                        position.character
                    );

                    Ok(Some(CompletionResponse::Array(completions)))
                }
                Err(e) => {
                    tracing::error!("Failed to parse YAML for completion: {}", e);
                    Ok(None)
                }
            }
        } else {
            tracing::warn!("Document not found in cache for completion: {}", uri);
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document from cache
        if let Some(doc) = self.cache.get(uri) {
            // Parse the document
            match parser::parse_yaml(&uri.to_string(), &doc.content) {
                Ok(yaml_doc) => {
                    // Get hover from provider
                    let hover = self.hover_provider.provide_hover(&yaml_doc, position);

                    tracing::debug!(
                        "Providing hover at {}:{}: {}",
                        position.line,
                        position.character,
                        hover.is_some()
                    );

                    Ok(hover)
                }
                Err(e) => {
                    tracing::error!("Failed to parse YAML for hover: {}", e);
                    Ok(None)
                }
            }
        } else {
            tracing::warn!("Document not found in cache for hover: {}", uri);
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document from cache
        if let Some(doc) = self.cache.get(uri) {
            // Parse the document
            match parser::parse_yaml(&uri.to_string(), &doc.content) {
                Ok(yaml_doc) => {
                    // Get definition from provider
                    let definition = self.definition_provider.provide_definition(&yaml_doc, position);

                    tracing::debug!(
                        "Providing definition at {}:{}: {}",
                        position.line,
                        position.character,
                        definition.is_some()
                    );

                    Ok(definition)
                }
                Err(e) => {
                    tracing::error!("Failed to parse YAML for definition: {}", e);
                    Ok(None)
                }
            }
        } else {
            tracing::warn!("Document not found in cache for definition: {}", uri);
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        // Get document from cache
        if let Some(doc) = self.cache.get(uri) {
            // Parse the document
            match parser::parse_yaml(&uri.to_string(), &doc.content) {
                Ok(yaml_doc) => {
                    // Get symbols from provider
                    let symbols = self.symbols_provider.provide_symbols(&yaml_doc);

                    tracing::debug!(
                        "Providing {} document symbols",
                        symbols.len()
                    );

                    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
                }
                Err(e) => {
                    tracing::error!("Failed to parse YAML for symbols: {}", e);
                    Ok(None)
                }
            }
        } else {
            tracing::warn!("Document not found in cache for symbols: {}", uri);
            Ok(None)
        }
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        // Get document from cache
        if let Some(doc) = self.cache.get(uri) {
            // Get formatting edits from provider
            let edits = self.formatting_provider.format(&doc.content);

            tracing::debug!(
                "Providing {} formatting edits",
                edits.as_ref().map(|e| e.len()).unwrap_or(0)
            );

            Ok(edits)
        } else {
            tracing::warn!("Document not found in cache for formatting: {}", uri);
            Ok(None)
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Document opened: {}", params.text_document.uri),
            )
            .await;

        // Add document to cache
        self.cache.insert(
            params.text_document.uri.clone(),
            params.text_document.language_id.clone(),
            params.text_document.version,
            params.text_document.text.clone(),
        );

        // Index document for go-to-definition
        if let Err(e) = self.definition_provider.index().index_document(
            &params.text_document.uri,
            &params.text_document.text,
        ) {
            tracing::warn!("Failed to index document: {}", e);
        }

        // Parse and validate the document
        if let Some(doc) = self.cache.get(&params.text_document.uri) {
            match parser::parse_yaml(&params.text_document.uri.to_string(), &doc.content) {
                Ok(yaml_doc) => {
                    tracing::debug!(
                        "Parsed document: kind={:?}, apiVersion={:?}",
                        yaml_doc.kind,
                        yaml_doc.api_version
                    );

                    // Validate and publish diagnostics
                    let diagnostics = self.validator.validate(&yaml_doc);

                    self.client
                        .publish_diagnostics(params.text_document.uri.clone(), diagnostics, None)
                        .await;
                }
                Err(e) => {
                    tracing::error!("Failed to parse YAML: {}", e);

                    // Publish parse error as diagnostic
                    self.client
                        .publish_diagnostics(
                            params.text_document.uri,
                            vec![Diagnostic {
                                range: Range {
                                    start: Position { line: 0, character: 0 },
                                    end: Position { line: 0, character: 0 },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: None,
                                code_description: None,
                                source: Some("tekton-lsp".to_string()),
                                message: format!("Failed to parse YAML: {}", e),
                                related_information: None,
                                tags: None,
                                data: None,
                            }],
                            None,
                        )
                        .await;
                }
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(
                MessageType::LOG,
                format!("Document changed: {}", params.text_document.uri),
            )
            .await;

        // Update document in cache
        self.cache.update(
            &params.text_document.uri,
            params.text_document.version,
            params.content_changes,
        );

        // Re-index document for go-to-definition
        if let Some(doc) = self.cache.get(&params.text_document.uri) {
            if let Err(e) = self.definition_provider.index().index_document(
                &params.text_document.uri,
                &doc.content,
            ) {
                tracing::warn!("Failed to re-index document: {}", e);
            }
        }

        // Re-validate after change
        if let Some(doc) = self.cache.get(&params.text_document.uri) {
            match parser::parse_yaml(&params.text_document.uri.to_string(), &doc.content) {
                Ok(yaml_doc) => {
                    // Validate and publish updated diagnostics
                    let diagnostics = self.validator.validate(&yaml_doc);

                    self.client
                        .publish_diagnostics(params.text_document.uri.clone(), diagnostics, None)
                        .await;
                }
                Err(e) => {
                    tracing::error!("Failed to parse YAML after change: {}", e);

                    // Publish parse error
                    self.client
                        .publish_diagnostics(
                            params.text_document.uri,
                            vec![Diagnostic {
                                range: Range {
                                    start: Position { line: 0, character: 0 },
                                    end: Position { line: 0, character: 0 },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: None,
                                code_description: None,
                                source: Some("tekton-lsp".to_string()),
                                message: format!("Failed to parse YAML: {}", e),
                                related_information: None,
                                tags: None,
                                data: None,
                            }],
                            None,
                        )
                        .await;
                }
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Document closed: {}", params.text_document.uri),
            )
            .await;

        // Remove document from workspace index
        self.definition_provider.index().remove_document(&params.text_document.uri);

        // Remove document from cache
        self.cache.remove(&params.text_document.uri);
    }
}
