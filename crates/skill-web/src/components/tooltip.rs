//! Tooltip component for inline help text
//!
//! Provides a simple hover tooltip with an info icon that displays
//! explanatory text for technical terms and configuration options.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    /// The tooltip text to display on hover
    pub text: String,
}

/// Tooltip component with info icon
///
/// Displays an info icon that shows explanatory text on hover.
/// The tooltip is positioned above and to the right of the icon.
///
/// # Example
///
/// ```ignore
/// <Tooltip text="This explains what the setting does" />
/// ```
#[function_component(Tooltip)]
pub fn tooltip(props: &TooltipProps) -> Html {
    html! {
        <span class="inline-flex items-center ml-1 group relative">
            // Info icon (SVG)
            <svg
                class="w-4 h-4 text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300 cursor-help transition-colors"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                    clip-rule="evenodd"
                />
            </svg>

            // Tooltip text (appears on hover)
            <span class="invisible group-hover:visible absolute z-50 w-64 p-2 text-xs text-white bg-gray-900 dark:bg-gray-800 rounded shadow-lg -top-10 left-5 pointer-events-none">
                { &props.text }
                // Arrow pointing to icon
                <svg
                    class="absolute text-gray-900 dark:text-gray-800 h-2 w-full left-0 top-full"
                    x="0px"
                    y="0px"
                    viewBox="0 0 255 255"
                >
                    <polygon class="fill-current" points="0,0 127.5,127.5 255,0"/>
                </svg>
            </span>
        </span>
    }
}
