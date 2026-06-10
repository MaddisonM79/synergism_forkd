//! Synergism Forkd — platform-agnostic UI components.
//!
//! Dioxus components, hooks, and signals. Consumed by
//! `synergismforkd_ui_web` (browser) and `synergismforkd_ui_desktop`
//! (dioxus-desktop). No platform-specific code lives here — no
//! `wasm-bindgen`, no `web-sys`, no renderer; the shells pick those.

use synergismforkd_common as _;
use synergismforkd_logic as _;
// Consumed as modules land (bridge/components); silence until then.
use dioxus as _;
use serde_json as _;

pub mod format;

pub fn placeholder() {}
