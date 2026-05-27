//! Accelerator cost formula and purchase loop.
//!
//! Verbatim port of `getCostAccelerator` + `buyAccelerator` from
//! `legacy/core_split/packages/logic/src/mechanics/accelerators.ts`.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::math::smallest_inc::smallest_inc;
use crate::state::{AcceleratorState, BuyAmount};

const BUYMAX: f64 = 1e15;

// Same f64 safe-integer-window guard as
// [`crate::mechanics::multipliers`] — the +1/-1 arithmetic in the
// binary-search recursion requires BUYMAX < 2^53.
const _: () = assert!(BUYMAX < (1_u64 << 53) as f64);

/// Input to [`get_cost_accelerator`]. Mirrors `GetCostAcceleratorInput` in
/// the TS source — the `player.*` / `G.*` reads from the legacy `web_ui`
/// hoisted into an explicit parameter for portability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetCostAcceleratorInput {
    /// `G.costDivisor` at call time (computed in the UI tick from
    /// runes/researches/ant upgrades).
    pub cost_divisor: f64,
    /// `CalcECC('transcend', player.challengecompletions[4])` — Eternal
    /// Challenge transcend completions.
    pub transcend_ecc: f64,
    /// `player.currentChallenge.transcension === 4`
    pub in_transcension_challenge_4: bool,
    /// `player.currentChallenge.reincarnation === 8`
    pub in_reincarnation_challenge_8: bool,
}

/// Cost in coins of buying the `buying_to`-th accelerator (1-indexed; the
/// formula internally decrements to 0-index, matching the TS source).
pub fn get_cost_accelerator(buying_to: f64, input: GetCostAcceleratorInput) -> Decimal {
    let buying_to = buying_to - 1.0;

    let original_cost = 500.0;
    let mut cost = Decimal::from_finite(original_cost);

    cost *= Decimal::from_finite(4.0 / input.cost_divisor).pow(Decimal::from_finite(buying_to));

    let transcend_break = 5.0 * input.transcend_ecc;
    if buying_to > 125.0 + transcend_break {
        let num = buying_to - 125.0 - transcend_break;
        let factorial_bit = Decimal::from_finite(num).factorial();
        let mult_bit = Decimal::from_finite(4.0).pow(Decimal::from_finite(num));
        cost *= mult_bit * factorial_bit;
    }

    if buying_to > 2000.0 + transcend_break {
        let sum_num = buying_to - 2000.0 - transcend_break;
        let sum_bit = sum_num * (sum_num + 1.0) / 2.0;
        cost *= Decimal::from_finite(2.0).pow(Decimal::from_finite(sum_bit));
    }

    if input.in_transcension_challenge_4 {
        let sum_bit = buying_to * (buying_to + 1.0) / 2.0;
        cost *= Decimal::from_finite(10.0).pow(Decimal::from_finite(sum_bit));
    }

    if input.in_reincarnation_challenge_8 {
        let sum_bit = buying_to * (buying_to + 1.0) / 2.0;
        cost *= Decimal::from_finite(1e50).pow(Decimal::from_finite(sum_bit));
    }

    if buying_to > BUYMAX {
        let diminishing_exponent = 1.0 / 8.0;
        // Recurse with `BUYMAX + 1.0` since the TS function decrements before
        // checking — passing `BUYMAX` would make the recursive call's
        // `buying_to` equal to `BUYMAX - 1`, below the diminishing-tail
        // threshold.
        let quadrillion_cost = get_cost_accelerator(BUYMAX + 1.0, input);
        let mut new_cost = quadrillion_cost.pow(Decimal::from_finite(
            (buying_to / BUYMAX).powf(1.0 / diminishing_exponent),
        ));
        // Re-normalize after the in-place mantissa/exponent rewrite. Matches
        // the legacy break_infinity.js massaging.
        let new_extra = new_cost.exponent() - new_cost.exponent().floor();
        new_cost.set_exponent(new_cost.exponent().floor());
        new_cost.set_mantissa(new_cost.mantissa() * 10.0_f64.powf(new_extra));
        new_cost.normalize();
        return cost.max(new_cost);
    }
    cost
}

