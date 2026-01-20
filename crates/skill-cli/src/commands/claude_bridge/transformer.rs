//! Transformer - Transform validated skills to Claude Agent Skills format
//!
//! This module transforms validated skills into the final ClaudeSkill format,
//! including categorization, example generation, and "when to use" triggers.

use anyhow::Result;
use std::collections::HashMap;

use super::types::{
    ClaudeSkill, ClaudeTool, ClaudeToolParameter, SkillRuntimeType, ToolExample, ValidatedSkill,
};

/// Transforms validated skills into Claude Agent Skills format
pub struct Transformer {
    /// Custom category mappings
    category_patterns: HashMap<String, Vec<String>>,
}

impl Transformer {
    /// Create a new transformer with default category patterns
    pub fn new() -> Self {
        Self {
            category_patterns: default_category_patterns(),
        }
    }

    /// Transform a validated skill into a ClaudeSkill
    pub fn transform(&self, skill: ValidatedSkill) -> Result<ClaudeSkill> {
        // Transform tools
        let tools: Vec<ClaudeTool> = skill
            .tools
            .into_iter()
            .map(|t| {
                let category = self.categorize_tool(&t.name, &skill.name);
                ClaudeTool {
                    name: t.name.clone(),
                    description: t.description,
                    parameters: t
                        .parameters
                        .into_iter()
                        .map(|p| ClaudeToolParameter {
                            name: p.name,
                            param_type: p.param_type,
                            description: p.description,
                            required: p.required,
                            default_value: p.default_value,
                            enum_values: None,
                        })
                        .collect(),
                    examples: self.generate_examples(&t.name, &skill.name),
                    category: Some(category),
                    streaming: t.streaming,
                }
            })
            .collect();

        // Build categories map
        let categories = self.build_categories(&tools);

        // Generate "when to use" triggers
        let when_to_use = self.generate_when_to_use(&skill.name, &skill.runtime);

        Ok(ClaudeSkill {
            name: skill.name,
            description: skill.description,
            tools,
            categories,
            when_to_use,
            runtime: skill.runtime,
        })
    }

    /// Categorize a tool based on its name
    fn categorize_tool(&self, tool_name: &str, _skill_name: &str) -> String {
        // Check custom patterns first
        for (category, patterns) in &self.category_patterns {
            for pattern in patterns {
                if tool_name.contains(pattern) {
                    return category.clone();
                }
            }
        }

        // Default categorization based on common prefixes
        let prefixes = [
            ("get", "Read Operations"),
            ("list", "Read Operations"),
            ("describe", "Read Operations"),
            ("show", "Read Operations"),
            ("create", "Write Operations"),
            ("update", "Write Operations"),
            ("delete", "Write Operations"),
            ("remove", "Write Operations"),
            ("apply", "Write Operations"),
            ("logs", "Debugging"),
            ("exec", "Debugging"),
            ("debug", "Debugging"),
            ("scale", "Scaling"),
            ("rollout", "Deployment"),
            ("deploy", "Deployment"),
        ];

        for (prefix, category) in prefixes {
            if tool_name.starts_with(prefix) {
                return category.to_string();
            }
        }

        // Default category
        "General".to_string()
    }

    /// Build a categories map from tools
    fn build_categories(&self, tools: &[ClaudeTool]) -> HashMap<String, Vec<String>> {
        let mut categories: HashMap<String, Vec<String>> = HashMap::new();

        for tool in tools {
            let category = tool.category.clone().unwrap_or_else(|| "General".to_string());
            categories
                .entry(category)
                .or_default()
                .push(tool.name.clone());
        }

        categories
    }

    /// Generate example calls for a tool
    fn generate_examples(&self, tool_name: &str, skill_name: &str) -> Vec<ToolExample> {
        // Generate a basic example
        let mcp_call = format!(
            "execute(skill='{}', tool='{}', args={{...}})",
            skill_name, tool_name
        );

        let script_call = format!("./scripts/{}.sh arg1=value1", tool_name);

        vec![ToolExample {
            description: format!("Basic {} usage", tool_name),
            mcp_call,
            script_call,
        }]
    }

