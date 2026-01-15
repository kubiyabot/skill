//! Context storage and persistence.
//!
//! This module provides functionality for persisting execution contexts to the
//! filesystem and loading them back. Contexts are stored in TOML format.
//!
//! # Storage Layout
//!
//! ```text
//! ~/.skill-engine/
//! ├── contexts/
//! │   ├── index.json              # Context index for fast listing
//! │   ├── {context-id}/
//! │   │   ├── context.toml        # Context definition
//! │   │   └── .backup/            # Backup versions
//! │   │       ├── context.toml.1  # Previous version
//! │   │       └── context.toml.2  # Older version
//! └── templates/
//!     └── contexts/
//!         └── default.toml        # Default context template
//! ```

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::ExecutionContext;
use crate::ContextError;

/// Default number of backup versions to keep.
const DEFAULT_BACKUP_COUNT: usize = 5;

/// Storage layer for execution contexts.
pub struct ContextStorage {
    /// Base directory for context storage.
    base_dir: PathBuf,
    /// Number of backup versions to keep.
    backup_count: usize,
}

impl ContextStorage {
    /// Create a new context storage with the default base directory.
    ///
    /// Uses `~/.skill-engine/contexts` as the base directory.
    pub fn new() -> Result<Self, ContextError> {
        let base_dir = dirs::home_dir()
            .ok_or_else(|| ContextError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine home directory",
            )))?
            .join(".skill-engine")
            .join("contexts");

        Self::with_base_dir(base_dir)
    }

    /// Create a new context storage with a custom base directory.
    pub fn with_base_dir(base_dir: PathBuf) -> Result<Self, ContextError> {
        fs::create_dir_all(&base_dir)?;

        Ok(Self {
            base_dir,
            backup_count: DEFAULT_BACKUP_COUNT,
        })
    }

    /// Set the number of backup versions to keep.
    pub fn with_backup_count(mut self, count: usize) -> Self {
        self.backup_count = count;
        self
    }

    /// Get the base directory.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Get the path for a context's directory.
    fn context_dir(&self, context_id: &str) -> PathBuf {
        self.base_dir.join(context_id)
    }

    /// Get the path for a context's TOML file.
    fn context_file(&self, context_id: &str) -> PathBuf {
        self.context_dir(context_id).join("context.toml")
    }

    /// Get the backup directory for a context.
    fn backup_dir(&self, context_id: &str) -> PathBuf {
        self.context_dir(context_id).join(".backup")
    }

    /// Get the index file path.
    fn index_file(&self) -> PathBuf {
        self.base_dir.join("index.json")
    }

    /// Save a context to storage.
    ///
    /// This performs an atomic write (write to temp file, then rename) to
    /// prevent corruption from interrupted writes.
    pub fn save(&self, context: &ExecutionContext) -> Result<(), ContextError> {
        let context_dir = self.context_dir(&context.id);
        fs::create_dir_all(&context_dir)?;

        let context_file = self.context_file(&context.id);

        // Create backup if file already exists
        if context_file.exists() {
            self.create_backup(&context.id)?;
        }

        // Serialize to TOML
        let toml_content = toml::to_string_pretty(context)?;

        // Atomic write: write to temp file, then rename
        let temp_file = context_dir.join(".context.toml.tmp");
        {
            let mut file = fs::File::create(&temp_file)?;
            file.write_all(toml_content.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(&temp_file, &context_file)?;

        // Update index
        self.update_index(&context.id, Some(context))?;

        Ok(())
    }

    /// Load a context from storage.
    pub fn load(&self, context_id: &str) -> Result<ExecutionContext, ContextError> {
        let context_file = self.context_file(context_id);

        if !context_file.exists() {
            return Err(ContextError::NotFound(context_id.to_string()));
        }

        let content = fs::read_to_string(&context_file)?;
        let context: ExecutionContext = toml::from_str(&content)?;

        Ok(context)
    }

    /// Delete a context from storage.
    pub fn delete(&self, context_id: &str) -> Result<(), ContextError> {
        let context_dir = self.context_dir(context_id);

        if !context_dir.exists() {
            return Err(ContextError::NotFound(context_id.to_string()));
        }

        fs::remove_dir_all(&context_dir)?;

        // Update index
        self.update_index(context_id, None)?;

        Ok(())
    }

    /// Check if a context exists.
    pub fn exists(&self, context_id: &str) -> bool {
        self.context_file(context_id).exists()
    }

    /// List all context IDs.
    pub fn list(&self) -> Result<Vec<String>, ContextError> {
        let index = self.load_index()?;
        Ok(index.contexts.keys().cloned().collect())
    }

    /// List all contexts with their metadata.
    pub fn list_with_metadata(&self) -> Result<Vec<ContextIndexEntry>, ContextError> {
        let index = self.load_index()?;
        Ok(index.contexts.into_values().collect())
    }

    /// Get the index entry for a context without loading the full context.
    pub fn get_metadata(&self, context_id: &str) -> Result<ContextIndexEntry, ContextError> {
        let index = self.load_index()?;
        index
            .contexts
            .get(context_id)
            .cloned()
            .ok_or_else(|| ContextError::NotFound(context_id.to_string()))
    }

    /// Export a context and all its parent contexts to a directory.
    pub fn export(&self, context_id: &str, output_dir: &Path) -> Result<Vec<String>, ContextError> {
        fs::create_dir_all(output_dir)?;

        let mut exported = Vec::new();
        let mut to_export = vec![context_id.to_string()];

        while let Some(id) = to_export.pop() {
            if exported.contains(&id) {
                continue;
            }

            let context = self.load(&id)?;

            // Export this context
            let output_file = output_dir.join(format!("{}.toml", id));
            let toml_content = toml::to_string_pretty(&context)?;
            fs::write(&output_file, toml_content)?;

            exported.push(id.clone());

            // Queue parent for export
            if let Some(parent_id) = &context.inherits_from {
                to_export.push(parent_id.clone());
            }
        }

        Ok(exported)
    }

    /// Import a context from a file.
    ///
    /// Returns the ID of the imported context.
    pub fn import(&self, file_path: &Path) -> Result<String, ContextError> {
        let content = fs::read_to_string(file_path)?;
        let context: ExecutionContext = toml::from_str(&content)?;

        // Check for conflicts
        if self.exists(&context.id) {
            return Err(ContextError::AlreadyExists(context.id.clone()));
        }

        self.save(&context)?;

        Ok(context.id)
    }

    /// Import a context, optionally overwriting if it exists.
    pub fn import_with_overwrite(
        &self,
        file_path: &Path,
        overwrite: bool,
    ) -> Result<String, ContextError> {
        let content = fs::read_to_string(file_path)?;
        let context: ExecutionContext = toml::from_str(&content)?;

        if self.exists(&context.id) && !overwrite {
            return Err(ContextError::AlreadyExists(context.id.clone()));
        }

        self.save(&context)?;

        Ok(context.id)
    }

    /// Create a backup of a context.
    fn create_backup(&self, context_id: &str) -> Result<(), ContextError> {
        let context_file = self.context_file(context_id);
        let backup_dir = self.backup_dir(context_id);

        if !context_file.exists() {
            return Ok(());
        }

        fs::create_dir_all(&backup_dir)?;

        // Rotate existing backups
        for i in (1..self.backup_count).rev() {
            let old = backup_dir.join(format!("context.toml.{}", i));
            let new = backup_dir.join(format!("context.toml.{}", i + 1));
            if old.exists() {
                if i + 1 >= self.backup_count {
                    fs::remove_file(&old)?;
                } else {
                    fs::rename(&old, &new)?;
                }
            }
        }

        // Create new backup
        let backup_file = backup_dir.join("context.toml.1");
        fs::copy(&context_file, &backup_file)?;

        Ok(())
    }

    /// Restore a context from a specific backup version.
    pub fn restore_backup(&self, context_id: &str, version: usize) -> Result<(), ContextError> {
        let backup_file = self.backup_dir(context_id).join(format!("context.toml.{}", version));

        if !backup_file.exists() {
            return Err(ContextError::NotFound(format!(
                "Backup version {} for context '{}'",
                version, context_id
            )));
        }

        // Read backup content first (before rotation shifts files)
        let backup_content = fs::read_to_string(&backup_file)?;

        let context_file = self.context_file(context_id);

        // Create backup of current before restoring
        self.create_backup(context_id)?;

        // Restore from the previously read backup content
        fs::write(&context_file, backup_content)?;

        // Update index
        let context = self.load(context_id)?;
        self.update_index(context_id, Some(&context))?;

        Ok(())
    }

    /// List available backup versions for a context.
    pub fn list_backups(&self, context_id: &str) -> Result<Vec<BackupInfo>, ContextError> {
        let backup_dir = self.backup_dir(context_id);

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(version_str) = name.strip_prefix("context.toml.") {
                    if let Ok(version) = version_str.parse::<usize>() {
                        let metadata = fs::metadata(&path)?;
                        let modified = metadata
                            .modified()
                            .ok()
                            .and_then(|t| DateTime::<Utc>::from(t).into());

                        backups.push(BackupInfo {
                            version,
                            path,
                            modified_at: modified,
                            size_bytes: metadata.len(),
                        });
                    }
                }
            }
        }

        backups.sort_by_key(|b| b.version);

        Ok(backups)
    }

    /// Load the context index.
    fn load_index(&self) -> Result<ContextIndex, ContextError> {
        let index_file = self.index_file();

        if !index_file.exists() {
            return Ok(ContextIndex::default());
        }

        let content = fs::read_to_string(&index_file)?;
        let index: ContextIndex = serde_json::from_str(&content)?;

        Ok(index)
    }

    /// Update the context index.
    fn update_index(
        &self,
        context_id: &str,
        context: Option<&ExecutionContext>,
    ) -> Result<(), ContextError> {
        let mut index = self.load_index()?;

        match context {
            Some(ctx) => {
                index.contexts.insert(
                    context_id.to_string(),
                    ContextIndexEntry {
                        id: ctx.id.clone(),
                        name: ctx.name.clone(),
                        description: ctx.description.clone(),
                        inherits_from: ctx.inherits_from.clone(),
                        tags: ctx.metadata.tags.clone(),
                        created_at: ctx.metadata.created_at,
                        updated_at: ctx.metadata.updated_at,
                    },
                );
            }
            None => {
                index.contexts.remove(context_id);
            }
        }

        // Atomic write
        let index_file = self.index_file();
        let temp_file = self.base_dir.join(".index.json.tmp");

        let content = serde_json::to_string_pretty(&index)?;
        {
            let mut file = fs::File::create(&temp_file)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }

        fs::rename(&temp_file, &index_file)?;

        Ok(())
    }

    /// Rebuild the index by scanning all contexts.
    ///
    /// Useful if the index becomes corrupted or out of sync.
    pub fn rebuild_index(&self) -> Result<usize, ContextError> {
        let mut index = ContextIndex::default();
        let mut count = 0;

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let context_file = path.join("context.toml");
                if context_file.exists() {
                    if let Ok(context) = self.load(entry.file_name().to_str().unwrap_or_default()) {
                        index.contexts.insert(
                            context.id.clone(),
                            ContextIndexEntry {
                                id: context.id.clone(),
                                name: context.name.clone(),
                                description: context.description.clone(),
                                inherits_from: context.inherits_from.clone(),
                                tags: context.metadata.tags.clone(),
                                created_at: context.metadata.created_at,
                                updated_at: context.metadata.updated_at,
                            },
                        );
                        count += 1;
                    }
                }
            }
        }

        // Write index
        let index_file = self.index_file();
        let content = serde_json::to_string_pretty(&index)?;
        fs::write(&index_file, content)?;

        Ok(count)
    }
}

