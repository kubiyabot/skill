//! Install Skill Modal component
//!
//! Modal dialog for installing skills from various sources.

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::api::{Api, InstallSkillRequest};
use crate::components::use_notifications;
use crate::store::ui::{UiAction, UiStore};

/// Source type for skill installation
#[derive(Clone, PartialEq, Default)]
pub enum SourceType {
    #[default]
    Git,
    Url,
    Local,
    Registry,
}

impl SourceType {
    fn label(&self) -> &'static str {
        match self {
            SourceType::Git => "Git Repository",
            SourceType::Url => "URL",
            SourceType::Local => "Local Path",
            SourceType::Registry => "Registry",
        }
    }

    fn placeholder(&self) -> &'static str {
        match self {
            SourceType::Git => "github:user/repo or https://github.com/user/repo.git",
            SourceType::Url => "https://example.com/skill.tar.gz",
            SourceType::Local => "/path/to/skill or ./relative/path",
            SourceType::Registry => "skill-name@1.0.0",
        }
    }

    fn help_text(&self) -> &'static str {
        match self {
            SourceType::Git => "Enter a GitHub shorthand (github:user/repo) or full git URL. Optionally specify a ref with @tag or @branch.",
            SourceType::Url => "Enter a direct URL to a skill archive (.tar.gz or .zip).",
            SourceType::Local => "Enter a local filesystem path to the skill directory.",
            SourceType::Registry => "Enter the skill name from the registry. Optionally specify version with @version.",
        }
    }
}

/// Props for the InstallSkillModal component
#[derive(Properties, PartialEq)]
pub struct InstallSkillModalProps {
    /// Callback when installation is complete
    #[prop_or_default]
    pub on_installed: Callback<String>,
    /// Callback when modal is closed
    #[prop_or_default]
    pub on_close: Callback<()>,
}

/// Installation state
#[derive(Clone, PartialEq)]
enum InstallState {
    Idle,
    Installing,
    Success(String),
    Error(String),
}

