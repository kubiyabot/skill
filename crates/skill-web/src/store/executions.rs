//! Executions state store
//!
//! Manages execution history and active executions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yewdux::prelude::*;

/// Execution status
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
    Cancelled,
}

impl ExecutionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failed | Self::Timeout | Self::Cancelled)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Execution history entry
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionEntry {
    /// Unique execution ID
    pub id: String,
    /// Skill name
    pub skill: String,
    /// Tool name
    pub tool: String,
    /// Instance used
    pub instance: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Input arguments
    pub args: HashMap<String, String>,
    /// Output content (if completed)
    pub output: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// When the execution started
    pub started_at: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Active execution state (for real-time updates)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveExecution {
    /// Execution entry
    pub entry: ExecutionEntry,
    /// Streaming output chunks
    pub output_chunks: Vec<String>,
    /// Progress percentage (0-100)
    pub progress: Option<u8>,
}

/// Executions store state
#[derive(Clone, Debug, PartialEq, Store)]
pub struct ExecutionsStore {
    /// Execution history (most recent first)
    pub history: Vec<ExecutionEntry>,
    /// Currently active execution (if any)
    pub active: Option<ActiveExecution>,
    /// Whether history is being loaded
    pub loading: bool,
    /// Error message
    pub error: Option<String>,
    /// Filter by skill
    pub skill_filter: Option<String>,
    /// Filter by status
    pub status_filter: Option<ExecutionStatus>,
    /// Maximum history entries to keep
    pub max_history: usize,
}

impl Default for ExecutionsStore {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            active: None,
            loading: false,
            error: None,
            skill_filter: None,
            status_filter: None,
            max_history: 1000,
        }
    }
}

impl ExecutionsStore {
    /// Get filtered history
    pub fn filtered_history(&self) -> Vec<&ExecutionEntry> {
        self.history
            .iter()
            .filter(|entry| {
                // Skill filter
                if let Some(ref skill) = self.skill_filter {
                    if &entry.skill != skill {
                        return false;
                    }
                }

                // Status filter
                if let Some(ref status) = self.status_filter {
                    if &entry.status != status {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get execution by ID
    pub fn get_execution(&self, id: &str) -> Option<&ExecutionEntry> {
        self.history.iter().find(|e| e.id == id)
    }

    /// Get recent executions for a skill
    pub fn recent_for_skill(&self, skill: &str, limit: usize) -> Vec<&ExecutionEntry> {
        self.history
            .iter()
            .filter(|e| e.skill == skill)
            .take(limit)
            .collect()
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        let successes = self.history.iter().filter(|e| e.status.is_success()).count();
        successes as f32 / self.history.len() as f32
    }

    /// Get average duration
    pub fn average_duration_ms(&self) -> u64 {
        if self.history.is_empty() {
            return 0;
        }
        let total: u64 = self.history.iter().map(|e| e.duration_ms).sum();
        total / self.history.len() as u64
    }

    /// Check if there's an active execution
    pub fn has_active(&self) -> bool {
        self.active.is_some()
    }
}

/// Executions store actions
pub enum ExecutionsAction {
    /// Set entire history
    SetHistory(Vec<ExecutionEntry>),
    /// Add execution to history
    AddExecution(ExecutionEntry),
    /// Update an execution
    UpdateExecution(ExecutionEntry),
    /// Remove execution by ID
    RemoveExecution(String),
    /// Clear all history
    ClearHistory,
    /// Start a new execution
    StartExecution(ExecutionEntry),
    /// Update active execution output
    AppendOutput(String),
    /// Update active execution progress
    SetProgress(u8),
    /// Complete active execution
    CompleteExecution(ExecutionEntry),
    /// Cancel active execution
    CancelExecution,
    /// Set loading state
    SetLoading(bool),
    /// Set error
    SetError(Option<String>),
    /// Set skill filter
    SetSkillFilter(Option<String>),
    /// Set status filter
    SetStatusFilter(Option<ExecutionStatus>),
    /// Clear filters
    ClearFilters,
}

impl Reducer<ExecutionsStore> for ExecutionsAction {
    fn apply(self, mut store: std::rc::Rc<ExecutionsStore>) -> std::rc::Rc<ExecutionsStore> {
        let state = std::rc::Rc::make_mut(&mut store);

        match self {
            ExecutionsAction::SetHistory(history) => {
                state.history = history;
                state.loading = false;
                state.error = None;
            }
            ExecutionsAction::AddExecution(entry) => {
                // Add to front (most recent first)
                state.history.insert(0, entry);
                // Trim to max
                if state.history.len() > state.max_history {
                    state.history.truncate(state.max_history);
                }
            }
            ExecutionsAction::UpdateExecution(entry) => {
                if let Some(existing) = state.history.iter_mut().find(|e| e.id == entry.id) {
                    *existing = entry;
                }
            }
            ExecutionsAction::RemoveExecution(id) => {
                state.history.retain(|e| e.id != id);
            }
            ExecutionsAction::ClearHistory => {
                state.history.clear();
            }
            ExecutionsAction::StartExecution(entry) => {
                state.active = Some(ActiveExecution {
                    entry,
                    output_chunks: Vec::new(),
                    progress: None,
                });
            }
            ExecutionsAction::AppendOutput(chunk) => {
                if let Some(ref mut active) = state.active {
                    active.output_chunks.push(chunk);
                }
            }
            ExecutionsAction::SetProgress(progress) => {
                if let Some(ref mut active) = state.active {
                    active.progress = Some(progress.min(100));
                }
            }
            ExecutionsAction::CompleteExecution(entry) => {
                // Add to history
                state.history.insert(0, entry);
                if state.history.len() > state.max_history {
                    state.history.truncate(state.max_history);
                }
                // Clear active
                state.active = None;
            }
            ExecutionsAction::CancelExecution => {
                if let Some(active) = state.active.take() {
                    // Add cancelled execution to history
                    let mut entry = active.entry;
                    entry.status = ExecutionStatus::Cancelled;
                    state.history.insert(0, entry);
                }
            }
            ExecutionsAction::SetLoading(loading) => {
                state.loading = loading;
            }
            ExecutionsAction::SetError(error) => {
                state.error = error;
                state.loading = false;
            }
            ExecutionsAction::SetSkillFilter(filter) => {
                state.skill_filter = filter;
            }
            ExecutionsAction::SetStatusFilter(filter) => {
                state.status_filter = filter;
            }
            ExecutionsAction::ClearFilters => {
                state.skill_filter = None;
                state.status_filter = None;
            }
        }

        store
    }
}