    /// Generate "when to use" triggers for a skill
    fn generate_when_to_use(&self, skill_name: &str, _runtime: &SkillRuntimeType) -> Vec<String> {
        // Skill-specific triggers
        match skill_name {
            "kubernetes" | "k8s" => vec![
                "User mentions pods, deployments, services, or namespaces".to_string(),
                "User wants to check cluster status or logs".to_string(),
                "User needs to scale or restart workloads".to_string(),
                "User asks about Kubernetes resources".to_string(),
            ],
            "docker" => vec![
                "User mentions containers, images, or Docker".to_string(),
                "User wants to run or manage containers".to_string(),
                "User needs to build or push images".to_string(),
            ],
            "aws" => vec![
                "User mentions AWS services (S3, EC2, Lambda, etc.)".to_string(),
                "User wants to manage cloud resources".to_string(),
                "User needs to deploy to AWS".to_string(),
            ],
            "git" | "github" => vec![
                "User mentions repositories, branches, or commits".to_string(),
                "User wants to check git status or history".to_string(),
                "User needs to manage pull requests or issues".to_string(),
            ],
            "terraform" => vec![
                "User mentions infrastructure as code".to_string(),
                "User wants to plan or apply Terraform changes".to_string(),
                "User needs to manage cloud infrastructure".to_string(),
            ],
            _ => vec![
                format!("User needs to work with {}", skill_name),
                format!("User mentions {} operations", skill_name),
            ],
        }
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Default category patterns for tool classification
fn default_category_patterns() -> HashMap<String, Vec<String>> {
    let mut patterns = HashMap::new();

    patterns.insert(
        "Read Operations".to_string(),
        vec![
            "get".to_string(),
            "list".to_string(),
            "describe".to_string(),
            "show".to_string(),
            "fetch".to_string(),
            "read".to_string(),
        ],
    );

    patterns.insert(
        "Write Operations".to_string(),
        vec![
            "create".to_string(),
            "update".to_string(),
            "delete".to_string(),
            "remove".to_string(),
            "apply".to_string(),
            "patch".to_string(),
            "put".to_string(),
        ],
    );

    patterns.insert(
        "Debugging".to_string(),
        vec![
            "logs".to_string(),
            "exec".to_string(),
            "debug".to_string(),
            "trace".to_string(),
            "inspect".to_string(),
        ],
    );

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::claude_bridge::types::{ValidatedParameter, ValidatedTool};

    // === Categorization Tests ===

    #[test]
    fn test_categorize_tool_read_operations() {
        let transformer = Transformer::new();

        assert_eq!(
            transformer.categorize_tool("get_pods", "kubernetes"),
            "Read Operations"
        );
        assert_eq!(
            transformer.categorize_tool("list_deployments", "kubernetes"),
            "Read Operations"
        );
        assert_eq!(
            transformer.categorize_tool("describe_service", "kubernetes"),
            "Read Operations"
        );
        assert_eq!(
            transformer.categorize_tool("show_config", "test"),
            "Read Operations"
        );
    }

    #[test]
    fn test_categorize_tool_write_operations() {
        let transformer = Transformer::new();

        assert_eq!(
            transformer.categorize_tool("create_deployment", "kubernetes"),
            "Write Operations"
        );
        assert_eq!(
            transformer.categorize_tool("update_service", "kubernetes"),
            "Write Operations"
        );
        assert_eq!(
            transformer.categorize_tool("delete_pod", "kubernetes"),
            "Write Operations"
        );
        assert_eq!(
            transformer.categorize_tool("remove_resource", "test"),
            "Write Operations"
        );
        assert_eq!(
            transformer.categorize_tool("apply_config", "test"),
            "Write Operations"
        );
    }

    #[test]
    fn test_categorize_tool_debugging() {
        let transformer = Transformer::new();

        assert_eq!(
            transformer.categorize_tool("logs", "kubernetes"),
            "Debugging"
        );
        assert_eq!(
            transformer.categorize_tool("exec_command", "kubernetes"),
            "Debugging"
        );
        assert_eq!(
            transformer.categorize_tool("debug_pod", "kubernetes"),
            "Debugging"
        );
    }

    #[test]
    fn test_categorize_tool_scaling() {
        let transformer = Transformer::new();

        assert_eq!(
            transformer.categorize_tool("scale_deployment", "kubernetes"),
            "Scaling"
        );
    }

    #[test]
    fn test_categorize_tool_deployment() {
        let transformer = Transformer::new();

        assert_eq!(
            transformer.categorize_tool("rollout_restart", "kubernetes"),
            "Deployment"
        );
        assert_eq!(
            transformer.categorize_tool("deploy_app", "test"),
            "Deployment"
        );
    }

    #[test]
    fn test_categorize_tool_general() {
        let transformer = Transformer::new();

        assert_eq!(
            transformer.categorize_tool("custom_tool", "test"),
            "General"
        );
        assert_eq!(
            transformer.categorize_tool("special_operation", "test"),
            "General"
        );
    }

    #[test]
    fn test_categorize_tool_empty_name() {
        let transformer = Transformer::new();
        assert_eq!(
            transformer.categorize_tool("", "test"),
            "General"
        );
    }

    // === Build Categories Tests ===

    #[test]
    fn test_build_categories_single() {
        let transformer = Transformer::new();
        let tools = vec![
            ClaudeTool {
                name: "get_pods".to_string(),
                description: "Get pods".to_string(),
                parameters: vec![],
                examples: vec![],
                category: Some("Read Operations".to_string()),
                streaming: false,
            }
        ];

        let categories = transformer.build_categories(&tools);
        assert_eq!(categories.len(), 1);
        assert_eq!(categories.get("Read Operations").unwrap().len(), 1);
        assert_eq!(categories.get("Read Operations").unwrap()[0], "get_pods");
    }

    #[test]
    fn test_build_categories_multiple() {
        let transformer = Transformer::new();
        let tools = vec![
            ClaudeTool {
                name: "get_pods".to_string(),
                description: "Get pods".to_string(),
                parameters: vec![],
                examples: vec![],
                category: Some("Read Operations".to_string()),
                streaming: false,
            },
            ClaudeTool {
                name: "list_services".to_string(),
                description: "List services".to_string(),
                parameters: vec![],
                examples: vec![],
                category: Some("Read Operations".to_string()),
                streaming: false,
            },
            ClaudeTool {
                name: "create_deployment".to_string(),
                description: "Create deployment".to_string(),
                parameters: vec![],
                examples: vec![],
                category: Some("Write Operations".to_string()),
                streaming: false,
            },
        ];

        let categories = transformer.build_categories(&tools);
        assert_eq!(categories.len(), 2);
        assert_eq!(categories.get("Read Operations").unwrap().len(), 2);
        assert_eq!(categories.get("Write Operations").unwrap().len(), 1);
    }

    #[test]
    fn test_build_categories_no_category() {
        let transformer = Transformer::new();
        let tools = vec![
            ClaudeTool {
                name: "tool1".to_string(),
                description: "Tool 1".to_string(),
                parameters: vec![],
                examples: vec![],
                category: None,
                streaming: false,
            }
        ];

        let categories = transformer.build_categories(&tools);
        assert!(categories.contains_key("General"));
    }

    #[test]
    fn test_build_categories_empty() {
        let transformer = Transformer::new();
        let tools = vec![];
        let categories = transformer.build_categories(&tools);
        assert_eq!(categories.len(), 0);
    }

    // === Example Generation Tests ===

    #[test]
    fn test_generate_examples_basic() {
        let transformer = Transformer::new();
        let examples = transformer.generate_examples("get_pods", "kubernetes");

        assert_eq!(examples.len(), 1);
        assert!(examples[0].description.contains("get_pods"));
        assert!(examples[0].mcp_call.contains("kubernetes"));
        assert!(examples[0].mcp_call.contains("get_pods"));
        assert!(examples[0].script_call.contains("get_pods.sh"));
    }

    #[test]
    fn test_generate_examples_format() {
        let transformer = Transformer::new();
        let examples = transformer.generate_examples("list", "test");

        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].description, "Basic list usage");
        assert!(examples[0].mcp_call.starts_with("execute(skill='test'"));
        assert!(examples[0].script_call.starts_with("./scripts/list.sh"));
    }

