//! Action builders: read-only derivations that turn a click + current state
//! into a fully-formed [`PlayerAction`].
//!
//! The buy functions take *input bundles* (challenge flags, the
//! `getReductionValue()` cost divisor) that the legacy UI computed at click
//! time. Composition at the boundary is this crate's job — but the actual
//! math lives in the logic crate ([`reduction_value`] /
//! [`producer_cost_input`] are the same functions the autobuyers use), so
//! manual and automatic purchases can never disagree on cost.

use synergismforkd_bignum::Decimal;
use synergismforkd_logic::events::ProducerType;
use synergismforkd_logic::mechanics::accelerators::BuyAcceleratorInput;
use synergismforkd_logic::mechanics::challenges::{calc_ecc, ChallengeType};
use synergismforkd_logic::mechanics::crystal_upgrades::BuyCrystalUpgradesInput;
use synergismforkd_logic::mechanics::multipliers::BuyMultiplierInput;
use synergismforkd_logic::mechanics::producers::{BuyMaxInput, BuyProducerInput};
use synergismforkd_logic::mechanics::rune_data::RuneUpgradeKind;
use synergismforkd_logic::mechanics::rune_levels::{
    max_rune_level_purchase, BuyRuneLevelsInput, MaxRuneLevelPurchaseInput,
};
use synergismforkd_logic::mechanics::rune_upgrade_progression::{
    max_rune_upgrade_purchase, BuyRuneUpgradeInput, MaxRuneUpgradePurchaseInput,
};
use synergismforkd_logic::state::BuyAmount as LogicBuyAmount;
use synergismforkd_logic::tick::{rune_effective_levels_per_oom, rune_exp_per_offering};
use synergismforkd_logic::{
    producer_cost_input, reduction_value, BuyRequest, ClickUpgradesUnlocks, GameState, PlayerAction,
};

use crate::bridge::BuyAmount;

/// Map the UI's buy-amount selector onto the logic tier's per-click cap.
/// `Max` is handled by the caller (it routes to a buy-max request), so it
/// maps to the largest legacy cap here as a safe fallback.
fn to_logic_amount(amount: BuyAmount) -> LogicBuyAmount {
    match amount {
        BuyAmount::One => LogicBuyAmount::One,
        BuyAmount::Ten => LogicBuyAmount::Ten,
        BuyAmount::Hundred => LogicBuyAmount::Hundred,
        BuyAmount::Thousand => LogicBuyAmount::Thousand,
        BuyAmount::Max => LogicBuyAmount::HundredThousand,
    }
}

fn transcend_ecc(state: &GameState) -> f64 {
    calc_ecc(
        ChallengeType::Transcend,
        state.challenges.challenge_completions[4],
    )
}

/// Producer purchase (`index` 1..=5). `Max` becomes a buy-max request;
/// anything else a capped manual-click loop.
#[must_use]
pub fn producer_buy(
    state: &GameState,
    producer_type: ProducerType,
    index: u8,
    amount: BuyAmount,
) -> PlayerAction {
    match amount.cap() {
        None => PlayerAction::Buy(BuyRequest::ProducerMax(BuyMaxInput {
            index,
            producer_type,
            cost_input: producer_cost_input(state),
        })),
        Some(buyamount) => PlayerAction::Buy(BuyRequest::Producer(BuyProducerInput {
            index,
            producer_type,
            autobuyer: false,
            buyamount,
            r: reduction_value(state),
            in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
            in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
            challengecompletions_4: state.challenges.challenge_completions[4],
            challengecompletions_8: state.challenges.challenge_completions[8],
        })),
    }
}

/// Accelerator purchase. The accelerator cost uses `G.costDivisor` (always
/// 1), not the reduction value — mirrors `auto_buy::accelerator_input`.
#[must_use]
pub fn accelerator_buy(state: &GameState, amount: BuyAmount) -> PlayerAction {
    PlayerAction::Buy(BuyRequest::Accelerator(BuyAcceleratorInput {
        autobuyer: false,
        coinbuyamount: to_logic_amount(amount),
        cost_divisor: 1.0,
        transcend_ecc: transcend_ecc(state),
        in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
        in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
    }))
}

