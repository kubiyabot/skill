//! Skill Engine Web Interface
//!
//! A WebAssembly-based web interface for managing AI agent skills,
//! built with the Yew framework.

// Allow dead code in the binary - all functionality is exercised through the Yew app,
// but not all types are directly referenced in main.rs
#![allow(dead_code)]

mod app;
mod components;
mod pages;
mod router;
mod store;
mod api;
mod utils;
mod hooks;

use wasm_bindgen::prelude::*;

/// Entry point for the WASM application
fn main() {
    // Initialize tracing for better debugging
    tracing_wasm::set_as_global_default();

    // Start the Yew application
    yew::Renderer::<app::App>::new().render();

    // Hide the loading screen
    hide_loading_screen();
}

/// Call JavaScript function to hide the loading screen
fn hide_loading_screen() {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::get(&window, &JsValue::from_str("hideLoadingScreen"))
            .ok()
            .and_then(|func| func.dyn_ref::<js_sys::Function>().cloned())
            .map(|func| func.call0(&JsValue::NULL));
    }
}
