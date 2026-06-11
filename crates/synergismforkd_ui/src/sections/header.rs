//! Top header bar: the reset-gain indicators (Prestige / Transcend /
//! Reincarnate) as compact hover-icons. Each icon appears once its tier is
//! reachable (same gates as the old StatsPanel RESETS section) and reveals its
//! "+gain" on hover. Display-only — the actual reset buttons live in the
//! Buildings reset strip.

use dioxus::prelude::*;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Resource, ResourceIcon, Tooltip};
use crate::i18n::t;

#[component]
pub fn HeaderBar() -> Element {
    // Gates mirror the reset progression (a gain shows once that reset is
    // reachable): prestige at coin-four, transcend after a prestige,
    // reincarnate after a transcension.
    let show_prestige = use_slice(|s| s.reset_counters.coin_four_unlocked);
    let show_transcend = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_reincarnate = use_slice(|s| s.reset_counters.transcend_unlocked);

    rsx! {
        header { class: "sf-header",
            if show_prestige() {
                ResetGainIcon { label_key: "buildings.prestige", resource: Resource::Diamonds }
            }
            if show_transcend() {
                ResetGainIcon { label_key: "buildings.transcend", resource: Resource::Mythos }
            }
            if show_reincarnate() {
                ResetGainIcon { label_key: "buildings.reincarnate", resource: Resource::Particles }
            }
        }
    }
}

/// One reset-gain hover-icon: the tier's currency icon, with a downward
/// tooltip showing "<Tier> +<gain> <icon>". Gain comes from the tick's derived
/// surface (read inline — three tiny nodes, re-renders at tick rate).
#[component]
fn ResetGainIcon(label_key: &'static str, resource: Resource) -> Element {
    let bridge = use_bridge();
    let derived = bridge.derived.read();
    let gain = match resource {
        Resource::Diamonds => derived.prestige_point_gain,
        Resource::Mythos => derived.transcend_point_gain,
        _ => derived.reincarnation_point_gain,
    };
    rsx! {
        Tooltip {
            down: true,
            tip: rsx! {
                span {
                    {t(label_key)}
                    " +"
                    Num { value: gain }
                }
            },
            span { class: "sf-header-icon",
                ResourceIcon { resource }
            }
        }
    }
}