/// Input to [`buy_accelerator`]. Mirrors `BuyAcceleratorInput` in the TS
/// source. Extends [`GetCostAcceleratorInput`] with the per-click cap and
/// the autobuyer flag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyAcceleratorInput {
    /// True when the autobuyer is driving — bypasses the per-click
    /// `coinbuyamount` cap.
    pub autobuyer: bool,
    /// Per-click purchase cap selected in the UI.
    pub coinbuyamount: BuyAmount,
    /// `G.costDivisor` at call time (computed in the UI tick from
    /// runes/researches/ant upgrades).
    pub cost_divisor: f64,
    /// `CalcECC('transcend', player.challengecompletions[4])` — Eternal
    /// Challenge transcend completions.
    pub transcend_ecc: f64,
    /// `player.currentChallenge.transcension === 4`
    pub in_transcension_challenge_4: bool,
    /// `player.currentChallenge.reincarnation === 8`
    pub in_reincarnation_challenge_8: bool,
}

impl BuyAcceleratorInput {
    fn cost_input(self) -> GetCostAcceleratorInput {
        GetCostAcceleratorInput {
            cost_divisor: self.cost_divisor,
            transcend_ecc: self.transcend_ecc,
            in_transcension_challenge_4: self.in_transcension_challenge_4,
            in_reincarnation_challenge_8: self.in_reincarnation_challenge_8,
        }
    }
}

