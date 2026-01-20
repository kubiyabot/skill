//! Side navigation component

use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;
use super::icons::{AnalyticsIcon, DashboardIcon, SkillsIcon, PlayIcon, HistoryIcon, SettingsIcon, SearchIcon};

/// Navigation item structure
struct NavItem {
    route: Route,
    label: &'static str,
    icon: fn(&'static str) -> Html,
}

/// Sidebar navigation component
#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let route = use_route::<Route>();

    let nav_items = [NavItem {
            route: Route::Dashboard,
            label: "Dashboard",
            icon: |class| html! { <DashboardIcon class={class} /> },
        },
        NavItem {
            route: Route::Skills,
            label: "Skills",
            icon: |class| html! { <SkillsIcon class={class} /> },
        },
        NavItem {
            route: Route::Run,
            label: "Run",
            icon: |class| html! { <PlayIcon class={class} /> },
        },
        NavItem {
            route: Route::History,
            label: "History",
            icon: |class| html! { <HistoryIcon class={class} /> },
        },
        NavItem {
            route: Route::SearchTest,
            label: "Search Test",
            icon: |class| html! { <SearchIcon class={class} /> },
        },
        NavItem {
            route: Route::Analytics,
            label: "Analytics",
            icon: |class| html! { <AnalyticsIcon class={class} /> },
        },
        NavItem {
            route: Route::Settings,
            label: "Settings",
            icon: |class| html! { <SettingsIcon class={class} /> },
        }];

    html! {
        <aside class="fixed left-0 top-16 bottom-0 w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-y-auto z-30">
            <nav class="p-4 space-y-1">
                { for nav_items.iter().map(|item| {
                    let is_active = route.as_ref().map(|r| is_route_match(r, &item.route)).unwrap_or(false);
                    let class = if is_active { "nav-link-active" } else { "nav-link" };

                    html! {
                        <Link<Route> to={item.route.clone()} classes={class}>
                            { (item.icon)("w-5 h-5") }
                            <span>{ item.label }</span>
                        </Link<Route>>
                    }
                }) }
            </nav>

            // Bottom section with quick actions
            <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-200 dark:border-gray-700">
                <Link<Route>
                    to={Route::Run}
                    classes="btn btn-primary w-full"
                >
                    <PlayIcon class="w-4 h-4 mr-2" />
                    { "Run Skill" }
                </Link<Route>>
            </div>
        </aside>
    }
}

/// Check if the current route matches the nav item route
fn is_route_match(current: &Route, target: &Route) -> bool {
    match (current, target) {
        (Route::Dashboard, Route::Dashboard) => true,
        (Route::Skills, Route::Skills) => true,
        (Route::SkillDetail { .. }, Route::Skills) => true,
        (Route::SkillInstance { .. }, Route::Skills) => true,
        (Route::Run, Route::Run) => true,
        (Route::RunSkill { .. }, Route::Run) => true,
        (Route::RunSkillTool { .. }, Route::Run) => true,
        (Route::History, Route::History) => true,
        (Route::HistoryDetail { .. }, Route::History) => true,
        (Route::SearchTest, Route::SearchTest) => true,
        (Route::Analytics, Route::Analytics) => true,
        (Route::Settings, Route::Settings) => true,
        _ => current == target,
    }
}
