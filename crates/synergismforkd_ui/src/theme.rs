//! Runtime theme selection. Themes are pure CSS-token override blocks in
//! `assets/styles/themes.css`, keyed off the `data-theme` attribute the app
//! root renders — switching is one attribute write, no style recomputation
//! in Rust.

use serde::{Deserialize, Serialize};

/// Available themes. The legacy game shipped 8; they port as token blocks
/// over time — the enum is the only Rust-side change each one needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    /// Dark is the default for an idle game that lives in a background tab.
    #[default]
    Dark,
    Light,
}

impl Theme {
    /// Every theme, in settings-menu order.
    pub const ALL: [Theme; 2] = [Theme::Dark, Theme::Light];

    /// The `data-theme` attribute value (must match a `themes.css` block).
    #[must_use]
    pub fn css_value(self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    /// Translation key for the settings menu.
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Theme::Dark => "settings.themes.dark",
            Theme::Light => "settings.themes.light",
        }
    }
}
