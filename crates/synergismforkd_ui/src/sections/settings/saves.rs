//! Saves section. Placeholder until the host-command wiring lands
//! (vertical slice task: export / import / hard reset).

use dioxus::prelude::*;

use crate::i18n::t;

#[component]
pub fn Saves() -> Element {
    rsx! {
        div { class: "sf-placeholder", {t("nav.placeholder")} }
    }
}
