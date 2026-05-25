//! Synergism Forkd — WASM entry point.
//!
//! Built as `cdylib` for `wasm32-unknown-unknown`; mounts the
//! `synergismforkd_ui` Dioxus root component into the page. Also built
//! as `rlib` so the desktop crate can depend on it.

pub fn placeholder() {}
