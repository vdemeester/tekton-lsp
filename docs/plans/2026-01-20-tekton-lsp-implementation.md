# Tekton Language Server Protocol (LSP) Implementation Plan

**Status**: Draft
**Created**: 2026-01-20
**Goal**: Build a production-ready LSP for Tekton YAML files with full IDE support
**Architecture**: Go-based LSP server using jsonrpc2, integrating Tekton API types for validation, YAML parsing with gopkg.in/yaml.v3, and comprehensive LSP feature support (diagnostics, completion, hover, go-to-definition, references, symbols, formatting)
**Tech Stack**: Go 1.21+, go-lsp/protocol, Tekton Pipeline/Triggers APIs, gopkg.in/yaml.v3, Kubernetes client-go

## Prerequisites

- [ ] Review LSP specification at https://microsoft.github.io/language-server-protocol/
- [ ] Review existing Go LSP implementations (gopls architecture)
- [ ] Understand Tekton CRD schemas (Pipeline, Task, Trigger, EventListener, etc.)
- [ ] Set up Go 1.21+ development environment
- [ ] Review YAML parsing with gopkg.in/yaml.v3

## Architecture Overview

```
tekton-lsp/
├── cmd/
│   └── tekton-lsp/          # LSP server binary entry point
├── pkg/
│   ├── server/              # LSP server implementation
│   ├── protocol/            # LSP protocol handlers
│   ├── parser/              # YAML parsing and AST building
│   ├── analyzer/            # Tekton resource analysis and validation
│   ├── completion/          # Completion provider
│   ├── hover/               # Hover documentation provider
│   ├── definition/          # Go-to-definition provider
│   ├── references/          # Find references provider
│   ├── symbols/             # Document/workspace symbols
│   └── formatting/          # YAML formatting
├── internal/
│   ├── cache/               # Document cache and workspace state
│   └── utils/               # Shared utilities
├── test/
│   ├── testdata/            # Sample Tekton YAML files for testing
│   └── integration/         # Integration tests
└── docs/
    ├── architecture.md
    └── plans/
```

**Key Design Decisions:**
1. **Reuse Tekton types**: Import Tekton Pipeline and Triggers as Go modules to leverage existing type definitions
2. **YAML-aware**: Build position-aware AST from YAML for accurate LSP features
3. **Workspace-aware**: Track all Tekton resources in workspace for cross-file navigation
4. **Incremental updates**: Support incremental document changes for performance
5. **Schema-driven**: Use Tekton CRD schemas for validation and completion

---

## Phase 1: Foundation

### Task 1: Project Setup and LSP Server Scaffold

**Purpose**: Initialize Go module with basic LSP server that can start and handle initialize/shutdown

**Files:**
- Create: `cmd/tekton-lsp/main.go`
- Create: `pkg/server/server.go`
- Create: `go.mod`
- Create: `go.sum`
- Create: `.gitignore`
- Test: Manual stdio test with LSP client

**Dependencies**: None

---

#### Step 1: Initialize Go module

```bash
cd /home/vincent/src/tektoncd/tekton-lsp
go mod init github.com/tektoncd/tekton-lsp
```

**Expected output:**
```
go: creating new go.mod: module github.com/tektoncd/tekton-lsp
```

---

#### Step 2: Add core dependencies

```bash
go get go.lsp.dev/protocol@latest
go get go.lsp.dev/jsonrpc2@latest
go get go.uber.org/zap@latest
go get gopkg.in/yaml.v3@latest
```

**Expected output:**
```
go: downloading go.lsp.dev/protocol v0.12.0
go: downloading go.lsp.dev/jsonrpc2 v0.10.0
...
```

---

#### Step 3: Create main entry point

Create `cmd/tekton-lsp/main.go`:

```go
package main

import (
	"context"
	"fmt"
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
	stream := jsonrpc2.NewStream(
		jsonrpc2.NewReader(os.Stdin),
		jsonrpc2.NewWriter(os.Stdout),
	)

	// Create and run connection
	conn := jsonrpc2.NewConn(stream)
	ctx := context.Background()

	logger.Info("Tekton LSP server starting")

	// Serve requests
	if err := srv.Serve(ctx, conn); err != nil {
		logger.Error("server error", zap.Error(err))
		os.Exit(1)
	}
}
```

**Implementation notes**: Using stdio for LSP communication (standard for LSP servers), zap for structured logging

---

#### Step 4: Create basic server structure

Create `pkg/server/server.go`:

```go
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
	initialized bool
	shuttingDown bool
}

// NewServer creates a new LSP server instance
func NewServer(logger *zap.Logger) *Server {
	return &Server{
		logger: logger,
	}
}

// Serve starts serving LSP requests
func (s *Server) Serve(ctx context.Context, conn jsonrpc2.Conn) error {
	s.logger.Info("LSP server ready to accept connections")

	// Handle requests using jsonrpc2
	handler := protocol.ServerHandler(&lspHandler{server: s}, nil)

	return conn.Run(ctx, handler)
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

func (h *lspHandler) ColorPresentation(ctx context.Context, params *protocol.ColorPresentationParams) ([]protocol.ColorPresentation, error) {
	return nil, nil
}

func (h *lspHandler) Completion(ctx context.Context, params *protocol.CompletionParams) (*protocol.CompletionList, error) {
	return nil, nil
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

func (h *lspHandler) DocumentSymbol(ctx context.Context, params *protocol.DocumentSymbolParams) ([]protocol.DocumentSymbol, error) {
	return nil, nil
}

func (h *lspHandler) FoldingRange(ctx context.Context, params *protocol.FoldingRangeParams) ([]protocol.FoldingRange, error) {
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

func (h *lspHandler) SignatureHelp(ctx context.Context, params *protocol.SignatureHelpParams) (*protocol.SignatureHelp, error) {
	return nil, nil
}

func (h *lspHandler) Symbols(ctx context.Context, params *protocol.WorkspaceSymbolParams) ([]protocol.SymbolInformation, error) {
	return nil, nil
}

func (h *lspHandler) TypeDefinition(ctx context.Context, params *protocol.TypeDefinitionParams) ([]protocol.Location, error) {
	return nil, nil
}

func (h *lspHandler) WillSave(ctx context.Context, params *protocol.WillSaveTextDocumentParams) error {
	return nil
}

func (h *lspHandler) WillSaveWaitUntil(ctx context.Context, params *protocol.WillSaveTextDocumentParams) ([]protocol.TextEdit, error) {
	return nil, nil
}
```

**Implementation notes**: Minimal LSP server that handles initialize/initialized/shutdown/exit lifecycle. All other methods are stubbed for now.

---

#### Step 5: Create .gitignore

Create `.gitignore`:

```gitignore
# Binaries
tekton-lsp
*.exe
*.exe~
*.dll
*.so
*.dylib

# Test binary, built with `go test -c`
*.test

# Output of the go coverage tool
*.out

# Go workspace file
go.work

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Build artifacts
bin/
dist/
```

---

#### Step 6: Build and test the server

**Build command:**
```bash
cd /home/vincent/src/tektoncd/tekton-lsp
go build -o bin/tekton-lsp ./cmd/tekton-lsp
```

**Expected output:**
```
(successful build with no output)
```

**Manual test:** Start the server and send an initialize request
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":"file:///tmp/test","capabilities":{}}}' | ./bin/tekton-lsp
```

**Expected behavior:**
- Server starts and waits for input
- Receives initialize request
- Responds with initialize result containing server capabilities
- Logs initialization to stderr

---

#### Step 7: Commit

```bash
git init
git add .
git commit -m "feat: initial LSP server scaffold

- Add basic Go module structure
- Implement LSP initialize/shutdown lifecycle
- Set up jsonrpc2 stdio communication
- Add stub implementations for LSP protocol methods"
```

---

### Task 2: Document Management and YAML Parsing

**Purpose**: Implement document lifecycle (open/change/close) and parse YAML with position tracking

**Files:**
- Create: `internal/cache/document.go`
- Create: `internal/cache/cache.go`
- Create: `pkg/parser/parser.go`
- Create: `pkg/parser/ast.go`
- Test: `pkg/parser/parser_test.go`
- Modify: `pkg/server/server.go` (add document handlers)

**Dependencies**: Task 1 must be completed

---

#### Step 1: Write test for document cache

Create `internal/cache/cache_test.go`:

```go
package cache

import (
	"testing"

	"go.lsp.dev/protocol"
	"go.lsp.dev/uri"
)