/// Multiplier purchase — same input shape as [`accelerator_buy`].
#[must_use]
pub fn multiplier_buy(state: &GameState, amount: BuyAmount) -> PlayerAction {
    PlayerAction::Buy(BuyRequest::Multiplier(BuyMultiplierInput {
        autobuyer: false,
        coinbuyamount: to_logic_amount(amount),
        cost_divisor: 1.0,
        transcend_ecc: transcend_ecc(state),
        in_transcension_challenge_4: state.challenges.current_transcension_challenge == 4,
        in_reincarnation_challenge_8: state.challenges.current_reincarnation_challenge == 8,
    }))
}

/// Crystal-upgrade log10 base costs (`G.crystalUpgradesCost`).
const CRYSTAL_BASE_COST: [f64; 8] = [6.0, 15.0, 20.0, 40.0, 100.0, 200.0, 500.0, 1000.0];
/// Crystal-upgrade per-level log10 increments (`G.crystalUpgradeCostIncrement`).
const CRYSTAL_COST_INCREMENT: [f64; 8] = [8.0, 15.0, 20.0, 40.0, 100.0, 200.0, 500.0, 1000.0];

/// Crystal-upgrade purchase (`i` in `1..=8`). Buys as many levels as the
/// prestige-shard balance affords (the mechanic solves the max analytically).
/// `prism_cost_divisor_log10` is `0.0` until the Prism rune effect is surfaced
/// (cost-neutral).
#[must_use]
pub fn crystal_upgrade_buy(state: &GameState, i: u8) -> PlayerAction {
    let u = usize::from(i - 1);
    PlayerAction::Buy(BuyRequest::CrystalUpgrade(BuyCrystalUpgradesInput {
        i,
        auto: false,
        prism_cost_divisor_log10: 0.0,
        crystal_upgrades_cost: CRYSTAL_BASE_COST[u],
        crystal_upgrade_cost_increment: CRYSTAL_COST_INCREMENT[u],
        upgrade_73: f64::from(state.upgrades.upgrades[73]),
        in_any_reincarnation_challenge: state.challenges.current_reincarnation_challenge != 0,
    }))
}

/// Shop-upgrade purchase (`idx` in `1..=125`). Routes through the logic tier's
/// `clickUpgrades` dispatcher, which picks the currency/mechanic from the index
/// (coin / diamond / mythos / particle / automation / generator) and gates on
/// the upgrade being unowned + the relevant tier unlock. The single buy path
/// for every shop, so manual and auto purchases can't disagree on routing.
#[must_use]
pub fn upgrade_buy(state: &GameState, idx: usize) -> PlayerAction {
    PlayerAction::Buy(BuyRequest::ClickUpgrade {
        i: idx,
        unlocks: ClickUpgradesUnlocks {
            prestige: state.reset_counters.prestige_unlocked,
            transcend: state.reset_counters.transcend_unlocked,
            reincarnate: state.reset_counters.reincarnate_unlocked,
        },
    })
}

// ─── Runes / blessings / spirits ────────────────────────────────────────────

/// Offering buy-amount for a rune/blessing/spirit spend (the legacy
/// `offeringbuyamount` toggle). `Fixed` adds exactly that many levels (capped
/// by the offerings budget); `Max` buys the most the balance affords.
#[derive(Clone, Copy, PartialEq)]
pub enum RuneBuyAmount {
    /// Add exactly this many levels (budget-capped).
    Fixed(f64),
    /// Buy the most levels the offerings balance affords.
    Max,
}

/// Top-level rune level purchase by spending offerings. Cost coefficient and
/// slope come from the logic rune-data table; the EXP-per-offering and any
/// OOM-increase come from the same logic helpers the (future) auto-sacrifice
/// path will use, so manual and auto can't disagree.
#[must_use]
pub fn rune_buy(state: &GameState, rune: usize, amount: RuneBuyAmount) -> PlayerAction {
    let kind = RuneUpgradeKind::Rune;
    let cost_coefficient = kind.cost_coefficient(rune);
    let levels_per_oom = rune_effective_levels_per_oom(state, rune, kind.levels_per_oom(rune));
    let exp_per_offering = rune_exp_per_offering(state, rune);
    let budget = state.automation.offerings;
    let current_level = state.runes.rune_levels.get(rune).copied().unwrap_or(0.0);
    let current_rune_exp =
        Decimal::from_finite(state.runes.rune_exp.get(rune).copied().unwrap_or(0.0));

    let levels_to_add = match amount {
        RuneBuyAmount::Fixed(n) => n,
        RuneBuyAmount::Max => {
            max_rune_level_purchase(MaxRuneLevelPurchaseInput {
                cost_coefficient,
                levels_per_oom,
                current_level,
                current_rune_exp,
                rune_exp_per_offering: exp_per_offering,
                budget,
                is_unlocked: true,
            })
            .levels
        }
    };

    PlayerAction::Buy(BuyRequest::RuneLevels(BuyRuneLevelsInput {
        index: rune,
        cost_coefficient,
        levels_per_oom,
        rune_exp_per_offering: exp_per_offering,
        levels_to_add,
        budget,
    }))
}

