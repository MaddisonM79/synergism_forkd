//! Buildings: a single page with one collapsible section per producer
//! family (Coin live; Diamond live; Mythos/Particle/Tesseract land later).
//! Each family reveals at its reset unlock.
//!
//! Coin reveal ladder (legacy `coinunlock1..4` CSS gates, `revealStuff()`):
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
use crate::components::{Collapsible, Num, Progress, Resource, ResourceIcon, Tooltip};
use crate::derive;
use crate::format::format_value;
use crate::i18n::t;

#[component]
pub fn Buildings() -> Element {
    let show_diamond = use_slice(|s| s.reset_counters.prestige_unlocked);
    // The legacy `coinunlock3` gate: the buy-amount selector is itself a
    // progression reward (coins ≥ 1e5).
    let amounts_unlocked = use_slice(|s| s.reset_counters.coin_three_unlocked);
    // Reset gates (lifted here so the reset buttons sit in a strip above the
    // producers rather than mixed into the coin building row). Prestige reveals
    // at coins ≥ 4e6; transcension after a prestige; reincarnation after a
    // transcension (mirrors the reset progression).
    let show_prestige = use_slice(|s| s.reset_counters.coin_four_unlocked);
    let show_transcend = use_slice(|s| s.reset_counters.prestige_unlocked);
    let show_reincarnate = use_slice(|s| s.reset_counters.transcend_unlocked);
    let any_reset = show_prestige() || show_transcend() || show_reincarnate();

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.buildings")} }
            if amounts_unlocked() {
                BuyAmountToggle {}
            }
        }
        if any_reset {
            div { class: "sf-reset-strip",
                if show_prestige() {
                    ResetCard { tier: ResetTier::Prestige }
                }
                if show_transcend() {
                    ResetCard { tier: ResetTier::Transcension }
                }
                if show_reincarnate() {
                    ResetCard { tier: ResetTier::Reincarnation }
                }
            }
        }
        Collapsible { title: t("buildings.subtab.coin").to_string(),
            CoinBuildings {}
        }
        if show_diamond() {
            Collapsible { title: t("buildings.subtab.diamond").to_string(),
                DiamondBuildings {}
            }
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

    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t(name_key)} }
            OwnedRow { owned: owned(), generated: generated() }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-row sf-persec",
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
            }
        }
    }
}

/// Diamond (prestige-tier) producers. Spend prestige points (diamonds);
/// all five tiers reveal together once prestige is unlocked (the legacy
/// `prestigeunlock` class — no per-tier gate). Bought via the same
/// `BuyRequest::Producer` path, routed to `diamond_producers` by
/// `ProducerType::Diamonds`.
#[component]
fn DiamondBuildings() -> Element {
    // Crystal upgrades 1–2 reveal with the Diamond subtab; 3–5 behind transcend
    // (legacy `transcendunlock` on `buycrystalupgrade3..5`).
    let show_crystal_345 = use_slice(|s| s.reset_counters.transcend_unlocked);
    rsx! {
        div { class: "sf-card-grid",
            for index in 1..=5u8 {
                DiamondProducerCard { key: "{index}", index }
            }
        }
        div { class: "sf-collapsible-title sf-crystal-head", {t("upgrades.crystalUpgrades.heading")} }
        div { class: "sf-card-grid",
            for i in 1..=5u8 {
                if i <= 2 || show_crystal_345() {
                    CrystalUpgradeCard { key: "c{i}", i }
                }
            }
        }
    }
}

/// A crystal upgrade (prestige-shard ladder shown under Diamonds): level, cost
/// in crystals, live effect (1/2/5), and a buy-to-max button.
#[component]
fn CrystalUpgradeCard(i: u8) -> Element {
    use crate::sections::production::upgrade_effects::{crystal_cost, crystal_effect_text};

    let bridge = use_bridge();
    let level = use_slice(move |s| s.crystal_upgrades.crystal_upgrades[(i - 1) as usize]);
    let cost = use_slice(move |s| crystal_cost(i, s));
    let affordable = use_slice(move |s| s.crystal_upgrades.prestige_shards >= crystal_cost(i, s));
    let notation = bridge.prefs.read().notation;
    let effect = use_slice(move |s| crystal_effect_text(i, s, bridge.prefs.peek().notation));
    let name = t(&format!("upgrades.crystalUpgrades.{i}")).to_string();
    // The legacy explicit math formula (crystals 1/2/3/5 only). A missing key
    // echoes itself, so render the line only when a real formula is present.
    let formula_key = format!("upgrades.crystalFormula.{i}");
    let formula = t(&formula_key);
    let has_formula = formula != formula_key;

    let buy = move |_| {
        bridge.dispatch(derive::crystal_upgrade_buy(&bridge.state.peek(), i));
    };

    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", "{name}" }
            if has_formula {
                div { class: "sf-upgrade-formula", "{formula}" }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("upgrades.crystal_level")} }
                span { {format_value(Decimal::from_finite(level()), notation)} }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.cost")} }
                span {
                    Num { value: cost() }
                    " "
                    ResourceIcon { resource: Resource::Crystals }
                }
            }
            if let Some(line) = effect() {
                div { class: "sf-card-row sf-upgrade-effect",
                    span { "{line}" }
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
            }
        }
    }
}