func TestCache_OpenDocument(t *testing.T) {
	cache := NewCache()

	testURI := uri.File("/test/pipeline.yaml")
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline`

	doc, err := cache.Open(testURI, "yaml", 1, content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if doc.URI != testURI {
		t.Errorf("expected URI %v, got %v", testURI, doc.URI)
	}

	if doc.Version != 1 {
		t.Errorf("expected version 1, got %d", doc.Version)
	}

	if doc.Content != content {
		t.Errorf("content mismatch")
	}

	// Verify document is retrievable
	retrieved, ok := cache.Get(testURI)
	if !ok {
		t.Fatal("document not found in cache")
	}

	if retrieved != doc {
		t.Error("retrieved document doesn't match opened document")
	}
}

func TestCache_UpdateDocument(t *testing.T) {
	cache := NewCache()

	testURI := uri.File("/test/pipeline.yaml")
	initialContent := "apiVersion: tekton.dev/v1"

	doc, _ := cache.Open(testURI, "yaml", 1, initialContent)

	// Apply incremental change
	changes := []protocol.TextDocumentContentChangeEvent{
		{
			Range: &protocol.Range{
				Start: protocol.Position{Line: 1, Character: 0},
				End:   protocol.Position{Line: 1, Character: 0},
			},
			Text: "kind: Pipeline\n",
		},
	}

	err := cache.Update(testURI, 2, changes)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if doc.Version != 2 {
		t.Errorf("expected version 2, got %d", doc.Version)
	}

	expectedContent := `apiVersion: tekton.dev/v1
kind: Pipeline
`
	if doc.Content != expectedContent {
		t.Errorf("expected content:\n%s\ngot:\n%s", expectedContent, doc.Content)
	}
}

func TestCache_CloseDocument(t *testing.T) {
	cache := NewCache()

	testURI := uri.File("/test/pipeline.yaml")
	cache.Open(testURI, "yaml", 1, "test content")

	cache.Close(testURI)

	_, ok := cache.Get(testURI)
	if ok {
		t.Error("document should have been removed from cache")
	}
}
```

**Why this test**: Verifies document lifecycle management in cache

---

#### Step 2: Run test to verify it fails

**Command:**
```bash
cd /home/vincent/src/tektoncd/tekton-lsp
go test ./internal/cache -v
```

**Expected output:**
```
# github.com/tektoncd/tekton-lsp/internal/cache [github.com/tektoncd/tekton-lsp/internal/cache.test]
./cache_test.go:11:17: undefined: NewCache
./cache_test.go:17:17: undefined: uri.File
```

---

#### Step 3: Implement document and cache

Create `internal/cache/document.go`:

```go
package cache

import (
	"fmt"
	"strings"

	"go.lsp.dev/protocol"
	"go.lsp.dev/uri"
)

// Document represents a text document in the workspace
type Document struct {
	URI     uri.URI
	LanguageID string
	Version int32
	Content string
}

// ApplyChanges applies incremental changes to the document content
func (d *Document) ApplyChanges(changes []protocol.TextDocumentContentChangeEvent) error {
	for _, change := range changes {
		if change.Range == nil {
			// Full document sync
			d.Content = change.Text
			continue
		}

		// Incremental sync
		lines := strings.Split(d.Content, "\n")

		// Extract the range
		start := change.Range.Start
		end := change.Range.End

		if int(start.Line) >= len(lines) {
			return fmt.Errorf("start line %d out of range", start.Line)
		}

		// Build new content
		var result strings.Builder

		// Lines before change
		for i := 0; i < int(start.Line); i++ {
			result.WriteString(lines[i])
			result.WriteString("\n")
		}

		// Start line with prefix before change
		startLine := lines[start.Line]
		if int(start.Character) > len(startLine) {
			return fmt.Errorf("start character %d out of range", start.Character)
		}
		result.WriteString(startLine[:start.Character])

		// New text
		result.WriteString(change.Text)

		// End line with suffix after change
		if int(end.Line) < len(lines) {
			endLine := lines[end.Line]
			if int(end.Character) <= len(endLine) {
				result.WriteString(endLine[end.Character:])
			}
			result.WriteString("\n")
		}

		// Lines after change
		for i := int(end.Line) + 1; i < len(lines); i++ {
			result.WriteString(lines[i])
			if i < len(lines)-1 {
				result.WriteString("\n")
			}
		}

		d.Content = strings.TrimSuffix(result.String(), "\n")
	}

	return nil
}
```

Create `internal/cache/cache.go`:

```go
package cache

import (
	"fmt"
	"sync"

	"go.lsp.dev/protocol"
	"go.lsp.dev/uri"
)

// Cache manages all open documents in the workspace
type Cache struct {
	mu        sync.RWMutex
	documents map[uri.URI]*Document
}

// NewCache creates a new document cache
func NewCache() *Cache {
	return &Cache{
		documents: make(map[uri.URI]*Document),
	}
}

// Open adds a new document to the cache
func (c *Cache) Open(uri uri.URI, languageID string, version int32, content string) (*Document, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if _, exists := c.documents[uri]; exists {
		return nil, fmt.Errorf("document %s already open", uri)
	}

	doc := &Document{
		URI:        uri,
		LanguageID: languageID,
		Version:    version,
		Content:    content,
	}

	c.documents[uri] = doc
	return doc, nil
}

// Get retrieves a document from the cache
func (c *Cache) Get(uri uri.URI) (*Document, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	doc, ok := c.documents[uri]
	return doc, ok
}

// Update applies changes to a document
func (c *Cache) Update(uri uri.URI, version int32, changes []protocol.TextDocumentContentChangeEvent) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	doc, ok := c.documents[uri]
	if !ok {
		return fmt.Errorf("document %s not found", uri)
	}

	doc.Version = version
	return doc.ApplyChanges(changes)
}

// Close removes a document from the cache
func (c *Cache) Close(uri uri.URI) {
	c.mu.Lock()
	defer c.mu.Unlock()

	delete(c.documents, uri)
}

// All returns all documents in the cache
func (c *Cache) All() []*Document {
	c.mu.RLock()
	defer c.mu.RUnlock()

	docs := make([]*Document, 0, len(c.documents))
	for _, doc := range c.documents {
		docs = append(docs, doc)
	}
	return docs
}
```

**Implementation notes**: Thread-safe document cache with incremental update support

---

#### Step 4: Run tests to verify they pass

**Command:**
```bash
go test ./internal/cache -v
```

**Expected output:**
```
=== RUN   TestCache_OpenDocument
--- PASS: TestCache_OpenDocument (0.00s)
=== RUN   TestCache_UpdateDocument
--- PASS: TestCache_UpdateDocument (0.00s)
=== RUN   TestCache_CloseDocument
--- PASS: TestCache_CloseDocument (0.00s)
PASS
ok      github.com/tektoncd/tekton-lsp/internal/cache   0.003s
```

---

#### Step 5: Write test for YAML parser

Create `pkg/parser/parser_test.go`:

```go
package parser

import (
	"testing"
)

func TestParser_ParsePipeline(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline
spec:
  tasks:
    - name: task1
      taskRef:
        name: build-task`

	doc, err := Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	if doc.APIVersion != "tekton.dev/v1" {
		t.Errorf("expected apiVersion tekton.dev/v1, got %s", doc.APIVersion)
	}

	if doc.Kind != "Pipeline" {
		t.Errorf("expected kind Pipeline, got %s", doc.Kind)
	}

	// Verify we can access nested fields
	if doc.Root == nil {
		t.Fatal("root node is nil")
	}

	// Check metadata.name
	metadata, ok := doc.Root.Get("metadata")
	if !ok {
		t.Fatal("metadata field not found")
	}

	name, ok := metadata.Get("name")
	if !ok {
		t.Fatal("metadata.name not found")
	}

	if name.Value != "test-pipeline" {
		t.Errorf("expected name test-pipeline, got %v", name.Value)
	}
}