/// Build the shared blessing/spirit buy input. Blessings/spirits spend
/// offerings at the salvage EXP rate (`= 1` until salvage unlocks), not the
/// universal rune mult, and have no OOM-increase term.
fn rune_upgrade_input(
    state: &GameState,
    kind: RuneUpgradeKind,
    index: usize,
    amount: RuneBuyAmount,
    current_level: f64,
    current_exp: f64,
) -> BuyRuneUpgradeInput {
    let cost_coefficient = kind.cost_coefficient(index);
    let levels_per_oom = kind.levels_per_oom(index);
    let exp_per_offering = Decimal::one();
    let budget = state.automation.offerings;
    let current_rune_exp = Decimal::from_finite(current_exp);

    let levels_to_add = match amount {
        RuneBuyAmount::Fixed(n) => n,
        RuneBuyAmount::Max => {
            max_rune_upgrade_purchase(MaxRuneUpgradePurchaseInput {
                cost_coefficient,
                levels_per_oom,
                current_level,
                current_rune_exp,
                rune_exp_per_offering: exp_per_offering,
                budget,
                upper_limit: 1e9,
                min_offerings_floor: Decimal::one(),
            })
            .levels
        }
    };

    BuyRuneUpgradeInput {
        index,
        cost_coefficient,
        levels_per_oom,
        rune_exp_per_offering: exp_per_offering,
        levels_to_add,
        budget,
    }
}

/// Rune blessing level purchase by spending offerings.
#[must_use]
pub fn rune_blessing_buy(state: &GameState, index: usize, amount: RuneBuyAmount) -> PlayerAction {
    let level = state
        .runes
        .rune_blessing_levels
        .get(index)
        .copied()
        .unwrap_or(0.0);
    let exp = state
        .runes
        .rune_blessing_exp
        .get(index)
        .copied()
        .unwrap_or(0.0);
    PlayerAction::Buy(BuyRequest::RuneBlessing(rune_upgrade_input(
        state,
        RuneUpgradeKind::Blessing,
        index,
        amount,
        level,
        exp,
    )))
}

/// Rune spirit level purchase by spending offerings.
#[must_use]
pub fn rune_spirit_buy(state: &GameState, index: usize, amount: RuneBuyAmount) -> PlayerAction {
    let level = state
        .runes
        .rune_spirit_levels
        .get(index)
        .copied()
        .unwrap_or(0.0);
    let exp = state
        .runes
        .rune_spirit_exp
        .get(index)
        .copied()
        .unwrap_or(0.0);
    PlayerAction::Buy(BuyRequest::RuneSpirit(rune_upgrade_input(
        state,
        RuneUpgradeKind::Spirit,
        index,
        amount,
        level,
        exp,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_reduction_value_is_one() {
        let state = GameState::default();
        assert_eq!(reduction_value(&state), 1.0);
    }

    #[test]
    fn max_routes_to_buy_max_others_to_manual() {
        let state = GameState::default();
        let max = producer_buy(&state, ProducerType::Coin, 1, BuyAmount::Max);
        assert!(matches!(max, PlayerAction::Buy(BuyRequest::ProducerMax(_))));

        let manual = producer_buy(&state, ProducerType::Coin, 2, BuyAmount::Hundred);
        match manual {
            PlayerAction::Buy(BuyRequest::Producer(input)) => {
                assert_eq!(input.index, 2);
                assert_eq!(input.buyamount, 100.0);
                assert!(!input.autobuyer);
                assert_eq!(input.r, 1.0);
            }
            other => panic!("expected manual producer buy, got {other:?}"),
        }
    }

    #[test]
    fn accelerator_input_uses_unit_cost_divisor() {
        let state = GameState::default();
        match accelerator_buy(&state, BuyAmount::Ten) {
            PlayerAction::Buy(BuyRequest::Accelerator(input)) => {
                assert_eq!(input.cost_divisor, 1.0);
                assert!(!input.autobuyer);
            }
            other => panic!("expected accelerator buy, got {other:?}"),
        }
    }
}
