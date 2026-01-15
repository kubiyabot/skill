//! Skills state store

use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

/// Skill status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SkillStatus {
    Configured,
    Unconfigured,
    Error,
    Loading,
}

/// Runtime type
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SkillRuntime {
    #[default]
    Wasm,
    Docker,
    Native,
}

/// Skill summary data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: String,
    pub runtime: SkillRuntime,
    pub tools_count: usize,
    pub instances_count: usize,
    pub status: SkillStatus,
    pub last_used: Option<String>,
    pub execution_count: u64,
}

/// Tool information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
    pub streaming: bool,
}

/// Parameter information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Instance information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub config_keys: Vec<String>,
}

/// Detailed skill information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillDetail {
    pub summary: SkillSummary,
    pub full_description: Option<String>,
    pub author: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub tools: Vec<ToolInfo>,
    pub instances: Vec<InstanceInfo>,
}

/// Skills store state
#[derive(Clone, Debug, Default, PartialEq, Store)]
pub struct SkillsStore {
    /// List of installed skills
    pub skills: Vec<SkillSummary>,
    /// Currently selected skill (for detail view)
    pub selected_skill: Option<SkillDetail>,
    /// Whether skills are being loaded
    pub loading: bool,
    /// Whether a skill detail is being loaded
    pub detail_loading: bool,
    /// Error message if loading failed
    pub error: Option<String>,
    /// Search query
    pub search_query: String,
    /// Selected filters
    pub status_filter: Option<SkillStatus>,
    pub source_filter: Option<String>,
    pub runtime_filter: Option<SkillRuntime>,
    /// Sort order
    pub sort_by: SkillSortBy,
    pub sort_ascending: bool,
}

/// Sort options for skills
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SkillSortBy {
    #[default]
    Name,
    LastUsed,
    ExecutionCount,
    ToolsCount,
}

impl SkillsStore {
    /// Get filtered and sorted skills based on current search and filters
    pub fn filtered_skills(&self) -> Vec<&SkillSummary> {
        let mut skills: Vec<&SkillSummary> = self.skills
            .iter()
            .filter(|skill| {
                // Search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !skill.name.to_lowercase().contains(&query)
                        && !skill.description.to_lowercase().contains(&query)
                    {
                        return false;
                    }
                }

                // Status filter
                if let Some(ref status) = self.status_filter {
                    if &skill.status != status {
                        return false;
                    }
                }

                // Source filter
                if let Some(ref source) = self.source_filter {
                    if !skill.source.contains(source) {
                        return false;
                    }
                }

                // Runtime filter
                if let Some(ref runtime) = self.runtime_filter {
                    if &skill.runtime != runtime {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Sort
        skills.sort_by(|a, b| {
            let cmp = match self.sort_by {
                SkillSortBy::Name => a.name.cmp(&b.name),
                SkillSortBy::LastUsed => a.last_used.cmp(&b.last_used),
                SkillSortBy::ExecutionCount => a.execution_count.cmp(&b.execution_count),
                SkillSortBy::ToolsCount => a.tools_count.cmp(&b.tools_count),
            };
            if self.sort_ascending { cmp } else { cmp.reverse() }
        });

        skills
    }

    /// Get skill by name
    pub fn get_skill(&self, name: &str) -> Option<&SkillSummary> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// Get total skills count
    pub fn total_count(&self) -> usize {
        self.skills.len()
    }

    /// Get count of filtered skills
    pub fn filtered_count(&self) -> usize {
        self.filtered_skills().len()
    }
}

/// Skills store actions
pub enum SkillsAction {
    SetSkills(Vec<SkillSummary>),
    AddSkill(SkillSummary),
    RemoveSkill(String),
    UpdateSkill(SkillSummary),
    SetSelectedSkill(Option<SkillDetail>),
    SetLoading(bool),
    SetDetailLoading(bool),
    SetError(Option<String>),
    SetSearchQuery(String),
    SetStatusFilter(Option<SkillStatus>),
    SetSourceFilter(Option<String>),
    SetRuntimeFilter(Option<SkillRuntime>),
    SetSortBy(SkillSortBy),
    ToggleSortOrder,
    ClearFilters,
}

impl Reducer<SkillsStore> for SkillsAction {
    fn apply(self, mut store: std::rc::Rc<SkillsStore>) -> std::rc::Rc<SkillsStore> {
        let state = std::rc::Rc::make_mut(&mut store);

        match self {
            SkillsAction::SetSkills(skills) => {
                state.skills = skills;
                state.loading = false;
                state.error = None;
            }
            SkillsAction::AddSkill(skill) => {
                // Remove existing if present, then add
                state.skills.retain(|s| s.name != skill.name);
                state.skills.push(skill);
            }
            SkillsAction::RemoveSkill(name) => {
                state.skills.retain(|s| s.name != name);
                // Clear selected if it was removed
                if let Some(ref selected) = state.selected_skill {
                    if selected.summary.name == name {
                        state.selected_skill = None;
                    }
                }
            }
            SkillsAction::UpdateSkill(skill) => {
                if let Some(existing) = state.skills.iter_mut().find(|s| s.name == skill.name) {
                    *existing = skill;
                }
            }
            SkillsAction::SetSelectedSkill(skill) => {
                state.selected_skill = skill;
                state.detail_loading = false;
            }
            SkillsAction::SetLoading(loading) => {
                state.loading = loading;
            }
            SkillsAction::SetDetailLoading(loading) => {
                state.detail_loading = loading;
            }
            SkillsAction::SetError(error) => {
                state.error = error;
                state.loading = false;
                state.detail_loading = false;
            }
            SkillsAction::SetSearchQuery(query) => {
                state.search_query = query;
            }
            SkillsAction::SetStatusFilter(filter) => {
                state.status_filter = filter;
            }
            SkillsAction::SetSourceFilter(filter) => {
                state.source_filter = filter;
            }
            SkillsAction::SetRuntimeFilter(filter) => {
                state.runtime_filter = filter;
            }
            SkillsAction::SetSortBy(sort_by) => {
                state.sort_by = sort_by;
            }
            SkillsAction::ToggleSortOrder => {
                state.sort_ascending = !state.sort_ascending;
            }
            SkillsAction::ClearFilters => {
                state.search_query = String::new();
                state.status_filter = None;
                state.source_filter = None;
                state.runtime_filter = None;
            }
        }

        store
    }
}