func TestParser_PositionTracking(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test-pipeline`

	doc, err := Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	// Find the "kind" field
	kindNode, ok := doc.Root.Get("kind")
	if !ok {
		t.Fatal("kind field not found")
	}

	// Verify position information
	if kindNode.Range.Start.Line != 1 {
		t.Errorf("expected kind at line 1, got %d", kindNode.Range.Start.Line)
	}
}
```

**Why this test**: Verifies YAML parsing with position tracking for LSP features

---

#### Step 6: Run test to verify it fails

**Command:**
```bash
go test ./pkg/parser -v
```

**Expected output:**
```
# github.com/tektoncd/tekton-lsp/pkg/parser [github.com/tektoncd/tekton-lsp/pkg/parser.test]
./parser_test.go:10:12: undefined: Parse
```

---

#### Step 7: Implement YAML parser with position tracking

Create `pkg/parser/ast.go`:

```go
package parser

import (
	"go.lsp.dev/protocol"
)

// Node represents a node in the YAML AST with position information
type Node struct {
	Key      string
	Value    interface{}
	Range    protocol.Range
	Children map[string]*Node
	Items    []*Node // for arrays
}

// Get retrieves a child node by key
func (n *Node) Get(key string) (*Node, bool) {
	if n.Children == nil {
		return nil, false
	}
	child, ok := n.Children[key]
	return child, ok
}

// Document represents a parsed YAML document
type Document struct {
	Filename   string
	APIVersion string
	Kind       string
	Root       *Node
}
```

Create `pkg/parser/parser.go`:

```go
package parser

import (
	"fmt"
	"strings"

	"go.lsp.dev/protocol"
	"gopkg.in/yaml.v3"
)

// Parse parses YAML content and builds an AST with position information
func Parse(filename string, content []byte) (*Document, error) {
	var root yaml.Node
	if err := yaml.Unmarshal(content, &root); err != nil {
		return nil, fmt.Errorf("yaml unmarshal error: %w", err)
	}

	// The root is typically a document node
	if root.Kind != yaml.DocumentNode || len(root.Content) == 0 {
		return nil, fmt.Errorf("invalid YAML structure")
	}

	// The document should contain a mapping node
	mappingNode := root.Content[0]
	if mappingNode.Kind != yaml.MappingNode {
		return nil, fmt.Errorf("expected mapping at root, got %v", mappingNode.Kind)
	}

	doc := &Document{
		Filename: filename,
	}

	// Build our AST
	doc.Root = buildNode(mappingNode)

	// Extract common Kubernetes/Tekton fields
	if apiVersion, ok := doc.Root.Get("apiVersion"); ok {
		if str, ok := apiVersion.Value.(string); ok {
			doc.APIVersion = str
		}
	}

	if kind, ok := doc.Root.Get("kind"); ok {
		if str, ok := kind.Value.(string); ok {
			doc.Kind = str
		}
	}

	return doc, nil
}

// buildNode converts a yaml.Node to our Node type with position tracking
func buildNode(n *yaml.Node) *Node {
	node := &Node{
		Range: protocol.Range{
			Start: protocol.Position{
				Line:      uint32(n.Line - 1),      // yaml.Node lines are 1-based
				Character: uint32(n.Column - 1),    // yaml.Node columns are 1-based
			},
			End: protocol.Position{
				Line:      uint32(n.Line - 1),
				Character: uint32(n.Column - 1 + len(n.Value)),
			},
		},
	}

	switch n.Kind {
	case yaml.ScalarNode:
		node.Value = n.Value

	case yaml.MappingNode:
		node.Children = make(map[string]*Node)
		// Process key-value pairs
		for i := 0; i < len(n.Content); i += 2 {
			keyNode := n.Content[i]
			valueNode := n.Content[i+1]

			key := keyNode.Value
			childNode := buildNode(valueNode)
			childNode.Key = key
			node.Children[key] = childNode
		}

	case yaml.SequenceNode:
		node.Items = make([]*Node, 0, len(n.Content))
		for _, item := range n.Content {
			node.Items = append(node.Items, buildNode(item))
		}
	}

	return node
}

// FindNodeAtPosition finds the node at a specific position in the document
func (d *Document) FindNodeAtPosition(pos protocol.Position) *Node {
	if d.Root == nil {
		return nil
	}
	return findNodeAtPosition(d.Root, pos)
}

func findNodeAtPosition(node *Node, pos protocol.Position) *Node {
	// Check if position is within this node's range
	if !positionInRange(pos, node.Range) {
		return nil
	}

	// Check children first (depth-first search for most specific node)
	if node.Children != nil {
		for _, child := range node.Children {
			if found := findNodeAtPosition(child, pos); found != nil {
				return found
			}
		}
	}

	// Check array items
	if node.Items != nil {
		for _, item := range node.Items {
			if found := findNodeAtPosition(item, pos); found != nil {
				return found
			}
		}
	}

	// This is the most specific node containing the position
	return node
}

func positionInRange(pos protocol.Position, r protocol.Range) bool {
	if pos.Line < r.Start.Line || pos.Line > r.End.Line {
		return false
	}
	if pos.Line == r.Start.Line && pos.Character < r.Start.Character {
		return false
	}
	if pos.Line == r.End.Line && pos.Character > r.End.Character {
		return false
	}
	return true
}
```

**Implementation notes**: Using gopkg.in/yaml.v3 which provides position information. Building custom AST for easier LSP operations.

---

#### Step 8: Run tests to verify they pass

**Command:**
```bash
go test ./pkg/parser -v
```

**Expected output:**
```
=== RUN   TestParser_ParsePipeline
--- PASS: TestParser_ParsePipeline (0.00s)
=== RUN   TestParser_PositionTracking
--- PASS: TestParser_PositionTracking (0.00s)
PASS
ok      github.com/tektoncd/tekton-lsp/pkg/parser       0.004s
```

---

#### Step 9: Integrate document cache with server

Modify `pkg/server/server.go` to add document handling:

```go
// Add to Server struct:
type Server struct {
	logger *zap.Logger
	client protocol.Client
	cache  *cache.Cache  // ADD THIS

	initialized bool
	shuttingDown bool
}

// Update NewServer:
func NewServer(logger *zap.Logger) *Server {
	return &Server{
		logger: logger,
		cache:  cache.NewCache(),  // ADD THIS
	}
}

// Implement DidOpen:
func (h *lspHandler) DidOpen(ctx context.Context, params *protocol.DidOpenTextDocumentParams) error {
	h.server.logger.Info("document opened",
		zap.String("uri", string(params.TextDocument.URI)),
	)

	doc, err := h.server.cache.Open(
		params.TextDocument.URI,
		params.TextDocument.LanguageID,
		params.TextDocument.Version,
		params.TextDocument.Text,
	)
	if err != nil {
		return err
	}

	h.server.logger.Debug("document cached",
		zap.String("uri", string(doc.URI)),
		zap.Int("size", len(doc.Content)),
	)

	return nil
}

// Implement DidChange:
func (h *lspHandler) DidChange(ctx context.Context, params *protocol.DidChangeTextDocumentParams) error {
	return h.server.cache.Update(
		params.TextDocument.URI,
		params.TextDocument.Version,
		params.ContentChanges,
	)
}

// Implement DidClose:
func (h *lspHandler) DidClose(ctx context.Context, params *protocol.DidCloseTextDocumentParams) error {
	h.server.logger.Info("document closed",
		zap.String("uri", string(params.TextDocument.URI)),
	)
	h.server.cache.Close(params.TextDocument.URI)
	return nil
}
```

Add import at top of file:
```go
import (
	// ... existing imports
	"github.com/tektoncd/tekton-lsp/internal/cache"
)
```

---

#### Step 10: Test document lifecycle

**Command:**
```bash
go build -o bin/tekton-lsp ./cmd/tekton-lsp
```

**Expected output:**
```
(successful build)
```

**Manual test**: Test document open/change/close lifecycle with LSP client

---

#### Step 11: Commit

```bash
git add .
git commit -m "feat: add document management and YAML parsing

- Implement document cache with incremental updates
- Add YAML parser with position tracking
- Build AST from YAML for LSP operations
- Integrate document lifecycle with LSP server
- Add FindNodeAtPosition for cursor-based operations"
```

---

## Phase 2: Diagnostics (Validation)

### Task 3: Tekton Resource Validation

**Purpose**: Add Tekton CRD schema validation and publish diagnostics for errors

**Files:**
- Create: `pkg/analyzer/validator.go`
- Create: `pkg/analyzer/validator_test.go`
- Create: `pkg/analyzer/tekton.go` (Tekton-specific validation rules)
- Modify: `pkg/server/server.go` (publish diagnostics on document changes)
- Add dependency: `github.com/tektoncd/pipeline` module

**Dependencies**: Task 2 must be completed

---

#### Step 1: Add Tekton dependencies

```bash
go get github.com/tektoncd/pipeline@latest
go get k8s.io/apimachinery@latest
```

---

#### Step 2: Write test for Pipeline validation

Create `pkg/analyzer/validator_test.go`:

```go
package analyzer

import (
	"testing"

	"github.com/tektoncd/tekton-lsp/pkg/parser"
)

func TestValidator_ValidPipeline(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: valid-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task`

	doc, err := parser.Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	validator := NewValidator()
	diagnostics := validator.Validate(doc)

	if len(diagnostics) != 0 {
		t.Errorf("expected no diagnostics, got %d: %+v", len(diagnostics), diagnostics)
	}
}

