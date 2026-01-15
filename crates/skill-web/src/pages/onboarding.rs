//! Onboarding wizard page

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::icons::{CheckIcon, ChevronRightIcon};
use crate::router::Route;

/// Onboarding page props
#[derive(Properties, PartialEq)]
pub struct OnboardingPageProps {
    #[prop_or("welcome".to_string())]
    pub step: String,
}

/// Onboarding page component
#[function_component(OnboardingPage)]
pub fn onboarding_page(props: &OnboardingPageProps) -> Html {
    let steps = vec![
        ("welcome", "Welcome"),
        ("search", "Search"),
        ("credentials", "Credentials"),
        ("skills", "Skills"),
        ("complete", "Complete"),
    ];

    let current_step_idx = steps.iter().position(|(id, _)| *id == props.step).unwrap_or(0);

    html! {
        <div class="min-h-screen bg-gradient-to-br from-primary-900 to-primary-950 flex flex-col">
            // Header
            <header class="p-6">
                <div class="flex items-center gap-3">
                    <span class="text-3xl">{ "⚡" }</span>
                    <span class="text-xl font-semibold text-white">{ "Skill Engine" }</span>
                </div>
            </header>

            // Progress indicator
            <div class="px-6 py-4">
                <div class="max-w-2xl mx-auto">
                    <div class="flex items-center justify-between">
                        { for steps.iter().enumerate().map(|(i, (id, label))| {
                            let is_complete = i < current_step_idx;
                            let is_current = i == current_step_idx;

                            html! {
                                <>
                                    <div class="flex flex-col items-center">
                                        <div class={classes!(
                                            "w-10", "h-10", "rounded-full", "flex", "items-center", "justify-center", "font-medium", "transition-colors",
                                            if is_complete {
                                                "bg-success-500 text-white"
                                            } else if is_current {
                                                "bg-white text-primary-900"
                                            } else {
                                                "bg-primary-800 text-primary-400"
                                            }
                                        )}>
                                            if is_complete {
                                                <CheckIcon class="w-5 h-5" />
                                            } else {
                                                { (i + 1).to_string() }
                                            }
                                        </div>
                                        <span class={classes!(
                                            "mt-2", "text-xs", "font-medium",
                                            if is_current { "text-white" } else { "text-primary-400" }
                                        )}>
                                            { *label }
                                        </span>
                                    </div>
                                    if i < steps.len() - 1 {
                                        <div class={classes!(
                                            "flex-1", "h-1", "mx-2", "rounded",
                                            if is_complete { "bg-success-500" } else { "bg-primary-800" }
                                        )} />
                                    }
                                </>
                            }
                        }) }
                    </div>
                </div>
            </div>

            // Content area
            <main class="flex-1 flex items-center justify-center p-6">
                <div class="w-full max-w-2xl">
                    {
                        match props.step.as_str() {
                            "welcome" => html! { <WelcomeStep /> },
                            "search" => html! { <SearchStep /> },
                            "credentials" => html! { <CredentialsStep /> },
                            "skills" => html! { <SkillsStep /> },
                            "complete" => html! { <CompleteStep /> },
                            _ => html! { <WelcomeStep /> },
                        }
                    }
                </div>
            </main>
        </div>
    }
}

/// Welcome step component
#[function_component(WelcomeStep)]
fn welcome_step() -> Html {
    let navigator = use_navigator().unwrap();

    let on_start = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "search".to_string() });
        })
    };

    let on_skip = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Dashboard);
        })
    };

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8 text-center">
            <div class="text-6xl mb-6">{ "⚡" }</div>
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-4">
                { "Welcome to Skill Engine" }
            </h1>
            <p class="text-lg text-gray-600 dark:text-gray-300 mb-8 max-w-md mx-auto">
                { "Give your AI agents superpowers with sandboxed WASM skill execution" }
            </p>

            <div class="space-y-3 text-left max-w-sm mx-auto mb-8">
                { for [
                    "Search Pipeline - How skills are discovered",
                    "AI Integration - Connect LLM providers",
                    "Starter Skills - Get productive immediately",
                    "Claude Code - Seamless integration",
                ].iter().map(|item| html! {
                    <div class="flex items-center gap-3">
                        <CheckIcon class="w-5 h-5 text-success-500 flex-shrink-0" />
                        <span class="text-gray-700 dark:text-gray-300">{ *item }</span>
                    </div>
                }) }
            </div>

            <p class="text-sm text-gray-500 mb-6">
                { "Estimated time: 3-5 minutes" }
            </p>

            <div class="flex flex-col gap-3">
                <button class="btn btn-primary w-full justify-center" onclick={on_start}>
                    { "Get Started" }
                    <ChevronRightIcon class="w-4 h-4 ml-2" />
                </button>
                <button class="btn btn-ghost w-full justify-center text-gray-500" onclick={on_skip}>
                    { "Skip to Dashboard" }
                </button>
            </div>
        </div>
    }
}

