//! Import Configuration Modal component
//!
//! A beautifully designed modal for importing skill configurations from TOML manifests.
//! Features:
//! - Multiple example templates with preview
//! - Code editor with line numbers
//! - Live validation with detailed feedback
//! - Drag-and-drop file upload
//! - Smooth animations and transitions

use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{File, FileReader, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::api::{Api, ParsedSkill};
use crate::components::use_notifications;
use crate::store::ui::{ModalType, UiAction, UiStore};

/// Import state
#[derive(Clone, PartialEq)]
enum ImportState {
    Input,
    Validating,
    Preview(Vec<ParsedSkill>),
    Importing,
    Complete(usize),
    Error(String),
}

/// Validation status for live validation
#[derive(Clone, PartialEq)]
enum ValidationStatus {
    None,
    Checking,
    Valid(usize),
    Invalid(String),
}

/// Example template
#[derive(Clone, PartialEq)]
struct ExampleTemplate {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    badge: &'static str,
    badge_color: &'static str,
    content: &'static str,
}

const EXAMPLE_TEMPLATES: &[ExampleTemplate] = &[
    ExampleTemplate {
        id: "quick-start",
        name: "Quick Start",
        description: "Working skills you can test now",
        badge: "Try Now",
        badge_color: "green",
        content: r#"# Quick Start - Testable Skills
# ==============================
# These skills use local examples that work immediately.
# Import this to test the UI right away!
#
version = "1"

# ─────────────────────────────────────
# Simple Skill (WASM) - Works Immediately!
# ─────────────────────────────────────
# Tools:
#   - hello(name, greeting) → Greet someone
#   - echo(message, repeat) → Echo text
#   - calculate(operation, a, b) → Math

[skills.simple]
source = "./examples/wasm-skills/simple-skill"
description = "Simple greeting and utilities - great for testing!"

[skills.simple.instances.default]

# ─────────────────────────────────────
# GitHub Skill (WASM) - Requires token
# ─────────────────────────────────────
# Tools:
#   - list_repos(org)
#   - get_repo(owner, repo)
#   - search_code(query)

[skills.github]
source = "./examples/wasm-skills/github-skill"
description = "GitHub API integration"

[skills.github.instances.default]
config.token = "${GITHUB_TOKEN}"

# ─────────────────────────────────────
# Docker Management (Native)
# ─────────────────────────────────────
# Tools:
#   - ps() → List containers
#   - images() → List images
#   - logs(container) → View logs

[skills.docker-cli]
source = "./examples/native-skills/docker-skill"
description = "Docker container management"

[skills.docker-cli.instances.default]
"#,
    },
    ExampleTemplate {
        id: "docker-runners",
        name: "Code Runners",
        description: "Python, Node.js sandboxed",
        badge: "Docker",
        badge_color: "purple",
        content: r#"# Code Runners (Docker Sandboxed)
# ================================
# Execute code in isolated containers.
# Uses example skills from the repository.
#
version = "1"

# ─────────────────────────────────────
# Python Runner
# ─────────────────────────────────────
# From: examples/docker-runtime-skills/python-runner
# Tools:
#   - run(code) → Execute Python code
#   - exec(command) → Run shell command

[skills.python-runner]
source = "./examples/docker-runtime-skills/python-runner"
runtime = "docker"
description = "Execute Python scripts in sandbox"

[skills.python-runner.instances.default]

# ─────────────────────────────────────
# Node.js Runner
# ─────────────────────────────────────
# From: examples/docker-runtime-skills/node-runner
# Tools:
#   - run(code) → Execute JavaScript
#   - exec(command) → Run shell command

[skills.node-runner]
source = "./examples/docker-runtime-skills/node-runner"
runtime = "docker"
description = "Execute Node.js scripts in sandbox"

[skills.node-runner.instances.default]

# ─────────────────────────────────────
# FFmpeg (Media Processing)
# ─────────────────────────────────────
# From: examples/docker-runtime-skills/ffmpeg-skill

[skills.ffmpeg]
source = "./examples/docker-runtime-skills/ffmpeg-skill"
runtime = "docker"
description = "Video/audio processing with FFmpeg"

[skills.ffmpeg.instances.default]

# ─────────────────────────────────────
# ImageMagick (Image Processing)
# ─────────────────────────────────────
# From: examples/docker-runtime-skills/imagemagick-skill

[skills.imagemagick]
source = "./examples/docker-runtime-skills/imagemagick-skill"
runtime = "docker"
description = "Image processing with ImageMagick"

[skills.imagemagick.instances.default]
"#,
    },
    ExampleTemplate {
        id: "devops",
        name: "DevOps Tools",
        description: "Docker, Kubernetes CLIs",
        badge: "Native",
        badge_color: "blue",
        content: r#"# DevOps Tools (Native CLI Wrappers)
# ===================================
# Native skills wrap system CLI tools with
# structured parameters and validation.
#
version = "1"

# ─────────────────────────────────────
# Docker Management
# ─────────────────────────────────────
# From: examples/native-skills/docker-skill
# Tools:
#   - ps(all, format) → List containers
#   - images(all) → List images
#   - run(image, name, detach, ports) → Run container
#   - exec(container, command) → Execute command
#   - logs(container, tail, follow) → View logs
#   - stop(container) → Stop container
#   - rm(container, force) → Remove container
#   - pull(image) → Pull image
#   - build(path, tag) → Build image

[skills.docker]
source = "./examples/native-skills/docker-skill"
description = "Docker container and image management"

[skills.docker.instances.default]
# Uses system Docker socket

# ─────────────────────────────────────
# Kubernetes Management
# ─────────────────────────────────────
# From: examples/native-skills/kubernetes-skill
# Tools:
#   - get(resource, name, namespace, output)
#   - describe(resource, name, namespace)
#   - logs(pod, container, tail, follow)
#   - exec(pod, command, namespace)
#   - apply(content, namespace, dry_run)
#   - delete(resource, name, namespace)
#   - scale(deployment, replicas, namespace)
#   - rollout(subcommand, deployment)

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes cluster management"

[skills.kubernetes.instances.default]
# Uses default kubeconfig

[skills.kubernetes.instances.production]
config.context = "prod-cluster"
"#,
    },
    ExampleTemplate {
        id: "databases",
        name: "Database Clients",
        description: "PostgreSQL, Redis (Docker)",
        badge: "Data",
        badge_color: "amber",
        content: r#"# Database Clients (Docker)
# =========================
# Connect to databases using containerized clients.
# Network access enabled for connectivity.
#
version = "1"

# ─────────────────────────────────────
# PostgreSQL Client
# ─────────────────────────────────────
# From: examples/docker-runtime-skills/postgres-skill
# Tools:
#   - query(sql) → Execute SQL query
#   - exec(command) → Run psql command

[skills.postgres]
source = "./examples/docker-runtime-skills/postgres-skill"
runtime = "docker"
description = "PostgreSQL database client"

[skills.postgres.instances.local]
config.host = "localhost"
config.port = "5432"
config.database = "postgres"
config.user = "postgres"

[skills.postgres.instances.docker]
config.host = "host.docker.internal"
config.port = "5432"
config.database = "myapp"
config.user = "postgres"

# ─────────────────────────────────────
# Redis Client
# ─────────────────────────────────────
# From: examples/docker-runtime-skills/redis-skill
# Tools:
#   - command(cmd) → Execute Redis command
#   - get(key) → Get value
#   - set(key, value) → Set value

[skills.redis]
source = "./examples/docker-runtime-skills/redis-skill"
runtime = "docker"
description = "Redis database client"

[skills.redis.instances.local]
config.host = "localhost"
config.port = "6379"
"#,
    },
    ExampleTemplate {
        id: "wasm-api",
        name: "API Integrations",
        description: "GitHub, Slack, AWS skills",
        badge: "WASM",
        badge_color: "indigo",
        content: r#"# API Integration Skills (WASM)
# =============================
# Custom skills that integrate with external APIs.
# Each requires appropriate API keys/tokens.
#
version = "1"

# ─────────────────────────────────────
# GitHub Skill
# ─────────────────────────────────────
# From: examples/wasm-skills/github-skill
# Tools:
#   - list_repos(org, visibility)
#   - get_repo(owner, repo)
#   - create_issue(owner, repo, title, body)
#   - list_issues(owner, repo, state)
#   - search_code(query, language)

[skills.github]
source = "./examples/wasm-skills/github-skill"
description = "GitHub API integration"

[skills.github.instances.default]
config.token = "${GITHUB_TOKEN}"

# ─────────────────────────────────────
# Slack Skill
# ─────────────────────────────────────
# From: examples/wasm-skills/slack-skill
# Tools:
#   - post_message(channel, text)
#   - list_channels()
#   - get_user(user_id)

[skills.slack]
source = "./examples/wasm-skills/slack-skill"
description = "Slack messaging integration"

[skills.slack.instances.default]
config.bot_token = "${SLACK_BOT_TOKEN}"

# ─────────────────────────────────────
# AWS Skill
# ─────────────────────────────────────
# From: examples/wasm-skills/aws-skill
# Tools:
#   - s3_list(bucket, prefix)
#   - s3_get(bucket, key)
#   - s3_put(bucket, key, content)
#   - lambda_invoke(function, payload)

[skills.aws]
source = "./examples/wasm-skills/aws-skill"
description = "AWS services integration"

[skills.aws.instances.default]
config.region = "us-east-1"
config.access_key = "${AWS_ACCESS_KEY_ID}"
config.secret_key = "${AWS_SECRET_ACCESS_KEY}"
"#,
    },
    ExampleTemplate {
        id: "complete",
        name: "Complete Setup",
        description: "All available example skills",
        badge: "Full",
        badge_color: "rose",
        content: r#"# Complete Skill Setup
# ====================
# All example skills from the repository.
# Import this to get everything at once!
#
version = "1"

# ═══════════════════════════════════════
# WASM SKILLS (Custom Tools)
# ═══════════════════════════════════════

[skills.simple]
source = "./examples/wasm-skills/simple-skill"
description = "Basic greeting and utilities"

[skills.simple.instances.default]

[skills.github]
source = "./examples/wasm-skills/github-skill"
description = "GitHub API"

[skills.github.instances.default]
config.token = "${GITHUB_TOKEN}"

[skills.slack]
source = "./examples/wasm-skills/slack-skill"
description = "Slack messaging"

[skills.slack.instances.default]
config.bot_token = "${SLACK_BOT_TOKEN}"

[skills.aws]
source = "./examples/wasm-skills/aws-skill"
description = "AWS services"

[skills.aws.instances.default]
config.region = "us-east-1"

# ═══════════════════════════════════════
# DOCKER SKILLS (Sandboxed Execution)
# ═══════════════════════════════════════

[skills.python-runner]
source = "./examples/docker-runtime-skills/python-runner"
runtime = "docker"
description = "Python sandbox"

[skills.python-runner.instances.default]

[skills.node-runner]
source = "./examples/docker-runtime-skills/node-runner"
runtime = "docker"
description = "Node.js sandbox"

[skills.node-runner.instances.default]

[skills.ffmpeg]
source = "./examples/docker-runtime-skills/ffmpeg-skill"
runtime = "docker"
description = "Video/audio processing"

[skills.ffmpeg.instances.default]

[skills.postgres]
source = "./examples/docker-runtime-skills/postgres-skill"
runtime = "docker"
description = "PostgreSQL client"

[skills.postgres.instances.local]
config.host = "localhost"

[skills.redis]
source = "./examples/docker-runtime-skills/redis-skill"
runtime = "docker"
description = "Redis client"

[skills.redis.instances.local]
config.host = "localhost"

# ═══════════════════════════════════════
# NATIVE SKILLS (CLI Wrappers)
# ═══════════════════════════════════════

[skills.docker-cli]
source = "./examples/native-skills/docker-skill"
description = "Docker management"

[skills.docker-cli.instances.default]

[skills.kubernetes]
source = "./examples/native-skills/kubernetes-skill"
description = "Kubernetes management"

[skills.kubernetes.instances.default]
"#,
    },
];

/// Props for the ImportConfigModal component
#[derive(Properties, PartialEq)]
pub struct ImportConfigModalProps {
    #[prop_or_default]
    pub on_imported: Callback<usize>,
    #[prop_or_default]
    pub on_close: Callback<()>,
}

/// Import Configuration Modal component
#[function_component(ImportConfigModal)]
pub fn import_config_modal(props: &ImportConfigModalProps) -> Html {
    let (ui_store, ui_dispatch) = use_store::<UiStore>();
    let notifications = use_notifications();

    // State
    let content = use_state(String::new);
    let import_state = use_state(|| ImportState::Input);
    let merge_mode = use_state(|| true);
    let is_dragging = use_state(|| false);
    let warnings = use_state(Vec::<String>::new);
    let validation_status = use_state(|| ValidationStatus::None);
    let active_tab = use_state(|| "editor"); // "editor" | "templates"
    let selected_template = use_state(|| Option::<String>::None);
    let debounce_timer = use_state(|| Option::<i32>::None);

    let api = use_memo((), |_| Rc::new(Api::new()));

    let is_open = ui_store.modal.open
        && ui_store.modal.modal_type == Some(ModalType::Import);

    // Live validation effect - debounced
    {
        let content = content.clone();
        let validation_status = validation_status.clone();
        let api = api.clone();
        let debounce_timer = debounce_timer.clone();

        use_effect_with((*content).clone(), move |content_value| {
            if let Some(timer_id) = *debounce_timer {
                let window = web_sys::window().unwrap();
                window.clear_timeout_with_handle(timer_id);
            }

            let content_value = content_value.clone();
            if content_value.trim().is_empty() {
                validation_status.set(ValidationStatus::None);
                return;
            }

            validation_status.set(ValidationStatus::Checking);

            let validation_status = validation_status.clone();
            let api = api.clone();
            let debounce_timer = debounce_timer.clone();

            let closure = Closure::wrap(Box::new(move || {
                let api = api.clone();
                let validation_status = validation_status.clone();
                let content_value = content_value.clone();

                spawn_local(async move {
                    match api.config.validate_manifest(&content_value).await {
                        Ok(response) => {
                            if response.valid {
                                validation_status.set(ValidationStatus::Valid(response.skills.len()));
                            } else {
                                let error = response.errors.first()
                                    .cloned()
                                    .unwrap_or_else(|| "Invalid manifest".to_string());
                                validation_status.set(ValidationStatus::Invalid(error));
                            }
                        }
                        Err(e) => {
                            validation_status.set(ValidationStatus::Invalid(e.to_string()));
                        }
                    }
                });
            }) as Box<dyn FnMut()>);

            let window = web_sys::window().unwrap();
            let timer_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                400,
            ).unwrap();

            debounce_timer.set(Some(timer_id));
            closure.forget();
        });
    }

    // Close handler
    let on_close = {
        let ui_dispatch = ui_dispatch.clone();
        let on_close_prop = props.on_close.clone();
        let import_state = import_state.clone();
        let content = content.clone();
        let validation_status = validation_status.clone();
        let active_tab = active_tab.clone();
        let selected_template = selected_template.clone();
        Callback::from(move |_: MouseEvent| {
            if !matches!(*import_state, ImportState::Validating | ImportState::Importing) {
                ui_dispatch.apply(UiAction::CloseModal);
                on_close_prop.emit(());
                content.set(String::new());
                validation_status.set(ValidationStatus::None);
                active_tab.set("editor");
                selected_template.set(None);
            }
        })
    };

    let on_backdrop_click = {
        let on_close = on_close.clone();
        Callback::from(move |e: MouseEvent| {
            let target = e.target().unwrap();
            let current_target = e.current_target().unwrap();
            if target == current_target {
                on_close.emit(e);
            }
        })
    };

    let on_content_input = {
        let content = content.clone();
        let import_state = import_state.clone();
        Callback::from(move |e: InputEvent| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            content.set(textarea.value());
            if matches!(*import_state, ImportState::Error(_)) {
                import_state.set(ImportState::Input);
            }
        })
    };

    let on_dragover = {
        let is_dragging = is_dragging.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging.set(true);
        })
    };

    let on_dragleave = {
        let is_dragging = is_dragging.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging.set(false);
        })
    };

    let on_drop = {
        let content = content.clone();
        let is_dragging = is_dragging.clone();
        let active_tab = active_tab.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging.set(false);
            active_tab.set("editor");

            if let Some(data_transfer) = e.data_transfer() {
                if let Some(files) = data_transfer.files() {
                    if files.length() > 0 {
                        if let Some(file) = files.get(0) {
                            read_file_content(file, content.clone());
                        }
                    }
                }
            }
        })
    };

    let on_file_select = {
        let content = content.clone();
        let active_tab = active_tab.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if files.length() > 0 {
                    if let Some(file) = files.get(0) {
                        read_file_content(file, content.clone());
                        active_tab.set("editor");
                    }
                }
            }
        })
    };

    let on_tab_change = {
        let active_tab = active_tab.clone();
        Callback::from(move |tab: &'static str| {
            active_tab.set(tab);
        })
    };

    let on_select_template = {
        let content = content.clone();
        let selected_template = selected_template.clone();
        let active_tab = active_tab.clone();
        Callback::from(move |(id, template_content): (String, String)| {
            selected_template.set(Some(id));
            content.set(template_content);
            active_tab.set("editor");
        })
    };

    let on_validate = {
        let api = api.clone();
        let content = content.clone();
        let import_state = import_state.clone();
        let warnings = warnings.clone();
        let notifications = notifications.clone();

        Callback::from(move |_| {
            let content_value = (*content).clone();
            if content_value.trim().is_empty() {
                notifications.error("Empty Content", "Please enter or upload a manifest file");
                return;
            }

            import_state.set(ImportState::Validating);

            let api = api.clone();
            let import_state = import_state.clone();
            let warnings = warnings.clone();
            let notifications = notifications.clone();

            spawn_local(async move {
                match api.config.validate_manifest(&content_value).await {
                    Ok(response) => {
                        if response.valid {
                            warnings.set(response.warnings);
                            import_state.set(ImportState::Preview(response.skills));
                        } else {
                            let error_msg = response.errors.join("\n");
                            import_state.set(ImportState::Error(error_msg.clone()));
                            notifications.error("Validation Failed", &error_msg);
                        }
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        import_state.set(ImportState::Error(error_msg.clone()));
                        notifications.error("Validation Error", &error_msg);
                    }
                }
            });
        })
    };

    let on_import = {
        let api = api.clone();
        let content = content.clone();
        let merge_mode = merge_mode.clone();
        let import_state = import_state.clone();
        let notifications = notifications.clone();
        let on_imported = props.on_imported.clone();
        let ui_dispatch = ui_dispatch.clone();

        Callback::from(move |_| {
            let content_value = (*content).clone();
            let merge = *merge_mode;

            import_state.set(ImportState::Importing);

            let api = api.clone();
            let import_state = import_state.clone();
            let notifications = notifications.clone();
            let on_imported = on_imported.clone();
            let ui_dispatch = ui_dispatch.clone();

            spawn_local(async move {
                match api.config.import_manifest(&content_value, merge, true).await {
                    Ok(response) => {
                        if response.success {
                            let count = response.installed_count;
                            import_state.set(ImportState::Complete(count));
                            notifications.success(
                                "Import Complete",
                                format!("Successfully imported {} skill(s)", count),
                            );
                            on_imported.emit(count);
                            ui_dispatch.apply(UiAction::CloseModal);
                        } else {
                            let error_msg = response.errors.join("\n");
                            import_state.set(ImportState::Error(error_msg.clone()));
                            notifications.error("Import Failed", &error_msg);
                        }
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        import_state.set(ImportState::Error(error_msg.clone()));
                        notifications.error("Import Error", &error_msg);
                    }
                }
            });
        })
    };

    let on_back = {
        let import_state = import_state.clone();
        Callback::from(move |_| {
            import_state.set(ImportState::Input);
        })
    };

    let on_toggle_merge = {
        let merge_mode = merge_mode.clone();
        Callback::from(move |_| {
            merge_mode.set(!*merge_mode);
        })
    };

    let on_clear = {
        let content = content.clone();
        let validation_status = validation_status.clone();
        let import_state = import_state.clone();
        Callback::from(move |_: MouseEvent| {
            content.set(String::new());
            validation_status.set(ValidationStatus::None);
            import_state.set(ImportState::Input);
        })
    };

    let is_loading = matches!(*import_state, ImportState::Validating | ImportState::Importing);
    let can_validate = !(*content).trim().is_empty() && !is_loading;
    let is_valid = matches!(*validation_status, ValidationStatus::Valid(_));

    if !is_open {
        return html! {};
    }

    html! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in"
            onclick={on_backdrop_click}
        >
            <div class="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-5xl max-h-[85vh] flex flex-col overflow-hidden animate-scale-in border border-gray-200 dark:border-gray-700">
                // Header
                <div class="relative px-6 py-5 border-b border-gray-100 dark:border-gray-800 bg-gradient-to-r from-gray-50 to-white dark:from-gray-800/50 dark:to-gray-900">
                    <div class="flex items-start justify-between">
                        <div class="flex items-center gap-4">
                            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary-500 to-primary-600 flex items-center justify-center shadow-lg shadow-primary-500/20">
                                <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                                </svg>
                            </div>
                            <div>
                                <h2 class="text-xl font-bold text-gray-900 dark:text-white">
                                    { "Import Configuration" }
                                </h2>
                                <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                                    { "Add skills from a TOML manifest file" }
                                </p>
                            </div>
                        </div>
                        <button
                            onclick={on_close.clone()}
                            class="p-2 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 transition-all"
                            disabled={is_loading}
                        >
                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                </div>

                // Body
                <div class="flex-1 overflow-hidden flex">
                    {
                        match &*import_state {
                            ImportState::Input | ImportState::Validating | ImportState::Error(_) => {
                                render_editor_view(
                                    &content,
                                    &is_dragging,
                                    &import_state,
                                    &validation_status,
                                    &active_tab,
                                    &selected_template,
                                    on_content_input.clone(),
                                    on_dragover.clone(),
                                    on_dragleave.clone(),
                                    on_drop.clone(),
                                    on_file_select.clone(),
                                    on_tab_change.clone(),
                                    on_select_template.clone(),
                                    on_clear.clone(),
                                )
                            }
                            ImportState::Preview(skills) => {
                                render_preview_view(
                                    skills,
                                    &warnings,
                                    &merge_mode,
                                    on_toggle_merge.clone(),
                                )
                            }
                            ImportState::Importing => render_importing_view(),
                            ImportState::Complete(count) => render_complete_view(*count),
                        }
                    }
                </div>

                // Footer
                <div class="px-6 py-4 border-t border-gray-100 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-800/30">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            {
                                match &*validation_status {
                                    ValidationStatus::None => html! {
                                        <span class="text-sm text-gray-400 dark:text-gray-500">
                                            { "Paste or upload a manifest to get started" }
                                        </span>
                                    },
                                    ValidationStatus::Checking => html! {
                                        <div class="flex items-center gap-2 text-gray-500 dark:text-gray-400">
                                            <div class="w-4 h-4 border-2 border-gray-300 border-t-primary-500 rounded-full animate-spin"></div>
                                            <span class="text-sm">{ "Validating..." }</span>
                                        </div>
                                    },
                                    ValidationStatus::Valid(count) => html! {
                                        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400">
                                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                                            </svg>
                                            <span class="text-sm font-medium">{ format!("{} skill{} ready", count, if *count == 1 { "" } else { "s" }) }</span>
                                        </div>
                                    },
                                    ValidationStatus::Invalid(err) => html! {
                                        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400 max-w-md">
                                            <svg class="w-4 h-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                            </svg>
                                            <span class="text-sm font-medium truncate" title={err.clone()}>{ err }</span>
                                        </div>
                                    },
                                }
                            }
                        </div>

                        <div class="flex items-center gap-3">
                            {
                                match &*import_state {
                                    ImportState::Input | ImportState::Validating | ImportState::Error(_) => {
                                        html! {
                                            <>
                                                <button
                                                    onclick={on_close.clone()}
                                                    class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
                                                    disabled={is_loading}
                                                >
                                                    { "Cancel" }
                                                </button>
                                                <button
                                                    onclick={on_validate}
                                                    disabled={!can_validate}
                                                    class={classes!(
                                                        "px-5", "py-2.5", "text-sm", "font-semibold", "rounded-xl", "transition-all", "flex", "items-center", "gap-2",
                                                        if can_validate && is_valid {
                                                            "bg-gradient-to-r from-green-500 to-emerald-500 text-white shadow-lg shadow-green-500/25 hover:shadow-green-500/40 hover:scale-[1.02]"
                                                        } else if can_validate {
                                                            "bg-gradient-to-r from-primary-500 to-primary-600 text-white shadow-lg shadow-primary-500/25 hover:shadow-primary-500/40 hover:scale-[1.02]"
                                                        } else {
                                                            "bg-gray-100 dark:bg-gray-800 text-gray-400 cursor-not-allowed"
                                                        }
                                                    )}
                                                >
                                                    if matches!(*import_state, ImportState::Validating) {
                                                        <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                                        { "Validating..." }
                                                    } else if is_valid {
                                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                                        </svg>
                                                        { "Continue to Import" }
                                                    } else {
                                                        { "Validate & Continue" }
                                                    }
                                                </button>
                                            </>
                                        }
                                    }
                                    ImportState::Preview(_) => {
                                        html! {
                                            <>
                                                <button
                                                    onclick={on_back}
                                                    class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors flex items-center gap-2"
                                                >
                                                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                                                    </svg>
                                                    { "Back" }
                                                </button>
                                                <button
                                                    onclick={on_import}
                                                    class="px-5 py-2.5 text-sm font-semibold rounded-xl bg-gradient-to-r from-primary-500 to-primary-600 text-white shadow-lg shadow-primary-500/25 hover:shadow-primary-500/40 hover:scale-[1.02] transition-all flex items-center gap-2"
                                                >
                                                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                                    </svg>
                                                    { "Import Skills" }
                                                </button>
                                            </>
                                        }
                                    }
                                    ImportState::Importing => {
                                        html! {
                                            <button
                                                disabled={true}
                                                class="px-5 py-2.5 text-sm font-semibold rounded-xl bg-gray-100 dark:bg-gray-800 text-gray-400 flex items-center gap-2"
                                            >
                                                <div class="w-4 h-4 border-2 border-gray-300 border-t-gray-500 rounded-full animate-spin"></div>
                                                { "Importing..." }
                                            </button>
                                        }
                                    }
                                    ImportState::Complete(_) => {
                                        html! {
                                            <button
                                                onclick={on_close}
                                                class="px-5 py-2.5 text-sm font-semibold rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 text-white shadow-lg shadow-green-500/25 hover:shadow-green-500/40 hover:scale-[1.02] transition-all"
                                            >
                                                { "Done" }
                                            </button>
                                        }
                                    }
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Render the editor view with tabs
#[allow(clippy::too_many_arguments)]
fn render_editor_view(
    content: &UseStateHandle<String>,
    is_dragging: &UseStateHandle<bool>,
    import_state: &UseStateHandle<ImportState>,
    _validation_status: &UseStateHandle<ValidationStatus>,
    active_tab: &UseStateHandle<&'static str>,
    selected_template: &UseStateHandle<Option<String>>,
    on_content_input: Callback<InputEvent>,
    on_dragover: Callback<DragEvent>,
    on_dragleave: Callback<DragEvent>,
    on_drop: Callback<DragEvent>,
    on_file_select: Callback<Event>,
    on_tab_change: Callback<&'static str>,
    on_select_template: Callback<(String, String)>,
    on_clear: Callback<MouseEvent>,
) -> Html {
    let tab_editor = {
        let on_tab_change = on_tab_change.clone();
        Callback::from(move |_: MouseEvent| on_tab_change.emit("editor"))
    };

    let tab_templates = {
        let on_tab_change = on_tab_change.clone();
        Callback::from(move |_: MouseEvent| on_tab_change.emit("templates"))
    };

    html! {
        <div class="flex-1 flex flex-col overflow-hidden">
            // Tab bar
            <div class="flex items-center justify-between px-6 py-3 border-b border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-900">
                <div class="flex items-center gap-1 p-1 bg-gray-100 dark:bg-gray-800 rounded-lg">
                    <button
                        onclick={tab_editor}
                        class={classes!(
                            "px-4", "py-2", "text-sm", "font-medium", "rounded-md", "transition-all",
                            if **active_tab == "editor" {
                                "bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm"
                            } else {
                                "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                            }
                        )}
                    >
                        <span class="flex items-center gap-2">
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                            </svg>
                            { "Editor" }
                        </span>
                    </button>
                    <button
                        onclick={tab_templates}
                        class={classes!(
                            "px-4", "py-2", "text-sm", "font-medium", "rounded-md", "transition-all",
                            if **active_tab == "templates" {
                                "bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm"
                            } else {
                                "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
                            }
                        )}
                    >
                        <span class="flex items-center gap-2">
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
                            </svg>
                            { "Templates" }
                        </span>
                    </button>
                </div>

                <div class="flex items-center gap-2">
                    if !(**content).is_empty() {
                        <button
                            onclick={on_clear}
                            class="flex items-center gap-2 px-3 py-1.5 text-sm text-gray-500 hover:text-red-600 dark:text-gray-400 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                        >
                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                            { "Clear" }
                        </button>
                    }
                    <label class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors cursor-pointer">
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                        </svg>
                        { "Upload File" }
                        <input
                            type="file"
                            accept=".toml,.txt"
                            class="hidden"
                            onchange={on_file_select}
                        />
                    </label>
                </div>
            </div>

            // Content area
            <div class="flex-1 overflow-hidden">
                if **active_tab == "editor" {
                    { render_code_editor(content, is_dragging, import_state, on_content_input, on_dragover, on_dragleave, on_drop) }
                } else {
                    { render_templates_view(selected_template, on_select_template) }
                }
            </div>
        </div>
    }
}

/// Render the code editor
fn render_code_editor(
    content: &UseStateHandle<String>,
    is_dragging: &UseStateHandle<bool>,
    import_state: &UseStateHandle<ImportState>,
    on_content_input: Callback<InputEvent>,
    on_dragover: Callback<DragEvent>,
    on_dragleave: Callback<DragEvent>,
    on_drop: Callback<DragEvent>,
) -> Html {
    let line_count = if (*content).is_empty() { 20 } else { (*content).lines().count().max(20) };

    html! {
        <div
            class="h-full flex relative"
            ondragover={on_dragover}
            ondragleave={on_dragleave}
            ondrop={on_drop}
        >
            // Error banner
            if let ImportState::Error(error) = &**import_state {
                <div class="absolute top-0 left-0 right-0 z-10 m-4">
                    <div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-xl p-4 shadow-lg">
                        <div class="flex items-start gap-3">
                            <div class="w-8 h-8 rounded-lg bg-red-100 dark:bg-red-900/50 flex items-center justify-center flex-shrink-0">
                                <svg class="w-4 h-4 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <div class="flex-1 min-w-0">
                                <p class="text-sm font-semibold text-red-800 dark:text-red-200">
                                    { "Validation Error" }
                                </p>
                                <p class="text-sm text-red-600 dark:text-red-400 mt-1 font-mono whitespace-pre-wrap">
                                    { error }
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            }

            // Line numbers
            <div class="w-14 bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 py-4 overflow-hidden select-none flex-shrink-0">
                <div class="space-y-0">
                    { for (1..=line_count).map(|n| {
                        html! {
                            <div class="h-6 leading-6 text-right pr-4 text-xs font-mono text-gray-400 dark:text-gray-600">
                                { n }
                            </div>
                        }
                    }) }
                </div>
            </div>

            // Editor
            <div class="flex-1 relative">
                <textarea
                    class="absolute inset-0 w-full h-full p-4 bg-white dark:bg-gray-900 text-sm font-mono text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-600 focus:outline-none resize-none leading-6"
                    placeholder="# Paste your .skill-engine.toml content here

version = \"1\"

[skills.my-skill]
source = \"github:username/my-skill\"
description = \"My awesome skill\"

[skills.my-skill.instances.default]
config.api_key = \"${API_KEY}\""
                    value={(**content).clone()}
                    oninput={on_content_input}
                    spellcheck="false"
                />
            </div>

            // Drag overlay
            if **is_dragging {
                <div class="absolute inset-0 z-20 flex items-center justify-center bg-primary-500/10 dark:bg-primary-500/20 backdrop-blur-sm border-2 border-dashed border-primary-500 rounded-lg m-2">
                    <div class="text-center">
                        <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-500/20 flex items-center justify-center mb-4">
                            <svg class="w-8 h-8 text-primary-600 dark:text-primary-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                            </svg>
                        </div>
                        <p class="text-lg font-semibold text-primary-700 dark:text-primary-300">
                            { "Drop your file here" }
                        </p>
                        <p class="text-sm text-primary-600/70 dark:text-primary-400/70 mt-1">
                            { "Supports .toml and .txt files" }
                        </p>
                    </div>
                </div>
            }
        </div>
    }
}

/// Render the templates view
fn render_templates_view(
    selected_template: &UseStateHandle<Option<String>>,
    on_select_template: Callback<(String, String)>,
) -> Html {
    html! {
        <div class="h-full overflow-y-auto p-6 bg-gray-50/50 dark:bg-gray-950/50">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                { for EXAMPLE_TEMPLATES.iter().map(|template| {
                    let is_selected = selected_template.as_ref().map(|s| s.as_str()) == Some(template.id);
                    let on_click = on_select_template.clone();
                    let id = template.id.to_string();
                    let content = template.content.to_string();

                    let badge_classes = match template.badge_color {
                        "green" => "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400",
                        "blue" => "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400",
                        "purple" => "bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400",
                        "amber" => "bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400",
                        "indigo" => "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-400",
                        "rose" => "bg-rose-100 dark:bg-rose-900/30 text-rose-700 dark:text-rose-400",
                        _ => "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-400",
                    };

                    html! {
                        <button
                            onclick={Callback::from(move |_| on_click.emit((id.clone(), content.clone())))}
                            class={classes!(
                                "group", "relative", "p-5", "rounded-xl", "border-2", "text-left", "transition-all", "hover:shadow-lg",
                                if is_selected {
                                    "border-primary-500 bg-primary-50 dark:bg-primary-900/20 shadow-lg shadow-primary-500/10"
                                } else {
                                    "border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 hover:border-gray-300 dark:hover:border-gray-600"
                                }
                            )}
                        >
                            // Selected indicator
                            if is_selected {
                                <div class="absolute top-3 right-3">
                                    <div class="w-6 h-6 rounded-full bg-primary-500 flex items-center justify-center">
                                        <svg class="w-4 h-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                                        </svg>
                                    </div>
                                </div>
                            }

                            <div class="flex items-start gap-4">
                                <div class={classes!(
                                    "w-12", "h-12", "rounded-xl", "flex", "items-center", "justify-center", "flex-shrink-0", "transition-transform", "group-hover:scale-110",
                                    match template.badge_color {
                                        "green" => "bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400",
                                        "blue" => "bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400",
                                        "purple" => "bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400",
                                        "amber" => "bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400",
                                        "indigo" => "bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400",
                                        "rose" => "bg-rose-100 dark:bg-rose-900/30 text-rose-600 dark:text-rose-400",
                                        _ => "bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400",
                                    }
                                )}>
                                    { render_template_icon(template.id) }
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2 mb-1">
                                        <h3 class="text-sm font-semibold text-gray-900 dark:text-white">
                                            { template.name }
                                        </h3>
                                        <span class={classes!("text-[10px]", "font-bold", "uppercase", "px-2", "py-0.5", "rounded-full", badge_classes)}>
                                            { template.badge }
                                        </span>
                                    </div>
                                    <p class="text-sm text-gray-500 dark:text-gray-400">
                                        { template.description }
                                    </p>
                                </div>
                            </div>

                            // Preview snippet
                            <div class="mt-4 p-3 rounded-lg bg-gray-50 dark:bg-gray-800/50 border border-gray-100 dark:border-gray-700/50">
                                <pre class="text-xs font-mono text-gray-600 dark:text-gray-400 overflow-hidden whitespace-pre-wrap line-clamp-3">
                                    { template.content.lines().take(4).collect::<Vec<_>>().join("\n") }
                                </pre>
                            </div>
                        </button>
                    }
                }) }
            </div>
        </div>
    }
}

/// Render template icon
fn render_template_icon(id: &str) -> Html {
    match id {
        "quick-start" => html! {
            // Rocket/play icon
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
        },
        "docker-runners" => html! {
            // Docker whale icon
            <svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
                <path d="M13 3h2v2h-2V3zm3 0h2v2h-2V3zm-6 0h2v2H10V3zM7 3h2v2H7V3zm0 3h2v2H7V6zm3 0h2v2h-2V6zm3 0h2v2h-2V6zm3 0h2v2h-2V6zm3 0h2v2h-2V6zM4 9h2v2H4V9zm3 0h2v2H7V9zm3 0h2v2h-2V9zm3 0h2v2h-2V9zm3 0h2v2h-2V9zm3 0h2v2h-2V9zm2.5 3c-.4 0-.8.1-1.2.2-.4-1.2-1.5-2-2.8-2H3c-1.1 0-2 .9-2 2v3c0 2.2 1.8 4 4 4h12c2.8 0 5-2.2 5-5 0-1.4-.6-2.8-1.5-3.8-.5-.3-1-.4-1.5-.4z"/>
            </svg>
        },
        "devops" => html! {
            // Terminal/CLI icon
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
        },
        "databases" => html! {
            // Database icon
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
            </svg>
        },
        "wasm-api" => html! {
            // API/cloud icon
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
            </svg>
        },
        "complete" => html! {
            // Stack/all icon
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
        },
        _ => html! {
            <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
        },
    }
}

/// Render the preview view
fn render_preview_view(
    skills: &[ParsedSkill],
    warnings: &UseStateHandle<Vec<String>>,
    merge_mode: &UseStateHandle<bool>,
    on_toggle_merge: Callback<MouseEvent>,
) -> Html {
    html! {
        <div class="flex-1 overflow-y-auto p-6">
            <div class="max-w-2xl mx-auto space-y-6">
                // Success banner
                <div class="text-center py-6">
                    <div class="w-16 h-16 mx-auto rounded-2xl bg-gradient-to-br from-green-400 to-emerald-500 flex items-center justify-center shadow-lg shadow-green-500/30 mb-4">
                        <svg class="w-8 h-8 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                    </div>
                    <h3 class="text-xl font-bold text-gray-900 dark:text-white">
                        { "Manifest Validated" }
                    </h3>
                    <p class="text-gray-500 dark:text-gray-400 mt-1">
                        { format!("Found {} skill{} ready to import", skills.len(), if skills.len() == 1 { "" } else { "s" }) }
                    </p>
                </div>

                // Warnings
                if !warnings.is_empty() {
                    <div class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-4">
                        <div class="flex items-start gap-3">
                            <div class="w-8 h-8 rounded-lg bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center flex-shrink-0">
                                <svg class="w-4 h-4 text-amber-600 dark:text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-amber-800 dark:text-amber-200">
                                    { "Warnings" }
                                </p>
                                <ul class="text-sm text-amber-700 dark:text-amber-300 mt-1 space-y-1">
                                    { for warnings.iter().map(|w| html! { <li class="flex items-start gap-2"><span class="text-amber-400">{ "•" }</span>{ w }</li> }) }
                                </ul>
                            </div>
                        </div>
                    </div>
                }

                // Merge mode toggle
                <div class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl border border-gray-200 dark:border-gray-700">
                    <div>
                        <p class="text-sm font-semibold text-gray-900 dark:text-white">
                            { "Merge with existing" }
                        </p>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            { if **merge_mode { "Keep existing skills, add new ones" } else { "Replace all with imported" } }
                        </p>
                    </div>
                    <button
                        type="button"
                        role="switch"
                        aria-checked={(**merge_mode).to_string()}
                        onclick={on_toggle_merge}
                        class={classes!(
                            "relative", "inline-flex", "h-7", "w-12", "flex-shrink-0",
                            "cursor-pointer", "rounded-full", "border-2", "border-transparent",
                            "transition-colors", "duration-200", "ease-in-out",
                            "focus:outline-none", "focus:ring-2", "focus:ring-primary-500", "focus:ring-offset-2",
                            if **merge_mode { "bg-primary-500" } else { "bg-gray-200 dark:bg-gray-600" }
                        )}
                    >
                        <span
                            class={classes!(
                                "pointer-events-none", "inline-block", "h-6", "w-6",
                                "transform", "rounded-full", "bg-white", "shadow-lg",
                                "ring-0", "transition", "duration-200", "ease-in-out",
                                if **merge_mode { "translate-x-5" } else { "translate-x-0" }
                            )}
                        />
                    </button>
                </div>

                // Skills list
                <div class="space-y-3">
                    { for skills.iter().map(render_skill_card) }
                </div>
            </div>
        </div>
    }
}

/// Render a skill card
fn render_skill_card(skill: &ParsedSkill) -> Html {
    let (badge_text, badge_class) = match skill.runtime.as_str() {
        "docker" => ("Docker", "bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400"),
        "native" => ("Native", "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400"),
        _ => ("WASM", "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400"),
    };

    html! {
        <div class="p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 hover:shadow-md transition-shadow">
            <div class="flex items-start justify-between">
                <div class="flex items-start gap-3">
                    <div class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center flex-shrink-0">
                        <svg class="w-5 h-5 text-gray-500 dark:text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                        </svg>
                    </div>
                    <div class="min-w-0">
                        <div class="flex items-center gap-2">
                            <h4 class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                                { &skill.name }
                            </h4>
                            <span class={classes!("text-[10px]", "font-bold", "uppercase", "px-2", "py-0.5", "rounded-full", badge_class)}>
                                { badge_text }
                            </span>
                        </div>
                        if let Some(ref desc) = skill.description {
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                { desc }
                            </p>
                        }
                        <p class="text-xs text-gray-400 dark:text-gray-500 mt-1 font-mono">
                            { &skill.source }
                        </p>
                    </div>
                </div>
                <div class="text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">
                    { format!("{} inst.", skill.instances.len()) }
                </div>
            </div>
        </div>
    }
}

/// Render the importing view
fn render_importing_view() -> Html {
    html! {
        <div class="flex-1 flex flex-col items-center justify-center p-6">
            <div class="relative">
                <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-primary-400 to-primary-600 flex items-center justify-center shadow-xl shadow-primary-500/30">
                    <svg class="w-10 h-10 text-white animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                    </svg>
                </div>
                <div class="absolute inset-0 rounded-2xl border-4 border-primary-500/30 animate-ping"></div>
            </div>
            <h3 class="mt-6 text-xl font-bold text-gray-900 dark:text-white">
                { "Importing Skills..." }
            </h3>
            <p class="text-gray-500 dark:text-gray-400 mt-1">
                { "This may take a moment" }
            </p>
        </div>
    }
}

/// Render the complete view
fn render_complete_view(count: usize) -> Html {
    html! {
        <div class="flex-1 flex flex-col items-center justify-center p-6">
            <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-green-400 to-emerald-500 flex items-center justify-center shadow-xl shadow-green-500/30">
                <svg class="w-10 h-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
            </div>
            <h3 class="mt-6 text-xl font-bold text-gray-900 dark:text-white">
                { "Import Complete!" }
            </h3>
            <p class="text-gray-500 dark:text-gray-400 mt-1">
                { format!("Successfully imported {} skill{}", count, if count == 1 { "" } else { "s" }) }
            </p>
        </div>
    }
}

/// Read file content using FileReader API
fn read_file_content(file: File, content: UseStateHandle<String>) {
    let reader = FileReader::new().unwrap();
    let reader_clone = reader.clone();

    let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
        if let Ok(result) = reader_clone.result() {
            if let Some(text) = result.as_string() {
                content.set(text);
            }
        }
    }) as Box<dyn FnMut(_)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader.read_as_text(&file).unwrap();
    onload.forget();
}

/// Hook to open the import config modal
#[hook]
pub fn use_import_config_modal() -> UseImportConfigModalHandle {
    let (_, dispatch) = use_store::<UiStore>();
    UseImportConfigModalHandle { dispatch }
}

/// Handle for the import config modal hook
pub struct UseImportConfigModalHandle {
    dispatch: Dispatch<UiStore>,
}

impl UseImportConfigModalHandle {
    pub fn open(&self) {
        self.dispatch.apply(UiAction::OpenModal(ModalType::Import, None));
    }
}

impl Clone for UseImportConfigModalHandle {
    fn clone(&self) -> Self {
        Self {
            dispatch: self.dispatch.clone(),
        }
    }
}
