//! Execution history page

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{Api, ExecutionHistoryEntry, ExecutionStatus as ApiExecutionStatus};
use crate::components::card::Card;
use crate::components::icons::{CheckIcon, RefreshIcon, SearchIcon};

/// History page props
#[derive(Properties, PartialEq)]
pub struct HistoryPageProps {
    #[prop_or_default]
    pub selected_id: Option<String>,
}

/// History page component
#[function_component(HistoryPage)]
pub fn history_page(_props: &HistoryPageProps) -> Html {
    let search_query = use_state(String::new);
    let executions = use_state(Vec::<ExecutionHistoryEntry>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // Load executions on mount
    {
        let api = api.clone();
        let executions = executions.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            loading.set(true);
            error.set(None);

            spawn_local(async move {
                match api.executions.list_all_history().await {
                    Ok(history) => {
                        executions.set(history);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }
                loading.set(false);
            });
        });
    }

    // Refresh handler
    let on_refresh = {
        let api = api.clone();
        let executions = executions.clone();
        let loading = loading.clone();
        let error = error.clone();

        Callback::from(move |_| {
            loading.set(true);
            error.set(None);

            let api = api.clone();
            let executions = executions.clone();
            let loading = loading.clone();
            let error = error.clone();

            spawn_local(async move {
                match api.executions.list_all_history().await {
                    Ok(history) => {
                        executions.set(history);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }
                loading.set(false);
            });
        })
    };

    let on_search = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    // Filter executions by search query
    let filtered_executions: Vec<&ExecutionHistoryEntry> = executions
        .iter()
        .filter(|exec| {
            if search_query.is_empty() {
                return true;
            }
            let query = search_query.to_lowercase();
            exec.skill.to_lowercase().contains(&query)
                || exec.tool.to_lowercase().contains(&query)
                || exec.instance.to_lowercase().contains(&query)
                || exec.id.to_lowercase().contains(&query)
        })
        .collect();

    html! {
        <div class="space-y-6 animate-fade-in">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                        { "Execution History" }
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 mt-1">
                        { "View past skill executions and their results" }
                    </p>
                </div>
                <button class="btn btn-secondary" onclick={on_refresh} disabled={*loading}>
                    <RefreshIcon class={classes!("w-4", "h-4", "mr-2", if *loading { "animate-spin" } else { "" })} />
                    { if *loading { "Loading..." } else { "Refresh" } }
                </button>
            </div>

            // Filters
            <Card>
                <div class="flex flex-col md:flex-row gap-4">
                    <div class="flex-1 relative">
                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <SearchIcon class="w-5 h-5 text-gray-400" />
                        </div>
                        <input
                            type="text"
                            placeholder="Search executions..."
                            class="input pl-10"
                            value={(*search_query).clone()}
                            oninput={on_search}
                        />
                    </div>
                </div>
            </Card>

            // Error state
            if let Some(err) = (*error).clone() {
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                    <p class="text-red-700 dark:text-red-300">{ format!("Failed to load history: {}", err) }</p>
                </div>
            }

            // Executions table
            <Card>
                if *loading && executions.is_empty() {
                    <div class="flex items-center justify-center py-12">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
                        <span class="ml-3 text-gray-500">{ "Loading executions..." }</span>
                    </div>
                } else if filtered_executions.is_empty() {
                    <div class="text-center py-12">
                        <svg class="w-12 h-12 mx-auto text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">
                            { "No executions found" }
                        </h3>
                        <p class="mt-2 text-gray-500 dark:text-gray-400">
                            { "Run some skills to see execution history here." }
                        </p>
                    </div>
                } else {
                    <div class="overflow-x-auto">
                        <table class="table">
                            <thead>
                                <tr>
                                    <th>{ "ID" }</th>
                                    <th>{ "Skill / Tool" }</th>
                                    <th>{ "Instance" }</th>
                                    <th>{ "Status" }</th>
                                    <th>{ "Duration" }</th>
                                    <th>{ "Time" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for filtered_executions.iter().map(|exec| html! {
                                    <ExecutionRow execution={(*exec).clone()} />
                                }) }
                            </tbody>
                        </table>
                    </div>

                    // Summary
                    <div class="mt-4 flex items-center justify-between">
                        <p class="text-sm text-gray-500">
                            { format!("Showing {} executions", filtered_executions.len()) }
                        </p>
                    </div>
                }
            </Card>
        </div>
    }
}

/// Execution row props
#[derive(Properties, PartialEq)]
struct ExecutionRowProps {
    execution: ExecutionHistoryEntry,
}

/// Execution table row component
#[function_component(ExecutionRow)]
fn execution_row(props: &ExecutionRowProps) -> Html {
    let exec = &props.execution;
    let expanded = use_state(|| false);

    let (status_badge, status_icon, status_text) = match exec.status {
        ApiExecutionStatus::Success => (
            "badge-success",
            html! { <CheckIcon class="w-3 h-3" /> },
            "Success",
        ),
        ApiExecutionStatus::Failed => (
            "badge-error",
            html! {
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
            },
            "Failed",
        ),
        ApiExecutionStatus::Running => ("badge-info", html! {}, "Running"),
        ApiExecutionStatus::Pending => ("badge-neutral", html! {}, "Pending"),
        ApiExecutionStatus::Timeout => ("badge-warning", html! {}, "Timeout"),
        ApiExecutionStatus::Cancelled => ("badge-neutral", html! {}, "Cancelled"),
    };

    // Format duration
    let duration_str = if exec.duration_ms < 1000 {
        format!("{}ms", exec.duration_ms)
    } else if exec.duration_ms < 60000 {
        format!("{:.1}s", exec.duration_ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", exec.duration_ms as f64 / 60000.0)
    };

    // Format timestamp
    let time_str = format_timestamp(&exec.started_at);

    // Truncate ID for display
    let short_id = if exec.id.len() > 8 {
        format!("{}...", &exec.id[..8])
    } else {
        exec.id.clone()
    };

    let toggle_expanded = {
        let expanded = expanded.clone();
        Callback::from(move |_| {
            expanded.set(!*expanded);
        })
    };

    html! {
        <>
            <tr class="hover:bg-gray-50 dark:hover:bg-gray-800/30 cursor-pointer" onclick={toggle_expanded}>
                <td class="font-mono text-xs text-gray-500" title={exec.id.clone()}>
                    <div class="flex items-center gap-2">
                        if *expanded {
                            <svg class="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                            </svg>
                        } else {
                            <svg class="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                            </svg>
                        }
                        { short_id }
                    </div>
                </td>
                <td>
                    <span class="font-medium text-gray-900 dark:text-white">{ &exec.skill }</span>
                    <span class="text-gray-400">{ ":" }</span>
                    <span class="text-gray-600 dark:text-gray-300">{ &exec.tool }</span>
                </td>
                <td>
                    <span class="badge badge-info">{ &exec.instance }</span>
                </td>
                <td>
                    <span class={classes!("badge", status_badge, "flex", "items-center", "gap-1")}>
                        { status_icon }
                        { status_text }
                    </span>
                </td>
                <td class="text-gray-500">{ duration_str }</td>
                <td class="text-gray-500">{ time_str }</td>
            </tr>
            if *expanded {
                <tr class="bg-gray-50 dark:bg-gray-900/50">
                    <td colspan="6" class="p-4">
                        <div class="space-y-4">
                            // Error display
                            if let Some(error) = &exec.error {
                                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
                                    <div class="flex items-start gap-2">
                                        <svg class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                        <div>
                                            <h4 class="font-medium text-red-800 dark:text-red-300">{ "Error" }</h4>
                                            <pre class="mt-1 text-sm text-red-700 dark:text-red-400 whitespace-pre-wrap font-mono">{ error }</pre>
                                        </div>
                                    </div>
                                </div>
                            }

                            // Output display
                            if let Some(output) = &exec.output {
                                <div>
                                    <div class="flex items-center justify-between mb-2">
                                        <h4 class="font-medium text-gray-900 dark:text-white">{ "Output" }</h4>
                                        <span class="text-xs text-gray-500">{ format!("{} characters", output.len()) }</span>
                                    </div>
                                    <div class="bg-gray-900 dark:bg-black rounded-lg p-4 overflow-x-auto">
                                        <pre class="text-sm text-gray-100 dark:text-gray-300 font-mono whitespace-pre-wrap">{ output }</pre>
                                    </div>
                                </div>
                            }

                            // Metadata
                            <div class="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                    <span class="text-gray-500">{ "Execution ID:" }</span>
                                    <span class="ml-2 font-mono text-gray-900 dark:text-white">{ &exec.id }</span>
                                </div>
                                <div>
                                    <span class="text-gray-500">{ "Started At:" }</span>
                                    <span class="ml-2 text-gray-900 dark:text-white">{ &exec.started_at }</span>
                                </div>
                            </div>
                        </div>
                    </td>
                </tr>
            }
        </>
    }
}

/// Format ISO timestamp to relative or readable time
fn format_timestamp(timestamp: &str) -> String {
    // Simple formatting - just show date and time part
    if timestamp.len() > 19 {
        // Format: YYYY-MM-DDTHH:MM:SS -> HH:MM:SS
        timestamp[11..19].to_string()
    } else if timestamp.len() > 10 {
        timestamp[..10].to_string()
    } else {
        timestamp.to_string()
    }
}
