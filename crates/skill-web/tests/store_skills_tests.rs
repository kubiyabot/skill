//! Tests for Skills Store
//!
//! These tests run in a WASM environment using wasm-bindgen-test

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use skill_web::store::skills::*;
use yewdux::prelude::*;
use std::rc::Rc;

wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Helper Functions
// ============================================================================

fn mock_skill(name: &str) -> SkillSummary {
    SkillSummary {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        description: format!("Test skill {}", name),
        source: format!("github:test/{}", name),
        runtime: SkillRuntime::Wasm,
        tools_count: 5,
        instances_count: 1,
        status: SkillStatus::Configured,
        last_used: None,
        execution_count: 0,
    }
}

// ============================================================================
// Store Initialization Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_skills_store_default_empty() {
    let store = SkillsStore::default();
    assert_eq!(store.skills.len(), 0);
    assert!(!store.loading);
    assert!(store.error.is_none());
}

#[wasm_bindgen_test]
fn test_skills_store_default_search_empty() {
    let store = SkillsStore::default();
    assert_eq!(store.search_query, "");
}

#[wasm_bindgen_test]
fn test_skills_store_default_no_filters() {
    let store = SkillsStore::default();
    assert!(store.status_filter.is_none());
    assert!(store.source_filter.is_none());
    assert!(store.runtime_filter.is_none());
}

// ============================================================================
// Add/Remove Skill Actions
// ============================================================================

#[wasm_bindgen_test]
fn test_add_skill_to_empty_store() {
    let store = Rc::new(SkillsStore::default());
    let skill = mock_skill("test-skill");

    let updated = SkillsAction::AddSkill(skill.clone()).apply(store);

    assert_eq!(updated.skills.len(), 1);
    assert_eq!(updated.skills[0].name, "test-skill");
}

#[wasm_bindgen_test]
fn test_add_multiple_skills() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("skill1"));
    let store_rc = Rc::new(store);

    let updated = SkillsAction::AddSkill(mock_skill("skill2")).apply(store_rc);

    assert_eq!(updated.skills.len(), 2);
}

#[wasm_bindgen_test]
fn test_add_duplicate_skill_replaces_existing() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("test-skill"));
    let store_rc = Rc::new(store);

    let mut updated_skill = mock_skill("test-skill");
    updated_skill.version = "0.2.0".to_string();

    let updated = SkillsAction::AddSkill(updated_skill).apply(store_rc);

    assert_eq!(updated.skills.len(), 1);
    assert_eq!(updated.skills[0].version, "0.2.0");
}

#[wasm_bindgen_test]
fn test_remove_skill_from_store() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("test-skill"));
    let store_rc = Rc::new(store);

    let updated = SkillsAction::RemoveSkill("test-skill".to_string()).apply(store_rc);

    assert_eq!(updated.skills.len(), 0);
}

#[wasm_bindgen_test]
fn test_remove_nonexistent_skill_no_error() {
    let store = Rc::new(SkillsStore::default());

    let updated = SkillsAction::RemoveSkill("nonexistent".to_string()).apply(store);

    assert_eq!(updated.skills.len(), 0);
}

// ============================================================================
// Set Skills Action
// ============================================================================

#[wasm_bindgen_test]
fn test_set_skills_replaces_all() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("old-skill"));
    let store_rc = Rc::new(store);

    let new_skills = vec![mock_skill("new-skill1"), mock_skill("new-skill2")];
    let updated = SkillsAction::SetSkills(new_skills).apply(store_rc);

    assert_eq!(updated.skills.len(), 2);
    assert!(updated.skills.iter().any(|s| s.name == "new-skill1"));
    assert!(updated.skills.iter().all(|s| s.name != "old-skill"));
}

#[wasm_bindgen_test]
fn test_set_skills_clears_loading_and_error() {
    let mut store = SkillsStore::default();
    store.loading = true;
    store.error = Some("error".to_string());
    let store_rc = Rc::new(store);

    let updated = SkillsAction::SetSkills(vec![]).apply(store_rc);

    assert!(!updated.loading);
    assert!(updated.error.is_none());
}

// ============================================================================
// Search and Filter Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_search_filter_by_name() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("aws-skill"));
    store.skills.push(mock_skill("github-skill"));
    store.search_query = "aws".to_string();

    let filtered = store.filtered_skills();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "aws-skill");
}

#[wasm_bindgen_test]
fn test_search_filter_case_insensitive() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("AWS-Skill"));
    store.search_query = "aws".to_string();

    let filtered = store.filtered_skills();

    assert_eq!(filtered.len(), 1);
}

