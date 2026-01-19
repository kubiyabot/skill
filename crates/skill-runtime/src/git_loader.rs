// Git Skill Loader - Clone and build skills from Git repositories
//
// Supports:
// - Cloning via git2 (pure Rust, no CLI dependency)
// - Auto-detection of skill type (Rust, JS/TS, Python, pre-built WASM)
// - Caching cloned repositories for fast subsequent access
// - Version pinning via tags, branches, or commits

use anyhow::{Context, Result};
use git2::{FetchOptions, RemoteCallbacks, Repository};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

use crate::git_source::GitSource;

/// Skill type detected from repository structure
#[derive(Debug, Clone, PartialEq)]
pub enum SkillType {
    /// Pre-built WASM component (no build needed)
    PrebuiltWasm(PathBuf),
    /// JavaScript skill (needs jco componentize)
    JavaScript(PathBuf),
    /// TypeScript skill (needs tsc + jco)
    TypeScript(PathBuf),
    /// Rust skill (needs cargo build --target wasm32-wasip1)
    Rust,
    /// Python skill (needs componentize-py)
    Python(PathBuf),
    /// Unknown - cannot determine how to build
    Unknown,
}

impl std::fmt::Display for SkillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillType::PrebuiltWasm(_) => write!(f, "Pre-built WASM"),
            SkillType::JavaScript(_) => write!(f, "JavaScript"),
            SkillType::TypeScript(_) => write!(f, "TypeScript"),
            SkillType::Rust => write!(f, "Rust"),
            SkillType::Python(_) => write!(f, "Python"),
            SkillType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Metadata about a cloned skill repository
#[derive(Debug, Clone)]
pub struct ClonedSkill {
    /// Original Git source
    pub source: GitSource,
    /// Local path to cloned repository
    pub local_path: PathBuf,
    /// Detected skill type
    pub skill_type: SkillType,
    /// Skill name (from manifest or repo name)
    pub skill_name: String,
    /// Skill version (if found in manifest)
    pub version: Option<String>,
}

/// Cache metadata for tracking cloned repositories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceCache {
    /// Map of cache keys to source cache entries
    pub entries: std::collections::HashMap<String, SourceCacheEntry>,
}

/// Metadata entry for a cached Git source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCacheEntry {
    /// Git repository URL
    pub url: String,
    /// Git reference (branch, tag, or commit)
    pub git_ref: String,
    /// Commit hash at time of clone
    pub commit: String,
    /// Timestamp when repository was cloned
    pub cloned_at: chrono::DateTime<chrono::Utc>,
    /// Name of the skill from manifest
    pub skill_name: String,
}

/// Loads skills from Git repositories
pub struct GitSkillLoader {
    /// Directory for cloned repositories
    sources_dir: PathBuf,
    /// Cache file path
    cache_path: PathBuf,
}

impl GitSkillLoader {
    /// Create a new GitSkillLoader
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let base_dir = home.join(".skill-engine");
        let sources_dir = base_dir.join("sources");
        let cache_path = base_dir.join("sources.json");

        std::fs::create_dir_all(&sources_dir)
            .with_context(|| format!("Failed to create sources directory: {}", sources_dir.display()))?;

