//! The persistent resource HUD: one slim bar, unlock-gated chips, exact
//! values + rates in tooltips. Compact rule: names live in tooltips, the
//! bar itself is icon + number only.

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Resource, ResourceIcon, Tooltip};
use crate::gating::{Group, Route, Section};
use crate::i18n::t;

/// One HUD chip: icon + compact value (plus an inline per-second rate when
/// given). The tooltip (opening downward, since the HUD hugs the top edge)
/// shows the resource name, exact rate, and — when given — the lifetime
/// total.
#[component]
fn Chip(
    resource: Resource,
    value: Decimal,
    #[props(default)] rate: Option<Decimal>,
    #[props(default)] lifetime: Option<Decimal>,
) -> Element {
    rsx! {
        Tooltip {
            down: true,
            tip: rsx! {
                div {
                    span { {t(resource.label_key())} }
                    if let Some(rate) = rate {
                        span { class: "sf-chip-rate",
                            " "
                            Num { value: rate, rate: true }
                            {t("hud.per_sec")}
                        }
                    }
                }
                if let Some(lifetime) = lifetime {
                    div { class: "sf-chip-rate",
                        {t("hud.lifetime")} ": "
                        Num { value: lifetime }
                    }
                }
            },
            span { class: "sf-chip",
                ResourceIcon { resource }
                Num { value }
                if let Some(rate) = rate {
                    span { class: "sf-chip-rate",
                        "+"
                        Num { value: rate, rate: true }
                        {t("hud.per_sec")}
                    }
                }
            }
        }
    }
}

/// The top bar. Chips appear as their feature unlocks; the settings gear is
/// always last.
#[component]
pub fn ResourceHud() -> Element {
    let bridge = use_bridge();

    let coins = use_slice(|s| s.upgrades.coins);
    let coins_lifetime = use_slice(|s| s.coin_counters.coins_total);
    let diamonds = use_slice(|s| s.upgrades.prestige_points);
    let crystals = use_slice(|s| s.crystal_upgrades.prestige_shards);
    let mythos = use_slice(|s| s.upgrades.transcend_points);
    let particles = use_slice(|s| s.upgrades.reincarnation_points);
    let offerings = use_slice(|s| s.automation.offerings);
    let obtainium = use_slice(|s| s.researches.obtainium);
    let quarks = use_slice(|s| s.quarks.worlds);
    let golden_quarks = use_slice(|s| s.golden_quarks.golden_quarks);

    let show_diamonds = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_mythos = use_slice(|s| s.reset_counters.transcend_unlocked);
    let show_particles = use_slice(|s| s.reset_counters.reincarnate_unlocked);
    let show_quarks = use_slice(|s| s.reset_counters.ascension_unlocked);
    let show_gq = use_slice(|s| s.singularity.highest_singularity_count > 0.0);

    let coins_rate = bridge.derived.read().coins_per_sec;

    rsx! {
        header { class: "sf-hud",
            Chip {
                resource: Resource::Coins,
                value: coins(),
                rate: Some(coins_rate),
                lifetime: Some(coins_lifetime()),
            }
            if show_diamonds() {
                Chip { resource: Resource::Diamonds, value: diamonds() }
                Chip { resource: Resource::Crystals, value: crystals() }
                Chip { resource: Resource::Offerings, value: offerings() }
            }
            if show_mythos() {
                Chip { resource: Resource::Mythos, value: mythos() }
            }
            if show_particles() {
                Chip { resource: Resource::Particles, value: particles() }
                Chip { resource: Resource::Obtainium, value: obtainium() }
            }
            if show_quarks() {
                Chip { resource: Resource::Quarks, value: quarks() }
            }
            if show_gq() {
                Chip { resource: Resource::GoldenQuarks, value: golden_quarks() }
            }
            div { class: "sf-hud-spacer" }
            button {
                onclick: move |_| {
                    bridge.navigate(Route {
                        group: Group::Settings,
                        section: Section::Settings,
                        subsection: 0,
                    });
                },
                "⚙"
            }
        }
    }
}
