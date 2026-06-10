//! Buildings: the vertical slice's playable section. Coin sub-tab is live;
//! the other family sub-tabs reveal with their unlocks and fill in at M2.
//!
//! Reveal ladder (legacy `coinunlock1..4` CSS gates, `revealStuff()`):
//! tier-2 producer + Accelerators at `coin_one` (coins ≥ 500), tier-3 +
//! Multipliers at `coin_two` (≥ 1e4), tier-4 + the buy-amount selector at
//! `coin_three` (≥ 1e5), tier-5 + Prestige at `coin_four` (≥ 4e6);
//! Accelerator Boost at prestige unlock. Per-row numbers come from
//! [`BuildingsDerived`] (the legacy `G.*` display surface).

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;
use synergismforkd_logic::events::ProducerType;
use synergismforkd_logic::{BuyRequest, PlayerAction, ResetRequest};

use crate::bridge::{use_bridge, use_slice, BuyAmount};
use crate::components::{Num, Resource, ResourceIcon, Tooltip};
use crate::derive;
use crate::format::format_value;
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
    let family_unlocked: Memo<[bool; 5]> = use_slice(|s| {
        [
            true,
            s.reset_counters.prestige_unlocked,
            s.reset_counters.transcend_unlocked,
            s.reset_counters.reincarnate_unlocked,
            s.reset_counters.ascension_count > 0.0,
        ]
    });
    // The legacy `coinunlock3` gate: the buy-amount selector is itself a
    // progression reward (coins ≥ 1e5).
    let amounts_unlocked = use_slice(|s| s.reset_counters.coin_three_unlocked);

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.buildings")} }
            if amounts_unlocked() {
                BuyAmountToggle {}
            }
        }
        if family_unlocked()[1] {
            div { class: "sf-seg", style: "margin-bottom: var(--space-4)",
                for (i, (key, id)) in SUBTABS.iter().enumerate() {
                    if family_unlocked()[i] {
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
/// the page). Revealed by the `coin_three` progression gate.
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
    let bridge = use_bridge();
    // Tier reveal ladder: tier 1 always; tiers 2..=5 behind coin_one..four.
    let tier_unlocked: Memo<[bool; 5]> = use_slice(|s| {
        let rc = &s.reset_counters;
        [
            true,
            rc.coin_one_unlocked,
            rc.coin_two_unlocked,
            rc.coin_three_unlocked,
            rc.coin_four_unlocked,
        ]
    });
    let show_accelerators = use_slice(|s| s.reset_counters.coin_one_unlocked);
    let show_multipliers = use_slice(|s| s.reset_counters.coin_two_unlocked);
    let show_prestige = use_slice(|s| s.reset_counters.coin_four_unlocked);
    let show_boost = use_slice(|s| s.reset_counters.prestige_unlocked);
    let tax_divisor = bridge.derived.read().buildings.tax_divisor;

    rsx! {
        div { class: "sf-card-grid",
            for index in 1..=5u8 {
                if tier_unlocked()[(index - 1) as usize] {
                    CoinProducerCard { key: "{index}", index }
                }
            }
        }
        if tax_divisor > Decimal::one() {
            TaxLine {}
        }
        div { class: "sf-buildings-meta",
            if show_accelerators() {
                AcceleratorCard {}
            }
            if show_multipliers() {
                MultiplierCard {}
            }
            if show_boost() {
                AcceleratorBoostCard {}
            }
            if show_prestige() {
                PrestigeCard {}
            }
        }
    }
}

/// "Owned N [+gen]" — the owned count plus the cascade-generated free
/// count, matching the legacy `amount [+gain]` display.
#[component]
fn OwnedRow(owned: f64, generated: Decimal) -> Element {
    rsx! {
        div { class: "sf-card-row",
            span { class: "label", {t("buildings.owned")} }
            span {
                Num { value: Decimal::from_finite(owned) }
                span { class: "sf-free", " +" , Num { value: generated } }
            }
        }
    }
}

#[component]
fn CostRow(cost: Decimal, resource: Resource) -> Element {
    rsx! {
        div { class: "sf-card-row",
            span { class: "label", {t("buildings.cost")} }
            span {
                Num { value: cost }
                " "
                ResourceIcon { resource }
            }
        }
    }
}

#[component]
fn CoinProducerCard(index: u8) -> Element {
    let bridge = use_bridge();
    let owned = use_slice(move |s| s.coin_producers.owned(index));
    let generated = use_slice(move |s| s.coin_producers.tiers[(index - 1) as usize].generated);
    let cost = use_slice(move |s| s.coin_producers.cost(index));
    let affordable = use_slice(move |s| s.upgrades.coins >= s.coin_producers.cost(index));
    let name_key = match index {
        1 => "buildings.coin.1",
        2 => "buildings.coin.2",
        3 => "buildings.coin.3",
        4 => "buildings.coin.4",
        _ => "buildings.coin.5",
    };

    // Per-second output + % of total production (legacy buildtext rows).
    let derived = bridge.derived.read();
    let b = &derived.buildings;
    let produce = b.coin_produce[(index - 1) as usize];
    let per_sec = produce / b.tax_divisor * Decimal::from_finite(40.0);
    let total = if b.coin_produce_total > Decimal::zero() {
        b.coin_produce_total
    } else {
        Decimal::one()
    };
    let percent = (produce / total).to_number() * 100.0;
    let notation = bridge.prefs.read().notation;

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
            OwnedRow { owned: owned(), generated: generated() }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.per_sec")} }
                Tooltip {
                    tip: rsx! { span { {format_value(Decimal::from_finite(percent), notation)} "% " {t("buildings.of_total")} } },
                    span {
                        Num { value: per_sec }
                        span { class: "sf-free", " ({format_value(Decimal::from_finite(percent), notation)}%)" }
                    }
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                button { disabled: !affordable(), onclick: buy_max, {t("buildings.buy_max")} }
            }
        }
    }
}

/// The wealth-tax banner (legacy `taxInfo`): production divisor + the
/// coins/sec hard cap.
#[component]
fn TaxLine() -> Element {
    let bridge = use_bridge();
    let derived = bridge.derived.read();
    let b = &derived.buildings;
    rsx! {
        div { class: "sf-tax-line",
            {t("buildings.tax_prefix")}
            " "
            Num { value: b.tax_divisor }
            " — "
            {t("buildings.tax_cap")}
            " "
            Num { value: b.coins_per_sec_cap }
            {t("hud.per_sec")}
        }
    }
}

#[component]
fn AcceleratorCard() -> Element {
    let bridge = use_bridge();
    let owned = use_slice(|s| s.accelerator.accelerator_bought);
    let cost = use_slice(|s| s.accelerator.accelerator_cost);
    let affordable = use_slice(|s| s.upgrades.coins >= s.accelerator.accelerator_cost);
    let derived = bridge.derived.read();
    let b = &derived.buildings;
    let notation = bridge.prefs.read().notation;
    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        bridge.dispatch(derive::accelerator_buy(&bridge.state.peek(), amount));
    };
    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t("buildings.accelerators")} }
            OwnedRow {
                owned: owned(),
                generated: Decimal::from_finite(b.free_accelerator),
            }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.power")} }
                span { class: "sf-num",
                    "{format_value(Decimal::from_finite(b.accelerator_power_percent), notation)}%"
                }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.effect")} }
                span {
                    Num { value: b.accelerator_effect, rate: true }
                    "×"
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
    let derived = bridge.derived.read();
    let b = &derived.buildings;
    let notation = bridge.prefs.read().notation;
    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        bridge.dispatch(derive::multiplier_buy(&bridge.state.peek(), amount));
    };
    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t("buildings.multipliers")} }
            OwnedRow {
                owned: owned(),
                generated: Decimal::from_finite(b.free_multiplier),
            }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.power")} }
                span { class: "sf-num",
                    {format_value(Decimal::from_finite(b.multiplier_power), notation)}
                    "×"
                }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.effect")} }
                span {
                    Num { value: b.multiplier_effect, rate: true }
                    "×"
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
            }
        }
    }
}