impl Default for ContextStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create default context storage")
    }
}

/// Context index for fast listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextIndex {
    /// Map of context ID to index entry.
    pub contexts: HashMap<String, ContextIndexEntry>,
}

/// Entry in the context index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIndexEntry {
    /// Context ID.
    pub id: String,
    /// Context name.
    pub name: String,
    /// Context description.
    pub description: Option<String>,
    /// Parent context ID.
    pub inherits_from: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Information about a backup version.
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Backup version number (1 = most recent).
    pub version: usize,
    /// Path to the backup file.
    pub path: PathBuf,
    /// When the backup was created.
    pub modified_at: Option<DateTime<Utc>>,
    /// Size in bytes.
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (ContextStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = ContextStorage::with_base_dir(temp_dir.path().to_path_buf()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_save_and_load() {
        let (storage, _temp) = create_test_storage();

        let context = ExecutionContext::new("test-context", "Test Context")
            .with_description("A test context")
            .with_tag("test");

        storage.save(&context).unwrap();
        assert!(storage.exists("test-context"));

        let loaded = storage.load("test-context").unwrap();
        assert_eq!(loaded.id, "test-context");
        assert_eq!(loaded.name, "Test Context");
        assert_eq!(loaded.description, Some("A test context".to_string()));
    }

    #[test]
    fn test_delete() {
        let (storage, _temp) = create_test_storage();

        let context = ExecutionContext::new("to-delete", "To Delete");
        storage.save(&context).unwrap();
        assert!(storage.exists("to-delete"));

        storage.delete("to-delete").unwrap();
        assert!(!storage.exists("to-delete"));
    }

    #[test]
    fn test_list() {
        let (storage, _temp) = create_test_storage();

        storage
            .save(&ExecutionContext::new("ctx-1", "Context 1"))
            .unwrap();
        storage
            .save(&ExecutionContext::new("ctx-2", "Context 2"))
            .unwrap();
        storage
            .save(&ExecutionContext::new("ctx-3", "Context 3"))
            .unwrap();

        let list = storage.list().unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"ctx-1".to_string()));
        assert!(list.contains(&"ctx-2".to_string()));
        assert!(list.contains(&"ctx-3".to_string()));
    }

    #[test]
    fn test_index_metadata() {
        let (storage, _temp) = create_test_storage();

        let context = ExecutionContext::new("indexed", "Indexed Context")
            .with_description("Has metadata")
            .with_tag("important")
            .with_tag("production");

        storage.save(&context).unwrap();

        let metadata = storage.get_metadata("indexed").unwrap();
        assert_eq!(metadata.name, "Indexed Context");
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_backup_creation() {
        let (storage, _temp) = create_test_storage();

        // Save initial version
        let mut context = ExecutionContext::new("backup-test", "Backup Test");
        storage.save(&context).unwrap();

        // Modify and save again
        context.description = Some("Modified".to_string());
        context.touch();
        storage.save(&context).unwrap();

        // Check backup exists
        let backups = storage.list_backups("backup-test").unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].version, 1);
    }

    #[test]
    fn test_backup_rotation() {
        let (storage, _temp) = create_test_storage();
        let storage = storage.with_backup_count(3);

        let mut context = ExecutionContext::new("rotation-test", "Rotation Test");

        // Create 5 versions
        for i in 0..5 {
            context.description = Some(format!("Version {}", i));
            context.touch();
            storage.save(&context).unwrap();
        }

        // Should only have 3 backups (backup_count - 1 = 2 older + current doesn't count)
        let backups = storage.list_backups("rotation-test").unwrap();
        assert!(backups.len() <= 3);
    }

    #[test]
    fn test_restore_backup() {
        let (storage, _temp) = create_test_storage();

        // Save initial
        let mut context = ExecutionContext::new("restore-test", "Restore Test");
        context.description = Some("Original".to_string());
        storage.save(&context).unwrap();

        // Modify
        context.description = Some("Modified".to_string());
        context.touch();
        storage.save(&context).unwrap();

        // Restore
        storage.restore_backup("restore-test", 1).unwrap();

        // Check restored value
        let restored = storage.load("restore-test").unwrap();
        assert_eq!(restored.description, Some("Original".to_string()));
    }

    #[test]
    fn test_export_import() {
        let (storage, _temp) = create_test_storage();

        // Create parent and child contexts
        let parent = ExecutionContext::new("parent", "Parent Context");
        let child = ExecutionContext::inheriting("child", "Child Context", "parent");

        storage.save(&parent).unwrap();
        storage.save(&child).unwrap();

        // Export child (should include parent)
        let export_dir = _temp.path().join("export");
        let exported = storage.export("child", &export_dir).unwrap();

        assert_eq!(exported.len(), 2);
        assert!(exported.contains(&"parent".to_string()));
        assert!(exported.contains(&"child".to_string()));

        // Create new storage and import
        let import_dir = _temp.path().join("import");
        fs::create_dir_all(&import_dir).unwrap();
        let import_storage = ContextStorage::with_base_dir(import_dir).unwrap();

        // Import parent first
        import_storage
            .import(&export_dir.join("parent.toml"))
            .unwrap();
        import_storage
            .import(&export_dir.join("child.toml"))
            .unwrap();

        assert!(import_storage.exists("parent"));
        assert!(import_storage.exists("child"));
    }

    #[test]
    fn test_import_conflict() {
        let (storage, _temp) = create_test_storage();

        let context = ExecutionContext::new("conflict", "Conflict Test");
        storage.save(&context).unwrap();

        // Export
        let export_dir = _temp.path().join("export");
        storage.export("conflict", &export_dir).unwrap();

        // Try to import again - should fail
        let result = storage.import(&export_dir.join("conflict.toml"));
        assert!(matches!(result, Err(ContextError::AlreadyExists(_))));

        // Import with overwrite should succeed
        let result = storage.import_with_overwrite(&export_dir.join("conflict.toml"), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebuild_index() {
        let (storage, _temp) = create_test_storage();

        // Save some contexts
        storage
            .save(&ExecutionContext::new("ctx-1", "Context 1"))
            .unwrap();
        storage
            .save(&ExecutionContext::new("ctx-2", "Context 2"))
            .unwrap();

        // Delete the index file manually
        fs::remove_file(storage.index_file()).ok();

        // Rebuild
        let count = storage.rebuild_index().unwrap();
        assert_eq!(count, 2);

        // Verify list works
        let list = storage.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_not_found() {
        let (storage, _temp) = create_test_storage();

        let result = storage.load("nonexistent");
        assert!(matches!(result, Err(ContextError::NotFound(_))));

        let result = storage.delete("nonexistent");
        assert!(matches!(result, Err(ContextError::NotFound(_))));
    }
}
