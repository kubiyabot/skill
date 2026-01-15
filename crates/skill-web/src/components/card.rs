//! Card component for content containers

use yew::prelude::*;

/// Card component props
#[derive(Properties, PartialEq)]
pub struct CardProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub subtitle: Option<AttrValue>,
    #[prop_or_default]
    pub actions: Option<Html>,
    #[prop_or(false)]
    pub hoverable: bool,
}

/// Card component for grouping content
#[function_component(Card)]
pub fn card(props: &CardProps) -> Html {
    let base_class = if props.hoverable {
        "card-hover"
    } else {
        "card"
    };

    html! {
        <div class={classes!(base_class, props.class.clone())}>
            if props.title.is_some() || props.actions.is_some() {
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <div>
                        if let Some(title) = &props.title {
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                { title }
                            </h3>
                        }
                        if let Some(subtitle) = &props.subtitle {
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                                { subtitle }
                            </p>
                        }
                    </div>
                    if let Some(actions) = &props.actions {
                        <div class="flex items-center gap-2">
                            { actions.clone() }
                        </div>
                    }
                </div>
            }
            <div class="p-6">
                { for props.children.iter() }
            </div>
        </div>
    }
}

/// Simple stat card for dashboard
#[derive(Properties, PartialEq)]
pub struct StatCardProps {
    pub title: AttrValue,
    pub value: AttrValue,
    #[prop_or_default]
    pub subtitle: Option<AttrValue>,
    #[prop_or_default]
    pub icon: Option<Html>,
    #[prop_or_default]
    pub trend: Option<Trend>,
}

#[derive(Clone, PartialEq)]
pub enum Trend {
    Up(String),
    Down(String),
    Neutral(String),
}

/// Stat card component for displaying metrics
#[function_component(StatCard)]
pub fn stat_card(props: &StatCardProps) -> Html {
    html! {
        <div class="card p-6">
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-500 dark:text-gray-400">
                        { &props.title }
                    </p>
                    <p class="mt-2 text-3xl font-semibold text-gray-900 dark:text-white">
                        { &props.value }
                    </p>
                    if let Some(subtitle) = &props.subtitle {
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            { subtitle }
                        </p>
                    }
                    if let Some(trend) = &props.trend {
                        <div class="mt-2 flex items-center text-sm">
                            { render_trend(trend) }
                        </div>
                    }
                </div>
                if let Some(icon) = &props.icon {
                    <div class="p-3 bg-primary-50 dark:bg-primary-900/30 rounded-lg">
                        { icon.clone() }
                    </div>
                }
            </div>
        </div>
    }
}

fn render_trend(trend: &Trend) -> Html {
    match trend {
        Trend::Up(text) => html! {
            <span class="text-success-600 dark:text-green-400 flex items-center gap-1">
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
                </svg>
                { text }
            </span>
        },
        Trend::Down(text) => html! {
            <span class="text-error-600 dark:text-red-400 flex items-center gap-1">
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                </svg>
                { text }
            </span>
        },
        Trend::Neutral(text) => html! {
            <span class="text-gray-500 dark:text-gray-400">{ text }</span>
        },
    }
}
