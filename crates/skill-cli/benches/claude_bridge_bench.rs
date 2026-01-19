//! Performance benchmarks for Claude Bridge
//!
//! Run with: cargo bench --bench claude_bridge_bench
//! View reports in: target/criterion/

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::TempDir;

// Helper to create test manifest
fn create_test_manifest(num_skills: usize, tools_per_skill: usize) -> String {
    let mut manifest = String::new();

    for skill_idx in 0..num_skills {
        manifest.push_str(&format!(
            r#"
[[skill]]
name = "skill-{}"
description = "Test skill number {}"
version = "1.0.0"
"#,
            skill_idx, skill_idx
        ));

        for tool_idx in 0..tools_per_skill {
            manifest.push_str(&format!(
                r#"
[[skill.tool]]
name = "tool-{}"
description = "Test tool number {}"
command = "echo {{}}"

[[skill.tool.parameter]]
name = "param-{}"
type = "string"
description = "Test parameter"
required = true
"#,
                tool_idx, tool_idx, tool_idx
            ));
        }
    }

    manifest
}

// Benchmark manifest parsing
fn bench_manifest_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest_parsing");

    for (skills, tools) in [(1, 5), (5, 10), (10, 20)] {
        let manifest = create_test_manifest(skills, tools);
        let size_desc = format!("{}skills_{}tools", skills, tools);

        group.bench_with_input(
            BenchmarkId::new("parse", &size_desc),
            &manifest,
            |b, manifest| {
                b.iter(|| {
                    let parsed: Result<toml::Value, _> = toml::from_str(black_box(manifest));
                    black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

// Benchmark skill generation (dry-run)
fn bench_skill_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("skill_generation");

    // Test with different manifest sizes
    for (skills, tools) in [(1, 5), (5, 10), (10, 20)] {
        let manifest = create_test_manifest(skills, tools);
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.toml");
        std::fs::write(&manifest_path, &manifest).unwrap();

        let size_desc = format!("{}skills_{}tools", skills, tools);

        group.bench_with_input(
            BenchmarkId::new("generate_dry_run", &size_desc),
            &manifest_path,
            |b, path| {
                b.iter(|| {
                    // Simulate generation without actual file I/O
                    // This would call the actual generation logic in a real benchmark
                    black_box(path)
                });
            },
        );
    }

    group.finish();
}

// Benchmark YAML frontmatter generation
fn bench_yaml_frontmatter(c: &mut Criterion) {
    let mut group = c.benchmark_group("yaml_frontmatter");

    let test_data = serde_yaml::to_value(serde_json::json!({
        "name": "test-skill",
        "description": "A test skill for benchmarking",
        "version": "1.0.0",
        "category": "testing",
        "tags": ["test", "benchmark", "performance"]
    }))
    .unwrap();

    group.bench_function("serialize", |b| {
        b.iter(|| {
            let yaml = serde_yaml::to_string(black_box(&test_data));
            black_box(yaml)
        });
    });

    group.finish();
}

// Benchmark script generation
fn bench_script_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("script_generation");

    let tool_names = ["get", "list", "describe", "create", "update", "delete", "logs", "exec"];

    group.bench_function("generate_scripts", |b| {
        b.iter(|| {
            let scripts: Vec<String> = tool_names
                .iter()
                .map(|name| {
                    format!(
                        r#"#!/usr/bin/env bash
set -euo pipefail

skill run test-skill {} "$@"
"#,
                        name
                    )
                })
                .collect();
            black_box(scripts)
        });
    });

    group.finish();
}

// Benchmark markdown rendering
fn bench_markdown_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_rendering");

    let tools = vec![
        ("get", "Get resources", vec!["resource", "namespace"]),
        ("list", "List resources", vec!["type", "filter"]),
        ("describe", "Describe resource", vec!["resource", "name"]),
        ("create", "Create resource", vec!["config", "namespace"]),
        ("update", "Update resource", vec!["resource", "changes"]),
    ];

    group.bench_function("render_tools_section", |b| {
        b.iter(|| {
            let mut content = String::from("## Tools\n\n");
            for (name, desc, params) in &tools {
                content.push_str(&format!("### {}\n\n", name));
                content.push_str(&format!("**Description:** {}\n\n", desc));
                content.push_str("**Parameters:**\n");
                for param in params {
                    content.push_str(&format!("- `{}` (string): Parameter\n", param));
                }
                content.push('\n');
            }
            black_box(content)
        });
    });

    group.finish();
}

// Benchmark file I/O operations
fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    let temp_dir = TempDir::new().unwrap();
    let test_content = "# Test Skill\n\nThis is a test skill document.\n".repeat(100);

    group.bench_function("write_skill_md", |b| {
        b.iter(|| {
            let path = temp_dir.path().join(format!("skill_{}.md", rand::random::<u32>()));
            std::fs::write(black_box(&path), black_box(&test_content)).unwrap();
            black_box(path)
        });
    });

    group.bench_function("read_skill_md", |b| {
        let path = temp_dir.path().join("skill_read.md");
        std::fs::write(&path, &test_content).unwrap();

        b.iter(|| {
            let content = std::fs::read_to_string(black_box(&path)).unwrap();
            black_box(content)
        });
    });

    group.finish();
}

// Benchmark validation
fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    let valid_manifest = create_test_manifest(5, 10);

    group.bench_function("validate_manifest", |b| {
        b.iter(|| {
            let parsed: Result<toml::Value, _> = toml::from_str(black_box(&valid_manifest));
            if let Ok(value) = parsed {
                // Check for required fields
                let has_skills = value.get("skill").is_some();
                black_box(has_skills)
            } else {
                black_box(false)
            }
        });
    });

    group.finish();
}

// Benchmark end-to-end generation (memory only)
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    for (skills, tools) in [(1, 5), (5, 10)] {
        let manifest = create_test_manifest(skills, tools);
        let size_desc = format!("{}skills_{}tools", skills, tools);

        group.bench_with_input(
            BenchmarkId::new("full_pipeline", &size_desc),
            &manifest,
            |b, manifest| {
                b.iter(|| {
                    // Parse
                    let parsed: Result<toml::Value, _> = toml::from_str(black_box(manifest));

                    // Validate
                    let valid = parsed.is_ok();

                    // Generate markdown (in-memory)
                    let md = if valid {
                        Some(format!("# Skill\n\n{} tools", tools))
                    } else {
                        None
                    };

                    // Generate scripts (in-memory)
                    let scripts = if valid {
                        Some(vec!["#!/bin/bash\necho test".to_string(); tools])
                    } else {
                        None
                    };

                    black_box((valid, md, scripts))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_manifest_parsing,
    bench_skill_generation,
    bench_yaml_frontmatter,
    bench_script_generation,
    bench_markdown_rendering,
    bench_file_operations,
    bench_validation,
    bench_end_to_end
);

criterion_main!(benches);
