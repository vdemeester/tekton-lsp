package main

import (
	"context"
	"fmt"
	"io"
	"os"

	"github.com/tektoncd/tekton-lsp/pkg/server"
	"go.lsp.dev/jsonrpc2"
	"go.uber.org/zap"
)

func main() {
	// Initialize logger
	logger, err := zap.NewDevelopment()
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer logger.Sync()

	// Create LSP server
	srv := server.NewServer(logger)

	// Create JSON-RPC 2.0 stream using stdio
	// Combine stdin and stdout into a single ReadWriteCloser
	stream := jsonrpc2.NewStream(struct {
		io.Reader
		io.Writer
		io.Closer
	}{
		Reader: os.Stdin,
		Writer: os.Stdout,
		Closer: os.Stdin,
	})

	ctx := context.Background()

	logger.Info("Tekton LSP server starting")

	// Serve requests
	if err := srv.Serve(ctx, stream); err != nil {
		logger.Error("server error", zap.Error(err))
		os.Exit(1)
	}
}
