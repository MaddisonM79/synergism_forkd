//! Buildings: the vertical slice's playable section. Coin sub-tab is live
//! (5 producer cards + accelerators/multipliers + prestige); the other
//! family sub-tabs reveal with their unlocks and fill in at M2.

use dioxus::prelude::*;
use synergismforkd_logic::events::ProducerType;
use synergismforkd_logic::{PlayerAction, ResetRequest};

use crate::bridge::{use_bridge, use_slice, BuyAmount};
use crate::components::{Num, Resource, ResourceIcon, Tooltip};
use crate::derive;
use crate::i18n::t;

/// Buildings sub-tabs, unlock-gated like everything else in the nav.
const SUBTABS: [(&str, &str); 5] = [
    ("buildings.subtab.coin", "coin"),
    ("buildings.subtab.diamond", "diamond"),
    ("buildings.subtab.mythos", "mythos"),
    ("buildings.subtab.particle", "particle"),
    ("buildings.subtab.tesseract", "tesseract"),
];

#[component]
pub fn Buildings() -> Element {
    let bridge = use_bridge();
    let subsection = bridge.route.read().subsection;
    let unlocked: Memo<[bool; 5]> = use_slice(|s| {
        [
            true,
            s.reset_counters.prestige_unlocked,
            s.reset_counters.transcend_unlocked,
            s.reset_counters.reincarnate_unlocked,
            s.reset_counters.ascension_count > 0.0,
        ]
    });

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.buildings")} }
            BuyAmountToggle {}
        }
        if unlocked()[1] {
            div { class: "sf-seg", style: "margin-bottom: var(--space-4)",
                for (i, (key, id)) in SUBTABS.iter().enumerate() {
                    if unlocked()[i] {
                        button {
                            key: "{id}",
                            class: if i == subsection { "active" } else { "" },
                            onclick: move |_| {
                                let mut route = bridge.route;
                                route.write().subsection = i;
                            },
                            {t(key)}
                        }
                    }
                }
            }
        }
        match subsection {
            0 => rsx! { CoinBuildings {} },
            _ => rsx! { div { class: "sf-placeholder", {t("nav.placeholder")} } },
        }
    }
}

/// The 1/10/100/1k/MAX selector (one preference, applies to every buy on
/// the page).
#[component]
fn BuyAmountToggle() -> Element {
    let bridge = use_bridge();
    let current = bridge.prefs.read().buy_amount;
    rsx! {
        span { class: "label", {t("buildings.buy_amount")} }
        div { class: "sf-seg",
            for amount in BuyAmount::ALL {
                button {
                    key: "{amount.label()}",
                    class: if amount == current { "active" } else { "" },
                    onclick: move |_| {
                        let mut prefs = bridge.prefs;
                        prefs.write().buy_amount = amount;
                    },
                    {amount.label()}
                }
            }
        }
    }
}

#[component]
fn CoinBuildings() -> Element {
    rsx! {
        div { class: "sf-card-grid",
            for index in 1..=5u8 {
                CoinProducerCard { key: "{index}", index }
            }
        }
        div { class: "sf-buildings-meta",
            AcceleratorCard {}
            MultiplierCard {}
            PrestigeCard {}
        }
    }
}

#[component]
fn CoinProducerCard(index: u8) -> Element {
    let bridge = use_bridge();
    let owned = use_slice(move |s| s.coin_producers.owned(index));
    let cost = use_slice(move |s| s.coin_producers.cost(index));
    let affordable = use_slice(move |s| s.upgrades.coins >= s.coin_producers.cost(index));
    let name_key = match index {
        1 => "buildings.coin.1",
        2 => "buildings.coin.2",
        3 => "buildings.coin.3",
        4 => "buildings.coin.4",
        _ => "buildings.coin.5",
    };

    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        let action = derive::producer_buy(&bridge.state.peek(), ProducerType::Coin, index, amount);
        bridge.dispatch(action);
    };
    let buy_max = move |_| {
        let action = derive::producer_buy(
            &bridge.state.peek(),
            ProducerType::Coin,
            index,
            BuyAmount::Max,
        );
        bridge.dispatch(action);
    };

    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t(name_key)} }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.owned")} }
                Num { value: synergismforkd_bignum::Decimal::from_finite(owned()) }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost() }
                    " "
                    ResourceIcon { resource: Resource::Coins }
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                button { disabled: !affordable(), onclick: buy_max, {t("buildings.buy_max")} }
            }
        }
    }
}

#[component]
fn AcceleratorCard() -> Element {
    let bridge = use_bridge();
    let owned = use_slice(|s| s.accelerator.accelerator_bought);
    let cost = use_slice(|s| s.accelerator.accelerator_cost);
    let affordable = use_slice(|s| s.upgrades.coins >= s.accelerator.accelerator_cost);
    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        bridge.dispatch(derive::accelerator_buy(&bridge.state.peek(), amount));
    };
    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t("buildings.accelerators")} }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.owned")} }
                Num { value: synergismforkd_bignum::Decimal::from_finite(owned()) }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost() }
                    " "
                    ResourceIcon { resource: Resource::Coins }
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
            }
        }
    }
}

#[component]
fn MultiplierCard() -> Element {
    let bridge = use_bridge();
    let owned = use_slice(|s| s.multiplier.multiplier_bought);
    let cost = use_slice(|s| s.multiplier.multiplier_cost);
    let affordable = use_slice(|s| s.upgrades.coins >= s.multiplier.multiplier_cost);
    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        bridge.dispatch(derive::multiplier_buy(&bridge.state.peek(), amount));
    };
    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t("buildings.multipliers")} }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.owned")} }
                Num { value: synergismforkd_bignum::Decimal::from_finite(owned()) }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost() }
                    " "
                    ResourceIcon { resource: Resource::Coins }
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
            }
        }
    }
}

/// Prestige: gain preview from `TickOutput.derived`, optional confirm,
/// dispatches the reset request.
#[component]
fn PrestigeCard() -> Element {
    let bridge = use_bridge();
    let gain = bridge.derived.read().prestige_gain;
    let available = gain >= synergismforkd_bignum::Decimal::one();

    let do_prestige = use_callback(move |()| {
        bridge.dispatch(PlayerAction::Reset(ResetRequest::Prestige));
    });
    let on_click = move |_| {
        if bridge.prefs.peek().confirm_resets {
            bridge.confirm(
                "dialogs.confirm_prestige.title",
                "dialogs.confirm_prestige.body",
                do_prestige,
            );
        } else {
            do_prestige.call(());
        }
    };

    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t("buildings.prestige")} }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.prestige_gain")} }
                Tooltip {
                    tip: rsx! { span { {t("hud.diamonds")} } },
                    span {
                        "+"
                        Num { value: gain }
                        " "
                        ResourceIcon { resource: Resource::Diamonds }
                    }
                }
            }
            div { class: "sf-card-actions",
                button {
                    class: "sf-prestige-btn",
                    disabled: !available,
                    onclick: on_click,
                    {t("buildings.prestige")}
                }
            }
        }
    }
}
