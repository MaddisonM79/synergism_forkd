//! The persistent right-side panel. Top section is the **resource ledger**
//! (every unlocked currency, icon + value + rate) — always shown, it's the
//! primary resource HUD now that the top bar is gone. Below it sit the
//! reset-gain and live production/system stats, which the `show_stats_panel`
//! pref toggles. All numbers come from state slices + the tick's derived
//! surface (`bridge.derived`) and re-render only when they change.

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Resource, ResourceIcon};
use crate::format::format_value;
use crate::i18n::t;

/// One currency row: icon + name on the left, value (and an optional inline
/// per-second rate) right-aligned. Compact inline layout.
#[component]
fn CurrencyRow(
    resource: Resource,
    value: Decimal,
    #[props(default)] rate: Option<Decimal>,
) -> Element {
    rsx! {
        div { class: "sf-res-row", style: "--row-accent: {resource.css_color()}",
            ResourceIcon { resource }
            span { class: "sf-res-name", {t(resource.label_key())} }
            span { class: "sf-res-val",
                Num { value }
                if let Some(rate) = rate {
                    span { class: "sf-res-rate",
                        "+"
                        Num { value: rate, rate: true }
                        {t("hud.per_sec")}
                    }
                }
            }
        }
    }
}

#[component]
pub fn StatsPanel() -> Element {
    let bridge = use_bridge();

    // Currency values (state slices) — paired point + shard per layer.
    let coins = use_slice(|s| s.upgrades.coins);
    let diamonds = use_slice(|s| s.upgrades.prestige_points);
    let crystals = use_slice(|s| s.crystal_upgrades.prestige_shards);
    let offerings = use_slice(|s| s.automation.offerings);
    let mythos = use_slice(|s| s.upgrades.transcend_points);
    let mythos_shards = use_slice(|s| s.reset_counters.transcend_shards);
    let particles = use_slice(|s| s.upgrades.reincarnation_points);
    let obtainium = use_slice(|s| s.researches.obtainium);
    let quarks = use_slice(|s| s.quarks.worlds);
    let golden_quarks = use_slice(|s| s.golden_quarks.golden_quarks);

    // Progressive reveal — one layer at a time, same gates as the rest of the UI.
    let show_diamonds = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_mythos = use_slice(|s| s.reset_counters.transcend_unlocked);
    let show_particles = use_slice(|s| s.reset_counters.reincarnate_unlocked);
    let show_quarks = use_slice(|s| s.reset_counters.ascension_unlocked);
    let show_gq = use_slice(|s| s.singularity.highest_singularity_count > 0.0);

    let derived = bridge.derived.read();
    let b = derived.buildings;
    let prefs = bridge.prefs.read();
    let notation = prefs.notation;
    let show_stats = prefs.show_stats_panel;
    // f64 → display string at the player's notation.
    let fv = move |x: f64| format_value(Decimal::from_finite(x), notation);

    let show_tax = b.tax_divisor > Decimal::one();

    rsx! {
        aside { class: "sf-stats",
            div { class: "sf-stats-title", {t("stats.title")} }

            // ── Resource ledger (always shown) ──────────────────────────────
            section { class: "sf-stats-section sf-stats-resources",
                div { class: "sf-stats-head", {t("stats.resources")} }
                CurrencyRow {
                    resource: Resource::Coins,
                    value: coins(),
                    rate: Some(derived.coins_per_sec),
                }
                if show_diamonds() {
                    CurrencyRow { resource: Resource::Diamonds, value: diamonds() }
                    CurrencyRow {
                        resource: Resource::Crystals,
                        value: crystals(),
                        rate: Some(derived.crystals_per_sec),
                    }
                    CurrencyRow {
                        resource: Resource::Offerings,
                        value: offerings(),
                        rate: Some(derived.offerings_per_sec),
                    }
                }
                if show_mythos() {
                    CurrencyRow { resource: Resource::Mythos, value: mythos() }
                    CurrencyRow { resource: Resource::MythosShards, value: mythos_shards() }
                }
                if show_particles() {
                    CurrencyRow { resource: Resource::Particles, value: particles() }
                    CurrencyRow {
                        resource: Resource::Obtainium,
                        value: obtainium(),
                        rate: Some(derived.obtainium_per_sec),
                    }
                }
                if show_quarks() {
                    CurrencyRow { resource: Resource::Quarks, value: quarks() }
                }
                if show_gq() {
                    CurrencyRow { resource: Resource::GoldenQuarks, value: golden_quarks() }
                }
            }

            if show_stats {
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

                if show_diamonds() {
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
}
