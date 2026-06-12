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
use synergismforkd_logic::{AutoToggle, BuyRequest, PlayerAction, ResetRequest};

use crate::bridge::{use_bridge, use_slice, use_slow_slice, BuyAmount};
use crate::components::{Collapsible, Num, Progress, Resource, ResourceIcon};
use crate::derive;
use crate::detail::{use_detail, BuildingDetail, DetailTarget, ResetKind};
use crate::format::format_value;
use crate::i18n::{t, t_args};

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
    // The Mythos building family reveals once you've transcended (legacy
    // `transcendunlock`), just as Diamond reveals at the prestige unlock.
    let show_mythos = use_slice(|s| s.reset_counters.transcend_unlocked);

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
        if show_mythos() {
            Collapsible { title: t("buildings.subtab.mythos").to_string(),
                MythosBuildings {}
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

/// A building autobuyer enable toggle, shown once the autobuyer is unlocked.
/// `index` is the legacy `player.toggles` slot (1..=26); flips
/// `automation.toggles[index]` so the `updateAll` driver buys this building.
#[component]
fn AutoBuyToggle(index: usize) -> Element {
    let bridge = use_bridge();
    let on = use_slice(move |s| s.automation.toggles[index]);
    rsx! {
        button {
            class: if on() { "sf-auto-toggle on" } else { "sf-auto-toggle" },
            "aria-pressed": "{on()}",
            onclick: move |_| {
                bridge.dispatch(PlayerAction::ToggleAuto {
                    target: AutoToggle::BuildingAutobuy(index),
                    enabled: !on(),
                });
            },
            {t(if on() { "buildings.auto_on" } else { "buildings.auto_off" })}
        }
    }
}

#[component]
fn CoinProducerCard(index: u8) -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let owned = use_slice(move |s| s.coin_producers.owned(index));
    // Autobuyer unlocks via automation upgrade 80+t (Upgrades → Automation).
    let auto_unlocked = use_slice(move |s| s.upgrades.upgrades[80 + index as usize] == 1);
    let generated = use_slice(move |s| s.coin_producers.tiers[(index - 1) as usize].generated);
    let cost = use_slice(move |s| s.coin_producers.cost(index));
    // Throttled to 5 Hz (legacy buttoncolorchange): drives the Buy button's
    // disabled state, which would otherwise strobe at 20 Hz under the autobuyer.
    let affordable = use_slow_slice(move |s| s.upgrades.coins >= s.coin_producers.cost(index));
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
    // Per-second output + % of total production live in the bottom panel now.
    let target = DetailTarget::Building(BuildingDetail::CoinProducer(index));
    let accent = Resource::Coins.css_color();

    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t(name_key)} }
            OwnedRow { owned: owned(), generated: generated() }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                if auto_unlocked() {
                    AutoBuyToggle { index: index as usize }
                }
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
        CrystalBonusLine {}
        div { class: "sf-card-grid",
            for i in 1..=5u8 {
                if i <= 2 || show_crystal_345() {
                    CrystalUpgradeCard { key: "c{i}", i }
                }
            }
        }
    }
}

/// The legacy "You have X Crystals, multiplying Coin production by Y×" line —
/// makes the Crystals → Coin bonus visible (it's applied in the coin
/// multiplier chain but was otherwise invisible).
#[component]
fn CrystalBonusLine() -> Element {
    let bridge = use_bridge();
    let crystals = use_slice(|s| s.crystal_upgrades.prestige_shards);
    let mult = bridge.derived.read().buildings.crystal_coin_multiplier;
    let notation = bridge.prefs.read().notation;
    rsx! {
        div { class: "sf-crystal-bonus",
            {t_args(
                "upgrades.crystal_bonus",
                &[
                    ("crystals", &format_value(crystals(), notation)),
                    ("mult", &format_value(mult, notation)),
                ],
            )}
        }
    }
}

