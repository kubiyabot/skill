//! 404 Not Found page

use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

/// Not Found page component
#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="min-h-[60vh] flex flex-col items-center justify-center text-center animate-fade-in">
            <div class="text-8xl mb-6 opacity-50">{ "🔍" }</div>
            <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                { "Page Not Found" }
            </h1>
            <p class="text-lg text-gray-500 dark:text-gray-400 mb-8 max-w-md">
                { "The page you're looking for doesn't exist or has been moved." }
            </p>
            <Link<Route> to={Route::Dashboard} classes="btn btn-primary">
                { "Go to Dashboard" }
            </Link<Route>>
        </div>
    }
}
