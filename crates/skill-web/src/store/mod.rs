//! Yewdux state management stores
//!
//! Global state management using Redux-like patterns.
//!
//! ## Available Stores
//!
//! - **SkillsStore**: Installed skills, filtering, sorting, selected skill
//! - **ExecutionsStore**: Execution history, active execution, streaming output
//! - **SettingsStore**: User preferences, persisted to localStorage
//! - **UiStore**: Transient UI state (sidebar, notifications, modals)
//!
//! ## Usage
//!
//! ```ignore
//! use yewdux::prelude::*;
//! use crate::store::{SkillsStore, SkillsAction};
//!
//! #[function_component(MyComponent)]
//! fn my_component() -> Html {
//!     let (skills, dispatch) = use_store::<SkillsStore>();
//!
//!     let on_search = {
//!         let dispatch = dispatch.clone();
//!         Callback::from(move |query: String| {
//!             dispatch.apply(SkillsAction::SetSearchQuery(query));
//!         })
//!     };
//!
//!     html! {
//!         // ...
//!     }
//! }
//! ```

pub mod executions;
pub mod settings;
pub mod skills;
pub mod ui;

// Re-export stores
pub use executions::{
    ActiveExecution, ExecutionEntry, ExecutionStatus, ExecutionsAction, ExecutionsStore,
};
pub use settings::{
    ApiSettings, EmbeddingProvider, OutputFormat, SearchSettings, SettingsAction, SettingsStore,
    Theme, VectorBackend,
};
pub use skills::{
    InstanceInfo, ParameterInfo, SkillDetail, SkillRuntime, SkillSortBy, SkillStatus, SkillSummary,
    SkillsAction, SkillsStore, ToolInfo,
};
pub use ui::{
    CommandPaletteState, ModalState, ModalType, Notification, NotificationLevel, UiAction, UiStore,
};
