//! Manifest loading and utilities for the CLI.

use anyhow::Result;
use skill_runtime::SkillManifest;
use std::path::Path;

/// Load manifest from path or auto-detect
pub fn load_manifest(path: Option<&Path>) -> Result<Option<SkillManifest>> {
    if let Some(path) = path {
        // Explicit path provided
        let manifest = SkillManifest::load(path)?;
        tracing::info!("Loaded manifest from {}", path.display());
        return Ok(Some(manifest));
    }

    // Try to auto-detect manifest in current directory or parents
    let cwd = std::env::current_dir()?;
    if let Some(manifest_path) = SkillManifest::find(&cwd) {
        let manifest = SkillManifest::load(&manifest_path)?;
        tracing::info!("Auto-detected manifest at {}", manifest_path.display());
        return Ok(Some(manifest));
    }

    Ok(None)
}
