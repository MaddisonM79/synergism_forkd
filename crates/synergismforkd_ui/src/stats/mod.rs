//! The resource ledger — the left half of the bottom bar. Every unlocked
//! currency as `icon + value (+rate)`, wrapping into columns to fill the
//! wide-short panel. The live production/system stats live on their own
//! [`Stats page`](crate::sections::production::stats). Numbers come from
//! state slices + the tick's derived surface (`bridge.derived`).

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Num, Resource, ResourceIcon};
use crate::detail::{use_detail, DetailTarget};
use crate::i18n::t;

/// One currency chip in the top bar: icon + value (+rate). Hovering/focusing
/// it writes the resource to the bottom detail box (where its name and the
/// fuller readout live — the chip itself is name-less to stay compact).
#[component]
fn CurrencyRow(
    resource: Resource,
    value: Decimal,
    #[props(default)] rate: Option<Decimal>,
) -> Element {
    let detail = use_detail();
    rsx! {
        div {
            class: "sf-res-row",
            style: "--row-accent: {resource.css_color()}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(DetailTarget::Resource(resource)),
            onfocus: move |_| detail.set(DetailTarget::Resource(resource)),
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

/// Resource body for the bottom detail box: the name the compact chip drops,
/// plus amount, per-second rate (where one exists), and lifetime total (coins).
#[component]
pub fn ResourceDetailBody(resource: Resource) -> Element {
    let bridge = use_bridge();
    let state = bridge.state.read();
    let derived = bridge.derived.read();

    let value = match resource {
        Resource::Coins => state.upgrades.coins,
        Resource::Diamonds => state.upgrades.prestige_points,
        Resource::Crystals => state.crystal_upgrades.prestige_shards,
        Resource::Mythos => state.upgrades.transcend_points,
        Resource::MythosShards => state.reset_counters.transcend_shards,
        Resource::Particles => state.upgrades.reincarnation_points,
        Resource::Offerings => state.automation.offerings,
        Resource::Obtainium => state.researches.obtainium,
        Resource::Quarks => state.quarks.worlds,
        Resource::GoldenQuarks => state.golden_quarks.golden_quarks,
        Resource::Ambrosia => Decimal::zero(),
    };
    let rate = match resource {
        Resource::Coins => Some(derived.coins_per_sec),
        Resource::Crystals => Some(derived.crystals_per_sec),
        Resource::Offerings => Some(derived.offerings_per_sec),
        Resource::Obtainium => Some(derived.obtainium_per_sec),
        _ => None,
    };
    let lifetime = match resource {
        Resource::Coins => Some(state.coin_counters.coins_total),
        _ => None,
    };

    rsx! {
        div { class: "sf-detail-card", style: "--row-accent: {resource.css_color()}",
            div { class: "sf-detail-head",
                ResourceIcon { resource }
                span { class: "sf-detail-title", {t(resource.label_key())} }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("detail.amount")} }
                span { Num { value } }
            }
            if let Some(rate) = rate {
                div { class: "sf-card-row",
                    span { class: "label", {t("detail.per_sec")} }
                    span {
                        Num { value: rate, rate: true }
                        {t("hud.per_sec")}
                    }
                }
            }
            if let Some(lifetime) = lifetime {
                div { class: "sf-card-row",
                    span { class: "label", {t("hud.lifetime")} }
                    span { Num { value: lifetime } }
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

    rsx! {
        aside { class: "sf-stats",
            // ── Resource ledger ─────────────────────────────────────────────
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
        }
    }
}
