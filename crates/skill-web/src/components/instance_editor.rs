//! Instance configuration editor components
//!
//! Provides a visual editor for creating and managing skill instance configurations
//! with environment variable preview and capabilities management.

use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

// ============================================================================
// Types
// ============================================================================

/// Instance data for editing
#[derive(Clone, Default, PartialEq)]
pub struct InstanceData {
    pub name: String,
    pub description: String,
    pub config: HashMap<String, String>,
    pub is_default: bool,
    pub capabilities: Capabilities,
}

/// Capability settings for an instance
#[derive(Clone, Default, PartialEq)]
pub struct Capabilities {
    pub network_access: bool,
    pub filesystem_access: bool,
    pub env_access: bool,
    pub network_allowlist: Vec<String>,
    pub filesystem_paths: Vec<String>,
    pub env_vars: Vec<String>,
}

// ============================================================================
// InstanceEditor - Main Component
// ============================================================================

#[derive(Properties, PartialEq)]
pub struct InstanceEditorProps {
    /// Skill name this instance belongs to
    pub skill: String,
    /// Existing instance data (None for creating new)
    #[prop_or_default]
    pub instance: Option<InstanceData>,
    /// Callback when save is clicked
    pub on_save: Callback<InstanceData>,
    /// Callback when cancel is clicked
    pub on_cancel: Callback<()>,
}

