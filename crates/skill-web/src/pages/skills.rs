//! Skills browser page with search and filtering

use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::api::{Api, SkillSummary as ApiSkillSummary};
use crate::components::card::Card;
use crate::components::icons::{PlusIcon, SearchIcon, SkillsIcon, PlayIcon};
use crate::components::{
    ImportConfigModal, InstallSkillModal, use_import_config_modal, use_install_skill_modal,
};
use crate::router::Route;
use crate::store::skills::{
    SkillRuntime, SkillSortBy, SkillStatus, SkillSummary, SkillsAction, SkillsStore,
};

/// Convert API skill summary to store skill summary
fn api_to_store_skill(api: ApiSkillSummary) -> SkillSummary {
    SkillSummary {
        name: api.name,
        version: api.version,
        description: api.description,
        source: api.source,
        runtime: match api.runtime.as_str() {
            "docker" => SkillRuntime::Docker,
            "native" => SkillRuntime::Native,
            _ => SkillRuntime::Wasm,
        },
        tools_count: api.tools_count,
        instances_count: api.instances_count,
        status: SkillStatus::Configured,
        last_used: api.last_used,
        execution_count: api.execution_count,
    }
}

/// Source filter options
#[derive(Clone, PartialEq, Default)]
pub enum SourceFilter {
    #[default]
    All,
    GitHub,
    Local,
    Http,
}

impl SourceFilter {
    fn matches(&self, source: &str) -> bool {
        match self {
            SourceFilter::All => true,
            SourceFilter::GitHub => {
                source.starts_with("github:") || source.contains("github.com")
            }
            SourceFilter::Local => source.starts_with("local:") || source.starts_with("./"),
            SourceFilter::Http => source.starts_with("http://") || source.starts_with("https://"),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            SourceFilter::All => "All Sources",
            SourceFilter::GitHub => "GitHub",
            SourceFilter::Local => "Local",
            SourceFilter::Http => "HTTP",
        }
    }
}

/// Status filter options
#[derive(Clone, PartialEq, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Configured,
    Unconfigured,
    Error,
}

impl StatusFilter {
    fn matches(&self, status: &SkillStatus) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::Configured => matches!(status, SkillStatus::Configured),
            StatusFilter::Unconfigured => matches!(status, SkillStatus::Unconfigured),
            StatusFilter::Error => matches!(status, SkillStatus::Error),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            StatusFilter::All => "All Status",
            StatusFilter::Configured => "Configured",
            StatusFilter::Unconfigured => "Unconfigured",
            StatusFilter::Error => "Error",
        }
    }
}

