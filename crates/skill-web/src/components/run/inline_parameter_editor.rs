//! Inline Parameter Editor - Compact parameter form that appears below search
//!
//! Features:
//! - Inline single-line inputs with validation
//! - Tab navigation between fields
//! - Real-time validation (green checkmark / red X)
//! - JSON editor for complex parameters
//! - Auto-focus first required field

use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlTextAreaElement, KeyboardEvent};
use wasm_bindgen::JsCast;
use std::collections::HashMap;
use crate::api::types::ParameterInfo;

#[derive(Properties, PartialEq)]
pub struct InlineParameterEditorProps {
    /// Parameter definitions from the tool
    pub parameters: Vec<ParameterInfo>,
    /// Current parameter values
    pub values: HashMap<String, serde_json::Value>,
    /// Callback when a parameter value changes
    pub on_change: Callback<(String, serde_json::Value)>,
    /// Validation errors for parameters
    #[prop_or_default]
    pub errors: HashMap<String, String>,
}

#[function_component(InlineParameterEditor)]
pub fn inline_parameter_editor(props: &InlineParameterEditorProps) -> Html {
    let first_input_ref = use_node_ref();
    let should_focus_first = use_state(|| true);

    // Focus first required field on mount
    use_effect_with((first_input_ref.clone(), should_focus_first.clone()), |(input_ref, should_focus)| {
        if **should_focus {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
                should_focus.set(false);
            }
        }
        || ()
    });

    // Handle Tab key for navigation (allow default browser behavior)
    let make_on_keydown = {
        |_param_name: String| {
            Callback::from(move |e: KeyboardEvent| {
                // Allow Tab for natural navigation, Enter submits the form
                if e.key() == "Enter" {
                    e.prevent_default(); // Prevent form submission, let parent handle execution
                }
            })
        }
    };

    // Handle input change for text/number parameters
    let make_on_input = {
        let on_change = props.on_change.clone();
        move |param_name: String, param_type: String| {
            let on_change = on_change.clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                let value = input.value();

                // Parse value based on type
                let json_value = match param_type.as_str() {
                    "number" | "integer" => {
                        value.parse::<i64>()
                            .map(|n| serde_json::json!(n))
                            .unwrap_or_else(|_| serde_json::json!(value))
                    }
                    "boolean" => {
                        let bool_val = value.to_lowercase() == "true" || value == "1";
                        serde_json::json!(bool_val)
                    }
                    "array" | "object" => {
                        // Try to parse as JSON
                        serde_json::from_str(&value)
                            .unwrap_or_else(|_| serde_json::json!(value))
                    }
                    _ => serde_json::json!(value),
                };

                on_change.emit((param_name.clone(), json_value));
            })
        }
    };

    // Handle textarea change for complex parameters
    let make_on_textarea_change = {
        let on_change = props.on_change.clone();
        move |param_name: String| {
            let on_change = on_change.clone();
            Callback::from(move |e: InputEvent| {
                let textarea: HtmlTextAreaElement = e.target().unwrap().dyn_into().unwrap();
                let value = textarea.value();

                // Try to parse as JSON for validation
                let json_value = serde_json::from_str(&value)
                    .unwrap_or_else(|_| serde_json::json!(value));

                on_change.emit((param_name.clone(), json_value));
            })
        }
    };

    if props.parameters.is_empty() {
        return html! {
            <div class="text-center py-4">
                <span class="text-sm text-gray-500 dark:text-gray-400">
                    { "No parameters required" }
                </span>
            </div>
        };
    }

    // Group parameters by required/optional
    let (required_params, optional_params): (Vec<_>, Vec<_>) =
        props.parameters.iter().partition(|p| p.required);

    html! {
        <div class="space-y-6">
            // Required Parameters Section
            if !required_params.is_empty() {
                <div class="bg-primary-50 dark:bg-primary-900/20 p-4 rounded-lg space-y-4">
                    <h4 class="text-sm font-semibold text-gray-900 dark:text-white">
                        { "Required Parameters" }
                    </h4>
                    { for required_params.iter().enumerate().map(|(idx, param)| {
                        let is_first = idx == 0;
                        render_parameter(param, is_first, props, &first_input_ref, &make_on_input, &make_on_textarea_change, &make_on_keydown)
                    }) }
                </div>
            }

            // Optional Parameters Section
            if !optional_params.is_empty() {
                <div class="space-y-4">
                    <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        { "Optional Parameters" }
                    </h4>
                    { for optional_params.iter().map(|param| {
                        render_parameter(param, false, props, &first_input_ref, &make_on_input, &make_on_textarea_change, &make_on_keydown)
                    }) }
                </div>
            }

            // Helper text
            <div class="text-xs text-gray-500 dark:text-gray-400 pt-2 border-t border-gray-200 dark:border-gray-700">
                { "Press Tab to move between fields. Required fields marked with " }
                <span class="text-error-500">{ "*" }</span>
            </div>
        </div>
    }
}

