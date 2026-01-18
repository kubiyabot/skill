use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use wasmtime::component::Component;

use crate::engine::SkillEngine;
use crate::skill_md::{find_skill_md, parse_skill_md, SkillMdContent};

/// Loads skills from local directories with automatic compilation
pub struct LocalSkillLoader {
    cache_dir: PathBuf,
}

impl LocalSkillLoader {
    /// Creates a new local skill loader with cache directory at `~/.skill-engine/local-cache`
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let cache_dir = home.join(".skill-engine").join("local-cache");
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Load a skill from a local directory
    /// Supports:
    /// - Pre-compiled .wasm files
    /// - skill.js files (compiled on-demand)
    /// - skill.ts files (compiled on-demand)
    pub async fn load_skill(
        &self,
        skill_path: impl AsRef<Path>,
        engine: &SkillEngine,
    ) -> Result<Component> {
        let skill_path = skill_path.as_ref();

        tracing::info!(path = %skill_path.display(), "Loading local skill");

        // Check if it's a direct .wasm file
        if skill_path.extension().map_or(false, |ext| ext == "wasm") {
            return engine.load_component(skill_path).await;
        }

        // Check if it's a directory
        if skill_path.is_dir() {
            return self.load_from_directory(skill_path, engine).await;
        }

        // Check if it's a .js or .ts file
        if let Some(ext) = skill_path.extension() {
            if ext == "js" || ext == "ts" {
                return self.compile_and_load(skill_path, engine).await;
            }
        }

        anyhow::bail!("Unsupported skill format: {}", skill_path.display());
    }

    /// Load skill from a directory
    /// Searches for: skill.wasm, skill.js, skill.ts, index.js, index.ts
    async fn load_from_directory(
        &self,
        dir: &Path,
        engine: &SkillEngine,
    ) -> Result<Component> {
        tracing::debug!(dir = %dir.display(), "Loading from directory");

        // Priority order: pre-compiled WASM, then source files
        let candidates = vec![
            dir.join("skill.wasm"),
            dir.join("dist/skill.wasm"),
            dir.join("skill.js"),
            dir.join("skill.ts"),
            dir.join("index.js"),
            dir.join("index.ts"),
            dir.join("src/index.js"),
            dir.join("src/index.ts"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                tracing::info!(file = %candidate.display(), "Found skill file");

                if candidate.extension().map_or(false, |ext| ext == "wasm") {
                    return engine.load_component(&candidate).await;
                } else {
                    return self.compile_and_load(&candidate, engine).await;
                }
            }
        }

        anyhow::bail!("No skill file found in directory: {}", dir.display());
    }

    /// Compile JavaScript/TypeScript to WASM and load
    async fn compile_and_load(
        &self,
        source_file: &Path,
        engine: &SkillEngine,
    ) -> Result<Component> {
        let source_abs = std::fs::canonicalize(source_file)
            .with_context(|| format!("Failed to resolve path: {}", source_file.display()))?;

        // Generate cache key from file path and modification time
        let cache_key = self.generate_cache_key(&source_abs)?;
        let cached_wasm = self.cache_dir.join(format!("{}.wasm", cache_key));

        // Check if cached version exists and is up-to-date
        if cached_wasm.exists() {
            let cache_mtime = std::fs::metadata(&cached_wasm)?.modified()?;
            let source_mtime = std::fs::metadata(&source_abs)?.modified()?;

            if cache_mtime >= source_mtime {
                tracing::info!(
                    source = %source_abs.display(),
                    cached = %cached_wasm.display(),
                    "Using cached WASM"
                );
                return engine.load_component(&cached_wasm).await;
            }
        }

        // Compile source to WASM
        tracing::info!(source = %source_abs.display(), "Compiling to WASM");
        self.compile_to_wasm(&source_abs, &cached_wasm).await?;

        // Load compiled WASM
        engine.load_component(&cached_wasm).await
    }

