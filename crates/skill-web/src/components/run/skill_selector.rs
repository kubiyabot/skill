//! Skill selector component - Step 1 of the execution workflow

use yew::prelude::*;
use crate::store::skills::SkillSummary;

#[derive(Properties, PartialEq)]
pub struct SkillSelectorProps {
    pub skills: Vec<SkillSummary>,
    pub selected: Option<String>,
    pub loading: bool,
    pub on_select: Callback<String>,
}

#[function_component(SkillSelector)]
pub fn skill_selector(props: &SkillSelectorProps) -> Html {
    let max_visible = 5;

    html! {
        <div class="space-y-4">
            // Step header
            <div class="flex items-center gap-2 mb-2">
                <div class="flex items-center justify-center w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 font-semibold text-sm">
                    { "1" }
                </div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                    { "Select Skill" }
                </h3>
            </div>

            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-sm">
                <div class="p-4 space-y-3">
                    if props.loading {
                        // Loading skeleton
                        { for (0..3).map(|_| html! {
                            <div class="animate-pulse">
                                <div class="h-16 bg-gray-200 dark:bg-gray-700 rounded-lg"></div>
                            </div>
                        })}
                    } else if props.skills.is_empty() {
                        <div class="text-center py-8 text-gray-500">
                            <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                            </svg>
                            <p class="text-sm">{ "No skills available" }</p>
                        </div>
                    } else {
                        // Skill cards
                        { for props.skills.iter().take(max_visible).map(|skill| {
                            let is_selected = props.selected.as_ref().map(|s| s == &skill.name).unwrap_or(false);
                            let skill_name = skill.name.clone();
                            let on_click = {
                                let on_select = props.on_select.clone();
                                Callback::from(move |_| {
                                    on_select.emit(skill_name.clone());
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

                                        // Skill info
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <span class="font-medium text-gray-900 dark:text-white">
                                                    { &skill.name }
                                                </span>
                                                <span class={format!(
                                                    "text-xs px-2 py-0.5 rounded-full font-medium {}",
                                                    match skill.runtime {
                                                        crate::store::skills::SkillRuntime::Native => "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300",
                                                        crate::store::skills::SkillRuntime::Docker => "bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300",
                                                        _ => "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300"
                                                    }
                                                )}>
                                                    { match skill.runtime {
                                                        crate::store::skills::SkillRuntime::Native => "native",
                                                        crate::store::skills::SkillRuntime::Docker => "docker",
                                                        crate::store::skills::SkillRuntime::Wasm => "wasm",
                                                    }}
                                                </span>
                                            </div>
                                            <div class="flex items-center gap-4 mt-1 text-xs text-gray-600 dark:text-gray-400">
                                                <span class="flex items-center gap-1">
                                                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                                                    </svg>
                                                    { format!("{} tools", skill.tools_count) }
                                                </span>
                                                if skill.execution_count > 0 {
                                                    <span class="flex items-center gap-1">
                                                        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                                        </svg>
                                                        { format!("{} runs", skill.execution_count) }
                                                    </span>
                                                }
                                            </div>
                                            if !skill.description.is_empty() {
                                                <p class="text-xs text-gray-500 dark:text-gray-500 mt-1.5 line-clamp-2">
                                                    { &skill.description }
                                                </p>
                                            }
                                        </div>
                                    </div>
                                </button>
                            }
                        })}

                        if props.skills.len() > max_visible {
                            <button class="w-full text-center py-2 text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 hover:bg-primary-50 dark:hover:bg-primary-900/10 rounded-lg transition-colors">
                                { format!("Show {} more skills...", props.skills.len() - max_visible) }
                            </button>
                        }
                    }
                </div>
            </div>
        </div>
    }
}
