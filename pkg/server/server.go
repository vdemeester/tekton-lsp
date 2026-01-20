package server

import (
	"context"
	"fmt"

	"go.lsp.dev/jsonrpc2"
	"go.lsp.dev/protocol"
	"go.uber.org/zap"
)

// Server implements the LSP server
type Server struct {
	logger *zap.Logger
	client protocol.Client

	// Server state
	initialized  bool
	shuttingDown bool
}

// NewServer creates a new LSP server instance
func NewServer(logger *zap.Logger) *Server {
	return &Server{
		logger: logger,
	}
}

// Serve starts serving LSP requests
func (s *Server) Serve(ctx context.Context, stream jsonrpc2.Stream) error {
	s.logger.Info("LSP server ready to accept connections")

	// Create handler
	handler := protocol.ServerHandler(&lspHandler{server: s}, nil)

	// Create connection with handler
	conn := jsonrpc2.NewConn(stream)
	conn.Go(ctx, handler)

	// Create client from connection
	client := protocol.ClientDispatcher(conn, s.logger)
	s.client = client

	<-conn.Done()
	return conn.Err()
}

// lspHandler implements protocol.Server interface
type lspHandler struct {
	server *Server
}

// Initialize handles the initialize request
func (h *lspHandler) Initialize(ctx context.Context, params *protocol.InitializeParams) (*protocol.InitializeResult, error) {
	h.server.logger.Info("received initialize request",
		zap.String("rootURI", string(params.RootURI)),
		zap.String("clientName", params.ClientInfo.Name),
	)

	if h.server.initialized {
		return nil, fmt.Errorf("server already initialized")
	}

	h.server.initialized = true

	return &protocol.InitializeResult{
		Capabilities: protocol.ServerCapabilities{
			TextDocumentSync: &protocol.TextDocumentSyncOptions{
				OpenClose: true,
				Change:    protocol.TextDocumentSyncKindIncremental,
			},
			// More capabilities will be added in later tasks
		},
		ServerInfo: &protocol.ServerInfo{
			Name:    "tekton-lsp",
			Version: "0.1.0",
		},
	}, nil
}

// Initialized handles the initialized notification
func (h *lspHandler) Initialized(ctx context.Context, params *protocol.InitializedParams) error {
	h.server.logger.Info("client confirmed initialization")
	return nil
}

// Shutdown handles the shutdown request
func (h *lspHandler) Shutdown(ctx context.Context) error {
	h.server.logger.Info("received shutdown request")
	h.server.shuttingDown = true
	return nil
}

// Exit handles the exit notification
func (h *lspHandler) Exit(ctx context.Context) error {
	h.server.logger.Info("received exit notification")
	if !h.server.shuttingDown {
		h.server.logger.Warn("exit without shutdown")
	}
	return nil
}

// Stub implementations for required protocol.Server methods
// These will be implemented in later tasks

func (h *lspHandler) CodeAction(ctx context.Context, params *protocol.CodeActionParams) ([]protocol.CodeAction, error) {
	return nil, nil
}

func (h *lspHandler) CodeLens(ctx context.Context, params *protocol.CodeLensParams) ([]protocol.CodeLens, error) {
	return nil, nil
}

func (h *lspHandler) CodeLensRefresh(ctx context.Context) error {
	return nil
}

func (h *lspHandler) CodeLensResolve(ctx context.Context, params *protocol.CodeLens) (*protocol.CodeLens, error) {
	return params, nil
}

func (h *lspHandler) ColorPresentation(ctx context.Context, params *protocol.ColorPresentationParams) ([]protocol.ColorPresentation, error) {
	return nil, nil
}

func (h *lspHandler) Completion(ctx context.Context, params *protocol.CompletionParams) (*protocol.CompletionList, error) {
	return nil, nil
}

func (h *lspHandler) CompletionResolve(ctx context.Context, params *protocol.CompletionItem) (*protocol.CompletionItem, error) {
	return params, nil
}

func (h *lspHandler) Declaration(ctx context.Context, params *protocol.DeclarationParams) ([]protocol.Location, error) {
	return nil, nil
}