/// Search setup step component
#[function_component(SearchStep)]
fn search_step() -> Html {
    let navigator = use_navigator().unwrap();
    let embedding_provider = use_state(|| "fastembed".to_string());
    let vector_store = use_state(|| "inmemory".to_string());

    let on_next = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "credentials".to_string() });
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "welcome".to_string() });
        })
    };

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                { "Search Pipeline" }
            </h2>
            <p class="text-gray-600 dark:text-gray-300 mb-8">
                { "Choose how skills are discovered and searched" }
            </p>

            <div class="space-y-6">
                // Embedding provider
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                        { "Embedding Provider" }
                    </label>
                    <div class="space-y-3">
                        { for [
                            ("fastembed", "FastEmbed (Recommended)", "Local, offline, no API keys required"),
                            ("openai", "OpenAI", "Cloud-based, requires API key"),
                            ("ollama", "Ollama", "Self-hosted, requires Ollama installation"),
                        ].iter().map(|(value, label, desc)| {
                            let is_selected = *embedding_provider == *value;
                            let provider = embedding_provider.clone();
                            let val = value.to_string();
                            let onclick = Callback::from(move |_| provider.set(val.clone()));

                            html! {
                                <label class={classes!(
                                    "flex", "items-start", "gap-4", "p-4", "rounded-lg", "border", "cursor-pointer", "transition-colors",
                                    if is_selected {
                                        "border-primary-500 bg-primary-50 dark:bg-primary-900/30"
                                    } else {
                                        "border-gray-200 dark:border-gray-700 hover:border-gray-300"
                                    }
                                )}>
                                    <input
                                        type="radio"
                                        name="embedding"
                                        value={*value}
                                        checked={is_selected}
                                        onclick={onclick}
                                        class="mt-1"
                                    />
                                    <div>
                                        <span class="font-medium text-gray-900 dark:text-white">{ *label }</span>
                                        <p class="text-sm text-gray-500 mt-1">{ *desc }</p>
                                    </div>
                                </label>
                            }
                        }) }
                    </div>
                </div>

                // Vector store
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                        { "Vector Store" }
                    </label>
                    <div class="space-y-3">
                        { for [
                            ("inmemory", "In-Memory (Recommended)", "Great for development and small deployments"),
                            ("qdrant", "Qdrant", "Production-ready, requires Qdrant server"),
                        ].iter().map(|(value, label, desc)| {
                            let is_selected = *vector_store == *value;
                            let store = vector_store.clone();
                            let val = value.to_string();
                            let onclick = Callback::from(move |_| store.set(val.clone()));

                            html! {
                                <label class={classes!(
                                    "flex", "items-start", "gap-4", "p-4", "rounded-lg", "border", "cursor-pointer", "transition-colors",
                                    if is_selected {
                                        "border-primary-500 bg-primary-50 dark:bg-primary-900/30"
                                    } else {
                                        "border-gray-200 dark:border-gray-700 hover:border-gray-300"
                                    }
                                )}>
                                    <input
                                        type="radio"
                                        name="store"
                                        value={*value}
                                        checked={is_selected}
                                        onclick={onclick}
                                        class="mt-1"
                                    />
                                    <div>
                                        <span class="font-medium text-gray-900 dark:text-white">{ *label }</span>
                                        <p class="text-sm text-gray-500 mt-1">{ *desc }</p>
                                    </div>
                                </label>
                            }
                        }) }
                    </div>
                </div>
            </div>

            <div class="flex justify-between mt-8">
                <button class="btn btn-ghost" onclick={on_back}>{ "Back" }</button>
                <button class="btn btn-primary" onclick={on_next}>
                    { "Next" }
                    <ChevronRightIcon class="w-4 h-4 ml-2" />
                </button>
            </div>
        </div>
    }
}

