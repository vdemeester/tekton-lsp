use clap::Parser;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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

#[derive(Debug)]
struct Backend {
    client: Client,
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Document opened: {}", params.text_document.uri),
            )
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(
                MessageType::LOG,
                format!("Document changed: {}", params.text_document.uri),
            )
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Document closed: {}", params.text_document.uri),
            )
            .await;
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
    let (service, socket) = LspService::new(|client| Backend { client });

    // Run the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
