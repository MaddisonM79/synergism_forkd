//! Upgrades: the six shop sections (Coin / Diamond / Mythos / Particle /
//! Automation / Generator), one [`Collapsible`] each, shown only when at least
//! one of the shop's upgrades is revealed. Each card shows its name, cost +
//! currency icon, and a buy button gated on owned/affordable. Live effect
//! values land in a later milestone; the shop metadata lives in
//! [`super::upgrade_data`].

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

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.upgrades")} }
        }
        if visible_shops().is_empty() {
            div { class: "sf-empty-state",
                div { class: "sf-empty-title", {t("upgrades.empty_title")} }
                div { class: "sf-empty-msg", {t("upgrades.empty_msg")} }
            }
        }
        for shop in visible_shops() {
            Collapsible { key: "{shop:?}", title: t(shop.title_key()).to_string(),
                ShopGrid { shop }
            }
        }
    }
}

/// The revealed upgrade cards for one shop, in ascending index (= legacy row)
/// order.
#[component]
fn ShopGrid(shop: Shop) -> Element {
    let indices = use_slice(move |s| {
        shop_upgrades(shop)
            .filter(|m| m.revealed(s))
            .map(|m| m.idx)
            .collect::<Vec<_>>()
    });
    rsx! {
        div { class: "sf-card-grid",
            for idx in indices() {
                UpgradeCard { key: "{idx}", idx }
            }
        }
    }
}

/// One shop upgrade: name, cost, owned/affordable state, buy.
#[component]
fn UpgradeCard(idx: usize) -> Element {
    let bridge = use_bridge();
    let m = meta(idx);
    let cost = m.cost();
    let owned = use_slice(move |s| meta(idx).owned(s));
    let affordable = use_slice(move |s| meta(idx).affordable(s));
    let name = t(&format!("upgrades.descriptions.{idx}")).to_string();

    let buy = move |_| {
        bridge.dispatch(derive::upgrade_buy(&bridge.state.peek(), idx));
    };

    rsx! {
        div { class: if owned() { "sf-card bought" } else { "sf-card" },
            div { class: "sf-card-title", "{name}" }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost }
                    " "
                    ResourceIcon { resource: m.resource }
                }
            }
            div { class: "sf-card-actions",
                button {
                    disabled: owned() || !affordable(),
                    onclick: buy,
                    if owned() { {t("upgrades.bought")} } else { {t("buildings.buy")} }
                }
            }
        }
    }
}
