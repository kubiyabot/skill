//! UI state store
//!
//! Manages transient UI state like sidebar visibility, notifications, and modals.

use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

/// Notification severity level
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum NotificationLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "check-circle",
            Self::Warning => "alert-triangle",
            Self::Error => "x-circle",
        }
    }
}

/// Notification message
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    /// Unique ID
    pub id: String,
    /// Title
    pub title: String,
    /// Message content
    pub message: String,
    /// Severity level
    pub level: NotificationLevel,
    /// Auto-dismiss after milliseconds (None = persistent)
    pub auto_dismiss_ms: Option<u32>,
    /// Whether the notification can be dismissed
    pub dismissible: bool,
    /// Optional action button text
    pub action_text: Option<String>,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
}

impl Notification {
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Info)
    }

    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Success)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Warning)
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Error)
    }

    fn new(title: impl Into<String>, message: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            message: message.into(),
            level,
            auto_dismiss_ms: Some(5000),
            dismissible: true,
            action_text: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn persistent(mut self) -> Self {
        self.auto_dismiss_ms = None;
        self
    }

    pub fn with_action(mut self, text: impl Into<String>) -> Self {
        self.action_text = Some(text.into());
        self
    }
}

/// Modal dialog state
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModalState {
    /// Whether any modal is open
    pub open: bool,
    /// Current modal type
    pub modal_type: Option<ModalType>,
    /// Modal data (JSON-serializable)
    pub data: Option<String>,
}

/// Types of modal dialogs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModalType {
    /// Confirm an action
    Confirm,
    /// Install a skill
    InstallSkill,
    /// Uninstall a skill
    UninstallSkill,
    /// Configure an instance
    ConfigureInstance,
    /// View execution details
    ExecutionDetails,
    /// Export data
    Export,
    /// Import data
    Import,
    /// About dialog
    About,
}

/// Command palette state
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandPaletteState {
    /// Whether the command palette is open
    pub open: bool,
    /// Current search query
    pub query: String,
    /// Selected index
    pub selected_index: usize,
}

/// UI store state
#[derive(Clone, Debug, Default, PartialEq, Store)]
pub struct UiStore {
    /// Whether the sidebar is open (for mobile)
    pub sidebar_open: bool,
    /// Whether the sidebar is collapsed (for desktop)
    pub sidebar_collapsed: bool,
    /// Active notifications
    pub notifications: Vec<Notification>,
    /// Modal state
    pub modal: ModalState,
    /// Command palette state
    pub command_palette: CommandPaletteState,
    /// Currently focused element ID
    pub focused_element: Option<String>,
    /// Whether dark mode is active (resolved from settings)
    pub dark_mode: bool,
    /// Loading overlay visible
    pub loading_overlay: bool,
    /// Loading message
    pub loading_message: Option<String>,
    /// Mobile breakpoint active
    pub is_mobile: bool,
    /// Touch device detected
    pub is_touch_device: bool,
}

impl UiStore {
    /// Maximum notifications to show
    const MAX_NOTIFICATIONS: usize = 5;

    /// Get visible notifications (limited)
    pub fn visible_notifications(&self) -> &[Notification] {
        let end = self.notifications.len().min(Self::MAX_NOTIFICATIONS);
        &self.notifications[..end]
    }

    /// Check if any modal is open
    pub fn has_modal(&self) -> bool {
        self.modal.open
    }

    /// Check if command palette is open
    pub fn is_command_palette_open(&self) -> bool {
        self.command_palette.open
    }
}

/// UI store actions
pub enum UiAction {
    // Sidebar
    ToggleSidebar,
    SetSidebarOpen(bool),
    ToggleSidebarCollapsed,
    SetSidebarCollapsed(bool),
    // Notifications
    AddNotification(Notification),
    DismissNotification(String),
    ClearNotifications,
    // Modal
    OpenModal(ModalType, Option<String>),
    CloseModal,
    // Command palette
    OpenCommandPalette,
    CloseCommandPalette,
    SetCommandPaletteQuery(String),
    SelectCommandPaletteItem(usize),
    CommandPaletteUp,
    CommandPaletteDown,
    // Loading
    ShowLoading(Option<String>),
    HideLoading,
    // Responsive
    SetIsMobile(bool),
    SetIsTouchDevice(bool),
    // Theme
    SetDarkMode(bool),
    // Focus
    SetFocusedElement(Option<String>),
}

impl Reducer<UiStore> for UiAction {
    fn apply(self, mut store: std::rc::Rc<UiStore>) -> std::rc::Rc<UiStore> {
        let state = std::rc::Rc::make_mut(&mut store);

        match self {
            // Sidebar
            UiAction::ToggleSidebar => {
                state.sidebar_open = !state.sidebar_open;
            }
            UiAction::SetSidebarOpen(open) => {
                state.sidebar_open = open;
            }
            UiAction::ToggleSidebarCollapsed => {
                state.sidebar_collapsed = !state.sidebar_collapsed;
            }
            UiAction::SetSidebarCollapsed(collapsed) => {
                state.sidebar_collapsed = collapsed;
            }
            // Notifications
            UiAction::AddNotification(notification) => {
                // Add to front
                state.notifications.insert(0, notification);
                // Limit total notifications
                if state.notifications.len() > 20 {
                    state.notifications.truncate(20);
                }
            }
            UiAction::DismissNotification(id) => {
                state.notifications.retain(|n| n.id != id);
            }
            UiAction::ClearNotifications => {
                state.notifications.clear();
            }
            // Modal
            UiAction::OpenModal(modal_type, data) => {
                state.modal = ModalState {
                    open: true,
                    modal_type: Some(modal_type),
                    data,
                };
            }
            UiAction::CloseModal => {
                state.modal = ModalState::default();
            }
            // Command palette
            UiAction::OpenCommandPalette => {
                state.command_palette = CommandPaletteState {
                    open: true,
                    query: String::new(),
                    selected_index: 0,
                };
            }
            UiAction::CloseCommandPalette => {
                state.command_palette = CommandPaletteState::default();
            }
            UiAction::SetCommandPaletteQuery(query) => {
                state.command_palette.query = query;
                state.command_palette.selected_index = 0;
            }
            UiAction::SelectCommandPaletteItem(index) => {
                state.command_palette.selected_index = index;
            }
            UiAction::CommandPaletteUp => {
                if state.command_palette.selected_index > 0 {
                    state.command_palette.selected_index -= 1;
                }
            }
            UiAction::CommandPaletteDown => {
                state.command_palette.selected_index += 1;
            }
            // Loading
            UiAction::ShowLoading(message) => {
                state.loading_overlay = true;
                state.loading_message = message;
            }
            UiAction::HideLoading => {
                state.loading_overlay = false;
                state.loading_message = None;
            }
            // Responsive
            UiAction::SetIsMobile(is_mobile) => {
                state.is_mobile = is_mobile;
                // Auto-close sidebar on mobile
                if is_mobile {
                    state.sidebar_open = false;
                }
            }
            UiAction::SetIsTouchDevice(is_touch) => {
                state.is_touch_device = is_touch;
            }
            // Theme
            UiAction::SetDarkMode(dark) => {
                state.dark_mode = dark;
            }
            // Focus
            UiAction::SetFocusedElement(element) => {
                state.focused_element = element;
            }
        }

        store
    }
}