#[wasm_bindgen_test]
fn test_search_filter_by_description() {
    let mut store = SkillsStore::default();
    let mut skill = mock_skill("test");
    skill.description = "Kubernetes management tool".to_string();
    store.skills.push(skill);
    store.search_query = "kubernetes".to_string();

    let filtered = store.filtered_skills();

    assert_eq!(filtered.len(), 1);
}

#[wasm_bindgen_test]
fn test_status_filter() {
    let mut store = SkillsStore::default();
    let mut skill1 = mock_skill("skill1");
    skill1.status = SkillStatus::Configured;
    let mut skill2 = mock_skill("skill2");
    skill2.status = SkillStatus::Unconfigured;

    store.skills.push(skill1);
    store.skills.push(skill2);
    store.status_filter = Some(SkillStatus::Configured);

    let filtered = store.filtered_skills();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "skill1");
}

#[wasm_bindgen_test]
fn test_source_filter() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("github-skill")); // source: github:test/github-skill

    let mut local_skill = mock_skill("local-skill");
    local_skill.source = "local:./local-skill".to_string();
    store.skills.push(local_skill);

    store.source_filter = Some("github:".to_string());

    let filtered = store.filtered_skills();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "github-skill");
}

#[wasm_bindgen_test]
fn test_runtime_filter() {
    let mut store = SkillsStore::default();
    let mut skill1 = mock_skill("wasm-skill");
    skill1.runtime = SkillRuntime::Wasm;
    let mut skill2 = mock_skill("docker-skill");
    skill2.runtime = SkillRuntime::Docker;

    store.skills.push(skill1);
    store.skills.push(skill2);
    store.runtime_filter = Some(SkillRuntime::Wasm);

    let filtered = store.filtered_skills();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "wasm-skill");
}

// ============================================================================
// Sorting Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_sort_by_name_ascending() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("zebra"));
    store.skills.push(mock_skill("alpha"));
    store.skills.push(mock_skill("middle"));
    store.sort_by = SkillSortBy::Name;
    store.sort_ascending = true;

    let sorted = store.filtered_skills();

    assert_eq!(sorted[0].name, "alpha");
    assert_eq!(sorted[1].name, "middle");
    assert_eq!(sorted[2].name, "zebra");
}

#[wasm_bindgen_test]
fn test_sort_by_name_descending() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("alpha"));
    store.skills.push(mock_skill("zebra"));
    store.sort_by = SkillSortBy::Name;
    store.sort_ascending = false;

    let sorted = store.filtered_skills();

    assert_eq!(sorted[0].name, "zebra");
    assert_eq!(sorted[1].name, "alpha");
}

#[wasm_bindgen_test]
fn test_sort_by_execution_count() {
    let mut store = SkillsStore::default();
    let mut skill1 = mock_skill("low");
    skill1.execution_count = 5;
    let mut skill2 = mock_skill("high");
    skill2.execution_count = 100;

    store.skills.push(skill1);
    store.skills.push(skill2);
    store.sort_by = SkillSortBy::ExecutionCount;
    store.sort_ascending = false;

    let sorted = store.filtered_skills();

    assert_eq!(sorted[0].name, "high");
    assert_eq!(sorted[1].name, "low");
}

// ============================================================================
// Loading and Error State Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_set_loading_state() {
    let store = Rc::new(SkillsStore::default());
    let updated = SkillsAction::SetLoading(true).apply(store);

    assert!(updated.loading);
}

#[wasm_bindgen_test]
fn test_set_error_clears_loading() {
    let mut store = SkillsStore::default();
    store.loading = true;
    let store_rc = Rc::new(store);

    let updated = SkillsAction::SetError(Some("error".to_string())).apply(store_rc);

    assert!(!updated.loading);
    assert_eq!(updated.error, Some("error".to_string()));
}

// ============================================================================
// Utility Method Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_get_skill_by_name() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("test-skill"));

    let found = store.get_skill("test-skill");

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test-skill");
}

#[wasm_bindgen_test]
fn test_get_skill_not_found() {
    let store = SkillsStore::default();
    let found = store.get_skill("nonexistent");

    assert!(found.is_none());
}

#[wasm_bindgen_test]
fn test_total_count() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("skill1"));
    store.skills.push(mock_skill("skill2"));

    assert_eq!(store.total_count(), 2);
}

#[wasm_bindgen_test]
fn test_filtered_count() {
    let mut store = SkillsStore::default();
    store.skills.push(mock_skill("aws-skill"));
    store.skills.push(mock_skill("github-skill"));
    store.search_query = "aws".to_string();

    assert_eq!(store.filtered_count(), 1);
    assert_eq!(store.total_count(), 2);
}