func TestValidator_MissingRequiredFields(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: invalid-pipeline
spec:
  tasks:
    - taskRef:
        name: build-task`

	doc, err := parser.Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	validator := NewValidator()
	diagnostics := validator.Validate(doc)

	if len(diagnostics) == 0 {
		t.Error("expected diagnostics for missing task name")
	}

	// Check that we got an error about missing 'name'
	found := false
	for _, diag := range diagnostics {
		if contains(diag.Message, "name") && contains(diag.Message, "required") {
			found = true
			break
		}
	}

	if !found {
		t.Errorf("expected diagnostic about missing 'name' field, got: %+v", diagnostics)
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > len(substr) &&
		(s[:len(substr)] == substr || s[len(s)-len(substr):] == substr ||
		 indexOf(s, substr) >= 0))
}

func indexOf(s, substr string) int {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return i
		}
	}
	return -1
}
```

**Why this test**: Verifies validation detects missing required fields

---

#### Step 3: Run test to verify it fails

**Command:**
```bash
go test ./pkg/analyzer -v
```

**Expected output:**
```
./validator_test.go:13:16: undefined: NewValidator
```

---

#### Step 4: Implement validator

Create `pkg/analyzer/validator.go`:

```go
package analyzer

import (
	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

// Validator validates Tekton resources
type Validator struct {
	// Future: add schema validation, CRD validation
}

// NewValidator creates a new validator instance
func NewValidator() *Validator {
	return &Validator{}
}

// Validate validates a parsed document and returns diagnostics
func (v *Validator) Validate(doc *parser.Document) []protocol.Diagnostic {
	var diagnostics []protocol.Diagnostic

	// Validate based on resource kind
	switch doc.Kind {
	case "Pipeline":
		diagnostics = append(diagnostics, v.validatePipeline(doc)...)
	case "Task":
		diagnostics = append(diagnostics, v.validateTask(doc)...)
	case "PipelineRun":
		diagnostics = append(diagnostics, v.validatePipelineRun(doc)...)
	case "TaskRun":
		diagnostics = append(diagnostics, v.validateTaskRun(doc)...)
	default:
		// Unknown or unsupported kind
		if doc.Kind != "" {
			kindNode, _ := doc.Root.Get("kind")
			if kindNode != nil {
				diagnostics = append(diagnostics, protocol.Diagnostic{
					Range:    kindNode.Range,
					Severity: protocol.DiagnosticSeverityWarning,
					Source:   "tekton-lsp",
					Message:  "Unknown Tekton resource kind: " + doc.Kind,
				})
			}
		}
	}

	return diagnostics
}

func (v *Validator) validatePipeline(doc *parser.Document) []protocol.Diagnostic {
	var diagnostics []protocol.Diagnostic

	// Validate metadata.name is present
	diagnostics = append(diagnostics, validateMetadataName(doc)...)

	// Validate spec exists
	spec, ok := doc.Root.Get("spec")
	if !ok {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    doc.Root.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "Pipeline spec is required",
		})
		return diagnostics
	}

	// Validate tasks array
	tasks, ok := spec.Get("tasks")
	if !ok {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    spec.Range,
			Severity: protocol.DiagnosticSeverityWarning,
			Source:   "tekton-lsp",
			Message:  "Pipeline should define tasks",
		})
		return diagnostics
	}

	// Validate each task
	if tasks.Items != nil {
		for i, task := range tasks.Items {
			// Each task must have a name
			name, ok := task.Get("name")
			if !ok {
				diagnostics = append(diagnostics, protocol.Diagnostic{
					Range:    task.Range,
					Severity: protocol.DiagnosticSeverityError,
					Source:   "tekton-lsp",
					Message:  "Task name is required",
				})
				continue
			}

			if nameStr, ok := name.Value.(string); !ok || nameStr == "" {
				diagnostics = append(diagnostics, protocol.Diagnostic{
					Range:    name.Range,
					Severity: protocol.DiagnosticSeverityError,
					Source:   "tekton-lsp",
					Message:  "Task name cannot be empty",
				})
			}

			// Task must have either taskRef or taskSpec
			hasTaskRef := false
			hasTaskSpec := false

			if _, ok := task.Get("taskRef"); ok {
				hasTaskRef = true
			}
			if _, ok := task.Get("taskSpec"); ok {
				hasTaskSpec = true
			}

			if !hasTaskRef && !hasTaskSpec {
				diagnostics = append(diagnostics, protocol.Diagnostic{
					Range:    task.Range,
					Severity: protocol.DiagnosticSeverityError,
					Source:   "tekton-lsp",
					Message:  "Task must have either taskRef or taskSpec",
				})
			}

			if hasTaskRef && hasTaskSpec {
				diagnostics = append(diagnostics, protocol.Diagnostic{
					Range:    task.Range,
					Severity: protocol.DiagnosticSeverityError,
					Source:   "tekton-lsp",
					Message:  "Task cannot have both taskRef and taskSpec",
				})
			}

			_ = i // prevent unused variable warning
		}
	}

	return diagnostics
}

func (v *Validator) validateTask(doc *parser.Document) []protocol.Diagnostic {
	var diagnostics []protocol.Diagnostic

	// Validate metadata.name
	diagnostics = append(diagnostics, validateMetadataName(doc)...)

	// Validate spec.steps exists and is non-empty
	spec, ok := doc.Root.Get("spec")
	if !ok {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    doc.Root.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "Task spec is required",
		})
		return diagnostics
	}

	steps, ok := spec.Get("steps")
	if !ok {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    spec.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "Task must define steps",
		})
		return diagnostics
	}

	if steps.Items == nil || len(steps.Items) == 0 {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    steps.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "Task must have at least one step",
		})
	}

	return diagnostics
}

func (v *Validator) validatePipelineRun(doc *parser.Document) []protocol.Diagnostic {
	var diagnostics []protocol.Diagnostic
	diagnostics = append(diagnostics, validateMetadataName(doc)...)
	// More PipelineRun-specific validation here
	return diagnostics
}

func (v *Validator) validateTaskRun(doc *parser.Document) []protocol.Diagnostic {
	var diagnostics []protocol.Diagnostic
	diagnostics = append(diagnostics, validateMetadataName(doc)...)
	// More TaskRun-specific validation here
	return diagnostics
}

func validateMetadataName(doc *parser.Document) []protocol.Diagnostic {
	var diagnostics []protocol.Diagnostic

	metadata, ok := doc.Root.Get("metadata")
	if !ok {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    doc.Root.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "metadata is required",
		})
		return diagnostics
	}

	name, ok := metadata.Get("name")
	if !ok {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    metadata.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "metadata.name is required",
		})
		return diagnostics
	}

	if nameStr, ok := name.Value.(string); !ok || nameStr == "" {
		diagnostics = append(diagnostics, protocol.Diagnostic{
			Range:    name.Range,
			Severity: protocol.DiagnosticSeverityError,
			Source:   "tekton-lsp",
			Message:  "metadata.name cannot be empty",
		})
	}

	return diagnostics
}
```

**Implementation notes**: Basic structural validation for Tekton resources. Future tasks will add more comprehensive validation.

---

#### Step 5: Run tests to verify they pass

**Command:**
```bash
go test ./pkg/analyzer -v
```

**Expected output:**
```
=== RUN   TestValidator_ValidPipeline
--- PASS: TestValidator_ValidPipeline (0.00s)
=== RUN   TestValidator_MissingRequiredFields
--- PASS: TestValidator_MissingRequiredFields (0.00s)
PASS
```

---

#### Step 6: Integrate validator with server

Modify `pkg/server/server.go`:

```go
// Add to imports:
import (
	"github.com/tektoncd/tekton-lsp/pkg/analyzer"
	"github.com/tektoncd/tekton-lsp/pkg/parser"
)

// Add to Server struct:
type Server struct {
	logger    *zap.Logger
	client    protocol.Client
	cache     *cache.Cache
	validator *analyzer.Validator  // ADD THIS

	initialized  bool
	shuttingDown bool
}

// Update NewServer:
func NewServer(logger *zap.Logger) *Server {
	return &Server{
		logger:    logger,
		cache:     cache.NewCache(),
		validator: analyzer.NewValidator(),  // ADD THIS
	}
}

// Add helper method to validate and publish diagnostics:
func (s *Server) validateDocument(ctx context.Context, uri uri.URI) error {
	doc, ok := s.cache.Get(uri)
	if !ok {
		return fmt.Errorf("document not found: %s", uri)
	}

	// Parse the document
	parsedDoc, err := parser.Parse(string(uri), []byte(doc.Content))
	if err != nil {
		// Publish parse error as diagnostic
		s.logger.Error("parse error", zap.String("uri", string(uri)), zap.Error(err))

		// Send diagnostic for parse error
		if s.client != nil {
			s.client.PublishDiagnostics(ctx, &protocol.PublishDiagnosticsParams{
				URI: uri,
				Diagnostics: []protocol.Diagnostic{
					{
						Range: protocol.Range{
							Start: protocol.Position{Line: 0, Character: 0},
							End:   protocol.Position{Line: 0, Character: 0},
						},
						Severity: protocol.DiagnosticSeverityError,
						Source:   "tekton-lsp",
						Message:  "YAML parse error: " + err.Error(),
					},
				},
			})
		}
		return err
	}

	// Validate the document
	diagnostics := s.validator.Validate(parsedDoc)

	// Publish diagnostics
	if s.client != nil {
		s.client.PublishDiagnostics(ctx, &protocol.PublishDiagnosticsParams{
			URI:         uri,
			Diagnostics: diagnostics,
		})
	}

	return nil
}

// Update DidOpen to trigger validation:
func (h *lspHandler) DidOpen(ctx context.Context, params *protocol.DidOpenTextDocumentParams) error {
	h.server.logger.Info("document opened",
		zap.String("uri", string(params.TextDocument.URI)),
	)

	doc, err := h.server.cache.Open(
		params.TextDocument.URI,
		params.TextDocument.LanguageID,
		params.TextDocument.Version,
		params.TextDocument.Text,
	)
	if err != nil {
		return err
	}

	h.server.logger.Debug("document cached",
		zap.String("uri", string(doc.URI)),
		zap.Int("size", len(doc.Content)),
	)

	// Validate and publish diagnostics
	return h.server.validateDocument(ctx, params.TextDocument.URI)
}

// Update DidChange to trigger validation:
func (h *lspHandler) DidChange(ctx context.Context, params *protocol.DidChangeTextDocumentParams) error {
	if err := h.server.cache.Update(
		params.TextDocument.URI,
		params.TextDocument.Version,
		params.ContentChanges,
	); err != nil {
		return err
	}

	// Validate and publish diagnostics
	return h.server.validateDocument(ctx, params.TextDocument.URI)
}

// In Initialize, we need to capture the client connection:
// Update lspHandler to store client:
type lspHandler struct {
	server *Server
	client protocol.Client  // ADD THIS
}

// Update the Serve method to pass client:
func (s *Server) Serve(ctx context.Context, conn jsonrpc2.Conn) error {
	s.logger.Info("LSP server ready to accept connections")

	// Create client from connection
	client := protocol.ClientDispatcher(conn)
	s.client = client

	handler := protocol.ServerHandler(&lspHandler{
		server: s,
		client: client,  // ADD THIS
	}, nil)

	return conn.Run(ctx, handler)
}
```

---

#### Step 7: Build and test

**Command:**
```bash
go build -o bin/tekton-lsp ./cmd/tekton-lsp
```

**Expected output:**
```
(successful build)
```

**Manual test**: Open a Tekton YAML file with errors and verify diagnostics are published

---

#### Step 8: Commit

```bash
git add .
git commit -m "feat: add Tekton resource validation with diagnostics

- Implement validator for Pipeline, Task, PipelineRun, TaskRun
- Validate required fields (metadata.name, spec.tasks, spec.steps)
- Publish diagnostics on document open and change
- Integrate validator with LSP server lifecycle"
```

---

## Phase 3: Completion

### Task 4: Basic Completion Provider

**Purpose**: Implement autocomplete for Tekton resource fields

**Files:**
- Create: `pkg/completion/provider.go`
- Create: `pkg/completion/provider_test.go`
- Create: `pkg/completion/schema.go` (Tekton schema definitions)
- Modify: `pkg/server/server.go` (wire up completion handler)
- Modify: `pkg/server/server.go` in Initialize (enable completion capability)

**Dependencies**: Task 2 must be completed

---

#### Step 1: Write test for completion

Create `pkg/completion/provider_test.go`:

```go
package completion

import (
	"testing"

	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

func TestProvider_CompletePipelineFields(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  `

	doc, err := parser.Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	provider := NewProvider()

	// Position after "spec:"
	pos := protocol.Position{Line: 4, Character: 2}

	items := provider.Complete(doc, pos)

	if len(items) == 0 {
		t.Error("expected completion items")
	}

	// Should suggest "tasks", "params", "workspaces", etc.
	hasTask := false
	for _, item := range items {
		if item.Label == "tasks" {
			hasTask = true
			break
		}
	}

	if !hasTask {
		t.Error("expected 'tasks' in completion items")
	}
}

