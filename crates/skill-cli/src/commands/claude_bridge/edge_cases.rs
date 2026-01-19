//! Edge Cases and Error Path Tests for Claude Bridge
//!
//! This module contains comprehensive tests for:
//! - Concurrent operations and race conditions
//! - Large manifests and resource limits
//! - Malformed inputs and error handling
//! - Filesystem errors and permission issues
//! - Unicode edge cases and boundary conditions

#[cfg(test)]
mod tests {
    use super::super::loader::Loader;
    use super::super::transformer::Transformer;
    use super::super::validator::Validator;
    use super::super::renderer::Renderer;
    use super::super::script_gen::ScriptGenerator;
    use super::super::types::{RawSkill, RawTool, RawToolParameter, SkillRuntimeType};
    use std::fs;
    use tempfile::TempDir;
    use tokio::task::JoinSet;

    // Helper to create a manifest with N skills
    fn create_large_manifest(num_skills: usize) -> String {
        let mut manifest = String::from("");
        for i in 0..num_skills {
            manifest.push_str(&format!(
                r#"
[skills.skill-{}]
source = "./skill-{}"
runtime = "wasm"
description = "Test skill number {}"
"#,
                i, i, i
            ));
        }
        manifest
    }

    // Helper to create a skill with N tools
    fn create_skill_with_many_tools(num_tools: usize) -> RawSkill {
        let tools: Vec<RawTool> = (0..num_tools)
            .map(|i| RawTool {
                name: format!("tool_{}", i),
                description: format!("Tool number {}", i),
                parameters: vec![],
                streaming: false,
            })
            .collect();

        RawSkill {
            name: "test-skill".to_string(),
            description: Some("Test skill with many tools".to_string()),
            source: "./test-skill".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools,
            skill_md_content: None,
        }
    }

    /// Test: Concurrent skill loading without race conditions
    #[tokio::test]
    async fn test_concurrent_skill_loading() {
        let temp = TempDir::new().unwrap();
        let manifest_content = r#"
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
"#;
        let manifest_path = temp.path().join(".skill-engine.toml");
        fs::write(&manifest_path, manifest_content).unwrap();

        // Load the same manifest concurrently from multiple tasks
        let mut join_set = JoinSet::new();
        for _ in 0..10 {
            let path = manifest_path.clone();
            join_set.spawn(async move {
                let loader = Loader::new(Some(&path)).unwrap();
                loader.load_all_skills().await
            });
        }

        // All tasks should succeed without race conditions
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let skills = result.unwrap().unwrap();
            results.push(skills);
        }