// Helper function to render a single parameter
fn render_parameter(
    param: &ParameterInfo,
    is_first: bool,
    props: &InlineParameterEditorProps,
    first_input_ref: &NodeRef,
    make_on_input: &impl Fn(String, String) -> Callback<InputEvent>,
    make_on_textarea_change: &impl Fn(String) -> Callback<InputEvent>,
    make_on_keydown: &impl Fn(String) -> Callback<KeyboardEvent>,
) -> Html {
    let param_name = param.name.clone();
    let param_type = param.param_type.clone();
    let current_value = props.values.get(&param.name)
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_default();
    let has_error = props.errors.contains_key(&param.name);
    let error_msg = props.errors.get(&param.name).cloned();
    let is_valid = !param.required || !current_value.is_empty();
    let is_complex = matches!(param_type.as_str(), "array" | "object");

    html! {
        <div class="space-y-2">
            // Parameter label with required indicator and type badge
            <label class="flex items-center gap-2">
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    { &param.name }
                </span>
                if param.required {
                    <span class="text-xs text-error-500">
                        { "*" }
                    </span>
                }
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300">
                    { &param.param_type }
                </span>
            </label>

            // Parameter description
            if !param.description.is_empty() {
                <p class="text-xs text-gray-600 dark:text-gray-400">
                    { &param.description }
                </p>
            }

            // Input field with validation indicator
            <div class="relative flex items-center gap-2">
                if is_complex {
                    // Textarea for arrays/objects
                    <textarea
                        class={classes!(
                            "input",
                            "w-full",
                            "font-mono",
                            "text-sm",
                            has_error.then_some("border-error-500 focus:border-error-500 focus:ring-error-500")
                        )}
                        placeholder={format!("Enter {} (JSON format)", param.param_type)}
                        value={current_value.clone()}
                        oninput={make_on_textarea_change(param_name.clone())}
                        rows="3"
                    />
                } else {
                    // Single-line input for simple types
                    if is_first {
                        <input
                            ref={first_input_ref.clone()}
                            type={match param_type.as_str() {
                                "number" | "integer" => "number",
                                "boolean" => "checkbox",
                                _ => "text"
                            }}
                            class={classes!(
                                "input",
                                "flex-1",
                                has_error.then_some("border-error-500 focus:border-error-500 focus:ring-error-500")
                            )}
                            placeholder={param.default_value.as_ref()
                                .map(|d| format!("Default: {}", d))
                                .unwrap_or_else(|| format!("Enter {}", param.name))}
                            value={current_value.clone()}
                            oninput={make_on_input(param_name.clone(), param_type.clone())}
                            onkeydown={make_on_keydown(param_name.clone())}
                        />
                    } else {
                        <input
                            type={match param_type.as_str() {
                                "number" | "integer" => "number",
                                "boolean" => "checkbox",
                                _ => "text"
                            }}
                            class={classes!(
                                if param_type == "boolean" { "checkbox checkbox-primary" } else { "input w-full" },
                                if param_type != "boolean" { Some("flex-1") } else { None },
                                has_error.then_some("border-error-500 focus:border-error-500 focus:ring-error-500")
                            )}
                            placeholder={if param_type == "boolean" {
                                String::new()
                            } else {
                                param.default_value.as_ref()
                                    .map(|d| format!("Default: {}", d))
                                    .unwrap_or_else(|| format!("Enter {}", param.name))
                            }}
                            checked={if param_type == "boolean" {
                                current_value == "true"
                            } else {
                                false
                            }}
                            value={if param_type == "boolean" {
                                "true".to_string()
                            } else {
                                current_value.clone()
                            }}
                            onchange={if param_type == "boolean" {
                                let on_change = props.on_change.clone();
                                let param_name = param_name.clone();
                                Some(Callback::from(move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    on_change.emit((param_name.clone(), serde_json::json!(input.checked())));
                                }))
                            } else {
                                None
                            }}
                            oninput={if param_type != "boolean" {
                                Some(make_on_input(param_name.clone(), param_type.clone()))
                            } else {
                                None
                            }}
                            onkeydown={make_on_keydown(param_name.clone())}
                        />
                    }
                }

                // Validation indicator
                <div class="flex-shrink-0 w-6 h-6">
                    if has_error {
                        // Red X for error
                        <svg class="w-6 h-6 text-error-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    } else if is_valid && !current_value.is_empty() {
                        // Green checkmark for valid
                        <svg class="w-6 h-6 text-success-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                    }
                </div>
            </div>

            // Error message
            if let Some(error) = error_msg {
                <p class="text-xs text-error-500 mt-1">
                    { error }
                </p>
            }
        </div>
    }
}
