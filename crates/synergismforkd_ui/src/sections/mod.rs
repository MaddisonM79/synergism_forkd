//! Section router: maps the active [`Section`](crate::gating::Section) to
//! its content panel. Sections beyond the current milestone render the
//! shared placeholder once their unlock gate opens.

pub mod production;
pub mod settings;

use dioxus::prelude::*;

use crate::bridge::use_bridge;
use crate::gating::Section;
use crate::i18n::t;

#[component]
pub fn SectionView() -> Element {
    let bridge = use_bridge();
    let section = bridge.route.read().section;
    rsx! {
        main { class: "sf-content",
            match section {
                Section::Buildings => rsx! { production::buildings::Buildings {} },
                Section::Achievements => rsx! { production::achievements::Achievements {} },
                Section::SettingsGeneral => rsx! { settings::general::General {} },
                Section::SettingsSaves => rsx! { settings::saves::Saves {} },
                Section::SettingsThemes => rsx! { settings::themes::Themes {} },
                _ => rsx! { div { class: "sf-placeholder", {t("nav.placeholder")} } },
            }
        }
    }
}
