use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    CredentialAccess,
    CredentialStore,
    CredentialDelete,
    InstanceCreate,
    InstanceDelete,
    ConfigLoad,
    ConfigUpdate,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub skill_name: String,
    pub instance_name: String,
    pub details: Option<String>,
    /// Redacted information (never contains actual secrets)
    pub metadata: Option<serde_json::Value>,
}

impl AuditEntry {
    pub fn new(
        event_type: AuditEventType,
        skill_name: String,
        instance_name: String,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            skill_name,
            instance_name,
            details: None,
            metadata: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Audit logger for security-sensitive operations
pub struct AuditLogger {
    log_file: Mutex<File>,
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let log_path = home.join(".skill-engine").join("audit.log");

        // Create parent directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open log file in append mode
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open audit log: {}", log_path.display()))?;

        Ok(Self {
            log_file: Mutex::new(log_file),
            log_path,
        })
    }

    /// Log an audit event
    pub fn log(&self, entry: AuditEntry) -> Result<()> {
        let json = serde_json::to_string(&entry)?;

        let mut file = self
            .log_file
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock audit log: {}", e))?;

        writeln!(file, "{}", json)?;
        file.flush()?;

        tracing::debug!(
            event = ?entry.event_type,
            skill = %entry.skill_name,
            instance = %entry.instance_name,
            "Audit event logged"
        );

        Ok(())
    }

    /// Log credential access
    pub fn log_credential_access(
        &self,
        skill_name: &str,
        instance_name: &str,
        key_name: &str,
    ) -> Result<()> {
        let entry = AuditEntry::new(
            AuditEventType::CredentialAccess,
            skill_name.to_string(),
            instance_name.to_string(),
        )
        .with_details(format!("Accessed credential key: {}", key_name));

        self.log(entry)
    }

    /// Log credential storage
    pub fn log_credential_store(
        &self,
        skill_name: &str,
        instance_name: &str,
        key_name: &str,
    ) -> Result<()> {
        let entry = AuditEntry::new(
            AuditEventType::CredentialStore,
            skill_name.to_string(),
            instance_name.to_string(),
        )
        .with_details(format!("Stored credential key: {}", key_name));

        self.log(entry)
    }

    /// Log credential deletion
    pub fn log_credential_delete(
        &self,
        skill_name: &str,
        instance_name: &str,
        key_name: &str,
    ) -> Result<()> {
        let entry = AuditEntry::new(
            AuditEventType::CredentialDelete,
            skill_name.to_string(),
            instance_name.to_string(),
        )
        .with_details(format!("Deleted credential key: {}", key_name));

        self.log(entry)
    }

    /// Get the audit log path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// Read recent audit entries
    pub fn read_recent(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        use std::io::{BufRead, BufReader};

        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);

        let entries: Vec<AuditEntry> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect();

        // Return last N entries
        Ok(entries.into_iter().rev().take(limit).rev().collect())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new().expect("Failed to create AuditLogger")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            AuditEventType::CredentialAccess,
            "test-skill".to_string(),
            "prod".to_string(),
        )
        .with_details("Test access".to_string());

        assert_eq!(entry.skill_name, "test-skill");
        assert_eq!(entry.instance_name, "prod");
        assert_eq!(entry.details, Some("Test access".to_string()));
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new(
            AuditEventType::CredentialStore,
            "test-skill".to_string(),
            "prod".to_string(),
        );

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.skill_name, entry.skill_name);
        assert_eq!(deserialized.instance_name, entry.instance_name);
    }
}
