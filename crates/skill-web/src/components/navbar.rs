//! Top navigation bar component

use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;
use super::icons::{SettingsIcon, SearchIcon};

/// Top navigation bar
#[function_component(Navbar)]
pub fn navbar() -> Html {
    let search_query = use_state(String::new);

    let on_search_input = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    html! {
        <nav class="fixed top-0 left-0 right-0 h-16 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 z-40">
            <div class="flex items-center justify-between h-full px-6">
                // Logo and title
                <div class="flex items-center gap-3">
                    <Link<Route> to={Route::Dashboard} classes="flex items-center gap-3 hover:opacity-80 transition-opacity">
                        <span class="text-2xl">{ "⚡" }</span>
                        <span class="text-xl font-semibold text-gray-900 dark:text-white">
                            { "Skill Engine" }
                        </span>
                    </Link<Route>>
                </div>

                // Search bar (centered)
                <div class="flex-1 max-w-xl mx-8">
                    <div class="relative">
                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <SearchIcon class="w-5 h-5 text-gray-400" />
                        </div>
                        <input
                            type="text"
                            placeholder="Search skills, tools, or commands..."
                            class="input pl-10"
                            value={(*search_query).clone()}
                            oninput={on_search_input}
                        />
                    </div>
                </div>

                // Right side actions
                <div class="flex items-center gap-4">
                    // Version badge
                    <span class="badge badge-info">{ "v0.2.2" }</span>

                    // Settings link
                    <Link<Route>
                        to={Route::Settings}
                        classes="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:bg-gray-700 transition-colors"
                    >
                        <SettingsIcon class="w-5 h-5" />
                    </Link<Route>>
                </div>
            </div>
        </nav>
    }
}