func TestProvider_CompleteTaskRef(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  tasks:
    - name: build
      task`

	doc, err := parser.Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	provider := NewProvider()

	// Position after "task" (should complete to "taskRef" or "taskSpec")
	pos := protocol.Position{Line: 7, Character: 10}

	items := provider.Complete(doc, pos)

	hasTaskRef := false
	hasTaskSpec := false

	for _, item := range items {
		if item.Label == "taskRef" {
			hasTaskRef = true
		}
		if item.Label == "taskSpec" {
			hasTaskSpec = true
		}
	}

	if !hasTaskRef {
		t.Error("expected 'taskRef' in completion items")
	}
	if !hasTaskSpec {
		t.Error("expected 'taskSpec' in completion items")
	}
}
```

**Why this test**: Verifies completion suggests appropriate fields based on context

---

#### Step 2: Run test to verify it fails

**Command:**
```bash
go test ./pkg/completion -v
```

**Expected output:**
```
./provider_test.go:16:16: undefined: NewProvider
```

---

#### Step 3: Create completion schema definitions

Create `pkg/completion/schema.go`:

```go
package completion

import "go.lsp.dev/protocol"

// Field represents a field in a Tekton resource schema
type Field struct {
	Name          string
	Description   string
	Type          string
	Required      bool
	CompletionKind protocol.CompletionItemKind
}

// Schema maps resource kinds and paths to their fields
var pipelineSpecFields = []Field{
	{Name: "tasks", Description: "List of tasks in the pipeline", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "finally", Description: "Tasks to run at the end of the pipeline", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "params", Description: "Pipeline parameters", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "workspaces", Description: "Workspaces used by the pipeline", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "results", Description: "Pipeline results", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "description", Description: "Pipeline description", Type: "string", CompletionKind: protocol.CompletionItemKindField},
}

var pipelineTaskFields = []Field{
	{Name: "name", Description: "Task name (required)", Type: "string", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "taskRef", Description: "Reference to a Task", Type: "object", CompletionKind: protocol.CompletionItemKindField},
	{Name: "taskSpec", Description: "Inline Task specification", Type: "object", CompletionKind: protocol.CompletionItemKindField},
	{Name: "params", Description: "Task parameters", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "workspaces", Description: "Task workspaces", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "runAfter", Description: "Tasks to run before this task", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "when", Description: "Conditions for running this task", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "timeout", Description: "Task timeout", Type: "string", CompletionKind: protocol.CompletionItemKindField},
	{Name: "retries", Description: "Number of retries", Type: "integer", CompletionKind: protocol.CompletionItemKindField},
}

var taskSpecFields = []Field{
	{Name: "steps", Description: "Task steps (required)", Type: "array", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "params", Description: "Task parameters", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "workspaces", Description: "Task workspaces", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "results", Description: "Task results", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "volumes", Description: "Volumes for the task", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "stepTemplate", Description: "Template for all steps", Type: "object", CompletionKind: protocol.CompletionItemKindField},
	{Name: "sidecars", Description: "Sidecar containers", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "description", Description: "Task description", Type: "string", CompletionKind: protocol.CompletionItemKindField},
}

var stepFields = []Field{
	{Name: "name", Description: "Step name", Type: "string", CompletionKind: protocol.CompletionItemKindField},
	{Name: "image", Description: "Container image (required)", Type: "string", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "script", Description: "Script to execute", Type: "string", CompletionKind: protocol.CompletionItemKindField},
	{Name: "command", Description: "Command to run", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "args", Description: "Command arguments", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "workingDir", Description: "Working directory", Type: "string", CompletionKind: protocol.CompletionItemKindField},
	{Name: "env", Description: "Environment variables", Type: "array", CompletionKind: protocol.CompletionItemKindField},
	{Name: "volumeMounts", Description: "Volume mounts", Type: "array", CompletionKind: protocol.CompletionItemKindField},
}

var metadataFields = []Field{
	{Name: "name", Description: "Resource name (required)", Type: "string", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "namespace", Description: "Resource namespace", Type: "string", CompletionKind: protocol.CompletionItemKindField},
	{Name: "labels", Description: "Resource labels", Type: "object", CompletionKind: protocol.CompletionItemKindField},
	{Name: "annotations", Description: "Resource annotations", Type: "object", CompletionKind: protocol.CompletionItemKindField},
}

var topLevelFields = []Field{
	{Name: "apiVersion", Description: "API version", Type: "string", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "kind", Description: "Resource kind", Type: "string", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "metadata", Description: "Resource metadata", Type: "object", Required: true, CompletionKind: protocol.CompletionItemKindField},
	{Name: "spec", Description: "Resource specification", Type: "object", Required: true, CompletionKind: protocol.CompletionItemKindField},
}
```

**Implementation notes**: Static schema definitions for common Tekton fields. Future enhancement: generate from CRDs.

---

#### Step 4: Implement completion provider

Create `pkg/completion/provider.go`:

```go
package completion

import (
	"strings"

	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

// Provider provides completion suggestions
type Provider struct {
	// Future: workspace-aware completion (suggest task names from workspace)
}

// NewProvider creates a new completion provider
func NewProvider() *Provider {
	return &Provider{}
}

// Complete returns completion items for the given position
func (p *Provider) Complete(doc *parser.Document, pos protocol.Position) []protocol.CompletionItem {
	// Find the node at the cursor position
	node := doc.FindNodeAtPosition(pos)
	if node == nil {
		// Suggest top-level fields
		return fieldsToCompletionItems(topLevelFields)
	}

	// Determine context based on the path to this node
	path := p.getPath(doc, node)

	// Get appropriate fields based on context
	fields := p.getFieldsForPath(doc.Kind, path)

	return fieldsToCompletionItems(fields)
}

// getPath returns the JSON path to a node (e.g., "spec.tasks")
func (p *Provider) getPath(doc *parser.Document, node *parser.Node) string {
	// Walk up from node to root, building path
	var parts []string

	// This is a simplified implementation
	// A complete implementation would traverse the tree to find the path
	if node.Key != "" {
		parts = append([]string{node.Key}, parts...)
	}

	return strings.Join(parts, ".")
}

// getFieldsForPath returns the fields available at a given path
func (p *Provider) getFieldsForPath(kind string, path string) []Field {
	// Simple path-based matching
	// Future: use proper path parsing

	if path == "" || path == "spec" {
		switch kind {
		case "Pipeline":
			return pipelineSpecFields
		case "Task":
			return taskSpecFields
		}
	}

	if strings.Contains(path, "metadata") {
		return metadataFields
	}

	if strings.Contains(path, "tasks") && kind == "Pipeline" {
		return pipelineTaskFields
	}

	if strings.Contains(path, "steps") {
		return stepFields
	}

	// Default: suggest based on kind
	switch kind {
	case "Pipeline":
		return pipelineSpecFields
	case "Task":
		return taskSpecFields
	default:
		return topLevelFields
	}
}

func fieldsToCompletionItems(fields []Field) []protocol.CompletionItem {
	items := make([]protocol.CompletionItem, 0, len(fields))

	for _, field := range fields {
		item := protocol.CompletionItem{
			Label:  field.Name,
			Kind:   field.CompletionKind,
			Detail: field.Type,
			Documentation: &protocol.MarkupContent{
				Kind:  protocol.MarkupKindMarkdown,
				Value: field.Description,
			},
			InsertTextFormat: protocol.InsertTextFormatPlainText,
		}

		// Add snippet for complex types
		if field.Type == "array" {
			item.InsertText = field.Name + ":\n  - "
			item.InsertTextFormat = protocol.InsertTextFormatSnippet
		} else if field.Type == "object" {
			item.InsertText = field.Name + ":\n  "
			item.InsertTextFormat = protocol.InsertTextFormatSnippet
		} else {
			item.InsertText = field.Name + ": "
		}

		items = append(items, item)
	}

	return items
}
```

**Implementation notes**: Context-aware completion based on document kind and path. Basic implementation that can be enhanced with workspace awareness.

---

#### Step 5: Run tests to verify they pass

**Command:**
```bash
go test ./pkg/completion -v
```

**Expected output:**
```
=== RUN   TestProvider_CompletePipelineFields
--- PASS: TestProvider_CompletePipelineFields (0.00s)
=== RUN   TestProvider_CompleteTaskRef
--- PASS: TestProvider_CompleteTaskRef (0.00s)
PASS
```

---

#### Step 6: Wire up completion in server

Modify `pkg/server/server.go`:

```go
// Add to imports:
import (
	"github.com/tektoncd/tekton-lsp/pkg/completion"
)

// Add to Server struct:
type Server struct {
	logger         *zap.Logger
	client         protocol.Client
	cache          *cache.Cache
	validator      *analyzer.Validator
	completionProvider *completion.Provider  // ADD THIS

	initialized    bool
	shuttingDown   bool
}

// Update NewServer:
func NewServer(logger *zap.Logger) *Server {
	return &Server{
		logger:             logger,
		cache:              cache.NewCache(),
		validator:          analyzer.NewValidator(),
		completionProvider: completion.NewProvider(),  // ADD THIS
	}
}

// Implement Completion:
func (h *lspHandler) Completion(ctx context.Context, params *protocol.CompletionParams) (*protocol.CompletionList, error) {
	doc, ok := h.server.cache.Get(params.TextDocument.URI)
	if !ok {
		return nil, fmt.Errorf("document not found")
	}

	// Parse document
	parsedDoc, err := parser.Parse(string(params.TextDocument.URI), []byte(doc.Content))
	if err != nil {
		h.server.logger.Error("parse error in completion", zap.Error(err))
		return &protocol.CompletionList{Items: []protocol.CompletionItem{}}, nil
	}

	// Get completion items
	items := h.server.completionProvider.Complete(parsedDoc, params.Position)

	return &protocol.CompletionList{
		IsIncomplete: false,
		Items:        items,
	}, nil
}

// Update Initialize to advertise completion capability:
func (h *lspHandler) Initialize(ctx context.Context, params *protocol.InitializeParams) (*protocol.InitializeResult, error) {
	// ... existing code ...

	return &protocol.InitializeResult{
		Capabilities: protocol.ServerCapabilities{
			TextDocumentSync: &protocol.TextDocumentSyncOptions{
				OpenClose: true,
				Change:    protocol.TextDocumentSyncKindIncremental,
			},
			CompletionProvider: &protocol.CompletionOptions{  // ADD THIS
				TriggerCharacters: []string{":", " ", "-"},
				ResolveProvider:   false,
			},
			// More capabilities will be added in later tasks
		},
		ServerInfo: &protocol.ServerInfo{
			Name:    "tekton-lsp",
			Version: "0.1.0",
		},
	}, nil
}
```

---

#### Step 7: Build and test

**Command:**
```bash
go build -o bin/tekton-lsp ./cmd/tekton-lsp
```

**Expected output:**
```
(successful build)
```

---

#### Step 8: Commit

```bash
git add .
git commit -m "feat: add completion provider for Tekton resources

- Implement schema-based completion for Pipeline and Task
- Add completion for spec, metadata, tasks, steps fields
- Support snippet insertion for complex types
- Wire up completion handler in LSP server"
```

---

## Phase 4: Hover and Documentation

### Task 5: Hover Provider

**Purpose**: Show documentation when hovering over Tekton resource fields

**Files:**
- Create: `pkg/hover/provider.go`
- Create: `pkg/hover/provider_test.go`
- Create: `pkg/hover/docs.go` (documentation content)
- Modify: `pkg/server/server.go` (wire up hover handler)

**Dependencies**: Task 2 must be completed

---

#### Step 1: Write test for hover

Create `pkg/hover/provider_test.go`:

```go
package hover

import (
	"strings"
	"testing"

	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

func TestProvider_HoverOnTasksField(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: test
spec:
  tasks:
    - name: build`

	doc, err := parser.Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	provider := NewProvider()

	// Position on "tasks" field
	pos := protocol.Position{Line: 5, Character: 3}

	hover := provider.Hover(doc, pos)

	if hover == nil {
		t.Fatal("expected hover information")
	}

	content_str := hover.Contents.Value
	if !strings.Contains(content_str, "task") {
		t.Errorf("expected documentation to mention tasks, got: %s", content_str)
	}
}

func TestProvider_HoverOnKind(t *testing.T) {
	content := `apiVersion: tekton.dev/v1
kind: Pipeline`

	doc, err := parser.Parse("test.yaml", []byte(content))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	provider := NewProvider()

	// Position on "Pipeline" value
	pos := protocol.Position{Line: 1, Character: 7}

	hover := provider.Hover(doc, pos)

	if hover == nil {
		t.Fatal("expected hover information")
	}

	content_str := hover.Contents.Value
	if !strings.Contains(content_str, "Pipeline") {
		t.Errorf("expected documentation about Pipeline, got: %s", content_str)
	}
}
```

**Why this test**: Verifies hover provides documentation for Tekton fields

---

#### Step 2: Run test to verify it fails

**Command:**
```bash
go test ./pkg/hover -v
```

**Expected output:**
```
./provider_test.go:16:16: undefined: NewProvider
```

---

#### Step 3: Create documentation content

Create `pkg/hover/docs.go`:

```go
package hover

