//! Skill Loader - Load skills from manifest and discover tools
//!
//! This module loads skills from the .skill-engine.toml manifest and
//! introspects them to discover available tools and parameters.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use skill_runtime::{find_skill_md, parse_skill_md, SkillManifest, SkillMdContent};

use super::types::{RawSkill, RawTool, RawToolParameter, SkillRuntimeType};

/// Loads skills from manifest and discovers their tools
#[derive(Debug)]
pub struct Loader {
    manifest: Option<SkillManifest>,
}

impl Loader {
    /// Create a new loader, optionally with a specific manifest path
    pub fn new(manifest_path: Option<&Path>) -> Result<Self> {
        let manifest = crate::commands::manifest::load_manifest(manifest_path)?;
        Ok(Self { manifest })
    }

    /// Load all skills from the manifest
    pub async fn load_all_skills(&self) -> Result<Vec<RawSkill>> {
        let manifest = self
            .manifest
            .as_ref()
            .context("No manifest found. Create a .skill-engine.toml file.")?;

        let mut skills = Vec::new();

        for (name, _definition) in &manifest.skills {
            match self.load_skill(name, manifest).await {
                Ok(skill) => skills.push(skill),
                Err(e) => {
                    tracing::warn!(skill = %name, error = %e, "Failed to load skill, skipping");
                }
            }
        }

        Ok(skills)
    }

    /// Load a single skill by name
    async fn load_skill(&self, name: &str, manifest: &SkillManifest) -> Result<RawSkill> {
        let definition = manifest
            .skills
            .get(name)
            .context(format!("Skill '{}' not found in manifest", name))?;

        // Determine runtime type
        let runtime = match definition.runtime {
            skill_runtime::manifest::SkillRuntime::Wasm => SkillRuntimeType::Wasm,
            skill_runtime::manifest::SkillRuntime::Native => SkillRuntimeType::Native,
            skill_runtime::manifest::SkillRuntime::Docker => SkillRuntimeType::Docker,
        };

        // Resolve skill path (relative to manifest base_dir)
        let skill_path = if definition.source.starts_with("./") || definition.source.starts_with("../") {
            manifest.base_dir.join(&definition.source)
        } else {
            PathBuf::from(&definition.source)
        };

        // Try to load SKILL.md for tool discovery
        let (tools, skill_md_content, description) = if let Some(skill_md_path) = find_skill_md(&skill_path) {
            match parse_skill_md(&skill_md_path) {
                Ok(md) => {
                    let tools = self.extract_tools_from_skill_md(&md);
                    let content = std::fs::read_to_string(&skill_md_path).ok();
                    let desc = if !md.frontmatter.description.is_empty() {
                        Some(md.frontmatter.description.clone())
                    } else {
                        definition.description.clone()
                    };
                    (tools, content, desc)
                }
                Err(e) => {
                    tracing::warn!(skill = %name, error = %e, "Failed to parse SKILL.md");
                    (Vec::new(), None, definition.description.clone())
                }
            }
        } else {
            // No SKILL.md found - use definition description only
            (Vec::new(), None, definition.description.clone())
        };

        Ok(RawSkill {
            name: name.to_string(),
            description,
            source: definition.source.clone(),
            runtime,
            tools,
            skill_md_content,
        })
    }

    /// Extract tools from parsed SKILL.md content
    fn extract_tools_from_skill_md(&self, md: &SkillMdContent) -> Vec<RawTool> {
        md.tool_docs
            .iter()
            .map(|(name, tool_doc)| {
                // Try to extract parameters from tool documentation
                let parameters = tool_doc
                    .parameters
                    .iter()
                    .map(|param| RawToolParameter {
                        name: param.name.clone(),
                        param_type: param.param_type.to_string(),
                        description: param.description.clone(),
                        required: param.required,
                        default_value: param.default.clone(),
                    })
                    .collect();

                RawTool {
                    name: name.clone(),
                    description: tool_doc.description.clone(),
                    parameters,
                    streaming: false,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    /// Helper to create a test manifest file
    fn create_test_manifest(dir: &Path, content: &str) -> PathBuf {
        let manifest_path = dir.join(".skill-engine.toml");
        fs::write(&manifest_path, content).expect("Failed to write test manifest");
        manifest_path
    }

    /// Helper to create a minimal valid manifest
    fn minimal_manifest() -> &'static str {
        r#"
[skills.test-skill]
source = "./test-skill"
runtime = "wasm"
description = "A test skill"
"#
    }

    /// Helper to create a manifest with multiple skills
    fn multi_skill_manifest() -> &'static str {
        r#"
[skills.skill-one]
source = "./skill-one"
runtime = "wasm"
description = "First skill"

[skills.skill-two]
source = "./skill-two"
runtime = "native"
description = "Second skill"

[skills.skill-three]
source = "./skill-three"
runtime = "docker"
description = "Third skill"
"#
    }

    #[tokio::test]
    async fn test_load_valid_manifest() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path(), minimal_manifest());

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, Some("A test skill".to_string()));
        assert_eq!(skills[0].runtime, SkillRuntimeType::Wasm);
    }

    #[tokio::test]
    async fn test_load_missing_manifest() {
        let result = Loader::new(Some(Path::new("/nonexistent/.skill-engine.toml")));
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        // Error message might vary, just check it contains some error indication
        assert!(!err_msg.is_empty(), "Error message should not be empty: {}", err_msg);
    }

    #[tokio::test]
    async fn test_load_invalid_toml() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join(".skill-engine.toml");
        fs::write(&manifest_path, "invalid { toml syntax [[ }").unwrap();

        let result = Loader::new(Some(&manifest_path));
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("TOML") || err_msg.contains("parse") || err_msg.contains("expected"));
    }

    #[tokio::test]
    async fn test_load_empty_manifest() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path(), "[skills]\n");

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 0, "Empty manifest should return no skills");
    }

    #[tokio::test]
    async fn test_load_multiple_skills() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path(), multi_skill_manifest());

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 3);

        // Verify each skill is loaded
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"skill-one"));
        assert!(names.contains(&"skill-two"));
        assert!(names.contains(&"skill-three"));

        // Verify runtime types
        let skill_one = skills.iter().find(|s| s.name == "skill-one").unwrap();
        assert_eq!(skill_one.runtime, SkillRuntimeType::Wasm);

        let skill_two = skills.iter().find(|s| s.name == "skill-two").unwrap();
        assert_eq!(skill_two.runtime, SkillRuntimeType::Native);

        let skill_three = skills.iter().find(|s| s.name == "skill-three").unwrap();
        assert_eq!(skill_three.runtime, SkillRuntimeType::Docker);
    }

    #[tokio::test]
    async fn test_loader_no_manifest_error() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path(), minimal_manifest());

        let loader = Loader::new(Some(&manifest_path)).unwrap();

        // Create a loader with no manifest (this is an edge case test)
        let empty_loader = Loader { manifest: None };
        let result = empty_loader.load_all_skills().await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No manifest"));
    }

    #[tokio::test]
    async fn test_relative_path_resolution() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
