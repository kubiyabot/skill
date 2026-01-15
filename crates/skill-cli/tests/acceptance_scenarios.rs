//! Acceptance Test Scenarios
//!
//! This file imports and re-exports the acceptance test scenarios from
//! the tests/claude_bridge/acceptance_tests directory.

#[path = "../../../tests/claude_bridge/acceptance_tests/mod.rs"]
mod acceptance_tests;

// Re-export scenarios so they're discovered by cargo test
pub use acceptance_tests::scenarios::*;