#[component]
fn DiamondProducerCard(index: u8) -> Element {
    let bridge = use_bridge();
    let owned = use_slice(move |s| s.diamond_producers.owned(index));
    let generated = use_slice(move |s| s.diamond_producers.tiers[(index - 1) as usize].generated);
    let cost = use_slice(move |s| s.diamond_producers.cost(index));
    let affordable =
        use_slice(move |s| s.upgrades.prestige_points >= s.diamond_producers.cost(index));
    let name_key = match index {
        1 => "buildings.diamond.1",
        2 => "buildings.diamond.2",
        3 => "buildings.diamond.3",
        4 => "buildings.diamond.4",
        _ => "buildings.diamond.5",
    };

    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        let action =
            derive::producer_buy(&bridge.state.peek(), ProducerType::Diamonds, index, amount);
        bridge.dispatch(action);
    };

    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t(name_key)} }
            OwnedRow { owned: owned(), generated: generated() }
            CostRow { cost: cost(), resource: Resource::Diamonds }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
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

/// Which reset tier a [`ResetCard`] drives.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ResetTier {
    Prestige,
    Transcension,
    Reincarnation,
}

impl ResetTier {
    /// Card title / button label key.
    fn title_key(self) -> &'static str {
        match self {
            ResetTier::Prestige => "buildings.prestige",
            ResetTier::Transcension => "buildings.transcend",
            ResetTier::Reincarnation => "buildings.reincarnate",
        }
    }

    /// Confirm-dialog keys (title, body).
    fn confirm_keys(self) -> (&'static str, &'static str) {
        match self {
            ResetTier::Prestige => (
                "dialogs.confirm_prestige.title",
                "dialogs.confirm_prestige.body",
            ),
            ResetTier::Transcension => (
                "dialogs.confirm_transcend.title",
                "dialogs.confirm_transcend.body",
            ),
            ResetTier::Reincarnation => (
                "dialogs.confirm_reincarnate.title",
                "dialogs.confirm_reincarnate.body",
            ),
        }
    }

    fn request(self) -> ResetRequest {
        match self {
            ResetTier::Prestige => ResetRequest::Prestige,
            ResetTier::Transcension => ResetRequest::Transcension,
            ResetTier::Reincarnation => ResetRequest::Reincarnation,
        }
    }

    /// The currency this reset awards.
    fn gain_resource(self) -> Resource {
        match self {
            ResetTier::Prestige => Resource::Diamonds,
            ResetTier::Transcension => Resource::Mythos,
            ResetTier::Reincarnation => Resource::Particles,
        }
    }

    /// The resource accumulated toward the first point, and the threshold
    /// at which the reset awards ≥ 1 (the divisor in the gain formula —
    /// `x^p ≥ 1 ⟺ x ≥ divisor` for `p > 0`). Coins for prestige/transcend,
    /// mythos shards for reincarnation.
    fn source(self, s: &synergismforkd_logic::GameState) -> (Decimal, f64, Resource) {
        match self {
            ResetTier::Prestige => (s.coin_counters.coins_this_prestige, 1e12, Resource::Coins),
            ResetTier::Transcension => (
                s.coin_counters.coins_this_transcension,
                1e100,
                Resource::Coins,
            ),
            ResetTier::Reincarnation => {
                (s.reset_counters.transcend_shards, 1e300, Resource::Mythos)
            }
        }
    }
}

/// A reset card (prestige / transcension / reincarnation): the gain
/// preview, a log-scale progress bar toward the minimum resource the reset
/// needs to award its first point, and the reset button (gated until the
/// minimum is met).
#[component]
fn ResetCard(tier: ResetTier) -> Element {
    let bridge = use_bridge();
    let gain = match tier {
        ResetTier::Prestige => bridge.derived.read().prestige_point_gain,
        ResetTier::Transcension => bridge.derived.read().transcend_point_gain,
        ResetTier::Reincarnation => bridge.derived.read().reincarnation_point_gain,
    };
    let available = gain >= Decimal::one();
    let (current, threshold, source_resource) = use_slice(move |s| tier.source(s))();
    let notation = bridge.prefs.read().notation;

    // Log-scale fill so the bar shows meaningful progress against the huge
    // thresholds (1e12 … 1e300) instead of sitting at ~0 until the end.
    let fraction = {
        let cur = current.to_number();
        if cur <= 1.0 {
            0.0
        } else {
            (cur.log10() / threshold.log10()).clamp(0.0, 1.0)
        }
    };

    let do_reset = use_callback(move |()| {
        bridge.dispatch(PlayerAction::Reset(tier.request()));
    });
    let on_click = move |_| {
        if bridge.prefs.peek().confirm_resets {
            let (tk, bk) = tier.confirm_keys();
            bridge.confirm(tk, bk, do_reset);
        } else {
            do_reset.call(());
        }
    };

    rsx! {
        div { class: "sf-card",
            div { class: "sf-card-title", {t(tier.title_key())} }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.prestige_gain")} }
                span {
                    "+"
                    Num { value: gain }
                    " "
                    ResourceIcon { resource: tier.gain_resource() }
                }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("buildings.requires")} }
                span { class: "sf-num",
                    {format_value(current, notation)}
                    " / "
                    {format_value(Decimal::from_finite(threshold), notation)}
                    " "
                    ResourceIcon { resource: source_resource }
                }
            }
            Progress { fraction }
            div { class: "sf-card-actions",
                button {
                    class: "sf-prestige-btn",
                    disabled: !available,
                    onclick: on_click,
                    {t(tier.title_key())}
                }
            }
        }
    }
}
