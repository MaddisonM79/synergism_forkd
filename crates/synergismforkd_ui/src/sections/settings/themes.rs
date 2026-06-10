//! Theme picker. Writing `prefs.theme` re-renders the app root's
//! `data-theme` attribute; the host persists prefs on change.

use dioxus::prelude::*;

use crate::bridge::use_bridge;
use crate::i18n::t;
use crate::theme::Theme;

#[component]
pub fn Themes() -> Element {
    let bridge = use_bridge();
    let current = bridge.prefs.read().theme;
    rsx! {
        div { class: "sf-section-head",
            h1 { {t("settings.themes.title")} }
        }
        div { class: "sf-seg",
            for theme in Theme::ALL {
                button {
                    key: "{theme.css_value()}",
                    class: if theme == current { "active" } else { "" },
                    onclick: move |_| {
                        let mut prefs = bridge.prefs;
                        prefs.write().theme = theme;
                    },
                    {t(theme.label_key())}
                }
            }
        }
    }
}
