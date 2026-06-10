//! Buildings section. Placeholder until the buy wiring lands (vertical
//! slice task: producer cards + accelerator/multiplier + prestige).

use dioxus::prelude::*;

use crate::i18n::t;

#[component]
pub fn Buildings() -> Element {
    rsx! {
        div { class: "sf-placeholder", {t("nav.placeholder")} }
    }
}
