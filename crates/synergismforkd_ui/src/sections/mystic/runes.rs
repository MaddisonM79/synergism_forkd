//! Runes: spend offerings to level the five core runes, plus their blessings
//! and spirits. Mirrors the building-card patterns — one shared offering
//! buy-amount selector drives per-rune spend buttons; affordability is
//! throttled to 5 Hz (legacy `buttoncolorchange`).

use dioxus::prelude::*;
use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::rune_data::RuneUpgradeKind;
use synergismforkd_logic::mechanics::rune_levels::rune_offerings_to_level;
use synergismforkd_logic::mechanics::rune_upgrade_progression::rune_upgrade_exp_left_to_level;
use synergismforkd_logic::tick::{first_five_effective_rune_level, rune_exp_per_offering};
use synergismforkd_logic::GameState;

use crate::bridge::{use_bridge, use_slice, use_slow_slice};
use crate::components::{Collapsible, Num, Resource, ResourceIcon};
use crate::derive::{self, RuneBuyAmount};
use crate::detail::{use_detail, DetailTarget, RuneKind};
use crate::format::format_value;
use crate::i18n::t;

use super::rune_data::{
    blessing_effect_line, blessings_unlocked, rune_effect_line, rune_name_key, rune_unlocked,
    spirit_effect_line, spirits_unlocked, CORE_RUNES,
};

/// Which family a rune card levels — selects the state arrays, cost
/// coefficients, and buy action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuneFamily {
    Rune,
    Blessing,
    Spirit,
}

impl RuneFamily {
    /// Map to the panel's pub family tag.
    fn to_kind(self) -> RuneKind {
        match self {
            RuneFamily::Rune => RuneKind::Rune,
            RuneFamily::Blessing => RuneKind::Blessing,
            RuneFamily::Spirit => RuneKind::Spirit,
        }
    }

    /// Recover the family from the panel's pub tag.
    fn from_kind(kind: RuneKind) -> Self {
        match kind {
            RuneKind::Rune => RuneFamily::Rune,
            RuneKind::Blessing => RuneFamily::Blessing,
            RuneKind::Spirit => RuneFamily::Spirit,
        }
    }

    fn kind(self) -> RuneUpgradeKind {
        match self {
            RuneFamily::Rune => RuneUpgradeKind::Rune,
            RuneFamily::Blessing => RuneUpgradeKind::Blessing,
            RuneFamily::Spirit => RuneUpgradeKind::Spirit,
        }
    }

    /// Current purchased level for rune `i` in this family.
    fn level(self, s: &GameState, i: usize) -> f64 {
        let arr = match self {
            RuneFamily::Rune => &s.runes.rune_levels,
            RuneFamily::Blessing => &s.runes.rune_blessing_levels,
            RuneFamily::Spirit => &s.runes.rune_spirit_levels,
        };
        arr.get(i).copied().unwrap_or(0.0)
    }

    fn current_exp(self, s: &GameState, i: usize) -> f64 {
        let arr = match self {
            RuneFamily::Rune => &s.runes.rune_exp,
            RuneFamily::Blessing => &s.runes.rune_blessing_exp,
            RuneFamily::Spirit => &s.runes.rune_spirit_exp,
        };
        arr.get(i).copied().unwrap_or(0.0)
    }

    /// EXP yielded per offering — universal mult for runes, salvage (`= 1`) for
    /// blessings/spirits.
    fn exp_per_offering(self, s: &GameState, i: usize) -> Decimal {
        match self {
            RuneFamily::Rune => rune_exp_per_offering(s, i),
            RuneFamily::Blessing | RuneFamily::Spirit => Decimal::one(),
        }
    }

    /// Offerings needed to reach the next level (display only).
    fn next_level_offerings(self, s: &GameState, i: usize) -> Decimal {
        let coeff = self.kind().cost_coefficient(i);
        let oom = self.kind().levels_per_oom(i);
        let current_exp = Decimal::from_finite(self.current_exp(s, i));
        let next = self.level(s, i) + 1.0;
        let per_off = self.exp_per_offering(s, i);
        match self {
            RuneFamily::Rune => rune_offerings_to_level(coeff, next, oom, current_exp, per_off),
            RuneFamily::Blessing | RuneFamily::Spirit => {
                let exp_left = rune_upgrade_exp_left_to_level(coeff, next, oom, current_exp);
                (exp_left / per_off).ceil().max(Decimal::one())
            }
        }
    }

    fn buy_action(
        self,
        s: &GameState,
        i: usize,
        amount: RuneBuyAmount,
    ) -> synergismforkd_logic::PlayerAction {
        match self {
            RuneFamily::Rune => derive::rune_buy(s, i, amount),
            RuneFamily::Blessing => derive::rune_blessing_buy(s, i, amount),
            RuneFamily::Spirit => derive::rune_spirit_buy(s, i, amount),
        }
    }
}