    /// Compile JavaScript/TypeScript to WASM using jco componentize
    async fn compile_to_wasm(&self, source: &Path, output: &Path) -> Result<()> {
        // Determine if TypeScript compilation is needed
        let is_typescript = source.extension().map_or(false, |ext| ext == "ts");

        let js_file = if is_typescript {
            // Compile TypeScript to JavaScript first
            let js_output = output.with_extension("js");
            self.compile_typescript(source, &js_output)?;
            js_output
        } else {
            source.to_path_buf()
        };

        // Find WIT interface file
        let wit_file = self.find_wit_interface(source)?;

        // Use jco componentize to create WASM component
        tracing::info!(
            js = %js_file.display(),
            wit = %wit_file.display(),
            output = %output.display(),
            "Running jco componentize"
        );

        let status = Command::new("npx")
            .args([
                "-y",
                "@bytecodealliance/jco",
                "componentize",
                js_file.to_str().unwrap(),
                "-w",
                wit_file.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .status()
            .context("Failed to run jco componentize. Is Node.js installed?")?;

        if !status.success() {
            anyhow::bail!("jco componentize failed with status: {}", status);
        }

        // Clean up temporary JS file if we compiled from TypeScript
        if is_typescript {
            let _ = std::fs::remove_file(&js_file);
        }

        tracing::info!(output = %output.display(), "Compilation successful");
        Ok(())
    }

    /// Compile TypeScript to JavaScript
    fn compile_typescript(&self, source: &Path, output: &Path) -> Result<()> {
        tracing::info!(
            source = %source.display(),
            output = %output.display(),
            "Compiling TypeScript"
        );

        let status = Command::new("npx")
            .args([
                "-y",
                "typescript",
                "tsc",
                source.to_str().unwrap(),
                "--outFile",
                output.to_str().unwrap(),
                "--target",
                "ES2020",
                "--module",
                "ES2020",
                "--moduleResolution",
                "node",
            ])
            .status()
            .context("Failed to run tsc. Is Node.js installed?")?;

        if !status.success() {
            anyhow::bail!("TypeScript compilation failed");
        }

        Ok(())
    }

    /// Find WIT interface file
    /// Searches in: current dir, parent dir, project root
    fn find_wit_interface(&self, source: &Path) -> Result<PathBuf> {
        let source_dir = source.parent().context("No parent directory")?;

        // Search locations (check both skill.wit and skill-interface.wit)
        let candidates = vec![
            source_dir.join("skill.wit"),
            source_dir.join("skill-interface.wit"),
            source_dir.join("../skill.wit"),
            source_dir.join("../skill-interface.wit"),
            source_dir.join("../wit/skill.wit"),
            source_dir.join("../wit/skill-interface.wit"),
            source_dir.join("../../wit/skill.wit"),
            source_dir.join("../../wit/skill-interface.wit"),
        ];

        for candidate in candidates {
            if let Ok(path) = std::fs::canonicalize(&candidate) {
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // Fall back to global WIT interface in skill-engine project
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let global_candidates = vec![
            home.join(".skill-engine/wit/skill.wit"),
            home.join(".skill-engine/wit/skill-interface.wit"),
        ];

        for global_wit in global_candidates {
            if global_wit.exists() {
                return Ok(global_wit);
            }
        }

        anyhow::bail!(
            "WIT interface file not found. Searched near: {}",
            source_dir.display()
        );
    }

    /// Generate cache key from file path and content hash
    fn generate_cache_key(&self, source: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        source.to_string_lossy().hash(&mut hasher);

        // Include file modification time in hash
        if let Ok(metadata) = std::fs::metadata(source) {
            if let Ok(mtime) = metadata.modified() {
                mtime.hash(&mut hasher);
            }
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Clear the local cache
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Load SKILL.md from a skill directory if it exists
    pub fn load_skill_md(&self, skill_path: impl AsRef<Path>) -> Option<SkillMdContent> {
        let skill_path = skill_path.as_ref();

        // Determine the skill directory
        let skill_dir = if skill_path.is_dir() {
            skill_path.to_path_buf()
        } else {
            skill_path.parent()?.to_path_buf()
        };

        // Find and parse SKILL.md
        if let Some(skill_md_path) = find_skill_md(&skill_dir) {
            match parse_skill_md(&skill_md_path) {
                Ok(content) => {
                    tracing::info!(
                        path = %skill_md_path.display(),
                        tools = content.tool_docs.len(),
                        "Loaded SKILL.md"
                    );
                    Some(content)
                }
                Err(e) => {
                    tracing::warn!(
                        path = %skill_md_path.display(),
                        error = %e,
                        "Failed to parse SKILL.md"
                    );
                    None
                }
            }
        } else {
            tracing::debug!(dir = %skill_dir.display(), "No SKILL.md found");
            None
        }
    }

    /// Load a skill with its documentation
    pub async fn load_skill_with_docs(
        &self,
        skill_path: impl AsRef<Path>,
        engine: &SkillEngine,
    ) -> Result<(Component, Option<SkillMdContent>)> {
        let skill_path = skill_path.as_ref();

        // Load the WASM component
        let component = self.load_skill(skill_path, engine).await?;

        // Try to load SKILL.md documentation
        let skill_md = self.load_skill_md(skill_path);

        Ok((component, skill_md))
    }
}

impl Default for LocalSkillLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create LocalSkillLoader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let loader = LocalSkillLoader::new().unwrap();
        let path = PathBuf::from("/tmp/test-skill.js");

        let key1 = loader.generate_cache_key(&path).unwrap();
        let key2 = loader.generate_cache_key(&path).unwrap();

        assert_eq!(key1, key2);
    }
}