/// Install Skill Modal component
#[function_component(InstallSkillModal)]
pub fn install_skill_modal(props: &InstallSkillModalProps) -> Html {
    let (ui_store, ui_dispatch) = use_store::<UiStore>();
    let notifications = use_notifications();

    // Form state
    let source_type = use_state(SourceType::default);
    let source_input = use_state(String::new);
    let git_ref = use_state(String::new);
    let instance_name = use_state(String::new);
    let force_reinstall = use_state(|| false);
    let install_state = use_state(|| InstallState::Idle);

    // API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // Check if modal should be shown
    let is_open = ui_store.modal.open
        && ui_store.modal.modal_type == Some(crate::store::ui::ModalType::InstallSkill);

    // Close handler
    let on_close = {
        let ui_dispatch = ui_dispatch.clone();
        let on_close_prop = props.on_close.clone();
        let install_state = install_state.clone();
        Callback::from(move |_: MouseEvent| {
            if *install_state != InstallState::Installing {
                ui_dispatch.apply(UiAction::CloseModal);
                on_close_prop.emit(());
            }
        })
    };

    // Backdrop click handler
    let on_backdrop_click = {
        let ui_dispatch = ui_dispatch.clone();
        let on_close_prop = props.on_close.clone();
        let install_state = install_state.clone();
        Callback::from(move |e: MouseEvent| {
            // Only close if clicking directly on the backdrop
            let target = e.target().unwrap();
            let current_target = e.current_target().unwrap();
            if target == current_target && *install_state != InstallState::Installing {
                ui_dispatch.apply(UiAction::CloseModal);
                on_close_prop.emit(());
            }
        })
    };

    // Source type change handler
    let on_source_type_change = {
        let source_type = source_type.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let new_type = match select.value().as_str() {
                "git" => SourceType::Git,
                "url" => SourceType::Url,
                "local" => SourceType::Local,
                "registry" => SourceType::Registry,
                _ => SourceType::Git,
            };
            source_type.set(new_type);
        })
    };

    // Input handlers
    let on_source_input = {
        let source_input = source_input.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            source_input.set(input.value());
        })
    };

    let on_git_ref_input = {
        let git_ref = git_ref.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            git_ref.set(input.value());
        })
    };

    let on_instance_input = {
        let instance_name = instance_name.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            instance_name.set(input.value());
        })
    };

    let on_force_toggle = {
        let force_reinstall = force_reinstall.clone();
        Callback::from(move |_| {
            force_reinstall.set(!*force_reinstall);
        })
    };

    // Install handler
    let on_install = {
        let api = api.clone();
        let source_type = source_type.clone();
        let source_input = source_input.clone();
        let git_ref = git_ref.clone();
        let instance_name = instance_name.clone();
        let force_reinstall = force_reinstall.clone();
        let install_state = install_state.clone();
        let notifications = notifications.clone();
        let on_installed = props.on_installed.clone();
        let ui_dispatch = ui_dispatch.clone();

        Callback::from(move |_| {
            let source = (*source_input).trim().to_string();
            if source.is_empty() {
                notifications.error("Validation Error", "Please enter a source");
                return;
            }

            // Build the source string based on type
            let full_source = match *source_type {
                SourceType::Git => {
                    // Handle github: shorthand
                    if source.starts_with("github:") || source.contains("github.com") {
                        source.clone()
                    } else {
                        format!("github:{}", source)
                    }
                }
                SourceType::Local => {
                    if source.starts_with("local:") {
                        source.clone()
                    } else {
                        format!("local:{}", source)
                    }
                }
                _ => source.clone(),
            };

            // Build request
            let request = InstallSkillRequest {
                source: full_source,
                name: if (*instance_name).is_empty() {
                    None
                } else {
                    Some((*instance_name).clone())
                },
                git_ref: if (*git_ref).is_empty() {
                    None
                } else {
                    Some((*git_ref).clone())
                },
                force: *force_reinstall,
            };

            install_state.set(InstallState::Installing);

            let api = api.clone();
            let install_state = install_state.clone();
            let notifications = notifications.clone();
            let on_installed = on_installed.clone();
            let ui_dispatch = ui_dispatch.clone();

            spawn_local(async move {
                match api.skills.install(&request).await {
                    Ok(response) => {
                        if response.success {
                            let name = response.name.unwrap_or_else(|| "skill".to_string());
                            let version = response.version.unwrap_or_else(|| "unknown".to_string());
                            install_state.set(InstallState::Success(name.clone()));
                            notifications.success(
                                "Skill Installed",
                                format!(
                                    "Successfully installed {} v{} with {} tools",
                                    name, version, response.tools_count
                                ),
                            );
                            on_installed.emit(name);
                            ui_dispatch.apply(UiAction::CloseModal);
                        } else {
                            let error = response.error.unwrap_or_else(|| "Unknown error".to_string());
                            install_state.set(InstallState::Error(error.clone()));
                            notifications.error("Installation Failed", &error);
                        }
                    }
                    Err(e) => {
                        let error = e.to_string();
                        install_state.set(InstallState::Error(error.clone()));
                        notifications.error("Installation Failed", &error);
                    }
                }
            });
        })
    };

    // Validation
    let is_valid = !(*source_input).trim().is_empty();
    let is_installing = *install_state == InstallState::Installing;

    if !is_open {
        return html! {};
    }

    html! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
            onclick={on_backdrop_click}
        >
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg mx-4 animate-scale-in">
                // Header
                <div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                        { "Install Skill" }
                    </h2>
                    <button
                        onclick={on_close.clone()}
                        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                        disabled={is_installing}
                    >
                        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                // Body
                <div class="p-6 space-y-5">
                    // Source Type
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            { "Source Type" }
                        </label>
                        <select
                            class="input"
                            onchange={on_source_type_change}
                            disabled={is_installing}
                        >
                            <option value="git" selected={*source_type == SourceType::Git}>
                                { SourceType::Git.label() }
                            </option>
                            <option value="url" selected={*source_type == SourceType::Url}>
                                { SourceType::Url.label() }
                            </option>
                            <option value="local" selected={*source_type == SourceType::Local}>
                                { SourceType::Local.label() }
                            </option>
                            <option value="registry" selected={*source_type == SourceType::Registry}>
                                { SourceType::Registry.label() }
                            </option>
                        </select>
                    </div>

                    // Source Input
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            { "Source" }
                            <span class="text-red-500">{ " *" }</span>
                        </label>
                        <input
                            type="text"
                            class="input"
                            placeholder={source_type.placeholder()}
                            value={(*source_input).clone()}
                            oninput={on_source_input}
                            disabled={is_installing}
                        />
                        <p class="mt-1.5 text-xs text-gray-500 dark:text-gray-400">
                            { source_type.help_text() }
                        </p>
                    </div>

                    // Git Ref (only for Git type)
                    if *source_type == SourceType::Git {
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                { "Branch / Tag / Commit" }
                                <span class="text-gray-400 text-xs ml-2">{ "(optional)" }</span>
                            </label>
                            <input
                                type="text"
                                class="input"
                                placeholder="main, v1.0.0, or commit hash"
                                value={(*git_ref).clone()}
                                oninput={on_git_ref_input}
                                disabled={is_installing}
                            />
                        </div>
                    }

                    // Instance Name
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            { "Instance Name" }
                            <span class="text-gray-400 text-xs ml-2">{ "(optional)" }</span>
                        </label>
                        <input
                            type="text"
                            class="input"
                            placeholder="default"
                            value={(*instance_name).clone()}
                            oninput={on_instance_input}
                            disabled={is_installing}
                        />
                        <p class="mt-1.5 text-xs text-gray-500 dark:text-gray-400">
                            { "Custom name for this skill installation. Leave empty for default." }
                        </p>
                    </div>

                    // Force Reinstall Toggle
                    if *source_type == SourceType::Git {
                        <div class="flex items-center gap-3">
                            <button
                                type="button"
                                role="switch"
                                aria-checked={(*force_reinstall).to_string()}
                                onclick={on_force_toggle}
                                disabled={is_installing}
                                class={classes!(
                                    "relative", "inline-flex", "h-6", "w-11", "flex-shrink-0",
                                    "cursor-pointer", "rounded-full", "border-2", "border-transparent",
                                    "transition-colors", "duration-200", "ease-in-out",
                                    "focus:outline-none", "focus:ring-2", "focus:ring-primary-500", "focus:ring-offset-2",
                                    if *force_reinstall { "bg-primary-600" } else { "bg-gray-200 dark:bg-gray-700" },
                                    if is_installing { "opacity-50 cursor-not-allowed" } else { "" }
                                )}
                            >
                                <span
                                    class={classes!(
                                        "pointer-events-none", "inline-block", "h-5", "w-5",
                                        "transform", "rounded-full", "bg-white", "shadow",
                                        "ring-0", "transition", "duration-200", "ease-in-out",
                                        if *force_reinstall { "translate-x-5" } else { "translate-x-0" }
                                    )}
                                />
                            </button>
                            <div>
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                    { "Force re-clone" }
                                </span>
                                <p class="text-xs text-gray-500 dark:text-gray-400">
                                    { "Delete existing installation and re-clone from source" }
                                </p>
                            </div>
                        </div>
                    }

                    // Error display
                    if let InstallState::Error(ref error) = *install_state {
                        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                            <div class="flex items-start gap-3">
                                <svg class="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                <div>
                                    <p class="text-sm font-medium text-red-700 dark:text-red-300">
                                        { "Installation failed" }
                                    </p>
                                    <p class="text-sm text-red-600 dark:text-red-400 mt-1">
                                        { error }
                                    </p>
                                </div>
                            </div>
                        </div>
                    }
                </div>

                // Footer
                <div class="flex items-center justify-end gap-3 p-6 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 rounded-b-xl">
                    <button
                        onclick={on_close}
                        class="btn btn-secondary"
                        disabled={is_installing}
                    >
                        { "Cancel" }
                    </button>
                    <button
                        onclick={on_install}
                        class="btn btn-primary"
                        disabled={!is_valid || is_installing}
                    >
                        if is_installing {
                            <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            { "Installing..." }
                        } else {
                            <svg class="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                            </svg>
                            { "Install Skill" }
                        }
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Hook to open the install skill modal
#[hook]
pub fn use_install_skill_modal() -> UseInstallSkillModalHandle {
    let (_, dispatch) = use_store::<UiStore>();
    UseInstallSkillModalHandle { dispatch }
}

/// Handle for the install skill modal hook
pub struct UseInstallSkillModalHandle {
    dispatch: Dispatch<UiStore>,
}

impl UseInstallSkillModalHandle {
    /// Open the install skill modal
    pub fn open(&self) {
        self.dispatch
            .apply(UiAction::OpenModal(crate::store::ui::ModalType::InstallSkill, None));
    }
}

impl Clone for UseInstallSkillModalHandle {
    fn clone(&self) -> Self {
        Self {
            dispatch: self.dispatch.clone(),
        }
    }
}