/// A crystal upgrade (prestige-shard ladder shown under Diamonds): level, cost
/// in crystals, live effect (1/2/5), and a buy-to-max button.
#[component]
fn CrystalUpgradeCard(i: u8) -> Element {
    use crate::sections::production::upgrade_effects::crystal_cost;

    let bridge = use_bridge();
    let detail = use_detail();
    let level = use_slice(move |s| s.crystal_upgrades.crystal_upgrades[(i - 1) as usize]);
    let cost = use_slice(move |s| crystal_cost(i, s));
    let affordable =
        use_slow_slice(move |s| s.crystal_upgrades.prestige_shards >= crystal_cost(i, s));
    let notation = bridge.prefs.read().notation;
    // Compact card title; the full descriptive name + formula + effect live in
    // the hover/detail box.
    let name = t_args("upgrades.crystal_short", &[("n", &i.to_string())]);

    let buy = move |_| {
        bridge.dispatch(derive::crystal_upgrade_buy(&bridge.state.peek(), i));
    };
    // Full name + formula + live effect live in the bottom panel now.
    let target = DetailTarget::CrystalUpgrade(i);
    let accent = Resource::Crystals.css_color();

    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", "{name}" }
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
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
            }
        }
    }
}

#[component]
fn DiamondProducerCard(index: u8) -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let owned = use_slice(move |s| s.diamond_producers.owned(index));
    let generated = use_slice(move |s| s.diamond_producers.tiers[(index - 1) as usize].generated);
    let cost = use_slice(move |s| s.diamond_producers.cost(index));
    let affordable =
        use_slow_slice(move |s| s.upgrades.prestige_points >= s.diamond_producers.cost(index));
    // Diamond-producer autobuyers unlock via Synergism-level milestones
    // (tier-N crystal autobuy); toggle slot 9 + tier.
    let auto_unlocked = use_slice(move |s| {
        use synergismforkd_logic::mechanics::achievement_levels::achievement_level_from_points;
        use synergismforkd_logic::mechanics::level_milestones::{
            get_level_milestone, LevelMilestoneKey,
        };
        let key = match index {
            1 => LevelMilestoneKey::Tier1CrystalAutobuy,
            2 => LevelMilestoneKey::Tier2CrystalAutobuy,
            3 => LevelMilestoneKey::Tier3CrystalAutobuy,
            4 => LevelMilestoneKey::Tier4CrystalAutobuy,
            _ => LevelMilestoneKey::Tier5CrystalAutobuy,
        };
        let level = achievement_level_from_points(s.achievements.achievement_points);
        get_level_milestone(key, level) > 0.5
    });
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
    let target = DetailTarget::Building(BuildingDetail::Diamond(index));
    let accent = Resource::Diamonds.css_color();

    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t(name_key)} }
            OwnedRow { owned: owned(), generated: generated() }
            CostRow { cost: cost(), resource: Resource::Diamonds }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                if auto_unlocked() {
                    AutoBuyToggle { index: 9 + index as usize }
                }
            }
        }
    }
}

/// Mythos (transcension-tier) producers. Spend Mythos (transcend points); all
/// five tiers reveal together once transcension is unlocked (legacy
/// `transcendunlock`). Bought via the shared `BuyRequest::Producer` path routed
/// to `mythos_producers` by `ProducerType::Mythos`.
#[component]
fn MythosBuildings() -> Element {
    rsx! {
        div { class: "sf-card-grid",
            for index in 1..=5u8 {
                MythosProducerCard { key: "{index}", index }
            }
        }
    }
}

