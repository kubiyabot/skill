//! Dashboard page - main overview with real-time statistics

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::api::{Api, ExecutionHistoryEntry as ApiExecutionEntry, SkillSummary as ApiSkillSummary};
use crate::components::card::{Card, StatCard, Trend};
use crate::components::icons::{CheckIcon, LightningIcon, PlayIcon, SkillsIcon};
use crate::router::Route;
use crate::store::executions::{ExecutionEntry, ExecutionStatus, ExecutionsAction, ExecutionsStore};
use crate::store::skills::{SkillRuntime, SkillStatus, SkillSummary, SkillsAction, SkillsStore};

/// Convert API skill summary to store skill summary
fn api_to_store_skill(api: ApiSkillSummary) -> SkillSummary {
    SkillSummary {
        name: api.name,
        version: api.version,
        description: api.description,
        source: api.source,
        runtime: match api.runtime.as_str() {
            "docker" => SkillRuntime::Docker,
            "native" => SkillRuntime::Native,
            _ => SkillRuntime::Wasm,
        },
        tools_count: api.tools_count,
        instances_count: api.instances_count,
        status: SkillStatus::Configured,
        last_used: api.last_used,
        execution_count: api.execution_count,
    }
}

/// Convert API execution entry to store execution entry
fn api_to_store_execution(api: ApiExecutionEntry) -> ExecutionEntry {
    ExecutionEntry {
        id: api.id,
        skill: api.skill,
        tool: api.tool,
        instance: api.instance,
        status: match api.status {
            crate::api::ExecutionStatus::Pending => ExecutionStatus::Pending,
            crate::api::ExecutionStatus::Running => ExecutionStatus::Running,
            crate::api::ExecutionStatus::Success => ExecutionStatus::Success,
            crate::api::ExecutionStatus::Failed => ExecutionStatus::Failed,
            crate::api::ExecutionStatus::Timeout => ExecutionStatus::Timeout,
            crate::api::ExecutionStatus::Cancelled => ExecutionStatus::Cancelled,
        },
        args: std::collections::HashMap::new(),
        output: None,
        error: api.error,
        duration_ms: api.duration_ms,
        started_at: api.started_at,
        metadata: std::collections::HashMap::new(),
    }
}