/// Skills browser page component
#[function_component(SkillsPage)]
pub fn skills_page() -> Html {
    let store = use_store_value::<SkillsStore>();
    let dispatch = use_dispatch::<SkillsStore>();

    // Local filter states
    let search_query = use_state(String::new);
    let source_filter = use_state(SourceFilter::default);
    let status_filter = use_state(StatusFilter::default);
    let sort_by = use_state(|| SkillSortBy::Name);
    let sort_ascending = use_state(|| true);

    // Install skill modal
    let install_modal = use_install_skill_modal();

    // Import config modal
    let import_modal = use_import_config_modal();

    // Create API client
    let api = use_memo((), |_| Rc::new(Api::new()));

    // Load data on mount
    {
        let api = api.clone();
        let dispatch = dispatch.clone();

        use_effect_with((), move |_| {
            dispatch.apply(SkillsAction::SetLoading(true));

            let api = api.clone();
            let dispatch = dispatch.clone();

            spawn_local(async move {
                match api.skills.list_all().await {
                    Ok(skills) => {
                        let store_skills: Vec<SkillSummary> =
                            skills.into_iter().map(api_to_store_skill).collect();
                        dispatch.apply(SkillsAction::SetSkills(store_skills));
                    }
                    Err(e) => {
                        dispatch.apply(SkillsAction::SetError(Some(e.to_string())));
                    }
                }
            });
        });
    }

    // Filter and sort skills
    let filtered_skills: Vec<&SkillSummary> = {
        let query = (*search_query).to_lowercase();
        let source_f = (*source_filter).clone();
        let status_f = (*status_filter).clone();
        let sort = (*sort_by).clone();
        let ascending = *sort_ascending;

        let mut skills: Vec<&SkillSummary> = store
            .skills
            .iter()
            .filter(|skill| {
                // Search filter
                if !query.is_empty() {
                    let matches_name = skill.name.to_lowercase().contains(&query);
                    let matches_desc = skill.description.to_lowercase().contains(&query);
                    if !matches_name && !matches_desc {
                        return false;
                    }
                }

                // Source filter
                if !source_f.matches(&skill.source) {
                    return false;
                }

                // Status filter
                if !status_f.matches(&skill.status) {
                    return false;
                }

                true
            })
            .collect();

        // Sort
        skills.sort_by(|a, b| {
            let cmp = match sort {
                SkillSortBy::Name => a.name.cmp(&b.name),
                SkillSortBy::LastUsed => a.last_used.cmp(&b.last_used),
                SkillSortBy::ExecutionCount => a.execution_count.cmp(&b.execution_count),
                SkillSortBy::ToolsCount => a.tools_count.cmp(&b.tools_count),
            };
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        skills
    };

    // Event handlers
    let on_search = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    let on_source_filter = {
        let source_filter = source_filter.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let filter = match select.value().as_str() {
                "github" => SourceFilter::GitHub,
                "local" => SourceFilter::Local,
                "http" => SourceFilter::Http,
                _ => SourceFilter::All,
            };
            source_filter.set(filter);
        })
    };

    let on_status_filter = {
        let status_filter = status_filter.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let filter = match select.value().as_str() {
                "configured" => StatusFilter::Configured,
                "unconfigured" => StatusFilter::Unconfigured,
                "error" => StatusFilter::Error,
                _ => StatusFilter::All,
            };
            status_filter.set(filter);
        })
    };

    let on_sort = {
        let sort_by = sort_by.clone();
        let sort_ascending = sort_ascending.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let (new_sort, ascending) = match select.value().as_str() {
                "name_asc" => (SkillSortBy::Name, true),
                "name_desc" => (SkillSortBy::Name, false),
                "last_used" => (SkillSortBy::LastUsed, false),
                "executions" => (SkillSortBy::ExecutionCount, false),
                "tools" => (SkillSortBy::ToolsCount, false),
                _ => (SkillSortBy::Name, true),
            };
            sort_by.set(new_sort);
            sort_ascending.set(ascending);
        })
    };

    let total_count = store.skills.len();
    let filtered_count = filtered_skills.len();
    let is_loading = store.loading;
    let error = store.error.clone();

    // Install skill button handlers
    let on_install_click = {
        let install_modal = install_modal.clone();
        Callback::from(move |_| {
            install_modal.open();
        })
    };

    let on_install_click_empty = {
        let install_modal = install_modal.clone();
        Callback::from(move |_| {
            install_modal.open();
        })
    };

    // Import config button handler
    let on_import_click = {
        let import_modal = import_modal.clone();
        Callback::from(move |_: MouseEvent| {
            import_modal.open();
        })
    };

    // Refresh skills list (shared helper)
    let refresh_skills = {
        let api = api.clone();
        let dispatch = dispatch.clone();
        move || {
            let api = api.clone();
            let dispatch = dispatch.clone();
            dispatch.apply(SkillsAction::SetLoading(true));
            spawn_local(async move {
                match api.skills.list_all().await {
                    Ok(skills) => {
                        let store_skills: Vec<SkillSummary> =
                            skills.into_iter().map(api_to_store_skill).collect();
                        dispatch.apply(SkillsAction::SetSkills(store_skills));
                    }
                    Err(e) => {
                        dispatch.apply(SkillsAction::SetError(Some(e.to_string())));
                    }
                }
            });
        }
    };

    // Refresh skills list after installation
    let on_skill_installed = {
        let refresh_skills = refresh_skills.clone();
        Callback::from(move |_name: String| {
            refresh_skills();
        })
    };

    // Refresh skills list after import
    let on_config_imported = {
        let refresh_skills = refresh_skills.clone();
        Callback::from(move |_count: usize| {
            refresh_skills();
        })
    };

    html! {
        <>
            // Modals
            <InstallSkillModal on_installed={on_skill_installed} />
            <ImportConfigModal on_imported={on_config_imported} />

            <div class="space-y-6 animate-fade-in">

            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                        { "Skills" }
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 mt-1">
                        if is_loading {
                            { "Loading skills..." }
                        } else if filtered_count != total_count {
                            { format!("Showing {} of {} skills", filtered_count, total_count) }
                        } else {
                            { format!("{} skills installed", total_count) }
                        }
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <button class="btn btn-secondary" onclick={on_import_click}>
                        <svg class="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                        </svg>
                        { "Import Config" }
                    </button>
                    <button class="btn btn-primary" onclick={on_install_click}>
                        <PlusIcon class="w-4 h-4 mr-2" />
                        { "Install Skill" }
                    </button>
                </div>
            </div>

            // Error alert
            if let Some(err) = error {
                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                    <div class="flex items-center gap-3">
                        <svg class="w-5 h-5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <p class="text-sm text-red-700 dark:text-red-300">{ err }</p>
                    </div>
                </div>
            }

            // Search and filters
            <Card>
                <div class="flex flex-col md:flex-row gap-4">
                    <div class="flex-1 relative">
                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <SearchIcon class="w-5 h-5 text-gray-400" />
                        </div>
                        <input
                            type="text"
                            placeholder="Search skills by name or description..."
                            class="input pl-10"
                            value={(*search_query).clone()}
                            oninput={on_search}
                        />
                    </div>
                    <div class="flex gap-2 flex-wrap">
                        <select class="input w-auto" onchange={on_source_filter}>
                            <option value="all" selected={*source_filter == SourceFilter::All}>
                                { SourceFilter::All.label() }
                            </option>
                            <option value="github" selected={*source_filter == SourceFilter::GitHub}>
                                { SourceFilter::GitHub.label() }
                            </option>
                            <option value="local" selected={*source_filter == SourceFilter::Local}>
                                { SourceFilter::Local.label() }
                            </option>
                            <option value="http" selected={*source_filter == SourceFilter::Http}>
                                { SourceFilter::Http.label() }
                            </option>
                        </select>
                        <select class="input w-auto" onchange={on_status_filter}>
                            <option value="all" selected={*status_filter == StatusFilter::All}>
                                { StatusFilter::All.label() }
                            </option>
                            <option value="configured" selected={*status_filter == StatusFilter::Configured}>
                                { StatusFilter::Configured.label() }
                            </option>
                            <option value="unconfigured" selected={*status_filter == StatusFilter::Unconfigured}>
                                { StatusFilter::Unconfigured.label() }
                            </option>
                            <option value="error" selected={*status_filter == StatusFilter::Error}>
                                { StatusFilter::Error.label() }
                            </option>
                        </select>
                        <select class="input w-auto" onchange={on_sort}>
                            <option value="name_asc">{ "Name (A-Z)" }</option>
                            <option value="name_desc">{ "Name (Z-A)" }</option>
                            <option value="last_used">{ "Last Used" }</option>
                            <option value="executions">{ "Most Executions" }</option>
                            <option value="tools">{ "Most Tools" }</option>
                        </select>
                    </div>
                </div>
            </Card>

            // Loading state
            if is_loading {
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    { for (0..4).map(|_| html! { <SkillCardSkeleton /> }) }
                </div>
            } else if filtered_skills.is_empty() {
                // Empty state
                <div class="text-center py-12">
                    <SkillsIcon class="w-12 h-12 mx-auto text-gray-400" />
                    <h3 class="mt-4 text-lg font-medium text-gray-900 dark:text-white">
                        if total_count == 0 {
                            { "No skills installed" }
                        } else {
                            { "No skills match your filters" }
                        }
                    </h3>
                    <p class="mt-2 text-gray-500 dark:text-gray-400">
                        if total_count == 0 {
                            { "Install your first skill to get started" }
                        } else {
                            { "Try adjusting your search or filters" }
                        }
                    </p>
                    if total_count == 0 {
                        <button class="btn btn-primary mt-4" onclick={on_install_click_empty}>
                            <PlusIcon class="w-4 h-4 mr-2" />
                            { "Install Skill" }
                        </button>
                    }
                </div>
            } else {
                // Skills grid
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    { for filtered_skills.iter().map(|skill| html! { <SkillCard skill={(*skill).clone()} /> }) }
                </div>
            }
            </div>
        </>
    }
}

