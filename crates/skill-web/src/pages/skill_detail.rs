//! Skill detail page with tabs

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::api::{
    Api, ExecutionHistoryEntry, InstanceInfo, ParameterInfo, SkillDetail, ToolInfo,
};
use crate::components::card::Card;
use crate::components::icons::{ChevronRightIcon, PlayIcon};
use crate::components::instance_editor::{InstanceData, InstanceEditorModal};
use crate::router::Route;
use crate::store::executions::{ExecutionEntry, ExecutionStatus, ExecutionsAction, ExecutionsStore};

/// Skill detail page props
#[derive(Properties, PartialEq)]
pub struct SkillDetailPageProps {
    pub name: String,
    #[prop_or_default]
    pub selected_instance: Option<String>,
}

/// Tab enum
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Overview,
    Tools,
    Instances,
    History,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Tools => "Tools",
            Tab::Instances => "Instances",
            Tab::History => "History",
        }
    }

    fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Tools, Tab::Instances, Tab::History]
    }
}

/// Skill detail page component
#[function_component(SkillDetailPage)]
pub fn skill_detail_page(props: &SkillDetailPageProps) -> Html {
    let skill_detail = use_state(|| None::<SkillDetail>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let active_tab = use_state(Tab::default);

    // Get executions store for history
    let executions_store = use_store_value::<ExecutionsStore>();
    let executions_dispatch = use_dispatch::<ExecutionsStore>();

    // API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // Load skill detail on mount or when name changes
    {
        let api = api.clone();
        let skill_detail = skill_detail.clone();
        let loading = loading.clone();
        let error = error.clone();
        let executions_dispatch = executions_dispatch.clone();
        let skill_name = props.name.clone();

        use_effect_with(props.name.clone(), move |_| {
            loading.set(true);
            error.set(None);
            executions_dispatch.apply(ExecutionsAction::SetLoading(true));

            let api = api.clone();
            let skill_detail = skill_detail.clone();
            let loading = loading.clone();
            let error = error.clone();
            let executions_dispatch = executions_dispatch.clone();
            let skill_name = skill_name.clone();

            spawn_local(async move {
                // Load skill detail
                match api.skills.get(&skill_name).await {
                    Ok(detail) => {
                        skill_detail.set(Some(detail));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        loading.set(false);
                    }
                }

                // Load execution history for this skill
                match api.executions.recent_for_skill(&skill_name, 50).await {
                    Ok(history) => {
                        let store_history: Vec<ExecutionEntry> =
                            history.into_iter().map(api_to_store_execution).collect();
                        executions_dispatch.apply(ExecutionsAction::SetHistory(store_history));
                    }
                    Err(_) => {
                        // Silently fail - history is secondary
                    }
                }
            });
        });
    }

    // Filter executions for this skill
    let skill_executions: Vec<&ExecutionEntry> = executions_store
        .history
        .iter()
        .filter(|e| e.skill == props.name)
        .collect();

    html! {
        <div class="space-y-6 animate-fade-in">
            // Breadcrumb
            <nav class="flex items-center gap-2 text-sm">
                <Link<Route> to={Route::Skills} classes="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200">
                    { "Skills" }
                </Link<Route>>
                <ChevronRightIcon class="w-4 h-4 text-gray-400" />
                <span class="text-gray-900 dark:text-white font-medium">{ &props.name }</span>
            </nav>

            // Loading state
            if *loading {
                <SkillDetailSkeleton />
            } else if let Some(err) = (*error).clone() {
                // Error state
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6">
                    <div class="flex items-center gap-3">
                        <svg class="w-6 h-6 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <div>
                            <h3 class="text-lg font-medium text-red-800 dark:text-red-200">
                                { "Failed to load skill" }
                            </h3>
                            <p class="text-sm text-red-700 dark:text-red-300 mt-1">{ err }</p>
                        </div>
                    </div>
                    <Link<Route> to={Route::Skills} classes="btn btn-secondary mt-4">
                        { "← Back to Skills" }
                    </Link<Route>>
                </div>
            } else if let Some(skill) = (*skill_detail).clone() {
                // Header
                <SkillHeader skill={skill.clone()} />

                // Tabs
                <TabNavigation
                    active={*active_tab}
                    on_change={Callback::from({
                        let active_tab = active_tab.clone();
                        move |tab| active_tab.set(tab)
                    })}
                    tools_count={skill.tools.len()}
                    instances_count={skill.instances.len()}
                    history_count={skill_executions.len()}
                />

                // Tab content
                <div>
                    {
                        match *active_tab {
                            Tab::Overview => html! { <OverviewTab skill={skill.clone()} /> },
                            Tab::Tools => html! { <ToolsTab skill_name={props.name.clone()} tools={skill.tools.clone()} /> },
                            Tab::Instances => html! { <InstancesTab skill_name={props.name.clone()} instances={skill.instances.clone()} /> },
                            Tab::History => html! { <HistoryTab executions={skill_executions.iter().map(|e| (*e).clone()).collect::<Vec<_>>()} /> },
                        }
                    }
                </div>
            }
        </div>
    }
}

/// Convert API execution to store execution
fn api_to_store_execution(api: ExecutionHistoryEntry) -> ExecutionEntry {
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

// ============================================================================
// Skill Header
// ============================================================================

#[derive(Properties, PartialEq)]
struct SkillHeaderProps {
    skill: SkillDetail,
}

#[function_component(SkillHeader)]
fn skill_header(props: &SkillHeaderProps) -> Html {
    let skill = &props.skill;
    let summary = &skill.summary;

    let runtime_badge = match summary.runtime.as_str() {
        "docker" => html! { <span class="text-xs px-2 py-0.5 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded">{ "Docker" }</span> },
        "native" => html! { <span class="text-xs px-2 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded">{ "Native" }</span> },
        _ => html! { <span class="text-xs px-2 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded">{ "WASM" }</span> },
    };

    html! {
        <div class="flex items-start justify-between">
            <div>
                <div class="flex items-center gap-3">
                    <span class="status-dot status-dot-success" />
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                        { &summary.name }
                    </h1>
                    <span class="badge badge-info">{ format!("v{}", &summary.version) }</span>
                    { runtime_badge }
                </div>
                <p class="mt-2 text-gray-500 dark:text-gray-400 max-w-2xl">
                    { &summary.description }
                </p>
                <p class="mt-1 text-xs text-gray-400 font-mono">
                    { &summary.source }
                </p>
            </div>
            <div class="flex gap-2">
                <Link<Route>
                    to={Route::RunSkill { skill: summary.name.clone() }}
                    classes="btn btn-primary"
                >
                    <PlayIcon class="w-4 h-4 mr-2" />
                    { "Run" }
                </Link<Route>>
                <button class="btn btn-secondary">{ "Configure" }</button>
                <button class="btn btn-ghost text-error-600">{ "Uninstall" }</button>
            </div>
        </div>
    }
}

// ============================================================================
// Tab Navigation
// ============================================================================

#[derive(Properties, PartialEq)]
struct TabNavigationProps {
    active: Tab,
    on_change: Callback<Tab>,
    tools_count: usize,
    instances_count: usize,
    history_count: usize,
}

#[function_component(TabNavigation)]
fn tab_navigation(props: &TabNavigationProps) -> Html {
    html! {
        <div class="border-b border-gray-200 dark:border-gray-700">
            <nav class="flex gap-6">
                { for Tab::all().iter().map(|tab| {
                    let is_active = props.active == *tab;
                    let on_change = props.on_change.clone();
                    let tab_value = *tab;
                    let onclick = Callback::from(move |_| on_change.emit(tab_value));

                    let count = match tab {
                        Tab::Tools => Some(props.tools_count),
                        Tab::Instances => Some(props.instances_count),
                        Tab::History => Some(props.history_count),
                        _ => None,
                    };

                    html! {
                        <button
                            class={classes!(
                                "py-3", "px-1", "text-sm", "font-medium", "border-b-2", "transition-colors",
                                "flex", "items-center", "gap-2",
                                if is_active {
                                    "border-primary-600 text-primary-600"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                }
                            )}
                            {onclick}
                        >
                            { tab.label() }
                            if let Some(c) = count {
                                <span class={classes!(
                                    "text-xs", "px-1.5", "py-0.5", "rounded-full",
                                    if is_active {
                                        "bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"
                                    } else {
                                        "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400"
                                    }
                                )}>
                                    { c }
                                </span>
                            }
                        </button>
                    }
                }) }
            </nav>
        </div>
    }
}

// ============================================================================
// Overview Tab
// ============================================================================

#[derive(Properties, PartialEq)]
struct OverviewTabProps {
    skill: SkillDetail,
}

#[function_component(OverviewTab)]
fn overview_tab(props: &OverviewTabProps) -> Html {
    let skill = &props.skill;
    let summary = &skill.summary;

    html! {
        <div class="space-y-6">
            // Description
            <Card title="Description">
                <div class="prose dark:prose-invert max-w-none">
                    if let Some(ref full_desc) = skill.full_description {
                        <p class="whitespace-pre-wrap">{ full_desc }</p>
                    } else {
                        <p>{ &summary.description }</p>
                    }
                </div>
            </Card>

            // Quick Start
            <Card title="Quick Start">
                <div class="bg-gray-900 dark:bg-gray-950 rounded-lg p-4 font-mono text-sm text-gray-100 overflow-x-auto">
                    <code>{ format!("skill run {}:{} <args>", summary.name, skill.tools.first().map(|t| t.name.as_str()).unwrap_or("tool_name")) }</code>
                </div>
            </Card>

            // Metadata
            <Card title="Metadata">
                <dl class="grid grid-cols-2 gap-4">
                    <div>
                        <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Source" }</dt>
                        <dd class="mt-1 font-mono text-sm truncate">{ &summary.source }</dd>
                    </div>
                    <div>
                        <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Runtime" }</dt>
                        <dd class="mt-1">{ &summary.runtime }</dd>
                    </div>
                    if let Some(ref author) = skill.author {
                        <div>
                            <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Author" }</dt>
                            <dd class="mt-1">{ author }</dd>
                        </div>
                    }
                    if let Some(ref license) = skill.license {
                        <div>
                            <dt class="text-sm text-gray-500 dark:text-gray-400">{ "License" }</dt>
                            <dd class="mt-1">{ license }</dd>
                        </div>
                    }
                    <div>
                        <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Tools" }</dt>
                        <dd class="mt-1">{ format!("{} available", summary.tools_count) }</dd>
                    </div>
                    <div>
                        <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Instances" }</dt>
                        <dd class="mt-1">{ format!("{} configured", summary.instances_count) }</dd>
                    </div>
                    <div>
                        <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Total Executions" }</dt>
                        <dd class="mt-1">{ format!("{}", summary.execution_count) }</dd>
                    </div>
                    if let Some(ref last_used) = summary.last_used {
                        <div>
                            <dt class="text-sm text-gray-500 dark:text-gray-400">{ "Last Used" }</dt>
                            <dd class="mt-1">{ format_date(last_used) }</dd>
                        </div>
                    }
                </dl>
            </Card>

            if let Some(ref repo) = skill.repository {
                <Card title="Repository">
                    <a href={repo.clone()} target="_blank" rel="noopener noreferrer" class="text-primary-600 dark:text-primary-400 hover:underline">
                        { repo }
                    </a>
                </Card>
            }
        </div>
    }
}

// ============================================================================
// Tools Tab
// ============================================================================

#[derive(Properties, PartialEq)]
struct ToolsTabProps {
    skill_name: String,
    tools: Vec<ToolInfo>,
}

#[function_component(ToolsTab)]
fn tools_tab(props: &ToolsTabProps) -> Html {
    if props.tools.is_empty() {
        return html! {
            <div class="text-center py-12">
                <svg class="w-12 h-12 mx-auto text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                </svg>
                <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">
                    { "No tools defined" }
                </h3>
                <p class="mt-2 text-gray-500 dark:text-gray-400">
                    { "This skill doesn't have any tools yet." }
                </p>
            </div>
        };
    }

    html! {
        <div class="space-y-4">
            { for props.tools.iter().map(|tool| {
                html! {
                    <ToolCard
                        skill_name={props.skill_name.clone()}
                        tool={tool.clone()}
                    />
                }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ToolCardProps {
    skill_name: String,
    tool: ToolInfo,
}

#[function_component(ToolCard)]
fn tool_card(props: &ToolCardProps) -> Html {
    let expanded = use_state(|| false);
    let toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(!*expanded))
    };

    let tool = &props.tool;
    let has_params = !tool.parameters.is_empty();

    html! {
        <div class="card">
            <div class="p-4">
                <div class="flex items-start justify-between">
                    <div class="flex-1">
                        <div class="flex items-center gap-2">
                            <h4 class="font-semibold text-gray-900 dark:text-white">
                                { &tool.name }
                            </h4>
                            if tool.streaming {
                                <span class="text-xs px-2 py-0.5 bg-cyan-100 dark:bg-cyan-900/30 text-cyan-700 dark:text-cyan-300 rounded">
                                    { "Streaming" }
                                </span>
                            }
                        </div>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            { &tool.description }
                        </p>
                    </div>
                    <Link<Route>
                        to={Route::RunSkillTool {
                            skill: props.skill_name.clone(),
                            tool: tool.name.clone()
                        }}
                        classes="btn btn-secondary btn-sm"
                    >
                        <PlayIcon class="w-3 h-3 mr-1" />
                        { "Run" }
                    </Link<Route>>
                </div>

                if has_params {
                    <button
                        class="mt-3 text-sm text-primary-600 dark:text-primary-400 hover:underline flex items-center gap-1"
                        onclick={toggle}
                    >
                        if *expanded {
                            { "Hide parameters" }
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
                            </svg>
                        } else {
                            { format!("Show {} parameters", tool.parameters.len()) }
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                            </svg>
                        }
                    </button>

                    if *expanded {
                        <div class="mt-4 border-t border-gray-200 dark:border-gray-700 pt-4">
                            <h5 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                                { "Parameters" }
                            </h5>
                            <div class="space-y-3">
                                { for tool.parameters.iter().map(|param| {
                                    html! { <ParameterRow param={param.clone()} /> }
                                }) }
                            </div>
                        </div>
                    }
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ParameterRowProps {
    param: ParameterInfo,
}

#[function_component(ParameterRow)]
fn parameter_row(props: &ParameterRowProps) -> Html {
    let param = &props.param;

    html! {
        <div class="flex items-start gap-4 text-sm">
            <div class="flex-shrink-0 w-32">
                <code class="text-primary-600 dark:text-primary-400">{ &param.name }</code>
                if param.required {
                    <span class="ml-1 text-red-500">{ "*" }</span>
                }
            </div>
            <div class="flex-shrink-0 w-24 text-gray-400 font-mono text-xs">
                { &param.param_type }
            </div>
            <div class="flex-1 text-gray-600 dark:text-gray-300">
                { &param.description }
                if let Some(ref default) = param.default_value {
                    <span class="ml-2 text-gray-400">
                        { format!("(default: {})", default) }
                    </span>
                }
            </div>
        </div>
    }
}

// ============================================================================
// Instances Tab
// ============================================================================

#[derive(Properties, PartialEq)]
struct InstancesTabProps {
    skill_name: String,
    instances: Vec<InstanceInfo>,
}

#[function_component(InstancesTab)]
fn instances_tab(props: &InstancesTabProps) -> Html {
    // Modal state
    let modal_open = use_state(|| false);
    let editing_instance = use_state(|| None::<InstanceData>);

    let open_create_modal = {
        let modal_open = modal_open.clone();
        let editing_instance = editing_instance.clone();
        Callback::from(move |_| {
            editing_instance.set(None);
            modal_open.set(true);
        })
    };

    let open_edit_modal = {
        let modal_open = modal_open.clone();
        let editing_instance = editing_instance.clone();
        Callback::from(move |instance: InstanceInfo| {
            editing_instance.set(Some(InstanceData {
                name: instance.name,
                description: instance.description.unwrap_or_default(),
                config: instance.config_keys.iter().map(|k| (k.clone(), String::new())).collect(),
                is_default: instance.is_default,
                capabilities: Default::default(),
            }));
            modal_open.set(true);
        })
    };

    let close_modal = {
        let modal_open = modal_open.clone();
        Callback::from(move |_| {
            modal_open.set(false);
        })
    };

    let on_save = {
        let modal_open = modal_open.clone();
        Callback::from(move |_data: InstanceData| {
            // TODO: Call API to save instance
            // For now just close the modal
            modal_open.set(false);
            // In real implementation would refresh the skill detail
        })
    };

    if props.instances.is_empty() {
        return html! {
            <>
                <div class="text-center py-12">
                    <svg class="w-12 h-12 mx-auto text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                    </svg>
                    <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">
                        { "No instances configured" }
                    </h3>
                    <p class="mt-2 text-gray-500 dark:text-gray-400">
                        { "Add an instance to configure this skill for different environments." }
                    </p>
                    <button class="btn btn-primary mt-4" onclick={open_create_modal.clone()}>
                        { "+ Add Instance" }
                    </button>
                </div>

                <InstanceEditorModal
                    open={*modal_open}
                    skill={props.skill_name.clone()}
                    instance={(*editing_instance).clone()}
                    on_save={on_save}
                    on_close={close_modal}
                />
            </>
        };
    }

    html! {
        <>
            <div class="space-y-4">
                <div class="flex justify-end">
                    <button class="btn btn-primary btn-sm" onclick={open_create_modal}>
                        { "+ Add Instance" }
                    </button>
                </div>

                { for props.instances.iter().map(|instance| {
                    let on_edit = {
                        let open_edit_modal = open_edit_modal.clone();
                        let instance = instance.clone();
                        Callback::from(move |_| {
                            open_edit_modal.emit(instance.clone());
                        })
                    };
                    html! { <InstanceCard instance={instance.clone()} on_edit={on_edit} /> }
                }) }
            </div>

            <InstanceEditorModal
                open={*modal_open}
                skill={props.skill_name.clone()}
                instance={(*editing_instance).clone()}
                on_save={on_save}
                on_close={close_modal}
            />
        </>
    }
}

#[derive(Properties, PartialEq)]
struct InstanceCardProps {
    instance: InstanceInfo,
    on_edit: Callback<()>,
}

#[function_component(InstanceCard)]
fn instance_card(props: &InstanceCardProps) -> Html {
    let instance = &props.instance;

    let on_edit_click = {
        let on_edit = props.on_edit.clone();
        Callback::from(move |_| on_edit.emit(()))
    };

    html! {
        <Card hoverable=true>
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <span class="status-dot status-dot-success" />
                    <div>
                        <div class="flex items-center gap-2">
                            <h4 class="font-semibold text-gray-900 dark:text-white">
                                { &instance.name }
                            </h4>
                            if instance.is_default {
                                <span class="badge badge-info">{ "Default" }</span>
                            }
                        </div>
                        if let Some(ref desc) = instance.description {
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                { desc }
                            </p>
                        }
                        if !instance.config_keys.is_empty() {
                            <p class="text-xs text-gray-400 mt-1">
                                { format!("Config: {}", instance.config_keys.join(", ")) }
                            </p>
                        }
                    </div>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-ghost btn-sm" onclick={on_edit_click}>{ "Edit" }</button>
                    if !instance.is_default {
                        <button class="btn btn-ghost btn-sm text-error-600">{ "Delete" }</button>
                    }
                </div>
            </div>
        </Card>
    }
}

// ============================================================================
// History Tab
// ============================================================================

#[derive(Properties, PartialEq)]
struct HistoryTabProps {
    executions: Vec<ExecutionEntry>,
}

#[function_component(HistoryTab)]
fn history_tab(props: &HistoryTabProps) -> Html {
    if props.executions.is_empty() {
        return html! {
            <Card>
                <div class="text-center py-8">
                    <PlayIcon class="w-12 h-12 mx-auto text-gray-400" />
                    <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">
                        { "No execution history" }
                    </h3>
                    <p class="mt-2 text-gray-500 dark:text-gray-400">
                        { "Run this skill to see execution history here." }
                    </p>
                </div>
            </Card>
        };
    }

    html! {
        <div class="space-y-3">
            { for props.executions.iter().map(|entry| {
                html! { <ExecutionRow entry={entry.clone()} /> }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ExecutionRowProps {
    entry: ExecutionEntry,
}

#[function_component(ExecutionRow)]
fn execution_row(props: &ExecutionRowProps) -> Html {
    let entry = &props.entry;

    let (status_class, status_text) = match entry.status {
        ExecutionStatus::Success => ("text-success-600", "Success"),
        ExecutionStatus::Failed => ("text-error-600", "Failed"),
        ExecutionStatus::Timeout => ("text-warning-600", "Timeout"),
        ExecutionStatus::Running => ("text-primary-600", "Running"),
        ExecutionStatus::Pending => ("text-gray-500", "Pending"),
        ExecutionStatus::Cancelled => ("text-gray-400", "Cancelled"),
    };

    let duration_str = if entry.duration_ms < 1000 {
        format!("{}ms", entry.duration_ms)
    } else if entry.duration_ms < 60000 {
        format!("{:.1}s", entry.duration_ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", entry.duration_ms as f64 / 60000.0)
    };

    html! {
        <div class="card p-4">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <div>
                        <div class="flex items-center gap-2">
                            <code class="text-sm font-medium">{ &entry.tool }</code>
                            <span class={classes!("text-xs", "font-medium", status_class)}>
                                { status_text }
                            </span>
                        </div>
                        <div class="text-xs text-gray-400 mt-1">
                            <span>{ &entry.instance }</span>
                            <span class="mx-2">{ "•" }</span>
                            <span>{ format_date(&entry.started_at) }</span>
                        </div>
                    </div>
                </div>
                <div class="text-sm text-gray-500">
                    { duration_str }
                </div>
            </div>
            if let Some(ref error) = entry.error {
                <div class="mt-2 text-xs text-error-600 dark:text-error-400 bg-red-50 dark:bg-red-900/20 rounded p-2">
                    { error }
                </div>
            }
        </div>
    }
}

// ============================================================================
// Skeleton Loader
// ============================================================================

#[function_component(SkillDetailSkeleton)]
fn skill_detail_skeleton() -> Html {
    html! {
        <div class="space-y-6 animate-pulse">
            // Header skeleton
            <div class="flex items-start justify-between">
                <div class="space-y-3">
                    <div class="flex items-center gap-3">
                        <div class="w-3 h-3 bg-gray-200 dark:bg-gray-700 rounded-full"></div>
                        <div class="h-8 w-48 bg-gray-200 dark:bg-gray-700 rounded"></div>
                        <div class="h-6 w-16 bg-gray-200 dark:bg-gray-700 rounded"></div>
                    </div>
                    <div class="h-4 w-96 bg-gray-200 dark:bg-gray-700 rounded"></div>
                    <div class="h-3 w-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
                </div>
                <div class="flex gap-2">
                    <div class="h-10 w-20 bg-gray-200 dark:bg-gray-700 rounded"></div>
                    <div class="h-10 w-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
                </div>
            </div>

            // Tabs skeleton
            <div class="flex gap-6 border-b border-gray-200 dark:border-gray-700 pb-3">
                { for (0..4).map(|_| html! {
                    <div class="h-4 w-20 bg-gray-200 dark:bg-gray-700 rounded"></div>
                }) }
            </div>

            // Content skeleton
            <div class="space-y-6">
                <div class="card p-6">
                    <div class="h-5 w-32 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
                    <div class="space-y-2">
                        <div class="h-4 w-full bg-gray-200 dark:bg-gray-700 rounded"></div>
                        <div class="h-4 w-3/4 bg-gray-200 dark:bg-gray-700 rounded"></div>
                        <div class="h-4 w-1/2 bg-gray-200 dark:bg-gray-700 rounded"></div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Utilities
// ============================================================================

fn format_date(timestamp: &str) -> String {
    // Simple formatting - just show date part
    if timestamp.len() > 10 {
        timestamp[..10].to_string()
    } else {
        timestamp.to_string()
    }
}