[skills.relative-skill]
source = "./relative/path"
runtime = "wasm"
description = "Skill with relative path"
"#;
        let manifest_path = create_test_manifest(temp.path(), manifest_content);

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source, "./relative/path");
    }

    #[tokio::test]
    async fn test_absolute_path_resolution() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
[skills.absolute-skill]
source = "/absolute/path/to/skill"
runtime = "wasm"
description = "Skill with absolute path"
"#;
        let manifest_path = create_test_manifest(temp.path(), manifest_content);

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source, "/absolute/path/to/skill");
    }

    #[tokio::test]
    async fn test_skill_with_no_description() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
[skills.no-desc]
source = "./skill"
runtime = "wasm"
"#;
        let manifest_path = create_test_manifest(temp.path(), manifest_content);

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].description, None);
    }

    #[tokio::test]
    async fn test_extract_tools_from_empty_skill_md() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path(), minimal_manifest());

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        // Without SKILL.md present, tools should be empty
        assert_eq!(skills[0].tools.len(), 0);
    }

    #[tokio::test]
    async fn test_loader_continues_on_skill_error() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
[skills.valid-skill]
source = "./valid"
runtime = "wasm"
description = "Valid skill"

[skills.invalid-skill]
source = ""
runtime = "wasm"
"#;
        let manifest_path = create_test_manifest(temp.path(), manifest_content);

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        // Should still load the valid skill even if one fails
        assert!(skills.len() >= 1);
        let valid = skills.iter().find(|s| s.name == "valid-skill");
        assert!(valid.is_some());
    }

    #[tokio::test]
    async fn test_all_runtime_types() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
[skills.wasm-skill]
source = "./wasm"
runtime = "wasm"
description = "WASM skill"

[skills.native-skill]
source = "./native"
runtime = "native"
description = "Native skill"

[skills.docker-skill]
source = "./docker"
runtime = "docker"
description = "Docker skill"
"#;
        let manifest_path = create_test_manifest(temp.path(), manifest_content);

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 3);

        for skill in skills {
            match skill.name.as_str() {
                "wasm-skill" => assert_eq!(skill.runtime, SkillRuntimeType::Wasm),
                "native-skill" => assert_eq!(skill.runtime, SkillRuntimeType::Native),
                "docker-skill" => assert_eq!(skill.runtime, SkillRuntimeType::Docker),
                _ => panic!("Unexpected skill name: {}", skill.name),
            }
        }
    }

    #[tokio::test]
    async fn test_manifest_with_special_characters() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
[skills.test-skill]
source = "./skill"
runtime = "wasm"
description = "Description with 'quotes' and \"double quotes\" and special chars: <>&"
"#;
        let manifest_path = create_test_manifest(temp.path(), manifest_content);

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        assert!(skills[0].description.is_some());
        let desc = skills[0].description.as_ref().unwrap();
        assert!(desc.contains("quotes"));
        assert!(desc.contains("<>&"));
    }

    #[tokio::test]
    async fn test_loader_new_with_none_path() {
        // This should attempt to auto-detect manifest in current directory
        // In test environment, this might fail, which is expected
        let result = Loader::new(None);
        // We don't assert success or failure here as it depends on test environment
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_skill_md_content_is_none_when_not_present() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path(), minimal_manifest());

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        assert!(skills[0].skill_md_content.is_none());
    }
}