/// Instance configuration editor component
#[function_component(InstanceEditor)]
pub fn instance_editor(props: &InstanceEditorProps) -> Html {
    // Initialize state from existing instance or defaults
    let name = use_state(|| {
        props
            .instance
            .as_ref()
            .map(|i| i.name.clone())
            .unwrap_or_default()
    });
    let description = use_state(|| {
        props
            .instance
            .as_ref()
            .map(|i| i.description.clone())
            .unwrap_or_default()
    });
    let config = use_state(|| {
        props
            .instance
            .as_ref()
            .map(|i| i.config.clone())
            .unwrap_or_default()
    });
    let is_default = use_state(|| {
        props
            .instance
            .as_ref()
            .map(|i| i.is_default)
            .unwrap_or(false)
    });
    let capabilities = use_state(|| {
        props
            .instance
            .as_ref()
            .map(|i| i.capabilities.clone())
            .unwrap_or_default()
    });

    // Validation state
    let validation_errors = use_state(HashMap::<String, String>::new);
    let is_testing = use_state(|| false);
    let test_result = use_state(|| None::<Result<String, String>>);

    // Callbacks
    let on_name_change = {
        let name = name.clone();
        let validation_errors = validation_errors.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            let value = input.value();
            name.set(value.clone());

            // Validate name
            let mut errors = (*validation_errors).clone();
            if value.is_empty() {
                errors.insert("name".to_string(), "Instance name is required".to_string());
            } else if !value.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                errors.insert(
                    "name".to_string(),
                    "Name can only contain letters, numbers, hyphens, and underscores".to_string(),
                );
            } else {
                errors.remove("name");
            }
            validation_errors.set(errors);
        })
    };

    let on_description_change = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            description.set(input.value());
        })
    };

    let on_config_change = {
        let config = config.clone();
        Callback::from(move |new_config: HashMap<String, String>| {
            config.set(new_config);
        })
    };

    let on_capabilities_change = {
        let capabilities = capabilities.clone();
        Callback::from(move |new_caps: Capabilities| {
            capabilities.set(new_caps);
        })
    };

    let on_default_change = {
        let is_default = is_default.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            is_default.set(input.checked());
        })
    };

    let on_test = {
        let is_testing = is_testing.clone();
        let test_result = test_result.clone();
        let config = config.clone();
        Callback::from(move |_| {
            is_testing.set(true);
            test_result.set(None);

            // Simulate test - in real implementation would call API
            let config = (*config).clone();
            let is_testing = is_testing.clone();
            let test_result = test_result.clone();

            // Check for unresolved env vars
            let unresolved_count = config
                .values()
                .filter(|v| v.contains("${") && v.contains("}"))
                .count();

            // Simulate async test completion
            gloo_timers::callback::Timeout::new(500, move || {
                is_testing.set(false);
                if unresolved_count == 0 {
                    test_result.set(Some(Ok("Configuration is valid".to_string())));
                } else {
                    test_result.set(Some(Err(format!(
                        "Unresolved environment variables in {} config value(s)",
                        unresolved_count
                    ))));
                }
            })
            .forget();
        })
    };

    let on_save = {
        let on_save = props.on_save.clone();
        let name = name.clone();
        let description = description.clone();
        let config = config.clone();
        let is_default = is_default.clone();
        let capabilities = capabilities.clone();
        let validation_errors = validation_errors.clone();
        Callback::from(move |_| {
            // Validate before saving
            let mut errors = HashMap::new();
            if (*name).is_empty() {
                errors.insert("name".to_string(), "Instance name is required".to_string());
            }

            if !errors.is_empty() {
                validation_errors.set(errors);
                return;
            }

            let data = InstanceData {
                name: (*name).clone(),
                description: (*description).clone(),
                config: (*config).clone(),
                is_default: *is_default,
                capabilities: (*capabilities).clone(),
            };
            on_save.emit(data);
        })
    };

    let on_cancel = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_| on_cancel.emit(()))
    };

    let is_editing = props.instance.is_some();
    let title = if is_editing {
        "Edit Instance"
    } else {
        "Create Instance"
    };

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg max-w-2xl w-full max-h-[90vh] overflow-hidden flex flex-col">
            // Header
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                    { title }
                </h2>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    { format!("Configure instance for {}", props.skill) }
                </p>
            </div>

            // Body - scrollable
            <div class="flex-1 overflow-y-auto p-6 space-y-6">
                // Instance Name
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        { "Instance Name" }
                        <span class="text-red-500 ml-1">{ "*" }</span>
                    </label>
                    <input
                        type="text"
                        value={(*name).clone()}
                        oninput={on_name_change}
                        placeholder="e.g., production, staging, dev"
                        disabled={is_editing}
                        class={classes!(
                            "w-full", "px-3", "py-2", "rounded-md", "border",
                            "bg-white", "dark:bg-gray-900",
                            "text-gray-900", "dark:text-white",
                            "focus:ring-2", "focus:ring-primary-500", "focus:border-primary-500",
                            if validation_errors.contains_key("name") {
                                "border-red-500"
                            } else {
                                "border-gray-300 dark:border-gray-600"
                            },
                            if is_editing { "bg-gray-100 dark:bg-gray-800 cursor-not-allowed" } else { "" }
                        )}
                    />
                    if let Some(error) = validation_errors.get("name") {
                        <p class="mt-1 text-sm text-red-500">{ error }</p>
                    }
                </div>

                // Description
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        { "Description" }
                    </label>
                    <input
                        type="text"
                        value={(*description).clone()}
                        oninput={on_description_change}
                        placeholder="Optional description for this instance"
                        class="w-full px-3 py-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    />
                </div>

                // Default checkbox
                <div class="flex items-center gap-2">
                    <input
                        type="checkbox"
                        checked={*is_default}
                        onchange={on_default_change}
                        class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                    />
                    <label class="text-sm text-gray-700 dark:text-gray-300">
                        { "Set as default instance" }
                    </label>
                </div>

                // Configuration key-value pairs
                <ConfigKeyValueEditor
                    pairs={(*config).clone()}
                    on_change={on_config_change}
                />

                // Environment variable preview
                <EnvironmentVariablePreview pairs={(*config).clone()} />

                // Capabilities editor
                <CapabilitiesEditor
                    capabilities={(*capabilities).clone()}
                    on_change={on_capabilities_change}
                />

                // Test result
                if let Some(result) = &*test_result {
                    <div class={classes!(
                        "p-3", "rounded-md", "text-sm",
                        match result {
                            Ok(_) => "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800",
                            Err(_) => "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-800",
                        }
                    )}>
                        { match result {
                            Ok(msg) => html! { <><span class="font-medium">{ "✓ " }</span>{ msg }</> },
                            Err(msg) => html! { <><span class="font-medium">{ "✗ " }</span>{ msg }</> },
                        }}
                    </div>
                }
            </div>

            // Footer
            <div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 flex items-center justify-between">
                <button
                    onclick={on_cancel}
                    class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-md transition-colors"
                >
                    { "Cancel" }
                </button>
                <div class="flex gap-2">
                    <button
                        onclick={on_test}
                        disabled={*is_testing}
                        class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-md transition-colors disabled:opacity-50"
                    >
                        if *is_testing {
                            { "Testing..." }
                        } else {
                            { "Test Configuration" }
                        }
                    </button>
                    <button
                        onclick={on_save}
                        class="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors"
                    >
                        { if is_editing { "Save Changes" } else { "Create Instance" } }
                    </button>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// ConfigKeyValueEditor - Key-value pairs editor
