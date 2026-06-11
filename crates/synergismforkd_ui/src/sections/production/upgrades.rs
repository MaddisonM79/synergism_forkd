//! Upgrades: the six shop sections (Coin / Diamond / Mythos / Particle /
//! Automation / Generator), each a [`Collapsible`] holding a dense grid of
//! numbered squares (mirrors the Achievements screen). A shared detail card at
//! the top describes the hovered/focused upgrade; clicking a square buys it.
//! Shop/reveal metadata lives in [`super::upgrade_data`]; live effect values in
//! [`super::upgrade_effects`].

use dioxus::prelude::*;

use crate::bridge::{use_bridge, use_slice};
use crate::components::{Collapsible, Num, ResourceIcon};
use crate::derive;
use crate::i18n::t;

use super::upgrade_data::{meta, shop_upgrades, Shop};

#[component]
pub fn Upgrades() -> Element {
    // The shops with at least one revealed upgrade (reactive to unlocks). Coin
    // upgrades 1–5 are always revealed, so the Coin shop shows from the start —
    // the empty state is a defensive fallback only.
    let visible_shops = use_slice(|s| {
        Shop::ALL
            .into_iter()
            .filter(|&shop| shop_upgrades(shop).any(|m| m.revealed(s)))
            .collect::<Vec<_>>()
    });
    // Which upgrade the detail card describes (hover/focus/click driven).
    let focused = use_signal(|| None::<usize>);

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.upgrades")} }
        }
        match focused() {
            Some(idx) => rsx! { UpgradeDetail { idx } },
            None => rsx! {
                div { class: "sf-upg-detail muted", {t("upgrades.hover_hint")} }
            },
        }
        if visible_shops().is_empty() {
            div { class: "sf-empty-state",
                div { class: "sf-empty-title", {t("upgrades.empty_title")} }
                div { class: "sf-empty-msg", {t("upgrades.empty_msg")} }
            }
        }
        for shop in visible_shops() {
            Collapsible { key: "{shop:?}", title: t(shop.title_key()).to_string(),
                ShopGrid { shop, focused }
            }
        }
    }
}

/// The revealed upgrade squares for one shop, in ascending index (= legacy row)
/// order.
#[component]
fn ShopGrid(shop: Shop, focused: Signal<Option<usize>>) -> Element {
    let indices = use_slice(move |s| {
        shop_upgrades(shop)
            .filter(|m| m.revealed(s))
            .map(|m| m.idx)
            .collect::<Vec<_>>()
    });
    rsx! {
        div { class: "sf-upg-grid",
            for idx in indices() {
                UpgradeCell { key: "{idx}", idx, focused }
            }
        }
    }
}

/// One upgrade square: shows the index, lights by state (owned / affordable /
/// locked), updates the detail card on hover/focus, and buys on click.
#[component]
fn UpgradeCell(idx: usize, focused: Signal<Option<usize>>) -> Element {
    let bridge = use_bridge();
    let mut focused = focused;
    let owned = use_slice(move |s| meta(idx).owned(s));
    let affordable = use_slice(move |s| meta(idx).affordable(s));

    let cls = if owned() {
        "sf-upg-cell owned"
    } else if affordable() {
        "sf-upg-cell can"
    } else {
        "sf-upg-cell"
    };

    rsx! {
        div {
            class: cls,
            tabindex: "0",
            onmouseenter: move |_| focused.set(Some(idx)),
            onfocus: move |_| focused.set(Some(idx)),
            onclick: move |_| bridge.dispatch(derive::upgrade_buy(&bridge.state.peek(), idx)),
            "{idx}"
        }
    }
}

/// The shared detail card above the grids: name, status, cost, and live effect
/// for the focused upgrade. Reads state/derived inline (one card, so re-rendering
/// each tick is cheap) — this also keeps it correct as the focused index changes
/// without per-index memo capture.
#[component]
fn UpgradeDetail(idx: usize) -> Element {
    let bridge = use_bridge();
    let m = meta(idx);
    let state = bridge.state.read();
    let derived = bridge.derived.read();
    let notation = bridge.prefs.read().notation;

    let owned = m.owned(&state);
    let affordable = m.affordable(&state);
    let cost = m.cost();
    let name = t(&format!("upgrades.descriptions.{idx}")).to_string();
    let effect = super::upgrade_effects::effect_text(idx, &state, &derived, notation);

    let (status_key, status_cls) = if owned {
        ("upgrades.bought", "sf-upg-status owned")
    } else if affordable {
        ("upgrades.status_affordable", "sf-upg-status can")
    } else {
        ("upgrades.status_locked", "sf-upg-status")
    };

    rsx! {
        div { class: "sf-upg-detail",
            div { class: "sf-upg-detail-head",
                span { class: "sf-upg-detail-num", "#{idx}" }
                span { class: "sf-upg-detail-name", "{name}" }
                span { class: status_cls, {t(status_key)} }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost }
                    " "
                    ResourceIcon { resource: m.resource }
                }
            }
            if let Some(line) = effect {
                div { class: "sf-upg-detail-effect", "{line}" }
            }
        }
    }
}