func (h *lspHandler) Definition(ctx context.Context, params *protocol.DefinitionParams) ([]protocol.Location, error) {
	return nil, nil
}

func (h *lspHandler) DidChange(ctx context.Context, params *protocol.DidChangeTextDocumentParams) error {
	return nil
}

func (h *lspHandler) DidChangeConfiguration(ctx context.Context, params *protocol.DidChangeConfigurationParams) error {
	return nil
}

func (h *lspHandler) DidChangeWatchedFiles(ctx context.Context, params *protocol.DidChangeWatchedFilesParams) error {
	return nil
}

func (h *lspHandler) DidChangeWorkspaceFolders(ctx context.Context, params *protocol.DidChangeWorkspaceFoldersParams) error {
	return nil
}

func (h *lspHandler) DidCreateFiles(ctx context.Context, params *protocol.CreateFilesParams) error {
	return nil
}

func (h *lspHandler) DidDeleteFiles(ctx context.Context, params *protocol.DeleteFilesParams) error {
	return nil
}

func (h *lspHandler) DidRenameFiles(ctx context.Context, params *protocol.RenameFilesParams) error {
	return nil
}

func (h *lspHandler) DidClose(ctx context.Context, params *protocol.DidCloseTextDocumentParams) error {
	return nil
}

func (h *lspHandler) DidOpen(ctx context.Context, params *protocol.DidOpenTextDocumentParams) error {
	return nil
}

func (h *lspHandler) DidSave(ctx context.Context, params *protocol.DidSaveTextDocumentParams) error {
	return nil
}

func (h *lspHandler) DocumentColor(ctx context.Context, params *protocol.DocumentColorParams) ([]protocol.ColorInformation, error) {
	return nil, nil
}

func (h *lspHandler) DocumentHighlight(ctx context.Context, params *protocol.DocumentHighlightParams) ([]protocol.DocumentHighlight, error) {
	return nil, nil
}

func (h *lspHandler) DocumentLink(ctx context.Context, params *protocol.DocumentLinkParams) ([]protocol.DocumentLink, error) {
	return nil, nil
}

func (h *lspHandler) DocumentLinkResolve(ctx context.Context, params *protocol.DocumentLink) (*protocol.DocumentLink, error) {
	return params, nil
}

func (h *lspHandler) DocumentSymbol(ctx context.Context, params *protocol.DocumentSymbolParams) ([]interface{}, error) {
	return nil, nil
}

func (h *lspHandler) ExecuteCommand(ctx context.Context, params *protocol.ExecuteCommandParams) (interface{}, error) {
	return nil, nil
}

func (h *lspHandler) FoldingRange(ctx context.Context, params *protocol.FoldingRangeParams) ([]protocol.FoldingRange, error) {
	return nil, nil
}

func (h *lspHandler) FoldingRanges(ctx context.Context, params *protocol.FoldingRangeParams) ([]protocol.FoldingRange, error) {
	return nil, nil
}

func (h *lspHandler) Formatting(ctx context.Context, params *protocol.DocumentFormattingParams) ([]protocol.TextEdit, error) {
	return nil, nil
}

func (h *lspHandler) Hover(ctx context.Context, params *protocol.HoverParams) (*protocol.Hover, error) {
	return nil, nil
}

func (h *lspHandler) Implementation(ctx context.Context, params *protocol.ImplementationParams) ([]protocol.Location, error) {
	return nil, nil
}

func (h *lspHandler) IncomingCalls(ctx context.Context, params *protocol.CallHierarchyIncomingCallsParams) ([]protocol.CallHierarchyIncomingCall, error) {
	return nil, nil
}

func (h *lspHandler) LinkedEditingRange(ctx context.Context, params *protocol.LinkedEditingRangeParams) (*protocol.LinkedEditingRanges, error) {
	return nil, nil
}

func (h *lspHandler) LogTrace(ctx context.Context, params *protocol.LogTraceParams) error {
	return nil
}

func (h *lspHandler) Moniker(ctx context.Context, params *protocol.MonikerParams) ([]protocol.Moniker, error) {
	return nil, nil
}