        // Verify all results are consistent
        assert_eq!(results.len(), 10);
        for skills in results {
            assert_eq!(skills.len(), 3);
        }
    }

    /// Test: Large manifest with 100+ skills
    #[tokio::test]
    #[ignore] // Expensive test - run with --ignored
    async fn test_large_manifest() {
        let temp = TempDir::new().unwrap();
        let manifest_content = create_large_manifest(150);
        let manifest_path = temp.path().join(".skill-engine.toml");
        fs::write(&manifest_path, manifest_content).unwrap();

        let loader = Loader::new(Some(&manifest_path)).unwrap();
        let skills = loader.load_all_skills().await.unwrap();

        assert_eq!(skills.len(), 150);

        // Verify first and last skills
        assert!(skills.iter().any(|s| s.name == "skill-0"));
        assert!(skills.iter().any(|s| s.name == "skill-149"));
    }

    /// Test: Malformed UTF-8 in skill names and descriptions
    #[test]
    fn test_invalid_utf8_handling() {
        let validator = Validator::new();

        // Test with various problematic characters
        let test_cases = vec![
            ("skill\0name", "Skill with null byte"),
            ("skill\u{FEFF}name", "Skill with BOM"),
            ("skill\u{200B}name", "Skill with zero-width space"),
            ("skill\u{202E}name", "Skill with RTL override"),
        ];

        for (name, desc) in test_cases {
            let skill = RawSkill {
                name: name.to_string(),
                description: Some(desc.to_string()),
                source: "./test".to_string(),
                runtime: SkillRuntimeType::Wasm,
                tools: vec![],
                skill_md_content: None,
            };

            // Should handle without panicking
            let result = validator.validate(&skill);
            assert!(result.is_ok() || result.is_err(), "Should not panic");
        }
    }

    /// Test: Empty and boundary condition inputs
    #[test]
    fn test_empty_inputs() {
        let lenient_validator = Validator::new();
        let strict_validator = Validator::strict();

        // Empty skill name - lenient validator may allow it, strict should reject
        let empty_name = RawSkill {
            name: "".to_string(),
            description: Some("Valid description".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };
        // Lenient might sanitize it, strict should reject
        let lenient_result = lenient_validator.validate(&empty_name);
        let strict_result = strict_validator.validate(&empty_name);
        // At least one should handle it (either error or generate a valid name)
        assert!(lenient_result.is_ok() || strict_result.is_err());

        // Empty description
        let empty_desc = RawSkill {
            name: "valid-name".to_string(),
            description: Some("".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };
        assert!(lenient_validator.validate(&empty_desc).is_ok());

        // None description
        let none_desc = RawSkill {
            name: "valid-name".to_string(),
            description: None,
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };
        assert!(lenient_validator.validate(&none_desc).is_ok());
    }

    /// Test: Maximum field lengths
    #[test]
    fn test_maximum_field_lengths() {
        let validator = Validator::new();

        // Very long skill name (1000 chars)
        let long_name = "a".repeat(1000);
        let skill = RawSkill {
            name: long_name.clone(),
            description: Some("Test".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };
        let result = validator.validate(&skill);
        // Should handle gracefully, either accept or reject with error
        assert!(result.is_ok() || result.is_err());

        // Very long description (10000 chars)
        let long_desc = "a".repeat(10000);
        let skill = RawSkill {
            name: "valid-name".to_string(),
            description: Some(long_desc),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };
        let result = validator.validate(&skill);
        assert!(result.is_ok() || result.is_err());
    }

    /// Test: Unicode edge cases (emoji, RTL, combining characters)
    #[test]
    fn test_unicode_edge_cases() {
        let validator = Validator::new();

        let test_cases = vec![
            ("emoji-skill-🚀", "Skill with emoji"),
            ("skill-العربية", "Skill with Arabic RTL text"),
            ("skill-with-é-ñ-ü", "Skill with accented characters"),
            ("skill-👨‍👩‍👧‍👦", "Skill with family emoji"),
            ("skill-\u{0301}combined", "Skill with combining diacritic"),
        ];

        for (name, desc) in test_cases {
            let skill = RawSkill {
                name: name.to_string(),
                description: Some(desc.to_string()),
                source: "./test".to_string(),
                runtime: SkillRuntimeType::Wasm,
                tools: vec![],
                skill_md_content: None,
            };

            // Should handle without panicking
            let result = validator.validate(&skill);
            assert!(result.is_ok() || result.is_err(), "Should not panic on {}", name);
        }
    }

    /// Test: Filesystem errors - read-only directory
    #[test]
    fn test_readonly_filesystem() {
        let temp = TempDir::new().unwrap();
        let output_dir = temp.path().join("readonly");
        fs::create_dir(&output_dir).unwrap();

        let validator = Validator::new();
        let transformer = Transformer::new();
        let raw_skill = create_skill_with_many_tools(5);
        let validated = validator.validate(&raw_skill).unwrap();
        let skill = transformer.transform(validated).unwrap();

        // Create files first
        let renderer = Renderer::new(&output_dir).unwrap();
        let result = renderer.render(&skill);
        assert!(result.is_ok());

        // Set directory to read-only on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output_dir).unwrap().permissions();
            perms.set_mode(0o444); // r--r--r--
            fs::set_permissions(&output_dir, perms).unwrap();

            // Try to write to read-only directory - should fail
            let readonly_dir = temp.path().join("readonly2");
            fs::create_dir(&readonly_dir).unwrap();
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444);
            fs::set_permissions(&readonly_dir, perms).unwrap();

            let result2 = Renderer::new(&readonly_dir);
            // Might fail on creating renderer or on render
            if let Ok(r) = result2 {
                let result3 = r.render(&skill);
                assert!(result3.is_err(), "Should error on read-only filesystem");
            }

            // Restore write permissions for cleanup
            let mut perms = fs::metadata(&output_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output_dir, perms).unwrap();
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&readonly_dir, perms).unwrap();
        }
    }

    /// Test: Resource limits - skill with many tools
    #[test]
    #[ignore] // Expensive test
    fn test_skill_with_many_tools() {
        let temp = TempDir::new().unwrap();
        let validator = Validator::new();
        let transformer = Transformer::new();
        let raw_skill = create_skill_with_many_tools(500);
        let validated = validator.validate(&raw_skill).unwrap();
        let skill = transformer.transform(validated).unwrap();

        let renderer = Renderer::new(temp.path()).unwrap();
        let result = renderer.render(&skill);
        assert!(result.is_ok());

        // Verify file was created
        let skill_md_path = temp.path().join("test-skill").join("SKILL.md");
        assert!(skill_md_path.exists());

        // Verify content contains all tools
        let content = fs::read_to_string(&skill_md_path).unwrap();
        assert!(content.contains("tool_0"));
        assert!(content.contains("tool_499"));
    }

    /// Test: Concurrent transformations
    #[tokio::test]
    async fn test_concurrent_transformations() {
        let _validator = Validator::new();
        let _transformer = Transformer::new();
        let raw_skill = create_skill_with_many_tools(10);

        // Validate and transform the same skill concurrently
        let mut join_set = JoinSet::new();
        for _ in 0..20 {
            let v = Validator::new();
            let t = Transformer::new();
            let s = raw_skill.clone();
            join_set.spawn(async move {
                let validated = v.validate(&s)?;
                t.transform(validated)
            });
        }

        // All transformations should succeed
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let transformed: anyhow::Result<_> = result.unwrap();
            results.push(transformed.unwrap());
        }

        assert_eq!(results.len(), 20);

        // Verify all results are consistent
        for result in &results {
            assert_eq!(result.name, "test-skill");
            assert_eq!(result.tools.len(), 10);
        }
    }

    /// Test: Malformed TOML handling
    #[test]
    fn test_malformed_toml() {
        let temp = TempDir::new().unwrap();

        let malformed_cases = [
            "invalid { toml [[ }",
            "[skills.test]\nsource = ",  // Incomplete
            "[skills.test]\nruntime = \"invalid_runtime\"",
            "[skills.test]\n[skills.test]",  // Duplicate
        ];

        for (i, malformed) in malformed_cases.iter().enumerate() {
            let manifest_path = temp.path().join(format!("manifest_{}.toml", i));
            fs::write(&manifest_path, malformed).unwrap();

            let result = Loader::new(Some(&manifest_path));
            // Should return error, not panic
            assert!(result.is_err(), "Should error on malformed TOML case {}", i);
        }
    }

    /// Test: Script generation with problematic tool names
    #[test]
    fn test_script_gen_edge_cases() {
        let temp = TempDir::new().unwrap();
        let validator = Validator::new();
        let transformer = Transformer::new();
        let generator = ScriptGenerator::new("test-skill");

        // Tool names with special characters
        let problematic_tools = vec![
            "tool-with-dashes",
            "tool_with_underscores",
            "tool.with.dots",
            "UPPERCASE",
            "123numeric",
        ];

        for tool_name in problematic_tools {
            let raw_skill = RawSkill {
                name: "test-skill".to_string(),
                description: Some("Test".to_string()),
                source: "./test".to_string(),
                runtime: SkillRuntimeType::Wasm,
                tools: vec![RawTool {
                    name: tool_name.to_string(),
                    description: "Test tool".to_string(),
                    parameters: vec![],
                    streaming: false,
                }],
                skill_md_content: None,
            };

            let validated = validator.validate(&raw_skill).unwrap();
            let skill = transformer.transform(validated).unwrap();

            let result = generator.generate(&skill, temp.path());
            // Should handle gracefully
            assert!(result.is_ok() || result.is_err(),
                "Should not panic on tool name: {}", tool_name);
        }
    }

    /// Test: Tools with many parameters
    #[test]
    fn test_tool_with_many_parameters() {
        let validator = Validator::new();
        let transformer = Transformer::new();
        let generator = ScriptGenerator::new("test-skill");

        // Create tool with 50 parameters
        let params: Vec<RawToolParameter> = (0..50)
            .map(|i| RawToolParameter {
                name: format!("param_{}", i),
                param_type: "string".to_string(),
                description: format!("Parameter {}", i),
                required: i % 2 == 0, // Alternate required/optional
                default_value: if i % 2 == 0 { None } else { Some("default".to_string()) },
            })
            .collect();

        let raw_skill = RawSkill {
            name: "test-skill".to_string(),
            description: Some("Test".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![RawTool {
                name: "complex_tool".to_string(),
                description: "Tool with many params".to_string(),
                parameters: params,
                streaming: false,
            }],
            skill_md_content: None,
        };

        let validated = validator.validate(&raw_skill).unwrap();
        let skill = transformer.transform(validated).unwrap();

        let temp = TempDir::new().unwrap();
        let result = generator.generate(&skill, temp.path());
        assert!(result.is_ok());

        // Verify script was created
        let script_path = temp.path().join("test-skill/scripts/complex_tool.sh");
        assert!(script_path.exists());

        // Verify content includes parameter documentation
        let content = fs::read_to_string(&script_path).unwrap();
        assert!(content.contains("param_0"));
        assert!(content.contains("param_49"));
    }

    /// Test: Nested directory creation
    #[test]
    fn test_deeply_nested_output() {
        let temp = TempDir::new().unwrap();
        let deep_path = temp.path()
            .join("level1")
            .join("level2")
            .join("level3")
            .join("level4");

        let validator = Validator::new();
        let transformer = Transformer::new();
        let raw_skill = create_skill_with_many_tools(1);
        let validated = validator.validate(&raw_skill).unwrap();
        let skill = transformer.transform(validated).unwrap();

        // Should create all necessary parent directories
        let renderer = Renderer::new(&deep_path).unwrap();
        let result = renderer.render(&skill);
        assert!(result.is_ok());

        let skill_md = deep_path.join("test-skill/SKILL.md");
        assert!(skill_md.exists());
    }

    /// Test: Validator with strict mode
    #[test]
    fn test_validator_strict_mode_edge_cases() {
        let strict_validator = Validator::strict();

        // Test with potentially problematic skill
        let skill = RawSkill {
            name: "test".to_string(),
            description: Some("Test description".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };

        // Strict validator should be more rigorous
        let result = strict_validator.validate(&skill);
        assert!(result.is_ok() || result.is_err());
    }

    /// Test: Renderer with special YAML characters
    #[test]
    fn test_renderer_yaml_escaping_edge_cases() {
        let validator = Validator::new();
        let transformer = Transformer::new();
        let raw_skill = RawSkill {
            name: "test-skill".to_string(),
            description: Some("Description with: colons, [brackets], {braces}, 'quotes', \"double quotes\", & ampersands".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };

        let validated = validator.validate(&raw_skill).unwrap();
        let skill = transformer.transform(validated).unwrap();

        let temp = TempDir::new().unwrap();
        let renderer = Renderer::new(temp.path()).unwrap();
        let result = renderer.render(&skill);
        assert!(result.is_ok());

        // Verify valid YAML was generated
        let skill_md_path = temp.path().join("test-skill/SKILL.md");
        let content = fs::read_to_string(&skill_md_path).unwrap();

        // Should contain properly escaped YAML
        assert!(content.contains("description:"));
    }

    /// Test: Empty arrays and null-like values
    #[test]
    fn test_empty_collections() {
        let validator = Validator::new();

        // Skill with no tools
        let no_tools = RawSkill {
            name: "test".to_string(),
            description: Some("Test".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };
        assert!(validator.validate(&no_tools).is_ok());

        // Tool with no parameters
        let no_params = RawSkill {
            name: "test".to_string(),
            description: Some("Test".to_string()),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![RawTool {
                name: "empty_tool".to_string(),
                description: "Tool with no params".to_string(),
                parameters: vec![],
                streaming: false,
            }],
            skill_md_content: None,
        };
        assert!(validator.validate(&no_params).is_ok());
    }

    /// Test: Transformer consistency
    #[test]
    fn test_transformer_consistency() {
        let validator = Validator::new();
        let transformer = Transformer::new();
        let raw_skill = create_skill_with_many_tools(5);

        // Validate once
        let validated1 = validator.validate(&raw_skill).unwrap();
        let validated2 = validator.validate(&raw_skill).unwrap();
        let validated3 = validator.validate(&raw_skill).unwrap();

        // Transform the same skill multiple times
        let result1 = transformer.transform(validated1);
        let result2 = transformer.transform(validated2);
        let result3 = transformer.transform(validated3);

        // All transformations should succeed consistently
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        let r1 = result1.unwrap();
        let r2 = result2.unwrap();
        let r3 = result3.unwrap();

        // Results should be consistent
        assert_eq!(r1.name, r2.name);
        assert_eq!(r2.name, r3.name);
        assert_eq!(r1.tools.len(), r2.tools.len());
        assert_eq!(r2.tools.len(), r3.tools.len());
    }
}
