//! Action builders: read-only derivations that turn a click + current state
//! into a fully-formed [`PlayerAction`].
//!
//! The buy functions take *input bundles* (challenge flags, the
//! `getReductionValue()` cost divisor) that the legacy UI computed at click
//! time. Composition at the boundary is this crate's job — but the actual
//! math lives in the logic crate ([`reduction_value`] /
//! [`producer_cost_input`] are the same functions the autobuyers use), so
//! manual and automatic purchases can never disagree on cost.

use synergismforkd_logic::events::ProducerType;
use synergismforkd_logic::mechanics::accelerators::BuyAcceleratorInput;
use synergismforkd_logic::mechanics::challenges::{calc_ecc, ChallengeType};
use synergismforkd_logic::mechanics::multipliers::BuyMultiplierInput;
use synergismforkd_logic::mechanics::producers::{BuyMaxInput, BuyProducerInput};
use synergismforkd_logic::state::BuyAmount as LogicBuyAmount;
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