/// Credentials step component
#[function_component(CredentialsStep)]
fn credentials_step() -> Html {
    let navigator = use_navigator().unwrap();

    let on_next = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "skills".to_string() });
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "search".to_string() });
        })
    };

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                { "Credentials" }
            </h2>
            <p class="text-gray-600 dark:text-gray-300 mb-8">
                { "Add API keys for enhanced features (optional)" }
            </p>

            <div class="space-y-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        { "OpenAI API Key" }
                        <span class="text-gray-400 ml-1">{ "(optional)" }</span>
                    </label>
                    <input
                        type="password"
                        class="input"
                        placeholder="sk-..."
                    />
                    <p class="text-xs text-gray-500 mt-1">
                        { "Used for OpenAI embeddings and skill enhancement" }
                    </p>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        { "Anthropic API Key" }
                        <span class="text-gray-400 ml-1">{ "(optional)" }</span>
                    </label>
                    <input
                        type="password"
                        class="input"
                        placeholder="sk-ant-..."
                    />
                    <p class="text-xs text-gray-500 mt-1">
                        { "Used for AI-powered skill enhancement" }
                    </p>
                </div>
            </div>

            <div class="flex justify-between mt-8">
                <button class="btn btn-ghost" onclick={on_back}>{ "Back" }</button>
                <button class="btn btn-primary" onclick={on_next}>
                    { "Next" }
                    <ChevronRightIcon class="w-4 h-4 ml-2" />
                </button>
            </div>
        </div>
    }
}

/// Skills step component
#[function_component(SkillsStep)]
fn skills_step() -> Html {
    let navigator = use_navigator().unwrap();

    let on_next = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "complete".to_string() });
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::OnboardingStep { step: "credentials".to_string() });
        })
    };

    let starter_skills = vec![
        ("kubernetes", "Kubernetes cluster management", true),
        ("github", "GitHub repository operations", true),
        ("docker", "Docker container management", false),
        ("aws", "AWS cloud services", false),
    ];

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                { "Starter Skills" }
            </h2>
            <p class="text-gray-600 dark:text-gray-300 mb-8">
                { "Select skills to install (you can add more later)" }
            </p>

            <div class="space-y-3">
                { for starter_skills.iter().map(|(name, desc, default)| {
                    html! {
                        <label class="flex items-start gap-4 p-4 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-gray-300 cursor-pointer transition-colors">
                            <input
                                type="checkbox"
                                checked={*default}
                                class="mt-1 rounded border-gray-300"
                            />
                            <div>
                                <span class="font-medium text-gray-900 dark:text-white">{ *name }</span>
                                <p class="text-sm text-gray-500 mt-1">{ *desc }</p>
                            </div>
                        </label>
                    }
                }) }
            </div>

            <div class="flex justify-between mt-8">
                <button class="btn btn-ghost" onclick={on_back}>{ "Back" }</button>
                <button class="btn btn-primary" onclick={on_next}>
                    { "Install & Continue" }
                    <ChevronRightIcon class="w-4 h-4 ml-2" />
                </button>
            </div>
        </div>
    }
}

/// Complete step component
#[function_component(CompleteStep)]
fn complete_step() -> Html {
    let navigator = use_navigator().unwrap();

    let on_finish = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Dashboard);
        })
    };

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8 text-center">
            <div class="w-16 h-16 bg-success-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
                <CheckIcon class="w-8 h-8 text-success-600 dark:text-green-400" />
            </div>
            <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                { "You're all set!" }
            </h2>
            <p class="text-gray-600 dark:text-gray-300 mb-8">
                { "Skill Engine is configured and ready to use" }
            </p>

            <div class="space-y-2 text-left max-w-sm mx-auto mb-8">
                <div class="flex items-center gap-3">
                    <CheckIcon class="w-5 h-5 text-success-500" />
                    <span class="text-gray-700 dark:text-gray-300">{ "Search pipeline configured" }</span>
                </div>
                <div class="flex items-center gap-3">
                    <CheckIcon class="w-5 h-5 text-success-500" />
                    <span class="text-gray-700 dark:text-gray-300">{ "2 skills installed" }</span>
                </div>
                <div class="flex items-center gap-3">
                    <CheckIcon class="w-5 h-5 text-success-500" />
                    <span class="text-gray-700 dark:text-gray-300">{ "Ready for Claude Code integration" }</span>
                </div>
            </div>

            <button class="btn btn-primary w-full justify-center" onclick={on_finish}>
                { "Go to Dashboard" }
                <ChevronRightIcon class="w-4 h-4 ml-2" />
            </button>
        </div>
    }
}
