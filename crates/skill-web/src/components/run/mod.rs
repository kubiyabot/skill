//! Run page components module

// TODO: These components are WIP and not yet integrated into the run page
#[allow(dead_code)]
mod skill_selector;
#[allow(dead_code)]
mod tool_selector;
#[allow(dead_code)]
mod wizard_stepper;
#[allow(dead_code)]
mod command_palette;
mod inline_parameter_editor;
mod terminal_output;

pub use inline_parameter_editor::InlineParameterEditor;
pub use terminal_output::TerminalOutput;

// Re-export WizardStep from hooks for convenience
