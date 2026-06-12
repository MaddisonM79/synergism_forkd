//! Stats page: the live production + system-multiplier readout. Moved off the
//! bottom bar (which is now a pure resource ledger) onto its own page, so the
//! numbers have room and the ledger isn't crowded. Values come from the tick's
//! derived surface (`bridge.derived`).

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;

use crate::bridge::{use_bridge, use_slice};
use crate::components::Num;
use crate::format::format_value;
use crate::i18n::t;

#[component]
pub fn StatsPage() -> Element {
    let bridge = use_bridge();
    // The Systems block (building power / speed / multipliers) reveals with the
    // Diamond tier, the global-Mythos row after a transcension — same gates the
    // old side panel used.
    let show_systems = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_mythos = use_slice(|s| s.reset_counters.transcend_unlocked);

    let derived = bridge.derived.read();
    let b = derived.buildings;
    let notation = bridge.prefs.read().notation;
    let fv = move |x: f64| format_value(Decimal::from_finite(x), notation);
    let show_tax = b.tax_divisor > Decimal::one();

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.stats")} }
        }
        div { class: "sf-card-grid",
            div { class: "sf-card",
                div { class: "sf-card-title", {t("stats.production")} }
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

            if show_systems() {
                div { class: "sf-card",
                    div { class: "sf-card-title", {t("stats.systems")} }
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
                    if show_mythos() {
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