func (h *lspHandler) OutgoingCalls(ctx context.Context, params *protocol.CallHierarchyOutgoingCallsParams) ([]protocol.CallHierarchyOutgoingCall, error) {
	return nil, nil
}

func (h *lspHandler) PrepareCallHierarchy(ctx context.Context, params *protocol.CallHierarchyPrepareParams) ([]protocol.CallHierarchyItem, error) {
	return nil, nil
}

func (h *lspHandler) OnTypeFormatting(ctx context.Context, params *protocol.DocumentOnTypeFormattingParams) ([]protocol.TextEdit, error) {
	return nil, nil
}

func (h *lspHandler) PrepareRename(ctx context.Context, params *protocol.PrepareRenameParams) (*protocol.Range, error) {
	return nil, nil
}

func (h *lspHandler) RangeFormatting(ctx context.Context, params *protocol.DocumentRangeFormattingParams) ([]protocol.TextEdit, error) {
	return nil, nil
}

func (h *lspHandler) References(ctx context.Context, params *protocol.ReferenceParams) ([]protocol.Location, error) {
	return nil, nil
}

func (h *lspHandler) Rename(ctx context.Context, params *protocol.RenameParams) (*protocol.WorkspaceEdit, error) {
	return nil, nil
}

func (h *lspHandler) Request(ctx context.Context, method string, params interface{}) (interface{}, error) {
	h.server.logger.Warn("unhandled request", zap.String("method", method))
	return nil, nil
}

func (h *lspHandler) SelectionRange(ctx context.Context, params *protocol.SelectionRangeParams) ([]protocol.SelectionRange, error) {
	return nil, nil
}

func (h *lspHandler) SemanticTokensFull(ctx context.Context, params *protocol.SemanticTokensParams) (*protocol.SemanticTokens, error) {
	return nil, nil
}

func (h *lspHandler) SemanticTokensFullDelta(ctx context.Context, params *protocol.SemanticTokensDeltaParams) (interface{}, error) {
	return nil, nil
}

func (h *lspHandler) SemanticTokensRange(ctx context.Context, params *protocol.SemanticTokensRangeParams) (*protocol.SemanticTokens, error) {
	return nil, nil
}

func (h *lspHandler) SemanticTokensRefresh(ctx context.Context) error {
	return nil
}

func (h *lspHandler) SetTrace(ctx context.Context, params *protocol.SetTraceParams) error {
	return nil
}

func (h *lspHandler) ShowDocument(ctx context.Context, params *protocol.ShowDocumentParams) (*protocol.ShowDocumentResult, error) {
	return nil, nil
}

func (h *lspHandler) SignatureHelp(ctx context.Context, params *protocol.SignatureHelpParams) (*protocol.SignatureHelp, error) {
	return nil, nil
}

func (h *lspHandler) Symbols(ctx context.Context, params *protocol.WorkspaceSymbolParams) ([]protocol.SymbolInformation, error) {
	return nil, nil
}

func (h *lspHandler) TypeDefinition(ctx context.Context, params *protocol.TypeDefinitionParams) ([]protocol.Location, error) {
	return nil, nil
}

func (h *lspHandler) WillCreateFiles(ctx context.Context, params *protocol.CreateFilesParams) (*protocol.WorkspaceEdit, error) {
	return nil, nil
}

func (h *lspHandler) WillDeleteFiles(ctx context.Context, params *protocol.DeleteFilesParams) (*protocol.WorkspaceEdit, error) {
	return nil, nil
}

func (h *lspHandler) WillRenameFiles(ctx context.Context, params *protocol.RenameFilesParams) (*protocol.WorkspaceEdit, error) {
	return nil, nil
}

func (h *lspHandler) WillSave(ctx context.Context, params *protocol.WillSaveTextDocumentParams) error {
	return nil
}

func (h *lspHandler) WillSaveWaitUntil(ctx context.Context, params *protocol.WillSaveTextDocumentParams) ([]protocol.TextEdit, error) {
	return nil, nil
}

func (h *lspHandler) WorkDoneProgressCancel(ctx context.Context, params *protocol.WorkDoneProgressCancelParams) error {
	return nil
}