// ============================================================================

#[derive(Properties, PartialEq)]
pub struct ConfigKeyValueEditorProps {
    pub pairs: HashMap<String, String>,
    pub on_change: Callback<HashMap<String, String>>,
}

#[function_component(ConfigKeyValueEditor)]
pub fn config_key_value_editor(props: &ConfigKeyValueEditorProps) -> Html {
    // Convert to vec for easier manipulation
    let pairs_vec: Vec<(String, String)> = props.pairs.clone().into_iter().collect();

    let add_pair = {
        let on_change = props.on_change.clone();
        let pairs = props.pairs.clone();
        Callback::from(move |_| {
            let mut new_pairs = pairs.clone();
            // Find a unique key
            let mut key_num = 1;
            let mut new_key = format!("KEY_{}", key_num);
            while new_pairs.contains_key(&new_key) {
                key_num += 1;
                new_key = format!("KEY_{}", key_num);
            }
            new_pairs.insert(new_key, String::new());
            on_change.emit(new_pairs);
        })
    };

    html! {
        <div class="space-y-3">
            <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    { "Configuration" }
                </label>
                <button
                    onclick={add_pair}
                    class="text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 font-medium"
                >
                    { "+ Add Key" }
                </button>
            </div>

            if pairs_vec.is_empty() {
                <div class="text-sm text-gray-500 dark:text-gray-400 italic py-4 text-center border border-dashed border-gray-300 dark:border-gray-600 rounded-md">
                    { "No configuration values. Click \"+ Add Key\" to add one." }
                </div>
            } else {
                <div class="space-y-2">
                    // Header row
                    <div class="grid grid-cols-12 gap-2 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                        <div class="col-span-4">{ "Key" }</div>
                        <div class="col-span-7">{ "Value" }</div>
                        <div class="col-span-1"></div>
                    </div>

                    { for pairs_vec.iter().map(|(key, value)| {
                        let key = key.clone();
                        let value = value.clone();
                        let on_change = props.on_change.clone();
                        let pairs = props.pairs.clone();

                        // Clone key for each callback that needs it
                        let key_for_row = key.clone();
                        let key_for_key_change = key.clone();
                        let key_for_value_change = key.clone();
                        let key_for_delete = key.clone();

                        html! {
                            <ConfigKeyValueRow
                                key_name={key_for_row}
                                value={value}
                                on_key_change={Callback::from({
                                    let on_change = on_change.clone();
                                    let pairs = pairs.clone();
                                    let old_key = key_for_key_change;
                                    move |new_key: String| {
                                        let mut new_pairs = pairs.clone();
                                        if let Some(val) = new_pairs.remove(&old_key) {
                                            new_pairs.insert(new_key, val);
                                        }
                                        on_change.emit(new_pairs);
                                    }
                                })}
                                on_value_change={Callback::from({
                                    let on_change = on_change.clone();
                                    let pairs = pairs.clone();
                                    let key = key_for_value_change;
                                    move |new_value: String| {
                                        let mut new_pairs = pairs.clone();
                                        new_pairs.insert(key.clone(), new_value);
                                        on_change.emit(new_pairs);
                                    }
                                })}
                                on_delete={Callback::from({
                                    let on_change = on_change.clone();
                                    let pairs = pairs.clone();
                                    let key = key_for_delete;
                                    move |_| {
                                        let mut new_pairs = pairs.clone();
                                        new_pairs.remove(&key);
                                        on_change.emit(new_pairs);
                                    }
                                })}
                            />
                        }
                    }) }
                </div>
            }

            <p class="text-xs text-gray-500 dark:text-gray-400">
                { "Use " }
                <code class="bg-gray-100 dark:bg-gray-700 px-1 rounded">{ "${VAR_NAME}" }</code>
                { " to reference environment variables." }
            </p>
        </div>
    }
}

// ============================================================================
// ConfigKeyValueRow - Single key-value row
// ============================================================================

