//! Terminal Output Panel - Slide-up terminal display for execution results
//!
//! Features:
//! - Slides up from bottom (60vh height)
//! - Dark terminal background with monospace font
//! - Syntax highlighting for JSON/YAML output
//! - Copy button, Re-run button, Close button
//! - Minimize to thin bar at bottom
//! - Auto-scroll to bottom for streaming output

use yew::prelude::*;
use crate::api::types::{ExecutionResponse, ExecutionStatus};

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    /// Whether the terminal is visible
    pub visible: bool,
    /// Execution result to display
    pub execution: Option<ExecutionResponse>,
    /// Callback to close the terminal
    pub on_close: Callback<()>,
    /// Callback to re-run the command
    #[prop_or_default]
    pub on_rerun: Option<Callback<()>>,
    /// Callback to copy output
    #[prop_or_default]
    pub on_copy: Option<Callback<String>>,
    /// Whether the terminal is minimized
    #[prop_or(false)]
    pub minimized: bool,
    /// Callback to toggle minimize
    pub on_toggle_minimize: Callback<()>,
}

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let terminal_ref = use_node_ref();

    // Auto-scroll to bottom when content changes
    use_effect_with((props.execution.clone(), terminal_ref.clone()), |(execution, terminal_ref)| {
        if execution.is_some() {
            if let Some(terminal) = terminal_ref.cast::<web_sys::HtmlElement>() {
                terminal.set_scroll_top(terminal.scroll_height());
            }
        }
        || ()
    });

    // Handle close
    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            on_close.emit(());
        })
    };

    // Handle minimize toggle
    let on_minimize_click = {
        let on_toggle = props.on_toggle_minimize.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            on_toggle.emit(());
        })
    };

    // Handle re-run
    let on_rerun_click = {
        let on_rerun = props.on_rerun.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(callback) = &on_rerun {
                callback.emit(());
            }
        })
    };

    // Handle copy
    let on_copy_click = {
        let on_copy = props.on_copy.clone();
        let output = props.execution.as_ref().map(|e| e.output.clone());
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(callback) = &on_copy {
                if let Some(output) = &output {
                    callback.emit(output.clone());
                }
            }
        })
    };

    // Format output with syntax highlighting (simple for now)
    let formatted_output = props.execution.as_ref().map(|exec| {
        // Try to parse as JSON for pretty printing
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&exec.output) {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| exec.output.clone())
        } else {
            exec.output.clone()
        }
    });

    // Get status color class
    let status_class = props.execution.as_ref().map(|exec| {
        match exec.status {
            ExecutionStatus::Success => "text-success-500",
            ExecutionStatus::Failed | ExecutionStatus::Timeout => "text-error-500",
            ExecutionStatus::Running => "text-primary-500",
            ExecutionStatus::Pending => "text-gray-500 dark:text-gray-400",
            ExecutionStatus::Cancelled => "text-warning-500",
        }
    });

    if !props.visible {
        return html! {};
    }

    html! {
        <div class={classes!(
            "fixed", "bottom-0", "left-0", "right-0",
            "bg-white", "dark:bg-gray-900",
            "border-t", "border-gray-200", "dark:border-gray-700",
            "shadow-lg",
            "z-50",
            "transition-all", "duration-200",
            if props.visible { "translate-y-0" } else { "translate-y-full" },
            if props.minimized { "h-14" } else { "h-[60vh]" }
        )}>
            // Header bar
            <div class="flex items-center justify-between px-6 py-3 bg-gray-100 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                <div class="flex items-center gap-4">
                    // Status indicator
                    if let Some(exec) = &props.execution {
                        <div class="flex items-center gap-2">
                            <span class={classes!("text-sm", "font-semibold", status_class)}>
                                { format!("{:?}", exec.status) }
                            </span>
                            <span class="text-xs text-gray-500 dark:text-gray-400">
                                { format!("({}ms)", exec.duration_ms) }
                            </span>
                        </div>
                    } else {
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                            { "Waiting for execution..." }
                        </span>
                    }
                </div>

                // Action buttons
                <div class="flex items-center gap-2">
                    // Copy button
                    if props.on_copy.is_some() && props.execution.is_some() {
                        <button
                            onclick={on_copy_click}
                            class="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-colors"
                            title="Copy output"
                        >
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                            </svg>
                        </button>
                    }

                    // Re-run button
                    if props.on_rerun.is_some() {
                        <button
                            onclick={on_rerun_click}
                            class="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-colors"
                            title="Re-run command"
                        >
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                            </svg>
                        </button>
                    }

                    // Minimize/Maximize button
                    <button
                        onclick={on_minimize_click}
                        class="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 transition-colors"
                        title={if props.minimized { "Maximize" } else { "Minimize" }}
                    >
                        if props.minimized {
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
                            </svg>
                        } else {
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                            </svg>
                        }
                    </button>

                    // Close button
                    <button
                        onclick={on_close_click}
                        class="p-2 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-error-500 transition-colors"
                        title="Close"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>
            </div>

            // Terminal content (hidden when minimized)
            if !props.minimized {
                <div
                    ref={terminal_ref}
                    class="p-6 overflow-y-auto h-[calc(100%-56px)] bg-gray-50 dark:bg-gray-900"
                >
                    if let Some(exec) = &props.execution {
                        // Command executed indicator
                        <div class="text-gray-600 dark:text-gray-400 mb-4 font-mono text-sm">
                            <span>{ "$ " }</span>
                            <span class="text-primary-500">{ &exec.id }</span>
                        </div>

                        // Output
                        <pre class={classes!(
                            "whitespace-pre-wrap",
                            "break-words",
                            "font-mono",
                            "text-sm",
                            "text-gray-900",
                            "dark:text-gray-100",
                            status_class
                        )}>
                            { formatted_output.as_ref().unwrap_or(&exec.output) }
                        </pre>

                        // Error message (if any)
                        if let Some(error) = &exec.error {
                            <div class="mt-4 p-4 bg-error-50 dark:bg-error-900/20 border-l-4 border-error-500 rounded">
                                <div class="text-error-500 font-semibold mb-2">
                                    { "Error:" }
                                </div>
                                <pre class="text-error-500 whitespace-pre-wrap font-mono text-sm">
                                    { error }
                                </pre>
                            </div>
                        }

                        // Success indicator
                        if exec.status == ExecutionStatus::Success {
                            <div class="mt-4 text-success-500 font-mono text-sm">
                                { format!("✓ Success ({}ms)", exec.duration_ms) }
                            </div>
                        }
                    } else {
                        // Waiting state
                        <div class="flex items-center gap-2 text-gray-600 dark:text-gray-400 font-mono text-sm">
                            <span>{ "Executing" }</span>
                            <span class="animate-pulse">{ "..." }</span>
                        </div>
                    }
                </div>
            }
        </div>
    }
}
