//! Main layout component with sidebar and navbar

use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;
use super::navbar::Navbar;
use super::sidebar::Sidebar;
use super::notifications::NotificationContainer;

/// Props for the Layout component
#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Children,
}

/// Main layout component that wraps all pages
#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    let route = use_route::<Route>();
    let show_sidebar = route.map(|r| r.show_sidebar()).unwrap_or(true);

    html! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            // Global notification container
            <NotificationContainer />

            if show_sidebar {
                <Navbar />
                <div class="flex">
                    <Sidebar />
                    <main class="flex-1 ml-64 mt-16 p-6">
                        <div class="max-w-7xl mx-auto">
                            { for props.children.iter() }
                        </div>
                    </main>
                </div>
            } else {
                // Full-screen layout for onboarding
                <main class="min-h-screen">
                    { for props.children.iter() }
                </main>
            }
        </div>
    }
}