        Ok(Self {
            sources_dir,
            cache_path,
        })
    }

    /// Get the directory for a cloned repo
    pub fn get_repo_dir(&self, source: &GitSource) -> PathBuf {
        self.sources_dir.join(&source.owner).join(&source.repo)
    }

    /// Check if a repo is already cloned
    pub fn is_cloned(&self, source: &GitSource) -> bool {
        self.get_repo_dir(source).join(".git").exists()
    }

    /// Clone or update a Git repository and prepare for loading
    pub async fn clone_skill(&self, source: &GitSource, force: bool) -> Result<ClonedSkill> {
        let repo_dir = self.get_repo_dir(source);

        if force && repo_dir.exists() {
            info!(path = %repo_dir.display(), "Force flag set, removing existing clone");
            std::fs::remove_dir_all(&repo_dir)?;
        }

        // Clone or update
        if repo_dir.join(".git").exists() {
            info!(
                repo = %source.repo,
                path = %repo_dir.display(),
                "Repository already cloned, checking ref..."
            );
            self.checkout_ref(&repo_dir, source)?;
        } else {
            info!(
                url = %source.url,
                path = %repo_dir.display(),
                "Cloning repository..."
            );
            self.clone_repo(source, &repo_dir)?;
        }

        // Detect skill type
        let skill_type = self.detect_skill_type(&repo_dir)?;
        info!(skill_type = %skill_type, "Detected skill type");

        // Extract metadata
        let (skill_name, version) = self.extract_metadata(&repo_dir, source)?;

        // Update cache
        self.update_cache(source, &repo_dir, &skill_name)?;

        Ok(ClonedSkill {
            source: source.clone(),
            local_path: repo_dir,
            skill_type,
            skill_name,
            version,
        })
    }

    /// Build the skill if necessary and return the WASM component path
    pub async fn build_skill(&self, cloned: &ClonedSkill) -> Result<PathBuf> {
        match &cloned.skill_type {
            SkillType::PrebuiltWasm(path) => {
                info!(path = %path.display(), "Using pre-built WASM");
                Ok(path.clone())
            }
            SkillType::JavaScript(entry) => {
                self.build_js_skill(&cloned.local_path, entry, false).await
            }
            SkillType::TypeScript(entry) => {
                self.build_js_skill(&cloned.local_path, entry, true).await
            }
            SkillType::Rust => self.build_rust_skill(&cloned.local_path).await,
            SkillType::Python(entry) => {
                self.build_python_skill(&cloned.local_path, entry).await
            }
            SkillType::Unknown => {
                anyhow::bail!(
                    "Cannot determine how to build this skill.\n\
                     Expected one of:\n\
                     - skill.wasm (pre-built)\n\
                     - Cargo.toml (Rust)\n\
                     - package.json + *.ts/*.js (JavaScript/TypeScript)\n\
                     - pyproject.toml + *.py (Python)"
                )
            }
        }
    }

    /// Remove a cloned repository
    pub fn remove_source(&self, source: &GitSource) -> Result<()> {
        let repo_dir = self.get_repo_dir(source);
        if repo_dir.exists() {
            std::fs::remove_dir_all(&repo_dir)?;
            info!(path = %repo_dir.display(), "Removed cloned repository");
        }
        Ok(())
    }

    // --- Private methods ---

    fn clone_repo(&self, source: &GitSource, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest.parent().unwrap())?;

        // Set up callbacks for progress
        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(|progress| {
            debug!(
                "Receiving objects: {}/{}",
                progress.received_objects(),
                progress.total_objects()
            );
            true
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Clone the repository
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        let repo = builder
            .clone(&source.url, dest)
            .with_context(|| format!("Failed to clone repository: {}", source.url))?;

        // Checkout specific ref if not default branch
        if let Some(refspec) = source.git_ref.as_refspec() {
            self.checkout_ref_in_repo(&repo, refspec)?;
        }

        Ok(())
    }

    fn checkout_ref(&self, repo_dir: &Path, source: &GitSource) -> Result<()> {
        let repo = Repository::open(repo_dir)
            .with_context(|| format!("Failed to open repository: {}", repo_dir.display()))?;

        // Fetch updates if not a pinned ref
        if !source.git_ref.is_pinned() {
            debug!("Fetching updates from origin...");
            let mut remote = repo.find_remote("origin")?;
            remote.fetch(&["refs/heads/*:refs/heads/*"], None, None)?;
        }

        if let Some(refspec) = source.git_ref.as_refspec() {
            self.checkout_ref_in_repo(&repo, refspec)?;
        }

        Ok(())
    }

    fn checkout_ref_in_repo(&self, repo: &Repository, refspec: &str) -> Result<()> {
        info!(refspec = %refspec, "Checking out ref");

        // Try to find the reference
        let reference = repo
            .resolve_reference_from_short_name(refspec)
            .or_else(|_| repo.find_reference(&format!("refs/tags/{}", refspec)))
            .or_else(|_| repo.find_reference(&format!("refs/heads/{}", refspec)))
            .with_context(|| format!("Could not find ref: {}", refspec))?;

        let commit = reference.peel_to_commit()?;

        // Checkout the commit
        repo.checkout_tree(commit.as_object(), None)?;
        repo.set_head_detached(commit.id())?;

        Ok(())
    }

    fn detect_skill_type(&self, repo_dir: &Path) -> Result<SkillType> {
        // Priority order for detection

        // 1. Pre-built WASM
        let wasm_candidates = [
            repo_dir.join("skill.wasm"),
            repo_dir.join("dist/skill.wasm"),
            repo_dir.join("build/skill.wasm"),
        ];
        for candidate in &wasm_candidates {
            if candidate.exists() {
                return Ok(SkillType::PrebuiltWasm(candidate.clone()));
            }
        }

        // 2. Check for Cargo.toml (Rust)
        let cargo_toml = repo_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            // Check if it's likely a WASM project
            if content.contains("cdylib") || content.contains("wasm32") || content.contains("wasm") {
                return Ok(SkillType::Rust);
            }
        }

        // 3. Check for package.json (JS/TS)
        let package_json = repo_dir.join("package.json");
        if package_json.exists() {
            // Look for TypeScript first
            let ts_candidates = [
                repo_dir.join("skill.ts"),
                repo_dir.join("src/skill.ts"),
                repo_dir.join("src/index.ts"),
                repo_dir.join("index.ts"),
            ];
            for candidate in ts_candidates {
                if candidate.exists() {
                    return Ok(SkillType::TypeScript(candidate));
                }
            }

            // Then JavaScript
            let js_candidates = [
                repo_dir.join("skill.js"),
                repo_dir.join("src/skill.js"),
                repo_dir.join("src/index.js"),
                repo_dir.join("index.js"),
            ];
            for candidate in js_candidates {
                if candidate.exists() {
                    return Ok(SkillType::JavaScript(candidate));
                }
            }
        }

        // 4. Check for Python (pyproject.toml or requirements.txt + main.py)
        let has_python_config =
            repo_dir.join("pyproject.toml").exists() || repo_dir.join("requirements.txt").exists();
        if has_python_config {
            let py_candidates = [
                repo_dir.join("skill.py"),
                repo_dir.join("src/main.py"),
                repo_dir.join("main.py"),
                repo_dir.join("src/skill.py"),
            ];
            for candidate in py_candidates {
                if candidate.exists() {
                    return Ok(SkillType::Python(candidate));
                }
            }
        }

        Ok(SkillType::Unknown)
    }

    fn extract_metadata(
        &self,
        repo_dir: &Path,
        source: &GitSource,
    ) -> Result<(String, Option<String>)> {
        // Try to read skill.yaml
        let skill_yaml_path = repo_dir.join("skill.yaml");
        if skill_yaml_path.exists() {
            let contents = std::fs::read_to_string(&skill_yaml_path)?;
            if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&contents) {
                let name = yaml["name"]
                    .as_str()
                    .unwrap_or(&source.repo)
                    .to_string();
                let version = yaml["version"].as_str().map(|s| s.to_string());
                return Ok((name, version));
            }
        }

        // Try SKILL.md frontmatter
        let skill_md_path = repo_dir.join("SKILL.md");
        if skill_md_path.exists() {
            let contents = std::fs::read_to_string(&skill_md_path)?;
            if let Some(frontmatter) = extract_yaml_frontmatter(&contents) {
                if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                    let name = yaml["name"]
                        .as_str()
                        .unwrap_or(&source.repo)
                        .to_string();
                    let version = yaml["version"].as_str().map(|s| s.to_string());
                    return Ok((name, version));
                }
            }
        }

        // Try package.json
        let package_json_path = repo_dir.join("package.json");
        if package_json_path.exists() {
            let contents = std::fs::read_to_string(&package_json_path)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                let name = json["name"]
                    .as_str()
                    .unwrap_or(&source.repo)
                    .to_string();
                let version = json["version"].as_str().map(|s| s.to_string());
                return Ok((name, version));
            }
        }

        // Try Cargo.toml
        let cargo_toml_path = repo_dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let contents = std::fs::read_to_string(&cargo_toml_path)?;
            if let Ok(toml) = toml::from_str::<toml::Value>(&contents) {
                if let Some(package) = toml.get("package") {
                    let name = package["name"]
                        .as_str()
                        .unwrap_or(&source.repo)
                        .to_string();
                    let version = package["version"].as_str().map(|s| s.to_string());
                    return Ok((name, version));
                }
            }
        }

        // Fall back to repo name
        Ok((source.repo.clone(), None))
    }

    fn update_cache(
        &self,
        source: &GitSource,
        repo_dir: &Path,
        skill_name: &str,
    ) -> Result<()> {
        let mut cache = self.load_cache();

        // Get current commit
        let commit = if let Ok(repo) = Repository::open(repo_dir) {
            repo.head()
                .ok()
                .and_then(|h| h.peel_to_commit().ok())
                .map(|c| c.id().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        cache.entries.insert(
            source.cache_key(),
            SourceCacheEntry {
                url: source.url.clone(),
                git_ref: source.git_ref.to_string(),
                commit,
                cloned_at: chrono::Utc::now(),
                skill_name: skill_name.to_string(),
            },
        );

        self.save_cache(&cache)?;
        Ok(())
    }

    fn load_cache(&self) -> SourceCache {
        std::fs::read_to_string(&self.cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_cache(&self, cache: &SourceCache) -> Result<()> {
        let content = serde_json::to_string_pretty(cache)?;
        std::fs::write(&self.cache_path, content)?;
        Ok(())
    }

    async fn build_js_skill(
        &self,
        repo_dir: &Path,
        entry: &Path,
        _is_typescript: bool,
    ) -> Result<PathBuf> {
        info!(entry = %entry.display(), "Building JavaScript/TypeScript skill");

        // Install dependencies if node_modules doesn't exist
        if !repo_dir.join("node_modules").exists() {
            info!("Installing npm dependencies...");
            let status = Command::new("npm")
                .args(["install"])
                .current_dir(repo_dir)
                .status()
                .context("Failed to run npm install. Is npm installed?")?;

            if !status.success() {
                anyhow::bail!("npm install failed");
            }
        }

        // Check if there's a build script
        let package_json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(repo_dir.join("package.json"))?,
        )?;

        // Run build if available
        if package_json
            .get("scripts")
            .and_then(|s| s.get("build"))
            .is_some()
        {
            info!("Running npm build...");
            let status = Command::new("npm")
                .args(["run", "build"])
                .current_dir(repo_dir)
                .status()?;

            if !status.success() {
                warn!("npm build failed, attempting direct componentize");
            }
        }

        // Check for componentize script
        if package_json
            .get("scripts")
            .and_then(|s| s.get("componentize"))
            .is_some()
        {
            info!("Running componentize script...");
            let status = Command::new("npm")
                .args(["run", "componentize"])
                .current_dir(repo_dir)
                .status()?;

            if status.success() {
                // Look for output WASM
                let wasm_candidates = [
                    repo_dir.join("skill.wasm"),
                    repo_dir.join("dist/skill.wasm"),
                ];
                for candidate in wasm_candidates {
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        // Direct componentize with jco
        let output_wasm = repo_dir.join("skill.wasm");

        info!("Running jco componentize...");
        let status = Command::new("npx")
            .args([
                "@bytecodealliance/jco",
                "componentize",
                entry.to_str().unwrap(),
                "-o",
                output_wasm.to_str().unwrap(),
            ])
            .current_dir(repo_dir)
            .status()
            .context("Failed to run jco componentize. Is jco installed?")?;

        if !status.success() {
            anyhow::bail!("jco componentize failed");
        }

        Ok(output_wasm)
    }

    async fn build_rust_skill(&self, repo_dir: &Path) -> Result<PathBuf> {
        info!("Building Rust skill...");

        let status = Command::new("cargo")
            .args(["build", "--release", "--target", "wasm32-wasip1"])
            .current_dir(repo_dir)
            .status()
            .context("Failed to run cargo build. Is cargo and wasm32-wasip1 target installed?")?;

        if !status.success() {
            anyhow::bail!(
                "cargo build failed. Make sure you have the wasm32-wasip1 target:\n\
                 rustup target add wasm32-wasip1"
            );
        }

        // Find the output WASM
        let target_dir = repo_dir.join("target/wasm32-wasip1/release");
        for entry in std::fs::read_dir(&target_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "wasm") {
                info!(wasm = %path.display(), "Found compiled WASM");
                return Ok(path);
            }
        }

        anyhow::bail!(
            "No .wasm file found in target/wasm32-wasip1/release/\n\
             Make sure Cargo.toml has crate-type = [\"cdylib\"]"
        )
    }

    async fn build_python_skill(&self, repo_dir: &Path, entry: &Path) -> Result<PathBuf> {
        info!(entry = %entry.display(), "Building Python skill");

        let output_wasm = repo_dir.join("skill.wasm");

        // Find WIT file
        let wit_candidates = [
            repo_dir.join("skill.wit"),
            repo_dir.join("wit/skill.wit"),
            repo_dir.join("skill-interface.wit"),
        ];

        let wit_path = wit_candidates
            .iter()
            .find(|p| p.exists())
            .context("No WIT interface file found. Expected skill.wit or wit/skill.wit")?;

        let status = Command::new("componentize-py")
            .args([
                "-d",
                wit_path.to_str().unwrap(),
                "-w",
                "skill",
                "componentize",
                entry.to_str().unwrap(),
                "-o",
                output_wasm.to_str().unwrap(),
            ])
            .current_dir(repo_dir)
            .status()
            .context("Failed to run componentize-py. Install it with: pip install componentize-py")?;

        if !status.success() {
            anyhow::bail!("componentize-py failed");
        }

        Ok(output_wasm)
    }
}

impl Default for GitSkillLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create GitSkillLoader")
    }
}

fn extract_yaml_frontmatter(content: &str) -> Option<&str> {
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    Some(rest[..end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_type_display() {
        assert_eq!(format!("{}", SkillType::Rust), "Rust");
        assert_eq!(
            format!("{}", SkillType::PrebuiltWasm(PathBuf::from("test.wasm"))),
            "Pre-built WASM"
        );
    }

    #[test]
    fn test_extract_yaml_frontmatter() {
        let content = "---\nname: test\nversion: 1.0\n---\n\n# Test";
        let fm = extract_yaml_frontmatter(content);
        assert!(fm.is_some());
        assert!(fm.unwrap().contains("name: test"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just markdown\n\nNo frontmatter here.";
        assert!(extract_yaml_frontmatter(content).is_none());
    }
}