#[derive(Properties, PartialEq)]
struct ConfigKeyValueRowProps {
    key_name: String,
    value: String,
    on_key_change: Callback<String>,
    on_value_change: Callback<String>,
    on_delete: Callback<()>,
}

#[function_component(ConfigKeyValueRow)]
fn config_key_value_row(props: &ConfigKeyValueRowProps) -> Html {
    let on_key_input = {
        let on_key_change = props.on_key_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            on_key_change.emit(input.value());
        })
    };

    let on_value_input = {
        let on_value_change = props.on_value_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            on_value_change.emit(input.value());
        })
    };

    let on_delete = {
        let on_delete = props.on_delete.clone();
        Callback::from(move |_| on_delete.emit(()))
    };

    // Check if value contains env var reference
    let has_env_ref = props.value.contains("${") && props.value.contains("}");

    html! {
        <div class="grid grid-cols-12 gap-2 items-center">
            <div class="col-span-4">
                <input
                    type="text"
                    value={props.key_name.clone()}
                    oninput={on_key_input}
                    placeholder="KEY"
                    class="w-full px-2 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white font-mono focus:ring-1 focus:ring-primary-500 focus:border-primary-500"
                />
            </div>
            <div class="col-span-7">
                <div class="relative">
                    <input
                        type="text"
                        value={props.value.clone()}
                        oninput={on_value_input}
                        placeholder="value or ${ENV_VAR}"
                        class={classes!(
                            "w-full", "px-2", "py-1.5", "text-sm", "rounded", "border",
                            "bg-white", "dark:bg-gray-900", "text-gray-900", "dark:text-white",
                            "focus:ring-1", "focus:ring-primary-500", "focus:border-primary-500",
                            if has_env_ref {
                                "border-amber-400 dark:border-amber-500 pr-8"
                            } else {
                                "border-gray-300 dark:border-gray-600"
                            }
                        )}
                    />
                    if has_env_ref {
                        <span class="absolute right-2 top-1/2 -translate-y-1/2 text-amber-500" title="Contains environment variable reference">
                            { "$" }
                        </span>
                    }
                </div>
            </div>
            <div class="col-span-1 flex justify-center">
                <button
                    onclick={on_delete}
                    class="p-1 text-gray-400 hover:text-red-500 transition-colors"
                    title="Delete"
                >
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
        </div>
    }
}

// ============================================================================
// EnvironmentVariablePreview - Shows resolved env vars
// ============================================================================

#[derive(Properties, PartialEq)]
pub struct EnvironmentVariablePreviewProps {
    pub pairs: HashMap<String, String>,
}

