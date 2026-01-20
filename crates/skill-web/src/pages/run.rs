//! Run page - Simplified execution workflow
//!
//! Clean, single-page interface with:
//! - Simple dropdown selection for Skill and Tool
//! - Dynamic parameter form
//! - Immediate execution feedback

use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::api::{Api, ExecutionResponse, SkillDetail};
use crate::components::run::{InlineParameterEditor, TerminalOutput};
use crate::components::notifications::use_notifications;
use crate::store::skills::{SkillsAction, SkillsStore};
use crate::components::SearchableSelect;

/// Run page props (for deep linking)
#[derive(Properties, PartialEq)]
pub struct RunPageProps {
    #[prop_or_default]
    pub selected_skill: Option<String>,
    #[prop_or_default]
    pub selected_tool: Option<String>,
}

/// Run page component
#[function_component(RunPage)]
pub fn run_page(props: &RunPageProps) -> Html {
    // Store for skills list
    let skills_store = use_store_value::<SkillsStore>();
    let skills_dispatch = use_dispatch::<SkillsStore>();

    // Form state
    let selected_skill = use_state(|| props.selected_skill.clone());
    let selected_tool = use_state(|| props.selected_tool.clone());
    let selected_instance = use_state(|| None::<String>);
    let parameters = use_state(HashMap::<String, serde_json::Value>::new);
    let validation_errors = use_state(HashMap::<String, String>::new);

    // All skill details (for tool lookup)
    let all_skill_details = use_state(Vec::<SkillDetail>::new);
    let current_skill_detail = use_state(|| None::<SkillDetail>);
    let skills_loading = use_state(|| true);

    // Execution state
    let execution_result = use_state(|| None::<ExecutionResponse>);
    let is_executing = use_state(|| false);
    let terminal_visible = use_state(|| false);
    let terminal_minimized = use_state(|| false);
    
    // UI Refs
    let result_ref = use_node_ref();

    // API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // Notifications
    let notifications = use_notifications();

    // Sync state with props (Deep Linking)
    {
        let selected_skill = selected_skill.clone();
        let selected_tool = selected_tool.clone();
        use_effect_with((props.selected_skill.clone(), props.selected_tool.clone()), move |(p_skill, p_tool)| {
            if let Some(s) = p_skill {
                selected_skill.set(Some(s.clone()));
            }
            if let Some(t) = p_tool {
                selected_tool.set(Some(t.clone()));
            }
            || ()
        });
    }

    // Load all skill details on mount
    {
        let api = api.clone();
        let skills_dispatch = skills_dispatch.clone();
        let all_skill_details = all_skill_details.clone();
        let skills_loading = skills_loading.clone();

        use_effect_with((), move |_| {
            skills_dispatch.apply(SkillsAction::SetLoading(true));
            skills_loading.set(true);

            let api = api.clone();
            let skills_dispatch = skills_dispatch.clone();
            let all_skill_details = all_skill_details.clone();
            let skills_loading = skills_loading.clone();

            spawn_local(async move {
                match api.skills.list_all().await {
                    Ok(skills) => {
                        let store_skills: Vec<crate::store::skills::SkillSummary> = skills
                            .iter()
                            .map(|s| crate::store::skills::SkillSummary {
                                name: s.name.clone(),
                                version: s.version.clone(),
                                description: s.description.clone(),
                                source: s.source.clone(),
                                runtime: match s.runtime.as_str() {
                                    "docker" => crate::store::skills::SkillRuntime::Docker,
                                    "native" => crate::store::skills::SkillRuntime::Native,
                                    _ => crate::store::skills::SkillRuntime::Wasm,
                                },
                                tools_count: s.tools_count,
                                instances_count: s.instances_count,
                                status: crate::store::skills::SkillStatus::Configured,
                                last_used: s.last_used.clone(),
                                execution_count: s.execution_count,
                            })
                            .collect();

                        skills_dispatch.apply(SkillsAction::SetSkills(store_skills));

                        let mut details = Vec::new();
                        for skill in skills.iter() {
                            match api.skills.get(&skill.name).await {
                                Ok(detail) => details.push(detail),
                                Err(e) => {
                                    web_sys::console::error_1(&format!("Failed to load skill {}: {}", skill.name, e).into());
                                }
                            }
                        }
                        all_skill_details.set(details);
                        skills_loading.set(false);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to load skills: {}", e).into());
                    }
                }
                skills_dispatch.apply(SkillsAction::SetLoading(false));
            });
            || ()
        });
    }

    // Auto-scroll to result when it arrives
    {
        let execution_result = execution_result.clone();
        let result_ref = result_ref.clone();
        use_effect_with(execution_result, move |result| {
            if result.is_some() {
                 if let Some(element) = result_ref.cast::<web_sys::Element>() {
                    let options = web_sys::ScrollIntoViewOptions::new();
                    options.set_behavior(web_sys::ScrollBehavior::Smooth);
                    element.scroll_into_view_with_scroll_into_view_options(&options);
                }
            }
            || ()
        });
    }

    // Update current skill detail when selection changes
    {
        let selected_skill = selected_skill.clone();
        let all_skill_details = all_skill_details.clone();
        let current_skill_detail = current_skill_detail.clone();
        let selected_tool = selected_tool.clone();
        let parameters = parameters.clone();

        use_effect_with((*selected_skill).clone(), move |skill_name| {
            if let Some(name) = skill_name {
                let detail = (*all_skill_details).iter()
                    .find(|d| d.summary.name == *name)
                    .cloned();
                current_skill_detail.set(detail);
                // Reset tool and params when skill changes
                selected_tool.set(None);
                parameters.set(HashMap::new());
            } else {
                current_skill_detail.set(None);
                selected_tool.set(None);
            }
            || ()
        });
    }

    // Handle parameter changes
    let on_parameter_change = {
        let parameters = parameters.clone();
        Callback::from(move |(name, value): (String, serde_json::Value)| {
            let mut params = (*parameters).clone();
            params.insert(name, value);
            parameters.set(params);
        })
    };

    // Execute command
    let on_execute = {
        let api = api.clone();
        let selected_skill = selected_skill.clone();
        let selected_tool = selected_tool.clone();
        let selected_instance = selected_instance.clone();
        let parameters = parameters.clone();
        let is_executing = is_executing.clone();
        let execution_result = execution_result.clone();
        let notifications = notifications.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let skill = (*selected_skill).clone();
            let tool = (*selected_tool).clone();

            if let (Some(skill_name), Some(tool_name)) = (skill, tool) {
                is_executing.set(true);

                let api = api.clone();
                let parameters = (*parameters).clone();
                let instance = (*selected_instance).clone();
                let is_executing = is_executing.clone();
                let execution_result = execution_result.clone();
                let notifications = notifications.clone();

                spawn_local(async move {
                    let request = crate::api::ExecutionRequest {
                        skill: skill_name.clone(),
                        tool: tool_name.clone(),
                        instance,
                        args: parameters,
                        stream: false,
                        timeout_secs: None,
                    };

                    match api.executions.execute(&request).await {
                        Ok(result) => {
                            let duration = result.duration_ms;
                            execution_result.set(Some(result));

                            // Show success toast
                            notifications.success(
                                "Execution completed",
                                format!("{}/{} executed successfully in {}ms", skill_name, tool_name, duration)
                            );
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to execute {}/{}: {}", skill_name, tool_name, e);
                            web_sys::console::error_1(&error_msg.clone().into());
                            notifications.error("Execution failed", error_msg);
                        }
                    }
                    is_executing.set(false);
                });
            }
        })
    };

    // Close terminal
    let on_terminal_close = {
        let terminal_visible = terminal_visible.clone();
        Callback::from(move |_| {
            terminal_visible.set(false);
        })
    };

    // Toggle terminal minimize
    let on_terminal_toggle_minimize = {
        let terminal_minimized = terminal_minimized.clone();
        Callback::from(move |_| {
            terminal_minimized.set(!*terminal_minimized);
        })
    };

    // Re-run command
    let on_rerun = {
        let api = api.clone();
        let selected_skill = selected_skill.clone();
        let selected_tool = selected_tool.clone();
        let selected_instance = selected_instance.clone();
        let parameters = parameters.clone();
        let is_executing = is_executing.clone();
        let execution_result = execution_result.clone();
        let notifications = notifications.clone();

        Some(Callback::from(move |_: ()| {
            let skill = (*selected_skill).clone();
            let tool = (*selected_tool).clone();

            if let (Some(skill_name), Some(tool_name)) = (skill, tool) {
                is_executing.set(true);

                let api = api.clone();
                let parameters = (*parameters).clone();
                let instance = (*selected_instance).clone();
                let is_executing = is_executing.clone();
                let execution_result = execution_result.clone();
                let notifications = notifications.clone();

                spawn_local(async move {
                    let request = crate::api::ExecutionRequest {
                        skill: skill_name.clone(),
                        tool: tool_name.clone(),
                        instance,
                        args: parameters,
                        stream: false,
                        timeout_secs: None,
                    };

                    match api.executions.execute(&request).await {
                        Ok(result) => {
                            let duration = result.duration_ms;
                            execution_result.set(Some(result));

                            notifications.success(
                                "Execution completed",
                                format!("{}/{} executed successfully in {}ms", skill_name, tool_name, duration)
                            );
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to execute {}/{}: {}", skill_name, tool_name, e);
                            web_sys::console::error_1(&error_msg.clone().into());
                            notifications.error("Execution failed", error_msg);
                        }
                    }
                    is_executing.set(false);
                });
            }
        }))
    };

    // Get current tool parameters
    let current_tool_params = current_skill_detail.as_ref()
        .and_then(|detail| {
            detail.tools.iter()
                .find(|t| Some(&t.name) == selected_tool.as_ref())
        })
        .map(|tool| tool.parameters.clone())
        .unwrap_or_default();

    // Get available tools for selected skill
    let available_tools = current_skill_detail.as_ref()
        .map(|detail| detail.tools.clone())
        .unwrap_or_default();

    // Check if form is complete
    let can_execute = selected_skill.is_some()
        && selected_tool.is_some()
        && !*is_executing;

    // Helper to handle select changes
    let on_skill_change = {
        let selected_skill = selected_skill.clone();
        Callback::from(move |value: String| {
            if value.is_empty() {
                selected_skill.set(None);
            } else {
                selected_skill.set(Some(value));
            }
        })
    };

    let on_tool_change = {
        let selected_tool = selected_tool.clone();
        let parameters = parameters.clone();
        Callback::from(move |value: String| {
            if value.is_empty() {
                selected_tool.set(None);
            } else {
                selected_tool.set(Some(value));
                parameters.set(HashMap::new()); // Reset params when tool changes
            }
        })
    };

    html! {
        <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900 overflow-hidden">
            // Header
            <div class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4 shrink-0">
                <div class="max-w-5xl mx-auto flex items-center justify-between">
                    <div>
                        <h1 class="text-xl font-bold text-gray-900 dark:text-white">
                            { "Run Skill" }
                        </h1>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                            { "Execute skills and tools with a simple dynamic form" }
                        </p>
                    </div>
                </div>
            </div>

            // Main Content
            <div class="flex-1 overflow-y-auto p-4 sm:p-6 lg:p-8">
                <div class="max-w-5xl mx-auto space-y-6">

                    // Selection Card
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                            // Skill Select
                            <div class="space-y-2">
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                    { "Select Skill" }
                                </label>
                                <div class="relative">
                                    <SearchableSelect
                                        options={skills_store.skills.iter().map(|s| s.name.clone()).collect::<Vec<_>>()}
                                        selected={selected_skill.as_deref().map(|s| s.to_string())}
                                        on_select={on_skill_change}
                                        placeholder="Choose a skill..."
                                        loading={*skills_loading}
                                    />
                                </div>
                                if let Some(_skill_name) = selected_skill.as_ref() {
                                    if let Some(detail) = current_skill_detail.as_ref() {
                                         <p class="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                                            { &detail.summary.description }
                                        </p>
                                    }
                                }
                            </div>

                            // Tool Select
                            <div class="space-y-2">
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                    { "Select Tool" }
                                </label>
                                <SearchableSelect
                                    options={available_tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>()}
                                    selected={selected_tool.as_deref().map(|s| s.to_string())}
                                    on_select={on_tool_change}
                                    placeholder="Choose a tool..."
                                    disabled={selected_skill.is_none()}
                                />
                                if let Some(tool_name) = selected_tool.as_ref() {
                                    if let Some(tool_detail) = available_tools.iter().find(|t| &t.name == tool_name) {
                                        <p class="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                                            { &tool_detail.description }
                                        </p>
                                    }
                                }
                            </div>
                        </div>
                    </div>

                    // Parameters & Execution
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 overflow-hidden">
                         <div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 flex items-center gap-2">
                            <svg class="w-4 h-4 text-primary-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            </svg>
                            <h3 class="font-medium text-gray-900 dark:text-white">
                                { "Configuration" }
                            </h3>
                        </div>

                        <div class="p-6">
                            if selected_skill.is_some() && selected_tool.is_some() {
                                if !current_tool_params.is_empty() {
                                    <InlineParameterEditor
                                        parameters={current_tool_params}
                                        values={(*parameters).clone()}
                                        on_change={on_parameter_change}
                                        errors={(*validation_errors).clone()}
                                    />
                                } else {
                                    <div class="text-center py-8 text-gray-500 dark:text-gray-400 text-sm">
                                        { "No parameters required for this tool." }
                                    </div>
                                }

                                <div class="mt-8 pt-6 border-t border-gray-200 dark:border-gray-700 flex justify-end">
                                    <button
                                        class={classes!(
                                            "btn",
                                            "btn-primary",
                                            "px-6",
                                            "py-2.5",
                                            "rounded-lg",
                                            "shadow-sm",
                                            "flex",
                                            "items-center",
                                            "gap-2",
                                            (!can_execute).then_some("opacity-50 cursor-not-allowed")
                                        )}
                                        onclick={on_execute}
                                        disabled={!can_execute}
                                    >
                                        if *is_executing {
                                            <svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                            </svg>
                                            { "Executing..." }
                                        } else {
                                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                            </svg>
                                            { "Run Command" }
                                        }
                                    </button>
                                </div>
                            } else {
                                <div class="text-center py-12 text-gray-400 dark:text-gray-500">
                                    <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                    <p class="text-sm">
                                        { "Select a skill and tool to configure parameters" }
                                    </p>
                                </div>
                            }
                        </div>
                    </div>

                    // Execution Result (Always visible placeholder)
                    <div ref={result_ref} class="card bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                         <div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <svg class="w-4 h-4 text-primary-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                </svg>
                                <h3 class="font-medium text-gray-900 dark:text-white">
                                    { "Execution Result" }
                                </h3>
                            </div>
                            if let Some(result) = &*execution_result {
                                <div class="text-xs font-mono text-gray-400 dark:text-gray-500">
                                    { format!("ID: {}", &result.id) }
                                </div>
                            }
                        </div>

                        <div class="p-6">
                            if let Some(result) = &*execution_result {
                                <div class="border-l-4 border-l-primary-500 pl-4 py-2">
                                     // Status header
                                    <div class="flex items-center justify-between mb-4">
                                        <div class="flex items-center gap-3">
                                            if result.status == crate::api::types::ExecutionStatus::Success {
                                                <div class="p-1.5 rounded-full bg-success-100 dark:bg-success-900/30 text-success-600 dark:text-success-400">
                                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                                    </svg>
                                                </div>
                                                <div>
                                                    <h4 class="font-semibold text-gray-900 dark:text-white">
                                                        { "Execution Successful" }
                                                    </h4>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400">
                                                        { format!("Completed in {}ms", result.duration_ms) }
                                                    </p>
                                                </div>
                                            } else {
                                                <div class="p-1.5 rounded-full bg-error-100 dark:bg-error-900/30 text-error-600 dark:text-error-400">
                                                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                                    </svg>
                                                </div>
                                                <div>
                                                    <h4 class="font-semibold text-gray-900 dark:text-white">
                                                        { "Execution Failed" }
                                                    </h4>
                                                    <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                                                        <span class="font-mono text-error-600 dark:text-error-400 bg-error-50 dark:bg-error-900/20 px-1.5 py-0.5 rounded">
                                                            { format!("{:?}", result.status) }
                                                        </span>
                                                        <span>{ format!("({}ms)", result.duration_ms) }</span>
                                                    </div>
                                                </div>
                                            }
                                        </div>
                                        <div class="flex gap-2">
                                             <button
                                                class="btn btn-secondary text-sm"
                                                onclick={{
                                                    let terminal_visible = terminal_visible.clone();
                                                    Callback::from(move |_| {
                                                        terminal_visible.set(true);
                                                    })
                                                }}
                                            >
                                                { "Show Terminal" }
                                            </button>
                                        </div>
                                    </div>

                                    // Output preview
                                    <div class="relative group mt-6">
                                        <div class="absolute -top-3 left-2 bg-white dark:bg-gray-800 px-2 text-xs font-semibold text-gray-500 dark:text-gray-400 tracking-wider uppercase">
                                            { "Output" }
                                        </div>
                                        <pre class="text-sm bg-gray-900 text-gray-50 p-5 rounded-lg overflow-x-auto max-h-96 font-mono shadow-inner border border-gray-800 leading-relaxed">
                                            { &result.output }
                                        </pre>
                                        <button
                                            class="absolute top-3 right-3 p-1.5 rounded bg-gray-800/80 text-gray-400 hover:text-white opacity-0 group-hover:opacity-100 transition-opacity backdrop-blur-sm"
                                            title="Copy output"
                                            onclick={{
                                                let output = result.output.clone();
                                                Callback::from(move |_| {
                                                    if let Some(window) = web_sys::window() {
                                                        let clipboard = window.navigator().clipboard();
                                                        let _ = clipboard.write_text(&output);
                                                    }
                                                })
                                            }}
                                        >
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                                            </svg>
                                        </button>
                                    </div>

                                    // Error message (if any)
                                    if let Some(error) = &result.error {
                                        <div class="mt-6">
                                            <div class="bg-error-50 dark:bg-error-900/10 border border-error-100 dark:border-error-900/30 rounded-lg overflow-hidden">
                                                 <div class="bg-error-100/50 dark:bg-error-900/20 px-4 py-2 border-b border-error-100 dark:border-error-900/30 flex items-center gap-2">
                                                    <svg class="w-4 h-4 text-error-600 dark:text-error-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                                    </svg>
                                                    <span class="text-xs font-bold text-error-700 dark:text-error-400 uppercase tracking-wide">
                                                        { "Error Details" }
                                                    </span>
                                                </div>
                                                <div class="p-4">
                                                    <pre class="text-error-600 dark:text-error-300 whitespace-pre-wrap font-mono text-sm leading-relaxed settings-scroll">
                                                        { error }
                                                    </pre>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                </div>
                            } else {
                                <div class="text-center py-12 text-gray-400 dark:text-gray-500">
                                    <p class="text-sm">
                                        { "Execution results will appear here" }
                                    </p>
                                </div>
                            }
                        </div>
                    </div>

                    // Bottom padding
                    <div class="h-8"></div>
                </div>
            </div>

            // Terminal Output (slide up from bottom)
            <TerminalOutput
                visible={*terminal_visible}
                execution={(*execution_result).clone()}
                on_close={on_terminal_close}
                on_rerun={on_rerun}
                minimized={*terminal_minimized}
                on_toggle_minimize={on_terminal_toggle_minimize}
            />
        </div>
    }
}
