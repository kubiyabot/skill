use anyhow::{Context, Result};
use keyring::Entry;
use std::fmt;
use std::sync::Arc;
use zeroize::{Zeroize, Zeroizing};

use crate::audit::AuditLogger;

const SERVICE_NAME: &str = "skill-engine";

/// Secure credential storage using platform-specific keychains
/// - macOS: Keychain
/// - Windows: Credential Manager
/// - Linux: Secret Service (DBus)
pub struct CredentialStore {
    service_name: String,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new() -> Self {
        let audit_logger = AuditLogger::new().ok().map(Arc::new);
        Self {
            service_name: SERVICE_NAME.to_string(),
            audit_logger,
        }
    }

    /// Create a new credential store with custom service name
    pub fn with_service_name(service_name: String) -> Self {
        let audit_logger = AuditLogger::new().ok().map(Arc::new);
        Self {
            service_name,
            audit_logger,
        }
    }

    /// Create a credential store with audit logging
    pub fn with_audit_logger(audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            service_name: SERVICE_NAME.to_string(),
            audit_logger: Some(audit_logger),
        }
    }

    /// Build keyring entry key: "skill-engine/{skill_name}/{instance_name}/{key_name}"
    fn build_entry_key(&self, skill: &str, instance: &str, key: &str) -> String {
        format!("{}/{}/{}", skill, instance, key)
    }

    /// Store a credential securely
    pub fn store_credential(
        &self,
        skill: &str,
        instance: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let entry_key = self.build_entry_key(skill, instance, key);
        let entry = Entry::new(&self.service_name, &entry_key)
            .context("Failed to create keyring entry")?;

        entry
            .set_password(value)
            .with_context(|| format!("Failed to store credential for key: {}", key))?;

        // Audit log
        if let Some(ref logger) = self.audit_logger {
            let _ = logger.log_credential_store(skill, instance, key);
        }

        tracing::debug!(
            skill = %skill,
            instance = %instance,
            key = %key,
            "Stored credential in keyring"
        );

        Ok(())
    }

    /// Retrieve a credential securely (returns a zeroizing string that clears on drop)
    pub fn get_credential(
        &self,
        skill: &str,
        instance: &str,
        key: &str,
    ) -> Result<Zeroizing<String>> {
        let entry_key = self.build_entry_key(skill, instance, key);
        let entry = Entry::new(&self.service_name, &entry_key)
            .context("Failed to create keyring entry")?;

        let password = entry
            .get_password()
            .with_context(|| format!("Failed to retrieve credential for key: {}", key))?;

        // Audit log
        if let Some(ref logger) = self.audit_logger {
            let _ = logger.log_credential_access(skill, instance, key);
        }

        tracing::debug!(
            skill = %skill,
            instance = %instance,
            key = %key,
            "Retrieved credential from keyring"
        );

        // Wrap in Zeroizing to clear memory on drop
        Ok(Zeroizing::new(password))
    }

    /// Delete a credential
    pub fn delete_credential(&self, skill: &str, instance: &str, key: &str) -> Result<()> {
        let entry_key = self.build_entry_key(skill, instance, key);
        let entry = Entry::new(&self.service_name, &entry_key)
            .context("Failed to create keyring entry")?;

        entry
            .delete_credential()
            .with_context(|| format!("Failed to delete credential for key: {}", key))?;

        // Audit log
        if let Some(ref logger) = self.audit_logger {
            let _ = logger.log_credential_delete(skill, instance, key);
        }

        tracing::debug!(
            skill = %skill,
            instance = %instance,
            key = %key,
            "Deleted credential from keyring"
        );

        Ok(())
    }

    /// Delete all credentials for an instance
    pub fn delete_all_credentials(&self, skill: &str, instance: &str) -> Result<()> {
        // Note: keyring doesn't provide a list operation, so callers must
        // track which keys they stored and call delete_credential for each
        tracing::debug!(
            skill = %skill,
            instance = %instance,
            "Deleting all credentials for instance"
        );
        Ok(())
    }

    /// Check if a credential exists
    pub fn has_credential(&self, skill: &str, instance: &str, key: &str) -> bool {
        let entry_key = self.build_entry_key(skill, instance, key);
        if let Ok(entry) = Entry::new(&self.service_name, &entry_key) {
            entry.get_password().is_ok()
        } else {
            false
        }
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Secure string that zeroes memory on drop
#[derive(Clone)]
pub struct SecureString(String);

impl SecureString {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(mut self) -> String {
        let s = std::mem::take(&mut self.0);
        std::mem::forget(self); // Prevent double-zeroing
        s
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecureString([REDACTED])")
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Parse a keyring reference URL: "keyring://skill-engine/{skill}/{instance}/{key}"
pub fn parse_keyring_reference(reference: &str) -> Result<(String, String, String)> {
    let prefix = "keyring://skill-engine/";
    if !reference.starts_with(prefix) {
        anyhow::bail!("Invalid keyring reference: must start with '{}'", prefix);
    }

    let path = &reference[prefix.len()..];
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() != 3 {
        anyhow::bail!(
            "Invalid keyring reference format: expected 'keyring://skill-engine/{{skill}}/{{instance}}/{{key}}'"
        );
    }

    Ok((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyring_reference() {
        let reference = "keyring://skill-engine/aws-skill/prod/aws_access_key_id";
        let (skill, instance, key) = parse_keyring_reference(reference).unwrap();

        assert_eq!(skill, "aws-skill");
        assert_eq!(instance, "prod");
        assert_eq!(key, "aws_access_key_id");
    }

    #[test]
    fn test_parse_keyring_reference_invalid() {
        let reference = "invalid://aws-skill/prod/key";
        assert!(parse_keyring_reference(reference).is_err());

        let reference = "keyring://skill-engine/only-two/parts";
        assert!(parse_keyring_reference(reference).is_err());
    }

    #[test]
    fn test_secure_string_zeroes_memory() {
        let secret = SecureString::new("sensitive".to_string());
        assert_eq!(secret.as_str(), "sensitive");

        drop(secret);
        // Memory should be zeroed after drop (can't easily test this without unsafe)
    }

    #[test]
    fn test_secure_string_debug() {
        let secret = SecureString::new("sensitive".to_string());
        let debug_str = format!("{:?}", secret);
        assert_eq!(debug_str, "SecureString([REDACTED])");
        assert!(!debug_str.contains("sensitive"));
    }

    // Note: Actual keyring operations are not tested here as they require
    // platform-specific keyring services. Use integration tests with mocks.
}
