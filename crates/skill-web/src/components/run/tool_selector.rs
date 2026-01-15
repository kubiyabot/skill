//! Tool selector component - Step 2 of the execution workflow

use yew::prelude::*;
use crate::api::ToolInfo;

#[derive(Properties, PartialEq)]
pub struct ToolSelectorProps {
    pub tools: Vec<ToolInfo>,
    pub selected: Option<String>,
    pub loading: bool,
    pub enabled: bool,
    pub on_select: Callback<String>,
}

#[function_component(ToolSelector)]
pub fn tool_selector(props: &ToolSelectorProps) -> Html {
    let max_visible = 8;

    html! {
        <div class="space-y-4">
            // Step header
            <div class="flex items-center gap-2 mb-2">
                <div class={format!(
                    "flex items-center justify-center w-8 h-8 rounded-full font-semibold text-sm {}",
                    if props.enabled {
                        "bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400"
                    } else {
                        "bg-gray-100 dark:bg-gray-800 text-gray-400"
                    }
                )}>
                    { "2" }
                </div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                    { "Select Tool" }
                </h3>
            </div>

            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-sm">
                <div class="p-4 space-y-3">
                    if !props.enabled {
                        <div class="text-center py-12 text-gray-400 dark:text-gray-500">
                            <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 11.5V14m0-2.5v-6a1.5 1.5 0 113 0m-3 6a1.5 1.5 0 00-3 0v2a7.5 7.5 0 0015 0v-5a1.5 1.5 0 00-3 0m-6-3V11m0-5.5v-1a1.5 1.5 0 013 0v1m0 0V11m0-5.5a1.5 1.5 0 013 0v3m0 0V11" />
                            </svg>
                            <p class="text-sm font-medium">{ "Select a skill first" }</p>
                            <p class="text-xs mt-1">{ "Choose a skill from the left to see available tools" }</p>
                        </div>
                    } else if props.loading {
                        // Loading skeleton
                        { for (0..4).map(|_| html! {
                            <div class="animate-pulse">
                                <div class="h-14 bg-gray-200 dark:bg-gray-700 rounded-lg"></div>
                            </div>
                        })}
                    } else if props.tools.is_empty() {
                        <div class="text-center py-8 text-gray-500">
                            <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                            </svg>
                            <p class="text-sm">{ "No tools available" }</p>
                        </div>
                    } else {
                        // Tool cards
                        { for props.tools.iter().take(max_visible).map(|tool| {
                            let is_selected = props.selected.as_ref().map(|s| s == &tool.name).unwrap_or(false);
                            let tool_name = tool.name.clone();
                            let on_click = {
                                let on_select = props.on_select.clone();
                                Callback::from(move |_| {
                                    on_select.emit(tool_name.clone());
                                })
                            };

                            html! {
                                <button
                                    onclick={on_click}
                                    class={format!(
                                        "w-full text-left p-3 rounded-lg border-2 transition-all hover:border-primary-300 dark:hover:border-primary-700 {}",
                                        if is_selected {
                                            "border-primary-500 bg-primary-50 dark:bg-primary-900/20 dark:border-primary-500"
                                        } else {
                                            "border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/30"
                                        }
                                    )}
                                >
                                    <div class="flex items-start gap-3">
                                        // Radio button
                                        <div class={format!(
                                            "w-5 h-5 rounded-full border-2 flex items-center justify-center flex-shrink-0 mt-0.5 transition-colors {}",
                                            if is_selected {
                                                "border-primary-500 bg-primary-500"
                                            } else {
                                                "border-gray-300 dark:border-gray-600"
                                            }
                                        )}>
                                            if is_selected {
                                                <div class="w-2.5 h-2.5 bg-white rounded-full"></div>
                                            }
                                        </div>

                                        // Tool info
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center justify-between gap-2">
                                                <span class="font-medium text-gray-900 dark:text-white">
                                                    { &tool.name }
                                                </span>
                                                if !tool.parameters.is_empty() {
                                                    <span class="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1">
                                                        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                                                        </svg>
                                                        { format!("{} params", tool.parameters.len()) }
                                                    </span>
                                                }
                                            </div>
                                            if !tool.description.is_empty() {
                                                <p class="text-xs text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">
                                                    { &tool.description }
                                                </p>
                                            }
                                        </div>
                                    </div>
                                </button>
                            }
                        })}

                        if props.tools.len() > max_visible {
                            <button class="w-full text-center py-2 text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 hover:bg-primary-50 dark:hover:bg-primary-900/10 rounded-lg transition-colors">
                                { format!("Show {} more tools...", props.tools.len() - max_visible) }
                            </button>
                        }
                    }
                </div>
            </div>
        </div>
    }
}
