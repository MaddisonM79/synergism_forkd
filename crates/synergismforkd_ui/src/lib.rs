//! Synergism Forkd — platform-agnostic UI components.
//!
//! Dioxus components, hooks, and signals. Consumed by
//! `synergismforkd_ui_web` (browser) and `synergismforkd_ui_desktop`
//! (dioxus-desktop). No platform-specific code lives here — no
//! `wasm-bindgen`, no `web-sys`, no renderer; the shells pick those.

use synergismforkd_common as _;

pub mod achievement_toast;
pub mod app;
pub mod bridge;
pub mod components;
pub mod derive;
pub mod events_map;
pub mod format;
pub mod gating;
pub mod hud;
pub mod i18n;
pub mod nav;
pub mod sections;
pub mod stats;
pub mod theme;

pub use app::App;
