use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

/// Represents a text document in the workspace
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub language_id: String,
    pub version: i32,
    pub content: String,
}

impl Document {
    /// Create a new document
    pub fn new(uri: Url, language_id: String, version: i32, content: String) -> Self {
        Self {
            uri,
            language_id,
            version,
            content,
        }
    }

    /// Apply incremental changes to the document
    pub fn apply_changes(&mut self, changes: Vec<TextDocumentContentChangeEvent>) {
        for change in changes {
            match change.range {
                // Full document sync
                None => {
                    self.content = change.text;
                }
                // Incremental sync
                Some(range) => {
                    self.apply_incremental_change(range, &change.text);
                }
            }
        }
    }

    /// Apply an incremental change to a specific range
    fn apply_incremental_change(&mut self, range: Range, text: &str) {
        let lines: Vec<&str> = self.content.lines().collect();
        let start = range.start;
        let end = range.end;

        let mut new_content = String::new();

        // Lines before the change
        for line in lines.iter().take(start.line as usize) {
            new_content.push_str(line);
            new_content.push('\n');
        }

        // Start line with prefix before change
        if let Some(start_line) = lines.get(start.line as usize) {
            let prefix = start_line
                .chars()
                .take(start.character as usize)
                .collect::<String>();
            new_content.push_str(&prefix);
        }

        // New text
        new_content.push_str(text);

        // End line with suffix after change
        if let Some(end_line) = lines.get(end.line as usize) {
            let suffix = end_line
                .chars()
                .skip(end.character as usize)
                .collect::<String>();
            new_content.push_str(&suffix);
            new_content.push('\n');
        }

        // Lines after the change
        for line in lines.iter().skip((end.line + 1) as usize) {
            new_content.push_str(line);
            new_content.push('\n');
        }

        // Remove trailing newline if original didn't have one
        if !self.content.ends_with('\n') && new_content.ends_with('\n') {
            new_content.pop();
        }

        self.content = new_content;
    }
}

/// Thread-safe cache for all open documents
#[derive(Debug, Clone)]
pub struct DocumentCache {
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

impl DocumentCache {
    /// Create a new empty document cache
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a document in the cache
    pub fn insert(&self, uri: Url, language_id: String, version: i32, content: String) {
        let doc = Document::new(uri.clone(), language_id, version, content);
        let mut documents = self.documents.write().unwrap();
        documents.insert(uri, doc);
    }

    /// Get a document from the cache
    pub fn get(&self, uri: &Url) -> Option<Document> {
        let documents = self.documents.read().unwrap();
        documents.get(uri).cloned()
    }

    /// Update a document with incremental changes
    pub fn update(&self, uri: &Url, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        let mut documents = self.documents.write().unwrap();
        if let Some(doc) = documents.get_mut(uri) {
            doc.version = version;
            doc.apply_changes(changes);
        }
    }

    /// Remove a document from the cache
    pub fn remove(&self, uri: &Url) {
        let mut documents = self.documents.write().unwrap();
        documents.remove(uri);
    }

    /// Get all documents in the cache
    pub fn all(&self) -> Vec<Document> {
        let documents = self.documents.read().unwrap();
        documents.values().cloned().collect()
    }
}

impl Default for DocumentCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let uri = Url::parse("file:///test.yaml").unwrap();
        let doc = Document::new(uri.clone(), "yaml".to_string(), 1, "test content".to_string());

        assert_eq!(doc.uri, uri);
        assert_eq!(doc.language_id, "yaml");
        assert_eq!(doc.version, 1);
        assert_eq!(doc.content, "test content");
    }

    #[test]
    fn test_full_document_change() {
        let uri = Url::parse("file:///test.yaml").unwrap();
        let mut doc = Document::new(uri, "yaml".to_string(), 1, "old content".to_string());

        doc.apply_changes(vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "new content".to_string(),
        }]);

        assert_eq!(doc.content, "new content");
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = DocumentCache::new();
        let uri = Url::parse("file:///test.yaml").unwrap();

        cache.insert(uri.clone(), "yaml".to_string(), 1, "test".to_string());

        let doc = cache.get(&uri).unwrap();
        assert_eq!(doc.content, "test");
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn test_cache_remove() {
        let cache = DocumentCache::new();
        let uri = Url::parse("file:///test.yaml").unwrap();

        cache.insert(uri.clone(), "yaml".to_string(), 1, "test".to_string());
        assert!(cache.get(&uri).is_some());

        cache.remove(&uri);
        assert!(cache.get(&uri).is_none());
    }

    #[test]
    fn test_cache_update() {
        let cache = DocumentCache::new();
        let uri = Url::parse("file:///test.yaml").unwrap();

        cache.insert(uri.clone(), "yaml".to_string(), 1, "line 1\nline 2".to_string());

        cache.update(
            &uri,
            2,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "updated content".to_string(),
            }],
        );

        let doc = cache.get(&uri).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.content, "updated content");
    }
}
