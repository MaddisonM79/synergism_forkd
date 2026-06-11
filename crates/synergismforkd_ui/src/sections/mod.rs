//! Section router: maps the active [`Section`](crate::gating::Section) to
//! its content panel. Sections beyond the current milestone render the
//! shared placeholder once their unlock gate opens.

pub mod header;
pub mod mystic;
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
                Section::Upgrades => rsx! { production::upgrades::Upgrades {} },
                Section::Stats => rsx! { production::stats::StatsPage {} },
                Section::Achievements => rsx! { production::achievements::Achievements {} },
                Section::Runes => rsx! { mystic::runes::Runes {} },
                Section::Settings => rsx! { settings::page::Settings {} },
                _ => rsx! { div { class: "sf-placeholder", {t("nav.placeholder")} } },
            }
        }
    }
}