#[component]
pub fn Runes() -> Element {
    let bridge = use_bridge();
    let offerings = use_slice(|s| s.automation.offerings);
    // Persisted UI pref (survives reload), like the buildings buy-amount.
    let amount = bridge.prefs.read().offering_buy_amount;

    // Which runes / panels are revealed (reactive to research / unlocks).
    let visible_runes = use_slice(|s| {
        (0..CORE_RUNES)
            .filter(|&i| rune_unlocked(s, i))
            .collect::<Vec<_>>()
    });
    let show_blessings = use_slice(blessings_unlocked);
    let show_spirits = use_slice(spirits_unlocked);

    rsx! {
        div { class: "sf-section-head",
            h1 { {t("nav.section.runes")} }
            OfferingAmountToggle {}
        }
        div { class: "sf-rune-offerings",
            {t("runes.offerings")} ": "
            Num { value: offerings() }
            " "
            ResourceIcon { resource: Resource::Offerings }
        }
        div { class: "sf-card-grid",
            for i in visible_runes() {
                RuneCard { key: "r{i}", family: RuneFamily::Rune, index: i, amount: amount }
            }
        }
        if show_blessings() {
            Collapsible { title: t("runes.blessings").to_string(),
                div { class: "sf-card-grid",
                    for i in 0..CORE_RUNES {
                        RuneCard { key: "b{i}", family: RuneFamily::Blessing, index: i, amount: amount }
                    }
                }
            }
        }
        if show_spirits() {
            Collapsible { title: t("runes.spirits").to_string(),
                div { class: "sf-card-grid",
                    for i in 0..CORE_RUNES {
                        RuneCard { key: "s{i}", family: RuneFamily::Spirit, index: i, amount: amount }
                    }
                }
            }
        }
    }
}

/// The 1/10/100/1k/10k/MAX offering buy-amount selector. Reads/writes the
/// persisted `offering_buy_amount` pref directly, so the choice survives a
/// reload (mirrors the buildings `BuyAmountToggle`).
#[component]
fn OfferingAmountToggle() -> Element {
    const OPTIONS: [(&str, RuneBuyAmount); 6] = [
        ("1", RuneBuyAmount::Fixed(1.0)),
        ("10", RuneBuyAmount::Fixed(10.0)),
        ("100", RuneBuyAmount::Fixed(100.0)),
        ("1k", RuneBuyAmount::Fixed(1000.0)),
        ("10k", RuneBuyAmount::Fixed(10_000.0)),
        ("MAX", RuneBuyAmount::Max),
    ];
    let bridge = use_bridge();
    let current = bridge.prefs.read().offering_buy_amount;
    rsx! {
        span { class: "label", {t("runes.buy_amount")} }
        div { class: "sf-seg",
            for (label, value) in OPTIONS {
                button {
                    key: "{label}",
                    class: if current == value { "active" } else { "" },
                    onclick: move |_| {
                        let mut prefs = bridge.prefs;
                        prefs.write().offering_buy_amount = value;
                    },
                    "{label}"
                }
            }
        }
    }
}

#[component]
fn RuneCard(family: RuneFamily, index: usize, amount: RuneBuyAmount) -> Element {
    let bridge = use_bridge();
    let detail = use_detail();
    let level = use_slice(move |s| family.level(s, index));
    // Effective level only applies to top-level runes.
    let effective = use_slice(move |s| match family {
        RuneFamily::Rune => first_five_effective_rune_level(s, index),
        _ => family.level(s, index),
    });
    let cost = use_slice(move |s| family.next_level_offerings(s, index));
    let affordable =
        use_slow_slice(move |s| s.automation.offerings >= family.next_level_offerings(s, index));
    let notation = bridge.prefs.read().notation;

    let buy = move |_| {
        let action = family.buy_action(&bridge.state.peek(), index, amount);
        bridge.dispatch(action);
    };
    // The live effect line lives in the bottom panel now.
    let target = DetailTarget::Rune {
        family: family.to_kind(),
        index,
    };

    rsx! {
        div {
            class: "sf-card",
            tabindex: "0",
            onmouseenter: move |_| detail.set(target),
            onfocus: move |_| detail.set(target),
            div { class: "sf-card-title", {t(rune_name_key(index))} }
            div { class: "sf-card-row",
                span { class: "label", {t("runes.level")} }
                span {
                    {format_value(Decimal::from_finite(level()), notation)}
                    if family == RuneFamily::Rune {
                        span { class: "sf-free",
                            " → " {format_value(Decimal::from_finite(effective()), notation)}
                        }
                    }
                }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("runes.next")} }
                span {
                    Num { value: cost() }
                    " "
                    ResourceIcon { resource: Resource::Offerings }
                }
            }
            div { class: "sf-card-actions",
                button { disabled: !affordable(), onclick: buy, {t("runes.spend")} }
            }
        }
    }
}

/// Rune/blessing/spirit body for the shared bottom detail panel: name, level
/// (→ effective for top runes), the live effect line, and offerings-to-next.
#[component]
pub fn RuneDetailBody(family: RuneKind, index: usize) -> Element {
    let bridge = use_bridge();
    let family = RuneFamily::from_kind(family);
    let notation = bridge.prefs.read().notation;
    let state = bridge.state.read();
    let level = family.level(&state, index);
    let effective = match family {
        RuneFamily::Rune => first_five_effective_rune_level(&state, index),
        _ => level,
    };
    let cost = family.next_level_offerings(&state, index);
    let effect = match family {
        RuneFamily::Rune => rune_effect_line(&state, index, notation),
        RuneFamily::Blessing => blessing_effect_line(&state, index, notation),
        RuneFamily::Spirit => spirit_effect_line(&state, index, notation),
    };

    rsx! {
        div { class: "sf-detail-card",
            div { class: "sf-upg-detail-head",
                span { class: "sf-upg-detail-name", {t(rune_name_key(index))} }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("runes.level")} }
                span {
                    {format_value(Decimal::from_finite(level), notation)}
                    if family == RuneFamily::Rune {
                        span { class: "sf-free",
                            " → " {format_value(Decimal::from_finite(effective), notation)}
                        }
                    }
                }
            }
            div { class: "sf-card-row sf-upgrade-effect",
                span { "{effect}" }
            }
            div { class: "sf-card-row",
                span { class: "label", {t("runes.next")} }
                span {
                    Num { value: cost }
                    " "
                    ResourceIcon { resource: Resource::Offerings }
                }
            }
        }
    }
}
