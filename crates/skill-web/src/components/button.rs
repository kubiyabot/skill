//! Button component with variants

use yew::prelude::*;

/// Button variant styles
#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// Button size options
#[derive(Clone, PartialEq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Button component props
#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub loading: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub r#type: Option<AttrValue>,
}

/// Button component
#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let variant_class = match props.variant {
        ButtonVariant::Primary => "btn-primary",
        ButtonVariant::Secondary => "btn-secondary",
        ButtonVariant::Ghost => "btn-ghost",
        ButtonVariant::Danger => "btn-danger",
    };

    let size_class = match props.size {
        ButtonSize::Small => "px-3 py-1.5 text-xs",
        ButtonSize::Medium => "px-4 py-2 text-sm",
        ButtonSize::Large => "px-6 py-3 text-base",
    };

    let button_type = props.r#type.clone().unwrap_or_else(|| "button".into());

    html! {
        <button
            type={button_type}
            class={classes!("btn", variant_class, size_class, props.class.clone())}
            disabled={props.disabled || props.loading}
            onclick={props.onclick.clone()}
        >
            if props.loading {
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
            }
            { for props.children.iter() }
        </button>
    }
}

/// Icon button component for toolbar actions
#[derive(Properties, PartialEq)]
pub struct IconButtonProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub title: Option<AttrValue>,
}

#[function_component(IconButton)]
pub fn icon_button(props: &IconButtonProps) -> Html {
    html! {
        <button
            type="button"
            class={classes!(
                "p-2", "rounded-lg", "text-gray-500", "hover:text-gray-700", "hover:bg-gray-100",
                "dark:text-gray-400", "dark:hover:text-gray-200", "dark:hover:bg-gray-700",
                "transition-colors", "disabled:opacity-50", "disabled:cursor-not-allowed",
                props.class.clone()
            )}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
            title={props.title.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
