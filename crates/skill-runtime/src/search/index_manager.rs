//! Persistent index manager with incremental updates
//!
//! Provides index management for persistent storage, incremental updates,
//! and automatic synchronization of skill embeddings.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for the index manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Directory for index storage
    pub index_path: PathBuf,
    /// Embedding model name
    pub embedding_model: String,
    /// Embedding dimensions
    pub embedding_dimensions: usize,
    /// Batch size for embedding generation
    pub chunk_size: usize,
    /// Whether to index on startup
    pub index_on_startup: bool,
    /// Whether to watch for changes
    pub watch_for_changes: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        let default_path = dirs::home_dir()
            .map(|p| p.join(".skill-engine").join("index"))
            .unwrap_or_else(|| PathBuf::from(".skill-engine/index"));

        Self {
            index_path: default_path,
            embedding_model: "all-minilm".to_string(),
            embedding_dimensions: 384,
            chunk_size: 32,
            index_on_startup: true,
            watch_for_changes: false,
        }
    }
}

impl IndexConfig {
    /// Create config with custom path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            index_path: path.into(),
            ..Default::default()
        }
    }

    /// Set embedding model
    pub fn with_model(mut self, model: impl Into<String>, dimensions: usize) -> Self {
        self.embedding_model = model.into();
        self.embedding_dimensions = dimensions;
        self
    }

    /// Set chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Disable startup indexing
    pub fn no_startup_index(mut self) -> Self {
        self.index_on_startup = false;
        self
    }
}

/// Checksum for a skill to detect changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillChecksum {
    /// SKILL.md content hash
    pub skill_md_hash: String,
    /// WASM binary hash (if exists)
    pub wasm_hash: Option<String>,
    /// Manifest hash
    pub manifest_hash: Option<String>,
    /// Last indexed timestamp
    pub indexed_at: DateTime<Utc>,
}

/// Index metadata stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Metadata version
    pub version: u32,
    /// Embedding model used
    pub embedding_model: String,
    /// Embedding dimensions
    pub dimensions: usize,
    /// Index creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
    /// Total document count
    pub document_count: usize,
    /// Checksums for indexed skills
    pub skill_checksums: HashMap<String, SkillChecksum>,
}

impl IndexMetadata {
    const CURRENT_VERSION: u32 = 1;
    const METADATA_FILE: &'static str = "index_metadata.json";

    /// Create new metadata
    pub fn new(embedding_model: impl Into<String>, dimensions: usize) -> Self {
        let now = Utc::now();
        Self {
            version: Self::CURRENT_VERSION,
            embedding_model: embedding_model.into(),
            dimensions,
            created_at: now,
            last_modified: now,
            document_count: 0,
            skill_checksums: HashMap::new(),
        }
    }

    /// Load metadata from disk
    pub fn load(index_path: &Path) -> Result<Option<Self>> {
        let metadata_path = index_path.join(Self::METADATA_FILE);
        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&metadata_path)
            .context("Failed to read index metadata")?;
        let metadata: Self = serde_json::from_str(&content)
            .context("Failed to parse index metadata")?;

        Ok(Some(metadata))
    }

    /// Save metadata to disk
    pub fn save(&self, index_path: &Path) -> Result<()> {
        fs::create_dir_all(index_path)
            .context("Failed to create index directory")?;

        let metadata_path = index_path.join(Self::METADATA_FILE);
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize index metadata")?;
        fs::write(&metadata_path, content)
            .context("Failed to write index metadata")?;

        Ok(())
    }

    /// Check if metadata is compatible with config
    pub fn is_compatible(&self, config: &IndexConfig) -> bool {
        self.version == Self::CURRENT_VERSION &&
        self.embedding_model == config.embedding_model &&
        self.dimensions == config.embedding_dimensions
    }

    /// Update last modified time
    pub fn touch(&mut self) {
        self.last_modified = Utc::now();
    }
}

/// Statistics about the index
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Total skills indexed
    pub total_skills: usize,
    /// Total documents (tools) indexed
    pub total_documents: usize,
    /// Skills that need re-indexing
    pub stale_skills: usize,
    /// Index size on disk (bytes)
    pub index_size_bytes: u64,
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Skills that were added
    pub added: Vec<String>,
    /// Skills that were updated
    pub updated: Vec<String>,
    /// Skills that were removed
    pub removed: Vec<String>,
    /// Skills that were skipped (unchanged)
    pub skipped: usize,
    /// Whether a full reindex was required
    pub full_reindex: bool,
}