    // === When to Use Generation Tests ===

    #[test]
    fn test_generate_when_to_use_kubernetes() {
        let transformer = Transformer::new();
        let triggers = transformer.generate_when_to_use("kubernetes", &SkillRuntimeType::Wasm);

        assert!(triggers.len() >= 2);
        assert!(triggers.iter().any(|t| t.contains("pods") || t.contains("deployments")));
    }

    #[test]
    fn test_generate_when_to_use_docker() {
        let transformer = Transformer::new();
        let triggers = transformer.generate_when_to_use("docker", &SkillRuntimeType::Native);

        assert!(triggers.len() >= 2);
        assert!(triggers.iter().any(|t| t.contains("container") || t.contains("image")));
    }

    #[test]
    fn test_generate_when_to_use_aws() {
        let transformer = Transformer::new();
        let triggers = transformer.generate_when_to_use("aws", &SkillRuntimeType::Docker);

        assert!(triggers.len() >= 2);
        assert!(triggers.iter().any(|t| t.contains("AWS")));
    }

    #[test]
    fn test_generate_when_to_use_git() {
        let transformer = Transformer::new();
        let triggers = transformer.generate_when_to_use("git", &SkillRuntimeType::Wasm);

        assert!(triggers.len() >= 2);
        assert!(triggers.iter().any(|t| t.contains("repositor") || t.contains("branch")));
    }

