//! The persistent right-side status panel: reset gains plus live
//! production/system stats, read-only. Lives in the app shell (visible on every
//! tab); the reset *buttons* are in the Buildings reset strip. Numbers come from
//! the tick's derived surface (`bridge.derived`) and re-render only when it
//! changes.

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Resource, ResourceIcon};
use crate::format::format_value;
use crate::i18n::t;

#[component]
pub fn StatsPanel() -> Element {
    let bridge = use_bridge();

    let show_prestige = use_slice(|s| s.reset_counters.coin_four_unlocked);
    let show_transcend = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_reincarnate = use_slice(|s| s.reset_counters.transcend_unlocked);

    let derived = bridge.derived.read();
    let b = derived.buildings;
    let notation = bridge.prefs.read().notation;
    // f64 → display string at the player's notation.
    let fv = move |x: f64| format_value(Decimal::from_finite(x), notation);

    let any_reset = show_prestige() || show_transcend() || show_reincarnate();
    let show_tax = b.tax_divisor > Decimal::one();

    rsx! {
        aside { class: "sf-stats",
            div { class: "sf-stats-title", {t("stats.title")} }

            if any_reset {
                section { class: "sf-stats-section",
                    div { class: "sf-stats-head", {t("stats.resets")} }
                    if show_prestige() {
                        div { class: "sf-card-row",
                            span { class: "label", {t("buildings.prestige")} }
                            span {
                                "+"
                                Num { value: derived.prestige_point_gain }
                                " "
                                ResourceIcon { resource: Resource::Diamonds }
                            }
                        }
                    }
                    if show_transcend() {
                        div { class: "sf-card-row",
                            span { class: "label", {t("buildings.transcend")} }
                            span {
                                "+"
                                Num { value: derived.transcend_point_gain }
                                " "
                                ResourceIcon { resource: Resource::Mythos }
                            }
                        }
                    }
                    if show_reincarnate() {
                        div { class: "sf-card-row",
                            span { class: "label", {t("buildings.reincarnate")} }
                            span {
                                "+"
                                Num { value: derived.reincarnation_point_gain }
                                " "
                                ResourceIcon { resource: Resource::Particles }
                            }
                        }
                    }
                }
            }

            section { class: "sf-stats-section",
                div { class: "sf-stats-head", {t("stats.production")} }
                div { class: "sf-card-row",
                    span { class: "label", {t("stats.coins_per_sec")} }
                    span {
                        Num { value: derived.coins_per_sec, rate: true }
                        {t("hud.per_sec")}
                    }
                }
                div { class: "sf-card-row",
                    span { class: "label", {t("buildings.accelerators")} }
                    span {
                        Num { value: b.accelerator_effect, rate: true }
                        "×"
                    }
                }
                div { class: "sf-card-row",
                    span { class: "label", {t("buildings.multipliers")} }
                    span {
                        Num { value: b.multiplier_effect, rate: true }
                        "×"
                    }
                }
                if show_tax {
                    div { class: "sf-card-row",
                        span { class: "label", {t("stats.tax_divisor")} }
                        span { Num { value: b.tax_divisor } }
                    }
                }
            }

            if show_prestige() {
                section { class: "sf-stats-section",
                    div { class: "sf-stats-head", {t("stats.systems")} }
                    div { class: "sf-card-row",
                        span { class: "label", {t("stats.building_power")} }
                        span { "{fv(b.building_power)}" }
                    }
                    div { class: "sf-card-row",
                        span { class: "label", {t("stats.speed")} }
                        span { "{fv(b.global_speed_mult)}×" }
                    }
                    div { class: "sf-card-row",
                        span { class: "label", {t("stats.total_mult")} }
                        span { "{fv(b.total_multiplier)}×" }
                    }
                    if show_transcend() {
                        div { class: "sf-card-row",
                            span { class: "label", {t("stats.global_mythos")} }
                            span {
                                Num { value: b.global_mythos_multiplier, rate: true }
                                "×"
                            }
                        }
                    }
                }
            }
        }
    }
}