/// Dashboard page component
#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {
    let skills_store = use_store_value::<SkillsStore>();
    let skills_dispatch = use_dispatch::<SkillsStore>();
    let executions_store = use_store_value::<ExecutionsStore>();
    let executions_dispatch = use_dispatch::<ExecutionsStore>();

    // Create API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // Load data on mount
    {
        let api = api.clone();
        let skills_dispatch = skills_dispatch.clone();
        let executions_dispatch = executions_dispatch.clone();

        use_effect_with((), move |_| {
            // Set loading states
            skills_dispatch.apply(SkillsAction::SetLoading(true));
            executions_dispatch.apply(ExecutionsAction::SetLoading(true));

            let api = api.clone();
            let skills_dispatch = skills_dispatch.clone();
            let executions_dispatch = executions_dispatch.clone();

            spawn_local(async move {
                // Load skills
                match api.skills.list_all().await {
                    Ok(skills) => {
                        let store_skills: Vec<SkillSummary> =
                            skills.into_iter().map(api_to_store_skill).collect();
                        skills_dispatch.apply(SkillsAction::SetSkills(store_skills));
                    }
                    Err(e) => {
                        skills_dispatch.apply(SkillsAction::SetError(Some(e.to_string())));
                    }
                }

                // Load execution history
                match api.executions.list_all_history().await {
                    Ok(history) => {
                        let store_history: Vec<ExecutionEntry> =
                            history.into_iter().map(api_to_store_execution).collect();
                        executions_dispatch.apply(ExecutionsAction::SetHistory(store_history));
                    }
                    Err(e) => {
                        executions_dispatch.apply(ExecutionsAction::SetError(Some(e.to_string())));
                    }
                }
            });
        });
    }

    // Calculate statistics
    let skill_count = skills_store.skills.len();
    let execution_count = executions_store.history.len();
    let success_rate = executions_store.success_rate();
    let success_rate_str = format!("{:.1}%", success_rate * 100.0);

    // Get recent executions (last 5)
    let recent_executions: Vec<&ExecutionEntry> =
        executions_store.history.iter().take(5).collect();

    // Loading state
    let is_loading = skills_store.loading || executions_store.loading;

    // Error state
    let error = skills_store
        .error
        .clone()
        .or_else(|| executions_store.error.clone());

    html! {
        <div class="space-y-6 animate-fade-in">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                        { "Dashboard" }
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 mt-1">
                        { "Overview of your Skill Engine" }
                    </p>
                </div>
                <Link<Route> to={Route::Run} classes="btn btn-primary">
                    <PlayIcon class="w-4 h-4 mr-2" />
                    { "Run Skill" }
                </Link<Route>>
            </div>

            // Error alert
            if let Some(err) = error {
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                    <div class="flex items-center gap-3">
                        <svg class="w-5 h-5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <p class="text-sm text-red-700 dark:text-red-300">{ err }</p>
                    </div>
                </div>
            }

            // Stats grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatCard
                    title="Total Skills"
                    value={if is_loading { "--".to_string() } else { skill_count.to_string() }}
                    subtitle={format!("{} tools available", skills_store.skills.iter().map(|s| s.tools_count).sum::<usize>())}
                    icon={html! { <SkillsIcon class="w-6 h-6 text-primary-600" /> }}
                />
                <StatCard
                    title="Total Executions"
                    value={if is_loading { "--".to_string() } else { execution_count.to_string() }}
                    subtitle="All time"
                    icon={html! { <PlayIcon class="w-6 h-6 text-primary-600" /> }}
                />
                <StatCard
                    title="Success Rate"
                    value={if is_loading { "--".to_string() } else { success_rate_str }}
                    subtitle="All executions"
                    icon={html! { <CheckIcon class="w-6 h-6 text-success-500" /> }}
                    trend={if success_rate >= 0.95 {
                        Some(Trend::Up("Excellent".to_string()))
                    } else if success_rate >= 0.80 {
                        Some(Trend::Neutral("Good".to_string()))
                    } else if execution_count > 0 {
                        Some(Trend::Down("Needs attention".to_string()))
                    } else {
                        None
                    }}
                />
                <StatCard
                    title="Search Ready"
                    value="RAG"
                    subtitle="FastEmbed active"
                    icon={html! { <LightningIcon class="w-6 h-6 text-warning-500" /> }}
                />
            </div>

            // Quick actions and recent activity
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Quick Actions
                <Card title="Quick Actions">
                    <div class="space-y-3">
                        <Link<Route>
                            to={Route::Skills}
                            classes="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                        >
                            <div class="p-2 bg-primary-50 dark:bg-primary-900/30 rounded-lg">
                                <SkillsIcon class="w-5 h-5 text-primary-600 dark:text-primary-400" />
                            </div>
                            <div class="flex-1">
                                <p class="font-medium text-gray-900 dark:text-white">
                                    { "Browse Skills" }
                                </p>
                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                    { format!("View and manage {} installed skills", skill_count) }
                                </p>
                            </div>
                        </Link<Route>>

                        <Link<Route>
                            to={Route::Run}
                            classes="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                        >
                            <div class="p-2 bg-success-50 dark:bg-green-900/30 rounded-lg">
                                <PlayIcon class="w-5 h-5 text-success-600 dark:text-green-400" />
                            </div>
                            <div class="flex-1">
                                <p class="font-medium text-gray-900 dark:text-white">
                                    { "Execute Tool" }
                                </p>
                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                    { "Run a skill tool with parameters" }
                                </p>
                            </div>
                        </Link<Route>>

                        <Link<Route>
                            to={Route::Settings}
                            classes="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                        >
                            <div class="p-2 bg-gray-100 dark:bg-gray-700 rounded-lg">
                                <LightningIcon class="w-5 h-5 text-gray-600 dark:text-gray-400" />
                            </div>
                            <div class="flex-1">
                                <p class="font-medium text-gray-900 dark:text-white">
                                    { "Configure Search" }
                                </p>
                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                    { "Tune RAG pipeline settings" }
                                </p>
                            </div>
                        </Link<Route>>
                    </div>
                </Card>

                // Recent Activity
                <Card title="Recent Activity">
                    if is_loading {
                        <div class="flex items-center justify-center py-8">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
                        </div>
                    } else if recent_executions.is_empty() {
                        <div class="text-center py-8">
                            <PlayIcon class="w-12 h-12 text-gray-300 dark:text-gray-600 mx-auto mb-3" />
                            <p class="text-gray-500 dark:text-gray-400">
                                { "No executions yet" }
                            </p>
                            <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
                                { "Run a skill to see activity here" }
                            </p>
                        </div>
                    } else {
                        <div class="space-y-4">
                            { for recent_executions.iter().map(|entry| {
                                html! {
                                    <ActivityItem
                                        skill={entry.skill.clone()}
                                        tool={entry.tool.clone()}
                                        status={entry.status.clone()}
                                        time={entry.started_at.clone()}
                                        duration_ms={entry.duration_ms}
                                    />
                                }
                            })}
                        </div>
                    }
                    <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                        <Link<Route>
                            to={Route::History}
                            classes="text-sm text-primary-600 dark:text-primary-400 hover:underline"
                        >
                            { "View all activity →" }
                        </Link<Route>>
                    </div>
                </Card>
            </div>
        </div>
    }
}