impl SyncResult {
    /// Check if any changes were made
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.updated.is_empty() || !self.removed.is_empty()
    }

    /// Total number of skills processed
    pub fn total_processed(&self) -> usize {
        self.added.len() + self.updated.len() + self.removed.len() + self.skipped
    }
}

/// Index manager for persistent skill indexing
pub struct IndexManager {
    config: IndexConfig,
    metadata: IndexMetadata,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new(config: IndexConfig) -> Result<Self> {
        // Load or create metadata
        let metadata = match IndexMetadata::load(&config.index_path)? {
            Some(meta) if meta.is_compatible(&config) => meta,
            _ => IndexMetadata::new(&config.embedding_model, config.embedding_dimensions),
        };

        Ok(Self { config, metadata })
    }

    /// Get the config
    pub fn config(&self) -> &IndexConfig {
        &self.config
    }

    /// Get the metadata
    pub fn metadata(&self) -> &IndexMetadata {
        &self.metadata
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        let index_size_bytes = self.calculate_index_size();

        IndexStats {
            total_skills: self.metadata.skill_checksums.len(),
            total_documents: self.metadata.document_count,
            stale_skills: 0, // Would be calculated during sync
            index_size_bytes,
        }
    }

    /// Compute checksum for a skill
    pub fn compute_skill_checksum(&self, skill_path: &Path) -> Result<SkillChecksum> {
        let mut skill_md_hash = String::new();
        let mut wasm_hash = None;
        let mut manifest_hash = None;

        // Hash SKILL.md
        let skill_md_path = skill_path.join("SKILL.md");
        if skill_md_path.exists() {
            let content = fs::read(&skill_md_path)
                .context("Failed to read SKILL.md")?;
            skill_md_hash = self.hash_content(&content);
        }

        // Hash WASM file (if exists)
        for entry in fs::read_dir(skill_path).into_iter().flatten() {
            if let Ok(entry) = entry {
                if entry.path().extension().map_or(false, |e| e == "wasm") {
                    let content = fs::read(entry.path())
                        .context("Failed to read WASM file")?;
                    wasm_hash = Some(self.hash_content(&content));
                    break;
                }
            }
        }

        // Hash manifest (skill.toml or skill.json)
        for filename in ["skill.toml", "skill.json"] {
            let manifest_path = skill_path.join(filename);
            if manifest_path.exists() {
                let content = fs::read(&manifest_path)
                    .context("Failed to read manifest")?;
                manifest_hash = Some(self.hash_content(&content));
                break;
            }
        }

        Ok(SkillChecksum {
            skill_md_hash,
            wasm_hash,
            manifest_hash,
            indexed_at: Utc::now(),
        })
    }

    /// Check if a skill needs re-indexing
    pub fn needs_reindex(&self, skill_name: &str, skill_path: &Path) -> Result<bool> {
        // Check if skill exists in metadata
        let existing = match self.metadata.skill_checksums.get(skill_name) {
            Some(checksum) => checksum,
            None => return Ok(true), // New skill
        };

        // Compute current checksum
        let current = self.compute_skill_checksum(skill_path)?;

        // Compare checksums
        Ok(existing.skill_md_hash != current.skill_md_hash ||
           existing.wasm_hash != current.wasm_hash ||
           existing.manifest_hash != current.manifest_hash)
    }

    /// Record that a skill was indexed
    pub fn record_indexed(&mut self, skill_name: &str, checksum: SkillChecksum, doc_count: usize) -> Result<()> {
        self.metadata.skill_checksums.insert(skill_name.to_string(), checksum);
        self.metadata.document_count = self.metadata.document_count.saturating_add(doc_count);
        self.metadata.touch();
        self.save_metadata()
    }

    /// Record that a skill was removed
    pub fn record_removed(&mut self, skill_name: &str, doc_count: usize) -> Result<()> {
        self.metadata.skill_checksums.remove(skill_name);
        self.metadata.document_count = self.metadata.document_count.saturating_sub(doc_count);
        self.metadata.touch();
        self.save_metadata()
    }

