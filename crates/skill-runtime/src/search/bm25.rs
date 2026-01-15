//! BM25 sparse retrieval using Tantivy
//!
//! Provides keyword-based search using the BM25 scoring algorithm
//! for complementing dense vector search.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, STORED, TEXT},
    Index, IndexReader, IndexWriter, TantivyDocument,
};

/// Configuration for BM25 index
#[derive(Debug, Clone)]
pub struct BM25Config {
    /// Directory for index storage (None = RAM)
    pub index_dir: Option<PathBuf>,
    /// BM25 k1 parameter (term frequency saturation)
    pub k1: f32,
    /// BM25 b parameter (document length normalization)
    pub b: f32,
    /// Number of indexing threads
    pub num_threads: usize,
    /// Heap size for indexing (in bytes)
    pub heap_size: usize,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            index_dir: None,
            k1: 1.2,
            b: 0.75,
            num_threads: 1,
            heap_size: 50_000_000, // 50MB
        }
    }
}

impl BM25Config {
    /// Create config for RAM-based index
    pub fn in_memory() -> Self {
        Self::default()
    }

    /// Create config for disk-based index
    pub fn persistent(path: impl Into<PathBuf>) -> Self {
        Self {
            index_dir: Some(path.into()),
            ..Default::default()
        }
    }
}

/// Search result from BM25 index
#[derive(Debug, Clone)]
pub struct BM25SearchResult {
    /// Document ID
    pub id: String,
    /// BM25 score
    pub score: f32,
    /// Tool name
    pub tool_name: String,
    /// Skill name
    pub skill_name: String,
}

/// BM25 index for sparse keyword retrieval
pub struct BM25Index {
    index: Index,
    reader: IndexReader,
    writer: Option<IndexWriter>,
    // Schema fields
    id_field: Field,
    tool_name_field: Field,
    skill_name_field: Field,
    description_field: Field,
    full_text_field: Field,
    config: BM25Config,
}

impl BM25Index {
    /// Create a new BM25 index
    pub fn new(config: BM25Config) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // Define fields
        let id_field = schema_builder.add_text_field("id", STORED);
        let tool_name_field = schema_builder.add_text_field("tool_name", TEXT | STORED);
        let skill_name_field = schema_builder.add_text_field("skill_name", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT);
        let full_text_field = schema_builder.add_text_field("full_text", TEXT);

        let schema = schema_builder.build();

        // Create index
        let index = if let Some(ref dir) = config.index_dir {
            std::fs::create_dir_all(dir).context("Failed to create index directory")?;
            Index::create_in_dir(dir, schema).context("Failed to create index in directory")?
        } else {
            Index::create_in_ram(schema)
        };

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        let writer = index
            .writer(config.heap_size)
            .context("Failed to create index writer")?;

        Ok(Self {
            index,
            reader,
            writer: Some(writer),
            id_field,
            tool_name_field,
            skill_name_field,
            description_field,
            full_text_field,
            config,
        })
    }

    /// Open an existing index from disk
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let index = Index::open_in_dir(&path).context("Failed to open index")?;

        let schema = index.schema();
        let id_field = schema.get_field("id").context("Missing id field")?;
        let tool_name_field = schema.get_field("tool_name").context("Missing tool_name field")?;
        let skill_name_field = schema.get_field("skill_name").context("Missing skill_name field")?;
        let description_field = schema.get_field("description").context("Missing description field")?;
        let full_text_field = schema.get_field("full_text").context("Missing full_text field")?;

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        let config = BM25Config::persistent(&path);

        Ok(Self {
            index,
            reader,
            writer: None,
            id_field,
            tool_name_field,
            skill_name_field,
            description_field,
            full_text_field,
            config,
        })
    }

    /// Add a document to the index
    pub fn add_document(
        &mut self,
        id: &str,
        tool_name: &str,
        skill_name: &str,
        description: &str,
        full_text: &str,
    ) -> Result<()> {
        let writer = self.writer.as_mut().context("Index not writable")?;

        let doc = doc!(
            self.id_field => id,
            self.tool_name_field => tool_name,
            self.skill_name_field => skill_name,
            self.description_field => description,
            self.full_text_field => full_text,
        );

        writer.add_document(doc).context("Failed to add document")?;
        Ok(())
    }

    /// Commit pending changes
    pub fn commit(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.commit().context("Failed to commit")?;
            self.reader.reload().context("Failed to reload reader")?;
        }
        Ok(())
    }

    /// Delete all documents
    pub fn clear(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.delete_all_documents().context("Failed to clear index")?;
            writer.commit().context("Failed to commit clear")?;
            self.reader.reload().context("Failed to reload reader")?;
        }
        Ok(())
    }

    /// Search the index
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<BM25SearchResult>> {
        let searcher = self.reader.searcher();

        // Search across all text fields
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.tool_name_field,
                self.description_field,
                self.full_text_field,
            ],
        );

        let parsed_query = query_parser
            .parse_query(query)
            .context("Failed to parse query")?;

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(top_k))
            .context("Search failed")?;

        let mut results = Vec::with_capacity(top_docs.len());

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher
                .doc(doc_address)
                .context("Failed to retrieve document")?;

            let id = retrieved_doc
                .get_first(self.id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tool_name = retrieved_doc
                .get_first(self.tool_name_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let skill_name = retrieved_doc
                .get_first(self.skill_name_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(BM25SearchResult {
                id,
                score,
                tool_name,
                skill_name,
            });
        }

        Ok(results)
    }

    /// Get document count
    pub fn document_count(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Get the config
    pub fn config(&self) -> &BM25Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_index() {
        let index = BM25Index::new(BM25Config::in_memory()).unwrap();
        assert_eq!(index.document_count(), 0);
    }

    #[test]
    fn test_add_and_search() {
        let mut index = BM25Index::new(BM25Config::in_memory()).unwrap();

        index
            .add_document(
                "k8s@default/list_pods",
                "list_pods",
                "kubernetes",
                "List all pods in the cluster",
                "List pods kubernetes k8s containers",
            )
            .unwrap();

        index
            .add_document(
                "k8s@default/get_deployment",
                "get_deployment",
                "kubernetes",
                "Get deployment details",
                "Get deployment kubernetes k8s",
            )
            .unwrap();

        index.commit().unwrap();

        assert_eq!(index.document_count(), 2);

        // Search for pods
        let results = index.search("pods", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "k8s@default/list_pods");

        // Search for deployment
        let results = index.search("deployment", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "k8s@default/get_deployment");
    }

    #[test]
    fn test_clear_index() {
        let mut index = BM25Index::new(BM25Config::in_memory()).unwrap();

        index
            .add_document("test", "test", "test", "test desc", "test full")
            .unwrap();
        index.commit().unwrap();

        assert_eq!(index.document_count(), 1);

        index.clear().unwrap();
        assert_eq!(index.document_count(), 0);
    }

    #[test]
    fn test_search_empty_index() {
        let index = BM25Index::new(BM25Config::in_memory()).unwrap();
        let results = index.search("anything", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_multi_term_search() {
        let mut index = BM25Index::new(BM25Config::in_memory()).unwrap();

        index
            .add_document(
                "doc1",
                "list_pods",
                "kubernetes",
                "List all pods in namespace",
                "kubernetes list pods namespace container",
            )
            .unwrap();

        index.commit().unwrap();

        // Multi-term should find the document
        let results = index.search("kubernetes pods", 10).unwrap();
        assert!(!results.is_empty());
    }
}