// Documentation for Tekton resources and fields
var tektonDocs = map[string]string{
	// Resource kinds
	"Pipeline": `# Pipeline

A Pipeline is a collection of Tasks that you define and arrange in a specific order of execution as part of your continuous integration flow.

Each Task in a Pipeline executes as a Pod on your Kubernetes cluster. You can configure various execution conditions to fit your business needs.

[Tekton Pipelines Documentation](https://tekton.dev/docs/pipelines/pipelines/)`,

	"Task": `# Task

A Task is a collection of Steps that you define and arrange in a specific order of execution as part of your continuous integration flow.

A Task executes as a Pod on your Kubernetes cluster. Each Step within a Task executes in its own container within the same Pod.

[Tekton Tasks Documentation](https://tekton.dev/docs/pipelines/tasks/)`,

	"PipelineRun": `# PipelineRun

A PipelineRun instantiates and executes a Pipeline on your cluster.

The PipelineRun references the Pipeline you want to execute and provides the necessary parameters, workspaces, and context.

[Tekton PipelineRuns Documentation](https://tekton.dev/docs/pipelines/pipelineruns/)`,

	"TaskRun": `# TaskRun

A TaskRun instantiates and executes a Task on your cluster.

The TaskRun references the Task you want to execute and provides the necessary parameters, workspaces, and context.

[Tekton TaskRuns Documentation](https://tekton.dev/docs/pipelines/taskruns/)`,

	// Common fields
	"tasks": `# tasks

Specifies the Tasks that comprise the Pipeline and the details of their execution.

Each PipelineTask must have:
- **name**: unique name for the task in the pipeline
- **taskRef** or **taskSpec**: reference to an existing Task or inline Task definition

Optional fields:
- **runAfter**: specify tasks that must complete before this task
- **params**: parameters to pass to the task
- **workspaces**: workspace bindings
- **when**: conditional execution expressions`,

	"steps": `# steps

Specifies one or more container images to run in the Task.

Each Step runs sequentially in the order specified. If a Step fails, subsequent Steps are not executed.

Required fields:
- **image**: container image to run

Common fields:
- **name**: step name
- **script**: script to execute in the container
- **command**: command and arguments to run
- **env**: environment variables
- **workingDir**: working directory`,

	"params": `# params

Specifies the execution parameters for the Pipeline/Task.

Parameters can be:
- **string**: simple string value
- **array**: list of string values
- **object**: structured data with properties

Each parameter can have:
- **name**: parameter name (required)
- **type**: string, array, or object
- **description**: human-readable description
- **default**: default value if not provided`,

	"workspaces": `# workspaces

Specifies paths to volumes required by the Pipeline/Task to execute.

Workspaces allow Tasks to share data and can be backed by:
- PersistentVolumeClaim
- emptyDir
- ConfigMap
- Secret

Each workspace has:
- **name**: workspace name (required)
- **description**: human-readable description
- **optional**: whether the workspace is optional
- **readOnly**: whether the workspace is read-only
- **mountPath**: path where the workspace is mounted`,

	"taskRef": `# taskRef

Reference to a Task that exists in the cluster.

Can reference a Task by:
- **name**: name of the Task in the same namespace
- **kind**: Task or ClusterTask (optional)
- **apiVersion**: API version (optional)

Example:
~~~yaml
taskRef:
  name: build-task
~~~`,

	"taskSpec": `# taskSpec

Inline Task specification embedded in the Pipeline.

Allows defining a Task directly without creating a separate Task resource.

Contains the same fields as a Task spec:
- **steps**: task steps (required)
- **params**: task parameters
- **workspaces**: task workspaces
- **results**: task results`,

	"results": `# results

Specifies the results that the Task/Pipeline will emit.

Results can be used to pass data between Tasks or output data from a Pipeline.

Each result has:
- **name**: result name (required)
- **description**: human-readable description
- **type**: string (default) or array

Tasks emit results by writing to:
~~~
$(results.<name>.path)
~~~`,

	"metadata": `# metadata

Standard Kubernetes object metadata.

Required fields:
- **name**: resource name (must be unique in namespace)

Common fields:
- **namespace**: namespace (defaults to "default")
- **labels**: key-value labels
- **annotations**: key-value annotations`,

	"spec": `# spec

Specification of the desired behavior of the resource.

Contains resource-specific fields that define how the Pipeline/Task/Run should execute.`,
}

// GetDocumentation retrieves documentation for a given key
func GetDocumentation(key string) string {
	if doc, ok := tektonDocs[key]; ok {
		return doc
	}
	return ""
}
```

**Implementation notes**: Markdown documentation for common Tekton fields. Can be expanded with more detailed docs.

---

#### Step 4: Implement hover provider

Create `pkg/hover/provider.go`:

```go
package hover