/// Accelerator Boost: resets diamonds + diamond upgrades in exchange for
/// permanent acceleration power and free accelerators per boost.
#[component]
fn AcceleratorBoostCard() -> Element {
    let bridge = use_bridge();
    let owned = use_slice(|s| s.accelerator.accelerator_boost_bought);
    let cost = use_slice(|s| s.accelerator.accelerator_boost_cost);
    let affordable =
        use_slice(|s| s.upgrades.prestige_points >= s.accelerator.accelerator_boost_cost);
    let derived = bridge.derived.read();
    let b = &derived.buildings;
    let notation = bridge.prefs.read().notation;
    let buy = move |_| {
        bridge.dispatch(PlayerAction::Buy(BuyRequest::AcceleratorBoost));
    };
    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t("buildings.accelerator_boost")} }
            OwnedRow {
                owned: owned(),
                generated: Decimal::from_finite(b.free_accelerator_boost),
            }
            CostRow { cost: cost(), resource: Resource::Diamonds }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.boost_grants")} }
                Tooltip {
                    tip: rsx! { span { {t("buildings.boost_warning")} } },
                    span { class: "sf-num",
                        "+{format_value(Decimal::from_finite(b.boost_power_percent), notation)}% · "
                        {format_value(Decimal::from_finite(b.accelerators_per_boost), notation)}
                        " "
                        {t("buildings.accelerators")}
                    }
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
    let gain = bridge.derived.read().prestige_point_gain;
    let available = gain >= Decimal::one();

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
