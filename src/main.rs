mod cache;
mod parser;
mod validator;
mod completion;

use cache::DocumentCache;
use completion::CompletionProvider;
use clap::Parser;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use validator::TektonValidator;

/// Tekton Language Server Protocol (LSP) implementation
#[derive(Parser, Debug)]
#[command(
    name = "tekton-lsp",
    version,
    about = "Language Server Protocol implementation for Tekton YAML files",
    long_about = "A Language Server providing IDE features for Tekton Pipelines, Tasks, and related resources.\n\nFeatures:\n  - Diagnostics (validation)\n  - Completion\n  - Hover documentation\n  - Go-to-definition\n  - Find references\n  - Document symbols"
)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone)]
struct Backend {
    client: Client,
    cache: DocumentCache,
    validator: TektonValidator,
    completion_provider: CompletionProvider,
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
            params.text_document.language_id,
            params.text_document.version,
            params.text_document.text,
        );

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

        // Remove document from cache
        self.cache.remove(&params.text_document.uri);
    }
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize tracing with appropriate level
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Tekton LSP server (version {})", env!("CARGO_PKG_VERSION"));

    // Create stdio transport
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create the LSP service
    let (service, socket) = LspService::new(|client| Backend {
        client,
        cache: DocumentCache::new(),
        validator: TektonValidator::new(),
        completion_provider: CompletionProvider::new(),
    });

    // Run the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