/// Skill card props
#[derive(Properties, PartialEq)]
struct SkillCardProps {
    skill: SkillSummary,
}

/// Skill card component
#[function_component(SkillCard)]
fn skill_card(props: &SkillCardProps) -> Html {
    let skill = &props.skill;

    let (status_badge, status_dot) = match skill.status {
        SkillStatus::Configured => (
            html! { <span class="badge badge-success">{ "Configured" }</span> },
            "status-dot-success",
        ),
        SkillStatus::Unconfigured => (
            html! { <span class="badge badge-warning">{ "Unconfigured" }</span> },
            "status-dot-warning",
        ),
        SkillStatus::Error => (
            html! { <span class="badge badge-error">{ "Error" }</span> },
            "status-dot-error",
        ),
        SkillStatus::Loading => (
            html! { <span class="badge badge-info">{ "Loading" }</span> },
            "status-dot-info",
        ),
    };

    let runtime_badge = match skill.runtime {
        SkillRuntime::Wasm => html! { <span class="text-xs px-2 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded">{ "WASM" }</span> },
        SkillRuntime::Docker => html! { <span class="text-xs px-2 py-0.5 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded">{ "Docker" }</span> },
        SkillRuntime::Native => html! { <span class="text-xs px-2 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded">{ "Native" }</span> },
    };

    // Format last used
    let _last_used_str = skill
        .last_used
        .as_ref()
        .map(|s| {
            if s.len() > 10 {
                s[..10].to_string()
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "Never".to_string());

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 shadow-sm hover:shadow-md transition-shadow">
            <div class="p-6">
                <div class="flex items-start justify-between">
                    <div class="flex items-center gap-3">
                        <span class={classes!("status-dot", status_dot)} />
                        <div>
                            <Link<Route>
                                to={Route::SkillDetail { name: skill.name.clone() }}
                                classes="text-lg font-semibold text-gray-900 dark:text-white hover:text-primary-600 dark:hover:text-primary-400"
                            >
                                { &skill.name }
                            </Link<Route>>
                            <div class="flex items-center gap-2 mt-0.5">
                                <span class="text-xs text-gray-500 dark:text-gray-400 font-mono">{ format!("v{}", &skill.version) }</span>
                                { runtime_badge }
                            </div>
                        </div>
                    </div>
                    { status_badge }
                </div>

                <p class="mt-4 text-sm text-gray-600 dark:text-gray-300 line-clamp-2 h-10">
                    { &skill.description }
                </p>

                <div class="mt-4 flex items-center justify-between pt-4 border-t border-gray-100 dark:border-gray-800">
                     <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                        <span title="Tools">{ format!("{} tools", skill.tools_count) }</span>
                        <span>{ "•" }</span>
                        <span title="Process Count">{ format!("{} instances", skill.instances_count) }</span>
                    </div>

                    <Link<Route>
                        to={Route::RunSkill { skill: skill.name.clone() }}
                        classes="btn btn-sm btn-primary flex items-center gap-1.5"
                    >
                        <PlayIcon class="w-3 h-3" />
                        { "Run" }
                    </Link<Route>>
                </div>
            </div>
        </div>
    }
}

/// Skeleton loader for skill cards
#[function_component(SkillCardSkeleton)]
fn skill_card_skeleton() -> Html {
    html! {
        <div class="card p-6 animate-pulse">
            <div class="flex items-start justify-between">
                <div class="flex items-center gap-3">
                    <div class="w-3 h-3 bg-gray-200 dark:bg-gray-700 rounded-full"></div>
                    <div>
                        <div class="h-5 w-32 bg-gray-200 dark:bg-gray-700 rounded"></div>
                        <div class="h-3 w-16 bg-gray-200 dark:bg-gray-700 rounded mt-1"></div>
                    </div>
                </div>
                <div class="h-6 w-20 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
            <div class="mt-3 space-y-2">
                <div class="h-4 w-full bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-4 w-2/3 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
            <div class="mt-2 h-3 w-48 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div class="mt-4 flex items-center gap-4">
                <div class="h-4 w-16 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-4 w-20 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
        </div>
    }
}