#[function_component(EnvironmentVariablePreview)]
pub fn environment_variable_preview(props: &EnvironmentVariablePreviewProps) -> Html {
    // Extract environment variable references
    let env_refs: Vec<(String, String, Option<String>)> = props
        .pairs
        .iter()
        .filter_map(|(key, value)| {
            // Find ${VAR} patterns
            let mut refs = Vec::new();
            let mut remaining = value.as_str();
            while let Some(start) = remaining.find("${") {
                if let Some(end) = remaining[start..].find('}') {
                    let var_name = &remaining[start + 2..start + end];
                    refs.push(var_name.to_string());
                    remaining = &remaining[start + end + 1..];
                } else {
                    break;
                }
            }
            if refs.is_empty() {
                None
            } else {
                Some((key.clone(), refs.join(", "), None)) // None = not resolved in browser
            }
        })
        .collect();

    if env_refs.is_empty() {
        return html! {};
    }

    html! {
        <div class="border border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-900/20 rounded-md p-4">
            <div class="flex items-start gap-2">
                <svg class="w-5 h-5 text-amber-500 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <div class="flex-1">
                    <h4 class="text-sm font-medium text-amber-800 dark:text-amber-200">
                        { "Environment Variable References" }
                    </h4>
                    <p class="text-xs text-amber-700 dark:text-amber-300 mt-1">
                        { "These will be resolved at runtime on the server." }
                    </p>
                    <div class="mt-3 space-y-1">
                        { for env_refs.iter().map(|(key, vars, _resolved)| {
                            html! {
                                <div class="flex items-center gap-2 text-sm">
                                    <code class="text-amber-700 dark:text-amber-300 font-mono">{ key }</code>
                                    <span class="text-amber-600 dark:text-amber-400">{ "→" }</span>
                                    <code class="text-amber-800 dark:text-amber-200 font-mono">{ format!("${{{}}}", vars) }</code>
                                </div>
                            }
                        }) }
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// CapabilitiesEditor - Sandbox capabilities editor
// ============================================================================

#[derive(Properties, PartialEq)]
pub struct CapabilitiesEditorProps {
    pub capabilities: Capabilities,
    pub on_change: Callback<Capabilities>,
}

#[function_component(CapabilitiesEditor)]
pub fn capabilities_editor(props: &CapabilitiesEditorProps) -> Html {
    let on_network_change = {
        let on_change = props.on_change.clone();
        let caps = props.capabilities.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            let mut new_caps = caps.clone();
            new_caps.network_access = input.checked();
            on_change.emit(new_caps);
        })
    };

    let on_filesystem_change = {
        let on_change = props.on_change.clone();
        let caps = props.capabilities.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            let mut new_caps = caps.clone();
            new_caps.filesystem_access = input.checked();
            on_change.emit(new_caps);
        })
    };

    let on_env_change = {
        let on_change = props.on_change.clone();
        let caps = props.capabilities.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            let mut new_caps = caps.clone();
            new_caps.env_access = input.checked();
            on_change.emit(new_caps);
        })
    };

    let on_network_allowlist_change = {
        let on_change = props.on_change.clone();
        let caps = props.capabilities.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            let mut new_caps = caps.clone();
            new_caps.network_allowlist = input
                .value()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            on_change.emit(new_caps);
        })
    };

    html! {
        <div class="space-y-4">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
                { "Capabilities" }
            </label>

            <div class="space-y-3 pl-1">
                // Network Access
                <div class="space-y-2">
                    <label class="flex items-center gap-2 cursor-pointer">
                        <input
                            type="checkbox"
                            checked={props.capabilities.network_access}
                            onchange={on_network_change}
                            class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                        />
                        <span class="text-sm text-gray-700 dark:text-gray-300">
                            { "Network Access" }
                        </span>
                    </label>

                    if props.capabilities.network_access {
                        <div class="ml-6">
                            <label class="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                                { "Allowed hosts (comma-separated)" }
                            </label>
                            <input
                                type="text"
                                value={props.capabilities.network_allowlist.join(", ")}
                                oninput={on_network_allowlist_change}
                                placeholder="api.example.com, *.internal.net"
                                class="w-full px-2 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:ring-1 focus:ring-primary-500 focus:border-primary-500"
                            />
                        </div>
                    }
                </div>

                // Filesystem Access
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        checked={props.capabilities.filesystem_access}
                        onchange={on_filesystem_change}
                        class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                    />
                    <span class="text-sm text-gray-700 dark:text-gray-300">
                        { "Filesystem Access" }
                    </span>
                </label>

                // Environment Variables
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        checked={props.capabilities.env_access}
                        onchange={on_env_change}
                        class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                    />
                    <span class="text-sm text-gray-700 dark:text-gray-300">
                        { "Environment Variables Access" }
                    </span>
                </label>
            </div>

            <p class="text-xs text-gray-500 dark:text-gray-400">
                { "Capabilities control what resources the skill can access at runtime." }
            </p>
        </div>
    }
}

// ============================================================================
// Modal Wrapper - For showing the editor in a modal
// ============================================================================

#[derive(Properties, PartialEq)]
pub struct InstanceEditorModalProps {
    /// Whether the modal is open
    pub open: bool,
    /// Skill name
    pub skill: String,
    /// Existing instance to edit (None for new)
    #[prop_or_default]
    pub instance: Option<InstanceData>,
    /// Callback when saved
    pub on_save: Callback<InstanceData>,
    /// Callback when closed/cancelled
    pub on_close: Callback<()>,
}

#[function_component(InstanceEditorModal)]
pub fn instance_editor_modal(props: &InstanceEditorModalProps) -> Html {
    if !props.open {
        return html! {};
    }

    let on_backdrop_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    let on_content_click = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

    html! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
            onclick={on_backdrop_click}
        >
            <div onclick={on_content_click} class="animate-scale-in">
                <InstanceEditor
                    skill={props.skill.clone()}
                    instance={props.instance.clone()}
                    on_save={props.on_save.clone()}
                    on_cancel={props.on_close.clone()}
                />
            </div>
        </div>
    }
}