#[component]
fn MythosProducerCard(index: u8) -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let owned = use_slice(move |s| s.mythos_producers.owned(index));
    let generated = use_slice(move |s| s.mythos_producers.tiers[(index - 1) as usize].generated);
    let cost = use_slice(move |s| s.mythos_producers.cost(index));
    let affordable =
        use_slow_slice(move |s| s.upgrades.transcend_points >= s.mythos_producers.cost(index));
    // Mythos-producer autobuyers unlock via shop upgrades 93 + index (94–98,
    // the "Automatically buy Augments/…/Grandmasters" upgrades); toggle slot
    // 15 + index (the legacy `player.toggles[16..=20]`).
    let auto_unlocked = use_slice(move |s| s.upgrades.upgrades[93 + index as usize] == 1);
    let name_key = match index {
        1 => "buildings.mythos.1",
        2 => "buildings.mythos.2",
        3 => "buildings.mythos.3",
        4 => "buildings.mythos.4",
        _ => "buildings.mythos.5",
    };

    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        let action =
            derive::producer_buy(&bridge.state.peek(), ProducerType::Mythos, index, amount);
        bridge.dispatch(action);
    };
    let target = DetailTarget::Building(BuildingDetail::Mythos(index));
    let accent = Resource::Mythos.css_color();

    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t(name_key)} }
            OwnedRow { owned: owned(), generated: generated() }
            CostRow { cost: cost(), resource: Resource::Mythos }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                if auto_unlocked() {
                    AutoBuyToggle { index: 15 + index as usize }
                }
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
    let detail = use_detail();
    let owned = use_slice(|s| s.accelerator.accelerator_bought);
    let cost = use_slice(|s| s.accelerator.accelerator_cost);
    let affordable = use_slow_slice(|s| s.upgrades.coins >= s.accelerator.accelerator_cost);
    let auto_unlocked = use_slice(|s| s.upgrades.upgrades[86] == 1);
    let generated = bridge.derived.read().buildings.free_accelerator;
    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        bridge.dispatch(derive::accelerator_buy(&bridge.state.peek(), amount));
    };
    // Power % + Effect × live in the bottom panel now.
    let target = DetailTarget::Building(BuildingDetail::Accelerator);
    let accent = Resource::Coins.css_color();
    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t("buildings.accelerators")} }
            OwnedRow {
                owned: owned(),
                generated: Decimal::from_finite(generated),
            }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                if auto_unlocked() {
                    AutoBuyToggle { index: 6 }
                }
            }
        }
    }
}

#[component]
fn MultiplierCard() -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let owned = use_slice(|s| s.multiplier.multiplier_bought);
    let cost = use_slice(|s| s.multiplier.multiplier_cost);
    let affordable = use_slow_slice(|s| s.upgrades.coins >= s.multiplier.multiplier_cost);
    let auto_unlocked = use_slice(|s| s.upgrades.upgrades[87] == 1);
    let generated = bridge.derived.read().buildings.free_multiplier;
    let buy = move |_| {
        let amount = bridge.prefs.peek().buy_amount;
        bridge.dispatch(derive::multiplier_buy(&bridge.state.peek(), amount));
    };
    // Power × + Effect × live in the bottom panel now.
    let target = DetailTarget::Building(BuildingDetail::Multiplier);
    let accent = Resource::Coins.css_color();
    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t("buildings.multipliers")} }
            OwnedRow {
                owned: owned(),
                generated: Decimal::from_finite(generated),
            }
            CostRow { cost: cost(), resource: Resource::Coins }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                if auto_unlocked() {
                    AutoBuyToggle { index: 7 }
                }
            }
        }
    }
}

