//! Command Palette component - Spotlight/Alfred-style search interface
//!
//! Features:
//! - Giant centered search bar with fuzzy search
//! - Keyboard navigation (arrows, enter, escape)
//! - Real-time skill+tool filtering
//! - Quick action buttons for recent commands
//! - Dark terminal aesthetic

use yew::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent};
use wasm_bindgen::JsCast;
use crate::api::types::SkillDetail;

/// Suggestion item combining skill and tool
#[derive(Debug, Clone, PartialEq)]
pub struct SuggestionItem {
    pub skill: String,
    pub tool: String,
    pub description: String,
}

#[derive(Properties, PartialEq)]
pub struct CommandPaletteProps {
    /// All skill details with tools
    pub skill_details: Vec<SkillDetail>,
    /// Callback when a skill+tool is selected
    pub on_select: Callback<(String, String)>,
    /// Recent executions for quick actions
    #[prop_or_default]
    pub recent_commands: Vec<(String, String)>,
    /// Whether to show the palette
    #[prop_or(true)]
    pub visible: bool,
}

#[function_component(CommandPalette)]
pub fn command_palette(props: &CommandPaletteProps) -> Html {
    let search_query = use_state(String::new);
    let selected_index = use_state(|| 0_usize);
    let input_ref = use_node_ref();

    // Build flattened list of all skill+tool combinations
    let all_items: Vec<SuggestionItem> = props.skill_details
        .iter()
        .flat_map(|skill| {
            skill.tools.iter().map(move |tool| SuggestionItem {
                skill: skill.summary.name.clone(),
                tool: tool.name.clone(),
                description: tool.description.clone(),
            })
        })
        .collect();

    // Fuzzy search filtering
    let filtered_suggestions: Vec<SuggestionItem> = if search_query.is_empty() {
        vec![]
    } else {
        let query_lower = search_query.to_lowercase();
        let mut results: Vec<(SuggestionItem, f32)> = all_items
            .iter()
            .filter_map(|item| {
                // Simple fuzzy matching - check if query is substring or chars appear in order
                let skill_tool = format!("{} {}", item.skill, item.tool).to_lowercase();

                // First check: simple substring match (highest score)
                if skill_tool.contains(&query_lower) {
                    return Some((item.clone(), 100.0));
                }

                // Second check: fuzzy char-by-char matching
                let mut query_chars = query_lower.chars();
                let mut current_char = query_chars.next()?;
                let mut score = 0.0_f32;

                for (idx, c) in skill_tool.chars().enumerate() {
                    if c == current_char {
                        score += 1.0 / (idx as f32 + 1.0); // Earlier matches score higher
                        match query_chars.next() {
                            Some(next) => current_char = next,
                            None => return Some((item.clone(), score)),
                        }
                    }
                }

                None // Not all query chars matched
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(item, _)| item).take(8).collect()
    };

    // Handle keyboard navigation
    let on_keydown = {
        let selected_index = selected_index.clone();
        let search_query = search_query.clone();
        let filtered_suggestions = filtered_suggestions.clone();
        let on_select = props.on_select.clone();

        Callback::from(move |e: KeyboardEvent| {
            match e.key().as_str() {
                "ArrowDown" => {
                    e.prevent_default();
                    let max_idx = filtered_suggestions.len().saturating_sub(1);
                    selected_index.set((*selected_index + 1).min(max_idx));
                }
                "ArrowUp" => {
                    e.prevent_default();
                    selected_index.set(selected_index.saturating_sub(1));
                }
                "Enter" => {
                    e.prevent_default();
                    if let Some(item) = filtered_suggestions.get(*selected_index) {
                        on_select.emit((item.skill.clone(), item.tool.clone()));
                    }
                }
                "Escape" => {
                    e.prevent_default();
                    search_query.set(String::new());
                    selected_index.set(0);
                }
                _ => {}
            }
        })
    };

    // Handle input changes
    let on_input = {
        let search_query = search_query.clone();
        let selected_index = selected_index.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            search_query.set(input.value());
            selected_index.set(0); // Reset selection when query changes
        })
    };

    // Handle suggestion click
    let make_on_suggestion_click = {
        let on_select = props.on_select.clone();
        move |skill: String, tool: String| {
            let on_select = on_select.clone();
            Callback::from(move |_: MouseEvent| {
                on_select.emit((skill.clone(), tool.clone()));
            })
        }
    };

    // Handle quick action click
    let make_on_quick_action_click = {
        let on_select = props.on_select.clone();
        move |skill: String, tool: String| {
            let on_select = on_select.clone();
            Callback::from(move |_: MouseEvent| {
                on_select.emit((skill.clone(), tool.clone()));
            })
        }
    };

    // Focus input on mount
    use_effect_with(input_ref.clone(), |input_ref| {
        if let Some(input) = input_ref.cast::<HtmlInputElement>() {
            let _ = input.focus();
        }
        || ()
    });

    if !props.visible {
        return html! {};
    }

    html! {
        <div class="command-palette flex flex-col items-center gap-4">
            // Giant search input
            <div class="w-full max-w-2xl">
                <input
                    ref={input_ref}
                    type="text"
                    class="search-input neon-focus w-full"
                    placeholder="🔍 Type skill or tool name..."
                    value={(*search_query).clone()}
                    oninput={on_input}
                    onkeydown={on_keydown}
                    autofocus=true
                />
            </div>

            // Suggestions list (only show if there's a query and results)
            if !search_query.is_empty() && !filtered_suggestions.is_empty() {
                <div class="suggestions-list glass-panel w-full max-w-2xl">
                    { for filtered_suggestions.iter().enumerate().map(|(idx, item)| {
                        let is_selected = idx == *selected_index;
                        let skill = item.skill.clone();
                        let tool = item.tool.clone();

                        html! {
                            <button
                                class={classes!(
                                    "suggestion-item",
                                    "w-full",
                                    is_selected.then_some("selected")
                                )}
                                onclick={make_on_suggestion_click(skill, tool)}
                            >
                                <span class="skill-name text-terminal-accent-cyan font-semibold">
                                    { &item.skill }
                                </span>
                                <span class="separator text-terminal-text-secondary mx-2">
                                    { "/" }
                                </span>
                                <span class="tool-name text-terminal-text-primary">
                                    { &item.tool }
                                </span>
                                <span class="flex-1"></span>
                                <span class="text-xs text-terminal-text-secondary truncate max-w-xs">
                                    { &item.description }
                                </span>
                            </button>
                        }
                    }) }
                </div>
            }

            // Show hint when no results
            if !search_query.is_empty() && filtered_suggestions.is_empty() {
                <div class="w-full max-w-2xl text-center py-8">
                    <p class="text-terminal-text-secondary text-sm">
                        { "No matching skills or tools found" }
                    </p>
                </div>
            }

            // Quick actions (recent commands) - only show when search is empty
            if !props.recent_commands.is_empty() && search_query.is_empty() {
                <div class="quick-actions w-full max-w-2xl mt-4">
                    <div class="flex items-center gap-2 mb-3">
                        <span class="text-sm text-terminal-text-secondary font-medium">
                            { "Recent:" }
                        </span>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        { for props.recent_commands.iter().take(6).map(|(skill, tool)| {
                            let skill_clone = skill.clone();
                            let tool_clone = tool.clone();

                            html! {
                                <button
                                    class="quick-action-btn"
                                    onclick={make_on_quick_action_click(skill_clone, tool_clone)}
                                >
                                    <span class="text-terminal-accent-cyan">{ skill }</span>
                                    <span class="text-terminal-text-secondary mx-1">{ "/" }</span>
                                    <span>{ tool }</span>
                                </button>
                            }
                        }) }
                    </div>
                </div>
            }

            // Keyboard hints
            <div class="flex items-center gap-6 text-xs text-terminal-text-secondary mt-4">
                <span>{ "↑↓ Navigate" }</span>
                <span>{ "↵ Select" }</span>
                <span>{ "Esc Clear" }</span>
            </div>
        </div>
    }
}
