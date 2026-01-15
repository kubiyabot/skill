//! Toast notification system
//!
//! Displays notifications in the top-right corner with auto-dismiss support.

use gloo_timers::callback::Timeout;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::store::ui::{Notification, NotificationLevel, UiAction, UiStore};

// ============================================================================
// NotificationContainer - Renders all notifications
// ============================================================================

/// Container for all toast notifications
#[function_component(NotificationContainer)]
pub fn notification_container() -> Html {
    let (store, _) = use_store::<UiStore>();
    let notifications = store.visible_notifications();

    html! {
        <div
            class="fixed top-4 right-4 z-50 flex flex-col gap-3 max-w-sm w-full pointer-events-none"
            aria-live="polite"
            aria-label="Notifications"
        >
            { for notifications.iter().map(|notification| {
                html! {
                    <NotificationToast
                        key={notification.id.clone()}
                        notification={notification.clone()}
                    />
                }
            }) }
        </div>
    }
}

// ============================================================================
// NotificationToast - Individual notification
// ============================================================================

#[derive(Properties, PartialEq)]
struct NotificationToastProps {
    notification: Notification,
}

#[function_component(NotificationToast)]
fn notification_toast(props: &NotificationToastProps) -> Html {
    let (_, dispatch) = use_store::<UiStore>();
    let notification = &props.notification;
    let is_exiting = use_state(|| false);

    // Auto-dismiss timer
    {
        let id = notification.id.clone();
        let auto_dismiss_ms = notification.auto_dismiss_ms;
        let dispatch = dispatch.clone();
        let is_exiting = is_exiting.clone();

        use_effect_with(id.clone(), move |_| {
            let cleanup: Option<Timeout> = if let Some(ms) = auto_dismiss_ms {
                let timeout = Timeout::new(ms, move || {
                    is_exiting.set(true);
                    // Delay actual removal to allow exit animation
                    let dispatch = dispatch.clone();
                    let id = id.clone();
                    Timeout::new(300, move || {
                        dispatch.apply(UiAction::DismissNotification(id));
                    })
                    .forget();
                });
                Some(timeout)
            } else {
                None
            };

            move || {
                drop(cleanup);
            }
        });
    }

    let on_dismiss = {
        let dispatch = dispatch.clone();
        let id = notification.id.clone();
        let is_exiting = is_exiting.clone();
        Callback::from(move |_| {
            is_exiting.set(true);
            let dispatch = dispatch.clone();
            let id = id.clone();
            Timeout::new(300, move || {
                dispatch.apply(UiAction::DismissNotification(id));
            })
            .forget();
        })
    };

    let (bg_class, border_class, icon_class) = match notification.level {
        NotificationLevel::Info => (
            "bg-blue-50 dark:bg-blue-900/30",
            "border-blue-200 dark:border-blue-800",
            "text-blue-500",
        ),
        NotificationLevel::Success => (
            "bg-green-50 dark:bg-green-900/30",
            "border-green-200 dark:border-green-800",
            "text-green-500",
        ),
        NotificationLevel::Warning => (
            "bg-amber-50 dark:bg-amber-900/30",
            "border-amber-200 dark:border-amber-800",
            "text-amber-500",
        ),
        NotificationLevel::Error => (
            "bg-red-50 dark:bg-red-900/30",
            "border-red-200 dark:border-red-800",
            "text-red-500",
        ),
    };

    let animation_class = if *is_exiting {
        "animate-slide-out-right"
    } else {
        "animate-slide-in-right"
    };

    html! {
        <div
            class={classes!(
                "pointer-events-auto",
                "rounded-lg", "border", "shadow-lg",
                "p-4", "flex", "gap-3",
                bg_class, border_class,
                animation_class
            )}
            role="alert"
        >
            // Icon
            <div class={classes!("flex-shrink-0", "mt-0.5", icon_class)}>
                <NotificationIcon level={notification.level.clone()} />
            </div>

            // Content
            <div class="flex-1 min-w-0">
                <h4 class="text-sm font-semibold text-gray-900 dark:text-white">
                    { &notification.title }
                </h4>
                <p class="text-sm text-gray-600 dark:text-gray-300 mt-0.5">
                    { &notification.message }
                </p>

                // Action button (optional)
                if let Some(ref action_text) = notification.action_text {
                    <button
                        class="text-sm font-medium text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 mt-2"
                    >
                        { action_text }
                    </button>
                }
            </div>

            // Dismiss button
            if notification.dismissible {
                <button
                    onclick={on_dismiss}
                    class="flex-shrink-0 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                    aria-label="Dismiss notification"
                >
                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            }
        </div>
    }
}

// ============================================================================
// NotificationIcon - Icon based on level
// ============================================================================

#[derive(Properties, PartialEq)]
struct NotificationIconProps {
    level: NotificationLevel,
}

#[function_component(NotificationIcon)]
fn notification_icon(props: &NotificationIconProps) -> Html {
    match props.level {
        NotificationLevel::Info => html! {
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        },
        NotificationLevel::Success => html! {
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        },
        NotificationLevel::Warning => html! {
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
        },
        NotificationLevel::Error => html! {
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        },
    }
}

// ============================================================================
// Hook for easy notification dispatch
// ============================================================================

/// Hook to easily show notifications from any component
#[hook]
pub fn use_notifications() -> UseNotificationsHandle {
    let (_, dispatch) = use_store::<UiStore>();
    UseNotificationsHandle { dispatch }
}

pub struct UseNotificationsHandle {
    dispatch: Dispatch<UiStore>,
}

impl UseNotificationsHandle {
    /// Show an info notification
    pub fn info(&self, title: impl Into<String>, message: impl Into<String>) {
        self.dispatch
            .apply(UiAction::AddNotification(Notification::info(title, message)));
    }

    /// Show a success notification
    pub fn success(&self, title: impl Into<String>, message: impl Into<String>) {
        self.dispatch
            .apply(UiAction::AddNotification(Notification::success(title, message)));
    }

    /// Show a warning notification
    pub fn warning(&self, title: impl Into<String>, message: impl Into<String>) {
        self.dispatch
            .apply(UiAction::AddNotification(Notification::warning(title, message)));
    }

    /// Show an error notification
    pub fn error(&self, title: impl Into<String>, message: impl Into<String>) {
        self.dispatch
            .apply(UiAction::AddNotification(Notification::error(title, message)));
    }

    /// Show a custom notification
    pub fn show(&self, notification: Notification) {
        self.dispatch.apply(UiAction::AddNotification(notification));
    }

    /// Dismiss a notification by ID
    pub fn dismiss(&self, id: &str) {
        self.dispatch
            .apply(UiAction::DismissNotification(id.to_string()));
    }

    /// Clear all notifications
    pub fn clear(&self) {
        self.dispatch.apply(UiAction::ClearNotifications);
    }
}

impl Clone for UseNotificationsHandle {
    fn clone(&self) -> Self {
        Self {
            dispatch: self.dispatch.clone(),
        }
    }
}