    /// Determine what sync operations are needed
    pub fn plan_sync(&self, current_skills: &HashMap<String, PathBuf>) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Check for skills that need to be added or updated
        for (skill_name, skill_path) in current_skills {
            if !self.metadata.skill_checksums.contains_key(skill_name) {
                result.added.push(skill_name.clone());
            } else if self.needs_reindex(skill_name, skill_path)? {
                result.updated.push(skill_name.clone());
            } else {
                result.skipped += 1;
            }
        }

        // Check for skills that need to be removed
        for skill_name in self.metadata.skill_checksums.keys() {
            if !current_skills.contains_key(skill_name) {
                result.removed.push(skill_name.clone());
            }
        }

        Ok(result)
    }

    /// Check if full reindex is needed
    pub fn needs_full_reindex(&self, config: &IndexConfig) -> bool {
        // Load existing metadata if any
        match IndexMetadata::load(&config.index_path) {
            Ok(Some(meta)) => !meta.is_compatible(config),
            Ok(None) => true, // No existing index
            Err(_) => true,   // Corrupted metadata
        }
    }

    /// Clear all index data
    pub fn clear(&mut self) -> Result<()> {
        self.metadata = IndexMetadata::new(&self.config.embedding_model, self.config.embedding_dimensions);
        self.save_metadata()?;

        // Clear data files (keep metadata)
        let data_dir = self.config.index_path.join("data");
        if data_dir.exists() {
            fs::remove_dir_all(&data_dir)
                .context("Failed to remove index data")?;
        }

        Ok(())
    }

    /// Save metadata to disk
    fn save_metadata(&self) -> Result<()> {
        self.metadata.save(&self.config.index_path)
    }

    /// Calculate total index size on disk
    fn calculate_index_size(&self) -> u64 {
        if !self.config.index_path.exists() {
            return 0;
        }

        walkdir::WalkDir::new(&self.config.index_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    }

    /// Hash content using blake3
    fn hash_content(&self, content: &[u8]) -> String {
        use std::io::Write;
        let mut hasher = blake3::Hasher::new();
        hasher.write_all(content).expect("write to hasher");
        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_config() -> (IndexConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = IndexConfig::with_path(temp_dir.path().join("index"));
        (config, temp_dir)
    }

    #[test]
    fn test_config_default() {
        let config = IndexConfig::default();
        assert!(config.index_path.to_str().unwrap().contains(".skill-engine"));
        assert_eq!(config.embedding_model, "all-minilm");
        assert_eq!(config.embedding_dimensions, 384);
        assert_eq!(config.chunk_size, 32);
        assert!(config.index_on_startup);
    }

    #[test]
    fn test_config_builder() {
        let config = IndexConfig::with_path("/tmp/test")
            .with_model("bge-small", 384)
            .with_chunk_size(64)
            .no_startup_index();

        assert_eq!(config.index_path, PathBuf::from("/tmp/test"));
        assert_eq!(config.embedding_model, "bge-small");
        assert_eq!(config.chunk_size, 64);
        assert!(!config.index_on_startup);
    }

    #[test]
    fn test_metadata_new() {
        let meta = IndexMetadata::new("test-model", 384);
        assert_eq!(meta.version, IndexMetadata::CURRENT_VERSION);
        assert_eq!(meta.embedding_model, "test-model");
        assert_eq!(meta.dimensions, 384);
        assert_eq!(meta.document_count, 0);
        assert!(meta.skill_checksums.is_empty());
    }

    #[test]
    fn test_metadata_save_load() {
        let (config, _temp) = temp_config();

        let mut meta = IndexMetadata::new(&config.embedding_model, config.embedding_dimensions);
        meta.document_count = 42;
        meta.skill_checksums.insert(
            "test-skill".to_string(),
            SkillChecksum {
                skill_md_hash: "abc123".to_string(),
                wasm_hash: Some("def456".to_string()),
                manifest_hash: None,
                indexed_at: Utc::now(),
            },
        );

        meta.save(&config.index_path).unwrap();
        let loaded = IndexMetadata::load(&config.index_path).unwrap().unwrap();

        assert_eq!(loaded.document_count, 42);
        assert!(loaded.skill_checksums.contains_key("test-skill"));
    }

    #[test]
    fn test_metadata_compatibility() {
        let config = IndexConfig::default();
        let meta = IndexMetadata::new(&config.embedding_model, config.embedding_dimensions);
        assert!(meta.is_compatible(&config));

        let mut incompatible_config = config.clone();
        incompatible_config.embedding_model = "different-model".to_string();
        assert!(!meta.is_compatible(&incompatible_config));
    }

    #[test]
    fn test_index_manager_creation() {
        let (config, _temp) = temp_config();
        let manager = IndexManager::new(config.clone()).unwrap();
        assert_eq!(manager.metadata().embedding_model, config.embedding_model);
    }

    #[test]
    fn test_skill_checksum() {
        let (config, temp) = temp_config();
        let manager = IndexManager::new(config).unwrap();

        // Create a test skill
        let skill_dir = temp.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill\n").unwrap();
        fs::write(skill_dir.join("skill.toml"), "name = \"test\"").unwrap();

        let checksum = manager.compute_skill_checksum(&skill_dir).unwrap();
        assert!(!checksum.skill_md_hash.is_empty());
        assert!(checksum.manifest_hash.is_some());
        assert!(checksum.wasm_hash.is_none());
    }

    #[test]
    fn test_needs_reindex() {
        let (config, temp) = temp_config();
        let mut manager = IndexManager::new(config).unwrap();

        // Create a test skill
        let skill_dir = temp.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill v1\n").unwrap();

        // New skill should need indexing
        assert!(manager.needs_reindex("test-skill", &skill_dir).unwrap());

        // Record it as indexed
        let checksum = manager.compute_skill_checksum(&skill_dir).unwrap();
        manager.record_indexed("test-skill", checksum, 5).unwrap();

        // Should not need re-indexing now
        assert!(!manager.needs_reindex("test-skill", &skill_dir).unwrap());

        // Modify skill
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill v2\n").unwrap();

        // Should need re-indexing now
        assert!(manager.needs_reindex("test-skill", &skill_dir).unwrap());
    }

    #[test]
    fn test_plan_sync() {
        let (config, temp) = temp_config();
        let mut manager = IndexManager::new(config).unwrap();

        // Setup: record some indexed skills
        let checksum = SkillChecksum {
            skill_md_hash: "old_hash".to_string(),
            wasm_hash: None,
            manifest_hash: None,
            indexed_at: Utc::now(),
        };
        manager.record_indexed("existing-skill", checksum.clone(), 3).unwrap();
        manager.record_indexed("removed-skill", checksum, 2).unwrap();

        // Create skill directories
        let existing_skill_dir = temp.path().join("existing-skill");
        let new_skill_dir = temp.path().join("new-skill");
        fs::create_dir_all(&existing_skill_dir).unwrap();
        fs::create_dir_all(&new_skill_dir).unwrap();
        fs::write(existing_skill_dir.join("SKILL.md"), "# Existing\n").unwrap();
        fs::write(new_skill_dir.join("SKILL.md"), "# New\n").unwrap();

        // Current skills (existing changed, new added, removed missing)
        let mut current_skills = HashMap::new();
        current_skills.insert("existing-skill".to_string(), existing_skill_dir);
        current_skills.insert("new-skill".to_string(), new_skill_dir);

        let result = manager.plan_sync(&current_skills).unwrap();

        assert!(result.added.contains(&"new-skill".to_string()));
        assert!(result.updated.contains(&"existing-skill".to_string())); // Hash changed
        assert!(result.removed.contains(&"removed-skill".to_string()));
    }

    #[test]
    fn test_sync_result() {
        let mut result = SyncResult::default();
        assert!(!result.has_changes());
        assert_eq!(result.total_processed(), 0);

        result.added.push("skill-1".to_string());
        result.skipped = 2;
        assert!(result.has_changes());
        assert_eq!(result.total_processed(), 3);
    }

    #[test]
    fn test_clear_index() {
        let (config, _temp) = temp_config();
        let mut manager = IndexManager::new(config).unwrap();

        // Add some data
        let checksum = SkillChecksum {
            skill_md_hash: "hash".to_string(),
            wasm_hash: None,
            manifest_hash: None,
            indexed_at: Utc::now(),
        };
        manager.record_indexed("test-skill", checksum, 10).unwrap();
        assert!(!manager.metadata().skill_checksums.is_empty());

        // Clear
        manager.clear().unwrap();
        assert!(manager.metadata().skill_checksums.is_empty());
        assert_eq!(manager.metadata().document_count, 0);
    }
}
