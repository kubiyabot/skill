//! Application routing configuration
//!
//! Defines all routes and provides navigation utilities.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{
    analytics::AnalyticsPage,
    dashboard::DashboardPage,
    skills::SkillsPage,
    skill_detail::SkillDetailPage,
    run::RunPage,
    history::HistoryPage,
    settings::SettingsPage,
    search_test::SearchTestPage,
    onboarding::OnboardingPage,
    not_found::NotFoundPage,
};

/// Application routes
#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    /// Dashboard - main overview page
    #[at("/")]
    Dashboard,

    /// Skills browser - list and search skills
    #[at("/skills")]
    Skills,

    /// Skill detail - view specific skill
    #[at("/skills/:name")]
    SkillDetail { name: String },

    /// Skill instance configuration
    #[at("/skills/:name/instances/:instance")]
    SkillInstance { name: String, instance: String },

    /// Run interface - execute skill tools
    #[at("/run")]
    Run,

    /// Run with pre-selected skill
    #[at("/run/:skill")]
    RunSkill { skill: String },

    /// Run with pre-selected skill and tool
    #[at("/run/:skill/:tool")]
    RunSkillTool { skill: String, tool: String },

    /// Execution history
    #[at("/history")]
    History,

    /// Execution detail
    #[at("/history/:id")]
    HistoryDetail { id: String },

    /// Settings page
    #[at("/settings")]
    Settings,

    /// Search testing page
    #[at("/search-test")]
    SearchTest,

    /// Analytics dashboard
    #[at("/analytics")]
    Analytics,

    /// Onboarding wizard
    #[at("/onboarding")]
    Onboarding,

    /// Onboarding step
    #[at("/onboarding/:step")]
    OnboardingStep { step: String },

    /// 404 - Not found
    #[not_found]
    #[at("/404")]
    NotFound,
}

/// Switch function to render the appropriate page based on route
pub fn switch(route: Route) -> Html {
    match route {
        Route::Dashboard => html! { <DashboardPage /> },
        Route::Skills => html! { <SkillsPage /> },
        Route::SkillDetail { name } => html! { <SkillDetailPage {name} /> },
        Route::SkillInstance { name, instance } => html! {
            <SkillDetailPage {name} selected_instance={Some(instance)} />
        },
        Route::Run => html! { <RunPage /> },
        Route::RunSkill { skill } => html! { <RunPage selected_skill={Some(skill)} /> },
        Route::RunSkillTool { skill, tool } => html! {
            <RunPage selected_skill={Some(skill)} selected_tool={Some(tool)} />
        },
        Route::History => html! { <HistoryPage /> },
        Route::HistoryDetail { id } => html! { <HistoryPage selected_id={Some(id)} /> },
        Route::Settings => html! { <SettingsPage /> },
        Route::SearchTest => html! { <SearchTestPage /> },
        Route::Analytics => html! { <AnalyticsPage /> },
        Route::Onboarding => html! { <OnboardingPage /> },
        Route::OnboardingStep { step } => html! { <OnboardingPage {step} /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}

/// Get the display name for a route (for breadcrumbs, etc.)
impl Route {
    pub fn display_name(&self) -> &'static str {
        match self {
            Route::Dashboard => "Dashboard",
            Route::Skills => "Skills",
            Route::SkillDetail { .. } => "Skill Details",
            Route::SkillInstance { .. } => "Instance Configuration",
            Route::Run | Route::RunSkill { .. } | Route::RunSkillTool { .. } => "Run",
            Route::History | Route::HistoryDetail { .. } => "History",
            Route::Settings => "Settings",
            Route::SearchTest => "Search Test",
            Route::Analytics => "Analytics",
            Route::Onboarding | Route::OnboardingStep { .. } => "Setup",
            Route::NotFound => "Not Found",
        }
    }

    /// Check if this route should show the sidebar
    pub fn show_sidebar(&self) -> bool {
        !matches!(self, Route::Onboarding | Route::OnboardingStep { .. })
    }
}