/// Accelerator Boost: resets diamonds + diamond upgrades in exchange for
/// permanent acceleration power and free accelerators per boost.
#[component]
fn AcceleratorBoostCard() -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let owned = use_slice(|s| s.accelerator.accelerator_boost_bought);
    let cost = use_slice(|s| s.accelerator.accelerator_boost_cost);
    let affordable =
        use_slow_slice(|s| s.upgrades.prestige_points >= s.accelerator.accelerator_boost_cost);
    let auto_unlocked = use_slice(|s| s.upgrades.upgrades[88] == 1 && s.upgrades.upgrades[46] == 1);
    let generated = bridge.derived.read().buildings.free_accelerator_boost;
    let buy = move |_| {
        bridge.dispatch(PlayerAction::Buy(BuyRequest::AcceleratorBoost));
    };
    // Boost grants + the reset/no-reset consequence note live in the panel now.
    let target = DetailTarget::Building(BuildingDetail::AcceleratorBoost);
    let accent = Resource::Diamonds.css_color();
    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t("buildings.accelerator_boost")} }
            OwnedRow {
                owned: owned(),
                generated: Decimal::from_finite(generated),
            }
            CostRow { cost: cost(), resource: Resource::Diamonds }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("buildings.buy")} }
                if auto_unlocked() {
                    AutoBuyToggle { index: 8 }
                }
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
    /// Map to the detail panel's pub reset tag.
    fn to_kind(self) -> ResetKind {
        match self {
            ResetTier::Prestige => ResetKind::Prestige,
            ResetTier::Transcension => ResetKind::Transcension,
            ResetTier::Reincarnation => ResetKind::Reincarnation,
        }
    }

    /// Recover the tier from the detail panel's pub tag.
    fn from_kind(kind: ResetKind) -> Self {
        match kind {
            ResetKind::Prestige => ResetTier::Prestige,
            ResetKind::Transcension => ResetTier::Transcension,
            ResetKind::Reincarnation => ResetTier::Reincarnation,
        }
    }

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

    /// Whether this tier's reset should pop a confirm dialog, per the
    /// independent Settings toggles.
    fn confirm_enabled(self, prefs: &crate::bridge::UiPrefs) -> bool {
        match self {
            ResetTier::Prestige => prefs.confirm_prestige,
            ResetTier::Transcension => prefs.confirm_transcension,
            ResetTier::Reincarnation => prefs.confirm_reincarnation,
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

    /// Reset availability + progress, faithful to the legacy `resetCheck`
    /// thresholds (Synergism.ts:3551 / 3559 / 3629):
    /// - prestige: `coinsThisPrestige ≥ 1e16` **or** gain ≥ 100
    /// - transcension: (`coinsThisTranscension ≥ 1e100` **or** gain ≥ 0.5) and
    ///   not inside a transcension challenge
    /// - reincarnation: gain > 0.5 and not inside a transcension/reincarnation
    ///   challenge (purely gain-gated — no coin threshold)
    fn gate(self, s: &synergismforkd_logic::GameState, gain: Decimal) -> ResetGate {
        let g = gain.to_number();
        match self {
            ResetTier::Prestige => {
                let current = s.coin_counters.coins_this_prestige;
                let threshold = Decimal::from_finite(1e16);
                ResetGate {
                    available: current >= threshold || g >= 100.0,
                    current,
                    threshold,
                    resource: Resource::Coins,
                    fraction: log_fraction(current, threshold),
                }
            }
            ResetTier::Transcension => {
                let current = s.coin_counters.coins_this_transcension;
                let threshold = Decimal::from_finite(1e100);
                let in_challenge = s.challenges.current_transcension_challenge != 0;
                ResetGate {
                    available: (current >= threshold || g >= 0.5) && !in_challenge,
                    current,
                    threshold,
                    resource: Resource::Coins,
                    fraction: log_fraction(current, threshold),
                }
            }
            ResetTier::Reincarnation => {
                let in_challenge = s.challenges.current_transcension_challenge != 0
                    || s.challenges.current_reincarnation_challenge != 0;
                let threshold = Decimal::from_finite(0.5);
                ResetGate {
                    available: g > 0.5 && !in_challenge,
                    current: gain,
                    threshold,
                    resource: Resource::Particles,
                    fraction: (g / 0.5).clamp(0.0, 1.0),
                }
            }
        }
    }
}

/// Reset availability + the "requires" / progress-bar inputs for one tier.
struct ResetGate {
    /// Whether the reset may be performed now (the legacy `resetCheck` gate).
    available: bool,
    /// The metric shown in the "requires" line (coins for prestige/transcend,
    /// the gain for reincarnation).
    current: Decimal,
    /// The threshold that metric is measured against.
    threshold: Decimal,
    /// Icon for the requires line.
    resource: Resource,
    /// Progress-bar fill, 0..=1.
    fraction: f64,
}

/// Log-scale progress toward a huge coin threshold, so the bar shows real
/// movement instead of sitting at ~0 until the very end.
fn log_fraction(current: Decimal, threshold: Decimal) -> f64 {
    let cur = current.to_number();
    let thr = threshold.to_number();
    if cur <= 1.0 || thr <= 1.0 {
        0.0
    } else {
        (cur.log10() / thr.log10()).clamp(0.0, 1.0)
    }
}

/// A reset card (prestige / transcension / reincarnation): the gain
/// preview, a log-scale progress bar toward the minimum resource the reset
/// needs to award its first point, and the reset button (gated until the
/// minimum is met).
#[component]
fn ResetCard(tier: ResetTier) -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let gain = match tier {
        ResetTier::Prestige => bridge.derived.read().prestige_point_gain,
        ResetTier::Transcension => bridge.derived.read().transcend_point_gain,
        ResetTier::Reincarnation => bridge.derived.read().reincarnation_point_gain,
    };
    let gate = tier.gate(&bridge.state.read(), gain);
    let available = gate.available;
    let current = gate.current;
    let threshold = gate.threshold;
    let source_resource = gate.resource;
    let fraction = gate.fraction;
    let notation = bridge.prefs.read().notation;

    let do_reset = use_callback(move |()| {
        bridge.dispatch(PlayerAction::Reset(tier.request()));
    });
    let on_click = move |_| {
        if tier.confirm_enabled(&bridge.prefs.peek()) {
            let (tk, bk) = tier.confirm_keys();
            bridge.confirm(tk, bk, do_reset);
        } else {
            do_reset.call(());
        }
    };

    let target = DetailTarget::Reset(tier.to_kind());
    let accent = tier.gain_resource().css_color();

    rsx! {
        div {
            class: "sf-card",
            style: "--card-accent: {accent}",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
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
                    {format_value(threshold, notation)}
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

/// Reset body for the shared bottom detail panel: what the reset does, the
/// gain, and the requirement.
#[component]
pub fn ResetDetailBody(kind: ResetKind) -> Element {
    let bridge = use_bridge();
    let tier = ResetTier::from_kind(kind);
    let gain = match tier {
        ResetTier::Prestige => bridge.derived.read().prestige_point_gain,
        ResetTier::Transcension => bridge.derived.read().transcend_point_gain,
        ResetTier::Reincarnation => bridge.derived.read().reincarnation_point_gain,
    };
    let gate = tier.gate(&bridge.state.read(), gain);
    let notation = bridge.prefs.read().notation;
    let (_, body_key) = tier.confirm_keys();

    rsx! {
        div { class: "sf-detail-card",
            div { class: "sf-detail-head",
                span { class: "sf-detail-title", {t(tier.title_key())} }
            }
            div { class: "sf-upgrade-effect", {t(body_key)} }
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
                span {
                    {format_value(gate.current, notation)}
                    " / "
                    {format_value(gate.threshold, notation)}
                    " "
                    ResourceIcon { resource: gate.resource }
                }
            }
        }
    }
}

/// Crystal-upgrade body for the shared bottom detail panel: name, the legacy
/// math formula (1/2/3/5 only), level, cost, and the live effect line.
#[component]
pub fn CrystalDetailBody(i: u8) -> Element {
    use crate::sections::production::upgrade_effects::{crystal_cost, crystal_effect_text};

    let bridge = use_bridge();
    let state = bridge.state.read();
    let notation = bridge.prefs.read().notation;
    let name = t(&format!("upgrades.crystalUpgrades.{i}")).to_string();
    let level = state.crystal_upgrades.crystal_upgrades[(i - 1) as usize];
    let cost = crystal_cost(i, &state);
    let effect = crystal_effect_text(i, &state, notation);
    let formula_key = format!("upgrades.crystalFormula.{i}");
    let formula = t(&formula_key);
    let has_formula = formula != formula_key;

    rsx! {
        div { class: "sf-detail-card",
            div { class: "sf-detail-head",
                span { class: "sf-detail-title", "{name}" }
            }
            if has_formula {
                div { class: "sf-upgrade-formula", "{formula}" }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("upgrades.crystal_level")} }
                span { {format_value(Decimal::from_finite(level), notation)} }
            }
            CostRow { cost, resource: Resource::Crystals }
            if let Some(line) = effect {
                div { class: "sf-card-row sf-upgrade-effect", span { "{line}" } }
            }
        }
    }
}

