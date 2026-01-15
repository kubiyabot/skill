use yew::prelude::*;
use web_sys::{HtmlInputElement, KeyboardEvent};
use wasm_bindgen::JsCast;

#[derive(Clone, PartialEq, Properties)]
pub struct SearchableSelectProps {
    pub options: Vec<String>,
    pub selected: Option<String>,
    pub on_select: Callback<String>,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub loading: bool,
}

#[function_component(SearchableSelect)]
pub fn searchable_select(props: &SearchableSelectProps) -> Html {
    let is_open = use_state(|| false);
    let search_term = use_state(|| String::new());
    let wrapper_ref = use_node_ref();
    let input_ref = use_node_ref();

    // Close dropdown when clicking outside
    {
        let is_open = is_open.clone();
        let wrapper_ref = wrapper_ref.clone();
        
        use_effect_with(wrapper_ref, move |wrapper_ref| {
            let wrapper_ref = wrapper_ref.clone();
            let is_open = is_open.clone();
            
            let listener = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
                move |event: web_sys::MouseEvent| {
                    if *is_open {
                        if let Some(target) = event.target() {
                            if let Some(wrapper) = wrapper_ref.cast::<web_sys::HtmlElement>() {
                                if !wrapper.contains(Some(&target.dyn_into().unwrap())) {
                                    is_open.set(false);
                                }
                            }
                        }
                    }
                },
            );

            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "mousedown",
                    listener.as_ref().unchecked_ref(),
                );
            }

            move || {
                if let Some(window) = web_sys::window() {
                    let _ = window.remove_event_listener_with_callback(
                        "mousedown",
                        listener.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let filtered_options = props.options.iter()
        .filter(|opt| {
            opt.to_lowercase().contains(&search_term.to_lowercase())
        })
        .collect::<Vec<_>>();

    let on_toggle = {
        let is_open = is_open.clone();
        let disabled = props.disabled;
        let input_ref = input_ref.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default(); // Prevent focus loss issues
            if !disabled {
                let new_state = !*is_open;
                is_open.set(new_state);
                // Focus input when opening
                if new_state {
                     if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        let _ = input.focus();
                    }
                }
            }
        })
    };

    let on_input = {
        let search_term = search_term.clone();
        let is_open = is_open.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_term.set(input.value());
            is_open.set(true); // Ensure open when typing
        })
    };

    let on_select_option = {
        let on_select = props.on_select.clone();
        let is_open = is_open.clone();
        let search_term = search_term.clone();
        
        move |option: String| {
            let on_select = on_select.clone();
            let is_open = is_open.clone();
            let search_term = search_term.clone();
            
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation(); // Prevent outside click handler
                on_select.emit(option.clone());
                is_open.set(false);
                search_term.set(String::new()); // Reset search on select
            })
        }
    };

    let display_value = props.selected.as_deref().unwrap_or(&props.placeholder);

    html! {
        <div class="relative" ref={wrapper_ref}>
            // Trigger
            <div
                class={classes!(
                    "flex", "items-center", "justify-between",
                    "w-full", "px-3", "py-2.5", "text-sm",
                    "bg-white", "dark:bg-gray-700",
                    "border", "rounded-lg", "shadow-sm",
                    "cursor-pointer", "transition-colors",
                    if props.disabled {
                        "bg-gray-100 dark:bg-gray-800 text-gray-500 cursor-not-allowed border-gray-200 dark:border-gray-700"
                    } else if *is_open {
                        "border-primary-500 ring-1 ring-primary-500"
                    } else {
                        "border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500"
                    },
                    if props.selected.is_none() { "text-gray-500 dark:text-gray-400" } else { "text-gray-900 dark:text-white" }
                )}
                onclick={on_toggle}
            >
                <div class="truncate mr-2">
                    { display_value }
                </div>
                
                if props.loading {
                    <div class="animate-spin h-4 w-4 border-2 border-primary-500 border-t-transparent rounded-full flex-shrink-0"></div>
                } else {
                    <svg
                        class={classes!(
                            "w-4", "h-4", "text-gray-400", "transition-transform",
                            is_open.then(|| "transform rotate-180")
                        )}
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                }
            </div>

            // Dropdown
            if *is_open && !props.disabled {
                <div class="absolute z-50 w-full mt-1 bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg shadow-lg overflow-hidden animate-in fade-in zoom-in-95 duration-100">
                    // Search input
                    <div class="p-2 border-b border-gray-200 dark:border-gray-600">
                        <input
                            ref={input_ref}
                            type="text"
                            class="w-full px-2 py-1.5 text-sm bg-gray-50 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500 text-gray-900 dark:text-white placeholder-gray-500"
                            placeholder="Search..."
                            value={(*search_term).clone()}
                            oninput={on_input}
                            onclick={Callback::from(|e: MouseEvent| e.stop_propagation())} // Prevent closing when clicking input
                        />
                    </div>

                    // Options list
                    <div class="max-h-60 overflow-y-auto">
                        if filtered_options.is_empty() {
                            <div class="px-3 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
                                { "No results found" }
                            </div>
                        } else {
                            { for filtered_options.iter().map(|option| {
                                let is_selected = props.selected.as_ref() == Some(option);
                                html! {
                                    <div
                                        class={classes!(
                                            "px-3", "py-2", "text-sm", "cursor-pointer",
                                            "flex", "items-center", "justify-between",
                                            if is_selected {
                                                "bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300"
                                            } else {
                                                "text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-600"
                                            }
                                        )}
                                        onclick={on_select_option(option.to_string())}
                                    >
                                        <span class="truncate">{ option }</span>
                                        if is_selected {
                                            <svg class="w-4 h-4 text-primary-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                            </svg>
                                        }
                                    </div>
                                }
                            })}
                        }
                    </div>
                </div>
            }
        </div>
    }
}
