mod cache;
mod completion;
mod definition;
mod hover;
mod parser;
mod server;
mod symbols;
mod validator;
mod workspace;

use clap::Parser;
use server::Backend;
use tower_lsp::{LspService, Server};

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
    let (service, socket) = LspService::new(Backend::new);

    // Run the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