/// Buy as many accelerators as possible given the current coin balance and
/// per-click cap. Verbatim port of `buyAccelerator` from
/// `legacy/core_split/packages/logic/src/mechanics/accelerators.ts` (in turn
/// hoisted from `packages/web_ui/src/Buy.ts:72`).
///
/// Two paths:
/// - **High-end** (`acceleratorBought >= BUYMAX`): binary-search for the
///   largest affordable count and snap to it. The cost function diminishes
///   so aggressively in this range that post-hoc cost accounting matches
///   the buy.
/// - **Normal**: bracket the target with a 4× doubling search, refine with
///   a stepdown loop, then walk forward subtracting cost each step.
#[must_use]
pub fn buy_accelerator(
    state: &mut AcceleratorState,
    coins: &mut Decimal,
    input: BuyAcceleratorInput,
) -> SmallVec<[CoreEvent; 4]> {
    let cost_input = input.cost_input();
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let starting_coins = *coins;
    let buy_start = state.accelerator_bought;

    // High-end binary search path.
    if buy_start >= BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        let log10_resource = coins.log10().to_number();
        let log10_quadrillion_cost = get_cost_accelerator(BUYMAX, cost_input).log10().to_number();

        let mut hi = (BUYMAX
            * (1.0_f64).max((log10_resource / log10_quadrillion_cost).powf(diminishing_exponent)))
        .floor();
        let mut lo = BUYMAX;
        // Iteration cap is defense-in-depth: the loop terminates
        // naturally when `mid` converges to an endpoint (inner break),
        // but at extreme magnitudes f64 precision can stall the
        // `hi - lo > 0.5` condition. 128 iters is well above
        // log2(f64::MAX) and bounds tick latency.
        for _ in 0..128 {
            if hi - lo <= 0.5 {
                break;
            }
            let mid = (lo + (hi - lo) / 2.0).floor();
            if mid == lo || mid == hi {
                break;
            }
            if *coins < get_cost_accelerator(mid, cost_input) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let buyable = lo;
        state.accelerator_bought = buyable;
        state.accelerator_cost = get_cost_accelerator(buyable, cost_input);
        if state.accelerator_bought > 0.0 {
            state.prestige_no_accelerator = false;
            state.transcend_no_accelerator = false;
            state.reincarnate_no_accelerator = false;
        }
        if state.accelerator_bought > buy_start {
            events.push(CoreEvent::AcceleratorsPurchased {
                before: buy_start,
                after: state.accelerator_bought,
                spent: starting_coins - *coins,
            });
        }
        return events;
    }

    // Normal path: bracket with 4× doubling, refine with stepdown, walk forward.
    let buydefault = buy_start + smallest_inc(buy_start);
    let mut buy_to = buydefault;

    let mut cash_to_buy = get_cost_accelerator(buy_to, cost_input);
    while *coins >= cash_to_buy {
        buy_to *= 4.0;
        cash_to_buy = get_cost_accelerator(buy_to, cost_input);
    }
    let mut stepdown = (buy_to / 8.0).floor();
    while stepdown >= smallest_inc(buy_to) {
        if get_cost_accelerator(buy_to - stepdown, cost_input) <= *coins {
            stepdown = (stepdown / 2.0).floor();
        } else {
            buy_to -= smallest_inc(buy_to).max(stepdown);
        }
    }

    // Per-click cap (only when not autobuying).
    if !input.autobuyer {
        let cap_to = state.accelerator_bought + input.coinbuyamount.as_f64();
        if cap_to < buy_to {
            buy_to = cap_to;
        }
    }

    let mut buy_from = (buy_to - 6.0 - smallest_inc(buy_to)).max(buydefault);
    let mut this_cost = get_cost_accelerator(buy_from, cost_input);
    while buy_from <= buy_to && *coins >= this_cost {
        if buy_from >= BUYMAX {
            buy_from = BUYMAX;
        }
        *coins -= this_cost;
        state.accelerator_bought = buy_from;
        buy_from += smallest_inc(buy_from);
        this_cost = get_cost_accelerator(buy_from, cost_input);
        state.accelerator_cost = this_cost;
        if buy_from >= BUYMAX {
            break;
        }
    }

    if state.accelerator_bought > 0.0 {
        state.prestige_no_accelerator = false;
        state.transcend_no_accelerator = false;
        state.reincarnate_no_accelerator = false;
    }

    if state.accelerator_bought > buy_start {
        events.push(CoreEvent::AcceleratorsPurchased {
            before: buy_start,
            after: state.accelerator_bought,
            spent: starting_coins - *coins,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> GetCostAcceleratorInput {
        GetCostAcceleratorInput {
            cost_divisor: 1.0,
            transcend_ecc: 0.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
        }
    }

    #[test]
    fn first_accelerator_is_original_cost() {
        // buying_to = 1 → decremented to 0 → cost = 500 * (4/1)^0 = 500
        let cost = get_cost_accelerator(1.0, baseline());
        assert!((cost.to_number() - 500.0).abs() < 1e-9);
    }

    #[test]
    fn second_accelerator_applies_first_factor() {
        // buying_to = 2 → decremented to 1 → cost = 500 * 4 = 2000
        let cost = get_cost_accelerator(2.0, baseline());
        assert!((cost.to_number() - 2000.0).abs() < 1e-6);
    }

    #[test]
    fn cost_divisor_reduces_growth() {
        // cost_divisor = 2 → growth factor = 4/2 = 2 (instead of 4).
        let input = GetCostAcceleratorInput {
            cost_divisor: 2.0,
            ..baseline()
        };
        // buying_to = 2 → cost = 500 * 2 = 1000
        let cost = get_cost_accelerator(2.0, input);
        assert!((cost.to_number() - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn cost_strictly_increases_below_threshold() {
        let inp = baseline();
        let a = get_cost_accelerator(5.0, inp);
        let b = get_cost_accelerator(6.0, inp);
        let c = get_cost_accelerator(50.0, inp);
        assert!(a < b);
        assert!(b < c);
    }

    #[test]
    fn factorial_branch_kicks_in_past_125() {
        // Past buying_to = 126 (level = 125 > 125 false; at 127, level = 126 > 125)
        let inp = baseline();
        let cost_126 = get_cost_accelerator(126.0, inp);
        let cost_127 = get_cost_accelerator(127.0, inp);
        // Big jump expected when factorial term kicks in.
        assert!(cost_127 > cost_126);
    }

    #[test]
    fn transcend_challenge_4_multiplies_cost() {
        let inp = baseline();
        let challenged = GetCostAcceleratorInput {
            in_transcension_challenge_4: true,
            ..baseline()
        };
        let a = get_cost_accelerator(3.0, inp);
        let b = get_cost_accelerator(3.0, challenged);
        assert!(b > a);
    }

    #[test]
    fn reincarnation_challenge_8_multiplies_more_than_t4() {
        let in_t4 = GetCostAcceleratorInput {
            in_transcension_challenge_4: true,
            ..baseline()
        };
        let in_r8 = GetCostAcceleratorInput {
            in_reincarnation_challenge_8: true,
            ..baseline()
        };
        let t4_cost = get_cost_accelerator(3.0, in_t4);
        let r8_cost = get_cost_accelerator(3.0, in_r8);
        // R8 uses 1e50 base vs T4's 10 — strictly bigger.
        assert!(r8_cost > t4_cost);
    }

    #[test]
    fn transcend_ecc_pushes_factorial_threshold() {
        // With transcend_ecc = 1, transcend_break = 5, threshold = 130.
        // At buying_to = 128 (level = 127), without ECC the factorial branch
        // fires; with ECC it doesn't.
        let no_ecc = baseline();
        let with_ecc = GetCostAcceleratorInput {
            transcend_ecc: 1.0,
            ..baseline()
        };
        let cost_no_ecc = get_cost_accelerator(128.0, no_ecc);
        let cost_with_ecc = get_cost_accelerator(128.0, with_ecc);
        assert!(cost_no_ecc > cost_with_ecc);
    }

    // ─── buy_accelerator ──────────────────────────────────────────────────

    fn empty_state() -> AcceleratorState {
        AcceleratorState {
            accelerator_bought: 0.0,
            accelerator_cost: get_cost_accelerator(1.0, baseline()),
            prestige_no_accelerator: true,
            transcend_no_accelerator: true,
            reincarnate_no_accelerator: true,
        }
    }

    fn buy_input() -> BuyAcceleratorInput {
        BuyAcceleratorInput {
            autobuyer: false,
            coinbuyamount: BuyAmount::HundredThousand,
            cost_divisor: 1.0,
            transcend_ecc: 0.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
        }
    }

    #[test]
    fn buy_is_noop_with_zero_coins() {
        let mut state = empty_state();
        let mut coins = Decimal::zero();
        let events = buy_accelerator(&mut state, &mut coins, buy_input());
        assert_eq!(state.accelerator_bought, 0.0);
        assert_eq!(coins, Decimal::zero());
        assert!(events.is_empty());
        // Flags stay raised when nothing was bought.
        assert!(state.prestige_no_accelerator);
        assert!(state.transcend_no_accelerator);
        assert!(state.reincarnate_no_accelerator);
    }

    #[test]
    fn buy_purchases_at_least_one_when_affordable() {
        // First accelerator costs 500 coins. Give the player 1000.
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1000.0);
        let baseline_coins = coins;
        let events = buy_accelerator(&mut state, &mut coins, buy_input());
        assert!(state.accelerator_bought > 0.0);
        assert!(coins < baseline_coins);
        assert_eq!(events.len(), 1);
        // First ownership flips the no-accelerator flags.
        assert!(!state.prestige_no_accelerator);
        assert!(!state.transcend_no_accelerator);
        assert!(!state.reincarnate_no_accelerator);
    }

    #[test]
    fn buy_event_spent_matches_resource_delta() {
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e6);
        let baseline_coins = coins;
        let events = buy_accelerator(&mut state, &mut coins, buy_input());
        let spent = baseline_coins - coins;
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::AcceleratorsPurchased {
                before,
                after,
                spent: ev_spent,
            } => {
                assert_eq!(*before, 0.0);
                assert_eq!(*after, state.accelerator_bought);
                assert_eq!(*ev_spent, spent);
            }
            other => panic!("expected AcceleratorsPurchased, got {other:?}"),
        }
    }

    #[test]
    fn per_click_cap_limits_purchases() {
        // Plenty of coins, but coinbuyamount = One should cap at 1 purchase.
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e10);
        let capped = BuyAcceleratorInput {
            coinbuyamount: BuyAmount::One,
            ..buy_input()
        };
        let _ = buy_accelerator(&mut state, &mut coins, capped);
        assert_eq!(state.accelerator_bought, 1.0);
    }

    #[test]
    fn autobuyer_ignores_per_click_cap() {
        let mut state = empty_state();
        let mut coins = Decimal::from_finite(1e10);
        let auto = BuyAcceleratorInput {
            autobuyer: true,
            coinbuyamount: BuyAmount::One,
            ..buy_input()
        };
        let _ = buy_accelerator(&mut state, &mut coins, auto);
        // Autobuyer with 1e10 coins buys far more than 1.
        assert!(state.accelerator_bought > 1.0);
    }
}
