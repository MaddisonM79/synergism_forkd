//! Upgrades section. The upgrade grid isn't ported yet, so this shows a
//! purposeful empty state rather than the generic "under construction"
//! placeholder — the tab is always visible (the legacy never gated it), so
//! it needs to explain itself when there's nothing to buy.

use dioxus::prelude::*;

use crate::i18n::t;

#[component]
pub fn Upgrades() -> Element {
    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.upgrades")} }
        }
        div { class: "sf-empty-state",
            div { class: "sf-empty-title", {t("upgrades.empty_title")} }
            div { class: "sf-empty-msg", {t("upgrades.empty_msg")} }
        }
    }
}