/// Building-card body for the shared bottom detail panel: the full readout
/// (owned/cost plus the verbose derived rows the slim cards dropped).
#[component]
pub fn BuildingDetailBody(which: BuildingDetail) -> Element {
    let bridge = use_bridge();
    let state = bridge.state.read();
    let derived = bridge.derived.read();
    let b = &derived.buildings;
    let notation = bridge.prefs.read().notation;

    match which {
        BuildingDetail::CoinProducer(tier) => {
            let name_key = match tier {
                1 => "buildings.coin.1",
                2 => "buildings.coin.2",
                3 => "buildings.coin.3",
                4 => "buildings.coin.4",
                _ => "buildings.coin.5",
            };
            let owned = state.coin_producers.owned(tier);
            let generated = state.coin_producers.tiers[(tier - 1) as usize].generated;
            let cost = state.coin_producers.cost(tier);
            let produce = b.coin_produce[(tier - 1) as usize];
            let per_sec = produce / b.tax_divisor * Decimal::from_finite(40.0);
            let total = if b.coin_produce_total > Decimal::zero() {
                b.coin_produce_total
            } else {
                Decimal::one()
            };
            let percent = (produce / total).to_number() * 100.0;
            rsx! {
                div { class: "sf-detail-card",
                    div { class: "sf-detail-head",
                        span { class: "sf-detail-title", {t(name_key)} }
                    }
                    OwnedRow { owned, generated }
                    CostRow { cost, resource: Resource::Coins }
                    div { class: "sf-card-row",
                        span { class: "label", {t("buildings.per_sec")} }
                        span {
                            Num { value: per_sec }
                            span { class: "sf-free",
                                " ({format_value(Decimal::from_finite(percent), notation)}%)"
                            }
                        }
                    }
                }
            }
        }
        BuildingDetail::Diamond(tier) => {
            let name_key = match tier {
                1 => "buildings.diamond.1",
                2 => "buildings.diamond.2",
                3 => "buildings.diamond.3",
                4 => "buildings.diamond.4",
                _ => "buildings.diamond.5",
            };
            let owned = state.diamond_producers.owned(tier);
            let generated = state.diamond_producers.tiers[(tier - 1) as usize].generated;
            let cost = state.diamond_producers.cost(tier);
            rsx! {
                div { class: "sf-detail-card",
                    div { class: "sf-detail-head",
                        span { class: "sf-detail-title", {t(name_key)} }
                    }
                    OwnedRow { owned, generated }
                    CostRow { cost, resource: Resource::Diamonds }
                }
            }
        }
        BuildingDetail::Mythos(tier) => {
            let name_key = match tier {
                1 => "buildings.mythos.1",
                2 => "buildings.mythos.2",
                3 => "buildings.mythos.3",
                4 => "buildings.mythos.4",
                _ => "buildings.mythos.5",
            };
            let owned = state.mythos_producers.owned(tier);
            let generated = state.mythos_producers.tiers[(tier - 1) as usize].generated;
            let cost = state.mythos_producers.cost(tier);
            rsx! {
                div { class: "sf-detail-card",
                    div { class: "sf-detail-head",
                        span { class: "sf-detail-title", {t(name_key)} }
                    }
                    OwnedRow { owned, generated }
                    CostRow { cost, resource: Resource::Mythos }
                }
            }
        }
        BuildingDetail::Accelerator => {
            let owned = state.accelerator.accelerator_bought;
            let cost = state.accelerator.accelerator_cost;
            rsx! {
                div { class: "sf-detail-card",
                    div { class: "sf-detail-head",
                        span { class: "sf-detail-title", {t("buildings.accelerators")} }
                    }
                    OwnedRow { owned, generated: Decimal::from_finite(b.free_accelerator) }
                    CostRow { cost, resource: Resource::Coins }
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
                }
            }
        }
        BuildingDetail::Multiplier => {
            let owned = state.multiplier.multiplier_bought;
            let cost = state.multiplier.multiplier_cost;
            rsx! {
                div { class: "sf-detail-card",
                    div { class: "sf-detail-head",
                        span { class: "sf-detail-title", {t("buildings.multipliers")} }
                    }
                    OwnedRow { owned, generated: Decimal::from_finite(b.free_multiplier) }
                    CostRow { cost, resource: Resource::Coins }
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
                }
            }
        }
        BuildingDetail::AcceleratorBoost => {
            let owned = state.accelerator.accelerator_boost_bought;
            let cost = state.accelerator.accelerator_boost_cost;
            let no_reset = state.upgrades.upgrades[46] >= 1;
            rsx! {
                div { class: "sf-detail-card",
                    div { class: "sf-detail-head",
                        span { class: "sf-detail-title", {t("buildings.accelerator_boost")} }
                    }
                    OwnedRow { owned, generated: Decimal::from_finite(b.free_accelerator_boost) }
                    CostRow { cost, resource: Resource::Diamonds }
                    div { class: "sf-card-row",
                        span { class: "label", {t("buildings.boost_grants")} }
                        span { class: "sf-num",
                            "+{format_value(Decimal::from_finite(b.boost_power_percent), notation)}% · "
                            {format_value(Decimal::from_finite(b.accelerators_per_boost), notation)}
                            " "
                            {t("buildings.accelerators")}
                        }
                    }
                    if no_reset {
                        div { class: "sf-boost-note", {t("buildings.boost_safe")} }
                    } else {
                        div { class: "sf-boost-note warn", {t("buildings.boost_warning")} }
                    }
                }
            }
        }
    }
}