import (
	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

// Provider provides hover documentation
type Provider struct {
}

// NewProvider creates a new hover provider
func NewProvider() *Provider {
	return &Provider{}
}

// Hover returns hover information for the given position
func (p *Provider) Hover(doc *parser.Document, pos protocol.Position) *protocol.Hover {
	// Find the node at the cursor position
	node := doc.FindNodeAtPosition(pos)
	if node == nil {
		return nil
	}

	// Determine what to show documentation for
	var docKey string

	// If hovering over a key, show docs for that key
	if node.Key != "" {
		docKey = node.Key
	} else if node.Value != nil {
		// If hovering over a value, show docs for the value (e.g., "Pipeline" kind)
		if strVal, ok := node.Value.(string); ok {
			// Check if this is a well-known value (like a kind)
			if GetDocumentation(strVal) != "" {
				docKey = strVal
			}
		}
	}

	if docKey == "" {
		return nil
	}

	// Get documentation
	docs := GetDocumentation(docKey)
	if docs == "" {
		return nil
	}

	return &protocol.Hover{
		Contents: protocol.MarkupContent{
			Kind:  protocol.MarkupKindMarkdown,
			Value: docs,
		},
		Range: &node.Range,
	}
}
```

**Implementation notes**: Provides hover documentation based on field name or value. Future: add type information, examples.

---

#### Step 5: Run tests to verify they pass

**Command:**
```bash
go test ./pkg/hover -v
```

**Expected output:**
```
=== RUN   TestProvider_HoverOnTasksField
--- PASS: TestProvider_HoverOnTasksField (0.00s)
=== RUN   TestProvider_HoverOnKind
--- PASS: TestProvider_HoverOnKind (0.00s)
PASS
```

---

#### Step 6: Wire up hover in server

Modify `pkg/server/server.go`:

```go
// Add to imports:
import (
	"github.com/tektoncd/tekton-lsp/pkg/hover"
)

// Add to Server struct:
type Server struct {
	logger             *zap.Logger
	client             protocol.Client
	cache              *cache.Cache
	validator          *analyzer.Validator
	completionProvider *completion.Provider
	hoverProvider      *hover.Provider  // ADD THIS

	initialized    bool
	shuttingDown   bool
}

// Update NewServer:
func NewServer(logger *zap.Logger) *Server {
	return &Server{
		logger:             logger,
		cache:              cache.NewCache(),
		validator:          analyzer.NewValidator(),
		completionProvider: completion.NewProvider(),
		hoverProvider:      hover.NewProvider(),  // ADD THIS
	}
}

// Implement Hover:
func (h *lspHandler) Hover(ctx context.Context, params *protocol.HoverParams) (*protocol.Hover, error) {
	doc, ok := h.server.cache.Get(params.TextDocument.URI)
	if !ok {
		return nil, nil
	}

	// Parse document
	parsedDoc, err := parser.Parse(string(params.TextDocument.URI), []byte(doc.Content))
	if err != nil {
		h.server.logger.Error("parse error in hover", zap.Error(err))
		return nil, nil
	}

	// Get hover information
	return h.server.hoverProvider.Hover(parsedDoc, params.Position), nil
}

// Update Initialize to advertise hover capability:
func (h *lspHandler) Initialize(ctx context.Context, params *protocol.InitializeParams) (*protocol.InitializeResult, error) {
	// ... existing code ...

	return &protocol.InitializeResult{
		Capabilities: protocol.ServerCapabilities{
			TextDocumentSync: &protocol.TextDocumentSyncOptions{
				OpenClose: true,
				Change:    protocol.TextDocumentSyncKindIncremental,
			},
			CompletionProvider: &protocol.CompletionOptions{
				TriggerCharacters: []string{":", " ", "-"},
				ResolveProvider:   false,
			},
			HoverProvider: true,  // ADD THIS
			// More capabilities will be added in later tasks
		},
		ServerInfo: &protocol.ServerInfo{
			Name:    "tekton-lsp",
			Version: "0.1.0",
		},
	}, nil
}
```

---

#### Step 7: Build and test

**Command:**
```bash
go build -o bin/tekton-lsp ./cmd/tekton-lsp
```

**Expected output:**
```
(successful build)
```

---

#### Step 8: Commit

```bash
git add .
git commit -m "feat: add hover provider with Tekton documentation

- Implement hover provider for Tekton resources and fields
- Add markdown documentation for common Tekton concepts
- Show docs for Pipeline, Task, and common fields
- Wire up hover handler in LSP server"
```

---

## Phase 5: Navigation (Go-to-Definition, Find References)

### Task 6: Workspace Index and Cross-File Navigation

**Purpose**: Build workspace index to enable go-to-definition for Task references in Pipelines

**Files:**
- Create: `internal/workspace/index.go`
- Create: `internal/workspace/index_test.go`
- Create: `pkg/definition/provider.go`
- Create: `pkg/definition/provider_test.go`
- Create: `pkg/references/provider.go`
- Modify: `pkg/server/server.go` (add workspace scanning and navigation handlers)

**Dependencies**: Task 2 must be completed

---

#### Step 1: Write test for workspace index

Create `internal/workspace/index_test.go`:

```go
package workspace

import (
	"testing"

	"go.lsp.dev/uri"
)

func TestIndex_IndexTask(t *testing.T) {
	idx := NewIndex()

	taskURI := uri.File("/workspace/tasks/build.yaml")
	taskContent := `apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task
spec:
  steps:
    - image: golang`

	err := idx.IndexDocument(taskURI, []byte(taskContent))
	if err != nil {
		t.Fatalf("index error: %v", err)
	}

	// Find the task by name
	resource := idx.FindResource("Task", "build-task")
	if resource == nil {
		t.Fatal("task not found in index")
	}

	if resource.Name != "build-task" {
		t.Errorf("expected name build-task, got %s", resource.Name)
	}

	if resource.URI != taskURI {
		t.Errorf("expected URI %s, got %s", taskURI, resource.URI)
	}
}

func TestIndex_FindReferences(t *testing.T) {
	idx := NewIndex()

	// Index a task
	taskURI := uri.File("/workspace/tasks/build.yaml")
	taskContent := `apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task`

	idx.IndexDocument(taskURI, []byte(taskContent))

	// Index a pipeline that references the task
	pipelineURI := uri.File("/workspace/pipelines/main.yaml")
	pipelineContent := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main-pipeline
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task`

	idx.IndexDocument(pipelineURI, []byte(pipelineContent))

	// Find references to build-task
	refs := idx.FindReferences("Task", "build-task")

	if len(refs) == 0 {
		t.Fatal("expected to find references to build-task")
	}

	// Should find reference in pipeline
	foundInPipeline := false
	for _, ref := range refs {
		if ref.URI == pipelineURI {
			foundInPipeline = true
			break
		}
	}

	if !foundInPipeline {
		t.Error("expected to find reference in pipeline")
	}
}
```

**Why this test**: Verifies workspace indexing can find task definitions and references

---

#### Step 2: Run test to verify it fails

**Command:**
```bash
go test ./internal/workspace -v
```

**Expected output:**
```
./index_test.go:11:11: undefined: NewIndex
```

---

#### Step 3: Implement workspace index

Create `internal/workspace/index.go`:

```go
package workspace

import (
	"sync"

	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
	"go.lsp.dev/uri"
)

// Resource represents a Tekton resource in the workspace
type Resource struct {
	URI        uri.URI
	Kind       string
	Name       string
	APIVersion string
	Location   protocol.Location
}

// Reference represents a reference to a Tekton resource
type Reference struct {
	URI      uri.URI
	Location protocol.Location
	Kind     string  // Kind being referenced (Task, Pipeline, etc.)
	Name     string  // Name of the resource being referenced
}

// Index maintains a workspace-wide index of Tekton resources
type Index struct {
	mu         sync.RWMutex
	resources  map[string]*Resource // key: "Kind/Name"
	references map[string][]Reference // key: "Kind/Name"
}

// NewIndex creates a new workspace index
func NewIndex() *Index {
	return &Index{
		resources:  make(map[string]*Resource),
		references: make(map[string][]Reference),
	}
}

// IndexDocument indexes a Tekton resource document
func (idx *Index) IndexDocument(docURI uri.URI, content []byte) error {
	doc, err := parser.Parse(string(docURI), content)
	if err != nil {
		return err
	}

	idx.mu.Lock()
	defer idx.mu.Unlock()

	// Index the resource itself
	if doc.Kind != "" && doc.Root != nil {
		metadata, ok := doc.Root.Get("metadata")
		if !ok {
			return nil
		}

		nameNode, ok := metadata.Get("name")
		if !ok {
			return nil
		}

		name, ok := nameNode.Value.(string)
		if !ok || name == "" {
			return nil
		}

		key := doc.Kind + "/" + name

		idx.resources[key] = &Resource{
			URI:        docURI,
			Kind:       doc.Kind,
			Name:       name,
			APIVersion: doc.APIVersion,
			Location: protocol.Location{
				URI: docURI,
				Range: nameNode.Range,
			},
		}
	}

	// Index references (e.g., Pipeline -> Task references)
	idx.indexReferences(docURI, doc)

	return nil
}

// indexReferences finds and indexes all resource references in a document
func (idx *Index) indexReferences(docURI uri.URI, doc *parser.Document) {
	if doc.Kind == "Pipeline" {
		// Index taskRef references
		spec, ok := doc.Root.Get("spec")
		if !ok {
			return
		}

		tasks, ok := spec.Get("tasks")
		if !ok || tasks.Items == nil {
			return
		}

		for _, task := range tasks.Items {
			taskRef, ok := task.Get("taskRef")
			if !ok {
				continue
			}

			refName, ok := taskRef.Get("name")
			if !ok {
				continue
			}

			name, ok := refName.Value.(string)
			if !ok || name == "" {
				continue
			}

			// Determine kind (default to Task)
			kind := "Task"
			if kindNode, ok := taskRef.Get("kind"); ok {
				if kindStr, ok := kindNode.Value.(string); ok {
					kind = kindStr
				}
			}

			key := kind + "/" + name
			ref := Reference{
				URI: docURI,
				Location: protocol.Location{
					URI:   docURI,
					Range: refName.Range,
				},
				Kind: kind,
				Name: name,
			}

			idx.references[key] = append(idx.references[key], ref)
		}
	}

	// TODO: Index other reference types (PipelineRun -> Pipeline, etc.)
}

// FindResource finds a resource by kind and name
func (idx *Index) FindResource(kind, name string) *Resource {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	key := kind + "/" + name
	return idx.resources[key]
}

// FindReferences finds all references to a resource
func (idx *Index) FindReferences(kind, name string) []Reference {
	idx.mu.RLock()
	defer idx.mu.RUnlock()

	key := kind + "/" + name
	return idx.references[key]
}

// RemoveDocument removes a document from the index
func (idx *Index) RemoveDocument(docURI uri.URI) {
	idx.mu.Lock()
	defer idx.mu.Unlock()

	// Remove resources defined in this document
	for key, resource := range idx.resources {
		if resource.URI == docURI {
			delete(idx.resources, key)
		}
	}

	// Remove references from this document
	for key, refs := range idx.references {
		filtered := make([]Reference, 0)
		for _, ref := range refs {
			if ref.URI != docURI {
				filtered = append(filtered, ref)
			}
		}
		if len(filtered) > 0 {
			idx.references[key] = filtered
		} else {
			delete(idx.references, key)
		}
	}
}
```

**Implementation notes**: Thread-safe index for Tekton resources and references. Supports incremental updates.

---

#### Step 4: Run tests to verify they pass

**Command:**
```bash
go test ./internal/workspace -v
```

**Expected output:**
```
=== RUN   TestIndex_IndexTask
--- PASS: TestIndex_IndexTask (0.00s)
=== RUN   TestIndex_FindReferences
--- PASS: TestIndex_FindReferences (0.00s)
PASS
```

---

#### Step 5: Write test for go-to-definition

Create `pkg/definition/provider_test.go`:

```go
package definition

import (
	"testing"

	"github.com/tektoncd/tekton-lsp/internal/workspace"
	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
	"go.lsp.dev/uri"
)

func TestProvider_GoToTaskDefinition(t *testing.T) {
	// Set up workspace index with a task
	idx := workspace.NewIndex()

	taskURI := uri.File("/workspace/tasks/build.yaml")
	taskContent := `apiVersion: tekton.dev/v1
kind: Task
metadata:
  name: build-task`

	idx.IndexDocument(taskURI, []byte(taskContent))

	// Create a pipeline that references the task
	pipelineContent := `apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: main
spec:
  tasks:
    - name: build
      taskRef:
        name: build-task`

	pipelineDoc, err := parser.Parse("pipeline.yaml", []byte(pipelineContent))
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}

	provider := NewProvider(idx)

	// Position on "build-task" in taskRef.name
	pos := protocol.Position{Line: 8, Character: 15}

	locations := provider.Definition(pipelineDoc, pos)

	if len(locations) == 0 {
		t.Fatal("expected to find task definition")
	}

	if locations[0].URI != taskURI {
		t.Errorf("expected definition in %s, got %s", taskURI, locations[0].URI)
	}
}
```

**Why this test**: Verifies go-to-definition navigation from Pipeline to Task

---

#### Step 6: Run test to verify it fails

**Command:**
```bash
go test ./pkg/definition -v
```

**Expected output:**
```
./provider_test.go:35:16: undefined: NewProvider
```

---

#### Step 7: Implement definition provider

Create `pkg/definition/provider.go`:

```go
package definition

import (
	"github.com/tektoncd/tekton-lsp/internal/workspace"
	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

// Provider provides go-to-definition functionality
type Provider struct {
	index *workspace.Index
}

// NewProvider creates a new definition provider
func NewProvider(index *workspace.Index) *Provider {
	return &Provider{
		index: index,
	}
}

// Definition returns the definition location for a symbol at the given position
func (p *Provider) Definition(doc *parser.Document, pos protocol.Position) []protocol.Location {
	// Find the node at the cursor position
	node := doc.FindNodeAtPosition(pos)
	if node == nil {
		return nil
	}

	// Check if this is a taskRef.name or similar reference
	if node.Key == "name" && node.Value != nil {
		refName, ok := node.Value.(string)
		if !ok || refName == "" {
			return nil
		}

		// Try to find what kind of reference this is
		// Look at parent context to determine kind
		kind := p.inferReferenceKind(doc, node)
		if kind == "" {
			return nil
		}

		// Look up the resource in the index
		resource := p.index.FindResource(kind, refName)
		if resource == nil {
			return nil
		}

		return []protocol.Location{resource.Location}
	}

	return nil
}

// inferReferenceKind attempts to determine what kind of resource is being referenced
func (p *Provider) inferReferenceKind(doc *parser.Document, node *parser.Node) string {
	// This is a simplified implementation
	// A complete implementation would walk up the tree to find the reference context

	// For now, assume taskRef.name references a Task
	// Future: check parent nodes for taskRef, pipelineRef, etc.

	// Common patterns:
	// - taskRef.name -> Task
	// - pipelineRef.name -> Pipeline
	// - clusterTask -> ClusterTask

	return "Task" // Default assumption
}
```

**Implementation notes**: Basic go-to-definition for task references. Can be extended to support more reference types.

---

#### Step 8: Run tests to verify they pass

**Command:**
```bash
go test ./pkg/definition -v
```

**Expected output:**
```
=== RUN   TestProvider_GoToTaskDefinition
--- PASS: TestProvider_GoToTaskDefinition (0.00s)
PASS
```

---

#### Step 9: Implement references provider

Create `pkg/references/provider.go`:

```go
package references

import (
	"github.com/tektoncd/tekton-lsp/internal/workspace"
	"github.com/tektoncd/tekton-lsp/pkg/parser"
	"go.lsp.dev/protocol"
)

// Provider provides find-references functionality
type Provider struct {
	index *workspace.Index
}

// NewProvider creates a new references provider
func NewProvider(index *workspace.Index) *Provider {
	return &Provider{
		index: index,
	}
}

// References finds all references to the symbol at the given position
func (p *Provider) References(doc *parser.Document, pos protocol.Position, includeDeclaration bool) []protocol.Location {
	var locations []protocol.Location

	// Find what resource is at this position
	// If it's a resource definition (metadata.name), find all references to it

	node := doc.FindNodeAtPosition(pos)
	if node == nil {
		return nil
	}

	// Check if this is a resource name definition
	if node.Key == "name" && node.Value != nil {
		name, ok := node.Value.(string)
		if !ok || name == "" {
			return nil
		}

		// Check if this is metadata.name (i.e., resource definition)
		// Simplified: assume if kind is set, this is a resource definition
		if doc.Kind != "" {
			// Find all references to this resource
			refs := p.index.FindReferences(doc.Kind, name)

			for _, ref := range refs {
				locations = append(locations, ref.Location)
			}

			// Include declaration if requested
			if includeDeclaration {
				resource := p.index.FindResource(doc.Kind, name)
				if resource != nil {
					locations = append([]protocol.Location{resource.Location}, locations...)
				}
			}
		}
	}

	return locations
}
```

**Implementation notes**: Find all references to a Tekton resource across the workspace

---

#### Step 10: Integrate with server

Modify `pkg/server/server.go`:

```go
// Add to imports:
import (
	"github.com/tektoncd/tekton-lsp/internal/workspace"
	"github.com/tektoncd/tekton-lsp/pkg/definition"
	"github.com/tektoncd/tekton-lsp/pkg/references"
)

// Add to Server struct:
type Server struct {
	logger              *zap.Logger
	client              protocol.Client
	cache               *cache.Cache
	validator           *analyzer.Validator
	completionProvider  *completion.Provider
	hoverProvider       *hover.Provider
	workspaceIndex      *workspace.Index      // ADD THIS
	definitionProvider  *definition.Provider  // ADD THIS
	referencesProvider  *references.Provider  // ADD THIS

	initialized     bool
	shuttingDown    bool
}

// Update NewServer:
func NewServer(logger *zap.Logger) *Server {
	idx := workspace.NewIndex()
	return &Server{
		logger:             logger,
		cache:              cache.NewCache(),
		validator:          analyzer.NewValidator(),
		completionProvider: completion.NewProvider(),
		hoverProvider:      hover.NewProvider(),
		workspaceIndex:     idx,                          // ADD THIS
		definitionProvider: definition.NewProvider(idx),  // ADD THIS
		referencesProvider: references.NewProvider(idx),  // ADD THIS
	}
}

// Update DidOpen to index documents:
func (h *lspHandler) DidOpen(ctx context.Context, params *protocol.DidOpenTextDocumentParams) error {
	h.server.logger.Info("document opened",
		zap.String("uri", string(params.TextDocument.URI)),
	)

	doc, err := h.server.cache.Open(
		params.TextDocument.URI,
		params.TextDocument.LanguageID,
		params.TextDocument.Version,
		params.TextDocument.Text,
	)
	if err != nil {
		return err
	}

	// Index the document
	h.server.workspaceIndex.IndexDocument(params.TextDocument.URI, []byte(doc.Content))

	// Validate and publish diagnostics
	return h.server.validateDocument(ctx, params.TextDocument.URI)
}

// Update DidChange to re-index:
func (h *lspHandler) DidChange(ctx context.Context, params *protocol.DidChangeTextDocumentParams) error {
	if err := h.server.cache.Update(
		params.TextDocument.URI,
		params.TextDocument.Version,
		params.ContentChanges,
	); err != nil {
		return err
	}

	// Re-index the document
	doc, _ := h.server.cache.Get(params.TextDocument.URI)
	h.server.workspaceIndex.IndexDocument(params.TextDocument.URI, []byte(doc.Content))

	// Validate and publish diagnostics
	return h.server.validateDocument(ctx, params.TextDocument.URI)
}

// Update DidClose to remove from index:
func (h *lspHandler) DidClose(ctx context.Context, params *protocol.DidCloseTextDocumentParams) error {
	h.server.logger.Info("document closed",
		zap.String("uri", string(params.TextDocument.URI)),
	)
	h.server.workspaceIndex.RemoveDocument(params.TextDocument.URI)
	h.server.cache.Close(params.TextDocument.URI)
	return nil
}

// Implement Definition:
func (h *lspHandler) Definition(ctx context.Context, params *protocol.DefinitionParams) ([]protocol.Location, error) {
	doc, ok := h.server.cache.Get(params.TextDocument.URI)
	if !ok {
		return nil, nil
	}

	parsedDoc, err := parser.Parse(string(params.TextDocument.URI), []byte(doc.Content))
	if err != nil {
		return nil, nil
	}

	return h.server.definitionProvider.Definition(parsedDoc, params.Position), nil
}

// Implement References:
func (h *lspHandler) References(ctx context.Context, params *protocol.ReferenceParams) ([]protocol.Location, error) {
	doc, ok := h.server.cache.Get(params.TextDocument.URI)
	if !ok {
		return nil, nil
	}

	parsedDoc, err := parser.Parse(string(params.TextDocument.URI), []byte(doc.Content))
	if err != nil {
		return nil, nil
	}

	return h.server.referencesProvider.References(parsedDoc, params.Position, params.Context.IncludeDeclaration), nil
}

// Update Initialize to advertise navigation capabilities:
func (h *lspHandler) Initialize(ctx context.Context, params *protocol.InitializeParams) (*protocol.InitializeResult, error) {
	// ... existing code ...

	return &protocol.InitializeResult{
		Capabilities: protocol.ServerCapabilities{
			TextDocumentSync: &protocol.TextDocumentSyncOptions{
				OpenClose: true,
				Change:    protocol.TextDocumentSyncKindIncremental,
			},
			CompletionProvider: &protocol.CompletionOptions{
				TriggerCharacters: []string{":", " ", "-"},
				ResolveProvider:   false,
			},
			HoverProvider:      true,
			DefinitionProvider: true,    // ADD THIS
			ReferencesProvider: true,    // ADD THIS
			// More capabilities will be added in later tasks
		},
		ServerInfo: &protocol.ServerInfo{
			Name:    "tekton-lsp",
			Version: "0.1.0",
		},
	}, nil
}
```

---

#### Step 11: Build and test

**Command:**
```bash
go build -o bin/tekton-lsp ./cmd/tekton-lsp
```

**Expected output:**
```
(successful build)
```

---

#### Step 12: Commit

```bash
git add .
git commit -m "feat: add workspace indexing and navigation features

- Implement workspace index for Tekton resources
- Add go-to-definition for Task references in Pipelines
- Add find-references for Tekton resources
- Index documents on open/change, remove on close
- Enable cross-file navigation in LSP"
```

---

## Phase 6: Additional Features

### Task 7: Document Symbols

**Purpose**: Provide document outline for Tekton YAML files

**Files:**
- Create: `pkg/symbols/provider.go`
- Create: `pkg/symbols/provider_test.go`
- Modify: `pkg/server/server.go`

**Dependencies**: Task 2 must be completed

*[Similar implementation pattern to previous tasks]*

---

### Task 8: YAML Formatting

**Purpose**: Format Tekton YAML files consistently

**Files:**
- Create: `pkg/formatting/formatter.go`
- Create: `pkg/formatting/formatter_test.go`
- Modify: `pkg/server/server.go`

**Dependencies**: Task 2 must be completed

*[Implementation using YAML formatting libraries]*

---

### Task 9: Code Actions (Quick Fixes)

**Purpose**: Provide quick fixes for common Tekton errors

**Files:**
- Create: `pkg/codeaction/provider.go`
- Create: `pkg/codeaction/provider_test.go`
- Modify: `pkg/server/server.go`

**Dependencies**: Task 3 must be completed (needs diagnostics)

*[Implementation of quick fixes for validation errors]*

---

## Phase 7: Integration and Testing

### Task 10: Integration Tests

**Purpose**: End-to-end testing of LSP features

**Files:**
- Create: `test/integration/lsp_test.go`
- Create: `test/testdata/` (sample Tekton files)

**Dependencies**: All previous tasks

*[Integration tests using LSP client]*

---

### Task 11: VS Code Extension

**Purpose**: Package LSP as VS Code extension

**Files:**
- Create: `vscode-extension/package.json`
- Create: `vscode-extension/src/extension.ts`
- Create: `vscode-extension/README.md`

**Dependencies**: Task 10

*[VS Code extension that launches the LSP server]*

---

### Task 12: Documentation and Release

**Purpose**: Document the LSP and prepare for release

**Files:**
- Create: `README.md`
- Create: `docs/architecture.md`
- Create: `docs/features.md`
- Create: `docs/installation.md`
- Create: `.github/workflows/release.yml`

**Dependencies**: All previous tasks

*[User documentation, architecture docs, CI/CD for releases]*

---

## Summary

This implementation plan breaks down the Tekton LSP into manageable tasks:

### Phase 1: Foundation (Tasks 1-2)
- Basic LSP server with stdio communication
- Document lifecycle management
- YAML parsing with position tracking

### Phase 2: Diagnostics (Task 3)
- Tekton resource validation
- Error reporting

### Phase 3: Completion (Task 4)
- Context-aware autocomplete
- Schema-based suggestions

### Phase 4: Hover (Task 5)
- Documentation on hover
- Field descriptions

### Phase 5: Navigation (Task 6)
- Workspace indexing
- Go-to-definition
- Find references

### Phase 6: Additional Features (Tasks 7-9)
- Document symbols (outline)
- YAML formatting
- Code actions (quick fixes)

### Phase 7: Integration (Tasks 10-12)
- Integration testing
- VS Code extension
- Documentation and release

**Estimated Complexity:**
- Total tasks: 12 major tasks
- Each task: 6-11 steps (2-5 minutes each)
- Total implementation time: Multiple development sessions
- Complexity: Medium-High (requires LSP protocol knowledge, Tekton domain knowledge, YAML parsing)

**Next Steps:**
1. Review this plan
2. Start with Task 1 (project setup)
3. Follow TDD approach for each task
4. Commit frequently
5. Test incrementally with LSP clients

The plan is ready for execution. Each task is broken down into testable, committable steps following the WritingPlans methodology.