/// Activity item props
#[derive(Properties, PartialEq)]
struct ActivityItemProps {
    skill: String,
    tool: String,
    status: ExecutionStatus,
    time: String,
    duration_ms: u64,
}

/// Activity item component
#[function_component(ActivityItem)]
fn activity_item(props: &ActivityItemProps) -> Html {
    let (status_class, status_icon) = match props.status {
        ExecutionStatus::Success => (
            "status-dot-success",
            html! { <CheckIcon class="w-4 h-4 text-success-500" /> },
        ),
        ExecutionStatus::Failed | ExecutionStatus::Timeout => (
            "status-dot-error",
            html! {
                <svg class="w-4 h-4 text-error-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
            },
        ),
        ExecutionStatus::Running | ExecutionStatus::Pending => (
            "status-dot-warning",
            html! {
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-warning-500"></div>
            },
        ),
        ExecutionStatus::Cancelled => (
            "status-dot-neutral",
            html! {
                <svg class="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                </svg>
            },
        ),
    };

    // Format duration
    let duration_str = if props.duration_ms < 1000 {
        format!("{}ms", props.duration_ms)
    } else if props.duration_ms < 60000 {
        format!("{:.1}s", props.duration_ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", props.duration_ms as f64 / 60000.0)
    };

    // Format time (simple relative time approximation)
    let time_str = format_relative_time(&props.time);

    html! {
        <div class="flex items-center gap-3">
            <span class={classes!("status-dot", status_class)} />
            <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-gray-900 dark:text-white truncate">
                    { format!("{}:{}", props.skill, props.tool) }
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400">
                    { time_str }
                </p>
            </div>
            <div class="flex items-center gap-2">
                <span class="text-xs text-gray-400">{ duration_str }</span>
                { status_icon }
            </div>
        </div>
    }
}

/// Format a timestamp as relative time (simplified)
fn format_relative_time(timestamp: &str) -> String {
    // For now, just return the timestamp
    // In a real app, we'd parse and calculate the difference
    if timestamp.len() > 16 {
        timestamp[..16].replace('T', " ")
    } else {
        timestamp.to_string()
    }
}