    #[test]
    fn test_generate_when_to_use_terraform() {
        let transformer = Transformer::new();
        let triggers = transformer.generate_when_to_use("terraform", &SkillRuntimeType::Native);

        assert!(triggers.len() >= 2);
        assert!(triggers.iter().any(|t| t.contains("infrastructure")));
    }

    #[test]
    fn test_generate_when_to_use_custom() {
        let transformer = Transformer::new();
        let triggers = transformer.generate_when_to_use("custom-skill", &SkillRuntimeType::Wasm);

        assert!(triggers.len() >= 2);
        assert!(triggers.iter().any(|t| t.contains("custom-skill")));
    }

    // === Full Transform Tests ===

    #[test]
    fn test_transform_basic_skill() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert_eq!(result.name, "test-skill");
        assert_eq!(result.description, "Test skill");
        assert_eq!(result.tools.len(), 0);
        assert_eq!(result.runtime, SkillRuntimeType::Wasm);
    }

    #[test]
    fn test_transform_skill_with_tools() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "kubernetes".to_string(),
            description: "Kubernetes management".to_string(),
            source: "./kubernetes".to_string(),
            runtime: SkillRuntimeType::Native,
            tools: vec![
                ValidatedTool {
                    name: "get_pods".to_string(),
                    description: "Get pods".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
                ValidatedTool {
                    name: "create_deployment".to_string(),
                    description: "Create deployment".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
            ],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert_eq!(result.tools.len(), 2);
        assert_eq!(result.tools[0].name, "get_pods");
        assert_eq!(result.tools[0].category, Some("Read Operations".to_string()));
        assert_eq!(result.tools[1].name, "create_deployment");
        assert_eq!(result.tools[1].category, Some("Write Operations".to_string()));
    }

    #[test]
    fn test_transform_tool_parameters() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![
                ValidatedTool {
                    name: "get".to_string(),
                    description: "Get resource".to_string(),
                    parameters: vec![
                        ValidatedParameter {
                            name: "resource".to_string(),
                            param_type: "string".to_string(),
                            description: "Resource type".to_string(),
                            required: true,
                            default_value: None,
                        },
                        ValidatedParameter {
                            name: "namespace".to_string(),
                            param_type: "string".to_string(),
                            description: "Namespace".to_string(),
                            required: false,
                            default_value: Some("default".to_string()),
                        },
                    ],
                    streaming: false,
                },
            ],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert_eq!(result.tools[0].parameters.len(), 2);
        assert_eq!(result.tools[0].parameters[0].name, "resource");
        assert_eq!(result.tools[0].parameters[0].param_type, "string");
        assert!(result.tools[0].parameters[0].required);
        assert_eq!(result.tools[0].parameters[1].name, "namespace");
        assert_eq!(result.tools[0].parameters[1].default_value, Some("default".to_string()));
    }

    #[test]
    fn test_transform_streaming_tool() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Docker,
            tools: vec![
                ValidatedTool {
                    name: "stream_logs".to_string(),
                    description: "Stream logs".to_string(),
                    parameters: vec![],
                    streaming: true,
                },
            ],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert!(result.tools[0].streaming);
    }

    #[test]
    fn test_transform_categories_created() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![
                ValidatedTool {
                    name: "get_item".to_string(),
                    description: "Get item".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
                ValidatedTool {
                    name: "create_item".to_string(),
                    description: "Create item".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
            ],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert!(result.categories.len() >= 2);
        assert!(result.categories.contains_key("Read Operations"));
        assert!(result.categories.contains_key("Write Operations"));
    }

    #[test]
    fn test_transform_when_to_use_generated() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "kubernetes".to_string(),
            description: "K8s management".to_string(),
            source: "./k8s".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert!(result.when_to_use.len() >= 2);
    }

    #[test]
    fn test_transform_examples_generated() {
        let transformer = Transformer::new();
        let validated = ValidatedSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Native,
            tools: vec![
                ValidatedTool {
                    name: "tool1".to_string(),
                    description: "Tool 1".to_string(),
                    parameters: vec![],
                    streaming: false,
                },
            ],
            skill_md_content: None,
        };

        let result = transformer.transform(validated).unwrap();
        assert!(!result.tools[0].examples.is_empty());
        assert!(result.tools[0].examples[0].mcp_call.contains("test"));
        assert!(result.tools[0].examples[0].script_call.contains("tool1"));
    }

    #[test]
    fn test_transform_all_runtime_types() {
        let transformer = Transformer::new();

        for runtime in [SkillRuntimeType::Wasm, SkillRuntimeType::Native, SkillRuntimeType::Docker] {
            let validated = ValidatedSkill {
                name: "test".to_string(),
                description: "Test".to_string(),
                source: "./test".to_string(),
                runtime: runtime.clone(),
                tools: vec![],
                skill_md_content: None,
            };

            let result = transformer.transform(validated).unwrap();
            assert_eq!(result.runtime, runtime);
        }
    }

    #[test]
    fn test_transformer_default() {
        let transformer = Transformer::default();
        let validated = ValidatedSkill {
            name: "test".to_string(),
            description: "Test".to_string(),
            source: "./test".to_string(),
            runtime: SkillRuntimeType::Wasm,
            tools: vec![],
            skill_md_content: None,
        };

        let result = transformer.transform(validated);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_category_patterns() {
        let patterns = default_category_patterns();
        assert!(patterns.contains_key("Read Operations"));
        assert!(patterns.contains_key("Write Operations"));
        assert!(patterns.contains_key("Debugging"));
    }
}
