//! Particle-building cost formula and purchase loop.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/particleBuildings.ts`.
//!
//! Particle buildings: five positions purchased with
//! `reincarnation_points`. The cost curve is independent from the producer
//! family (separate base list and a quadratic-in-exponent growth above a
//! challenge-gated threshold), so this lives in its own module rather than
//! reusing [`crate::mechanics::producers::get_producer_cost`].

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::math::smallest_inc::smallest_inc;
use crate::state::{BuyAmount, ParticleBuildingsState};

/// Threshold past which the cost formula switches to the
/// "diminishing-returns" tail.
const BUYMAX: f64 = 1e15;

/// Base costs by position. Same constants as the legacy
/// `mythosAndParticleBuildingCosts`; mythos buildings reuse them via a
/// different codepath in [`crate::mechanics::producers`].
const ORIGINAL_COSTS: [f64; 5] = [1.0, 1e2, 1e4, 1e8, 1e16];

/// Inputs to [`get_particle_cost`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetParticleCostInput {
    /// Which of the five particle buildings (1..=5). Picks the base cost.
    pub index: u8,
    /// `player.currentChallenge.ascension === 15` — flips the
    /// diminishing-return threshold `325000 → 1000`.
    pub in_ascension_challenge_15: bool,
}

/// Inputs to [`buy_particle_building`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyParticleBuildingInput {
    /// Which of the five particle buildings (1..=5).
    pub index: u8,
    /// `player.currentChallenge.ascension === 15`.
    pub in_ascension_challenge_15: bool,
    /// True when the autobuyer is driving — bypasses the per-click cap.
    pub autobuyer: bool,
    /// Per-click purchase cap selected in the UI.
    pub particlebuyamount: BuyAmount,
}

/// Internal cost helper — takes `original_cost` separately so the
/// diminishing-tail recursion can pass it through without an extra index
/// lookup.
fn get_cost_internal(
    original_cost: f64,
    buying_to: f64,
    in_ascension_challenge_15: bool,
) -> Decimal {
    let buying_to = buying_to - 1.0;
    let base = Decimal::from_finite(original_cost);
    let mut cost = base * Decimal::from_finite(2.0).pow(Decimal::from_finite(buying_to));

    let dr = if in_ascension_challenge_15 {
        1_000.0
    } else {
        325_000.0
    };

    if buying_to > dr {
        cost *= Decimal::from_finite(1.001).pow(Decimal::from_finite(
            (buying_to - dr) * ((buying_to - dr + 1.0) / 2.0),
        ));
    }

    if buying_to > BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        // Off-by-one: TS passes BUYMAX then decrements inside. We
        // pre-decrement above, so the recursive call gets BUYMAX + 1.0
        // to land at the same internal value.
        let quadrillion_cost =
            get_cost_internal(original_cost, BUYMAX + 1.0, in_ascension_challenge_15);
        let new_cost = quadrillion_cost.pow(Decimal::from_finite(
            (buying_to / BUYMAX).powf(1.0 / diminishing_exponent),
        ));
        return cost.max(new_cost);
    }
    cost
}

/// Cost in `reincarnation_points` of buying the `buying_to`-th particle
/// building of the given index.
#[must_use]
pub fn get_particle_cost(buying_to: f64, input: GetParticleCostInput) -> Decimal {
    debug_assert!(
        matches!(input.index, 1..=5),
        "particle index out of range: {}",
        input.index
    );
    let original_cost = ORIGINAL_COSTS[usize::from(input.index - 1)];
    get_cost_internal(original_cost, buying_to, input.in_ascension_challenge_15)
}

/// Buy as many of the selected particle building as possible. Same
/// two-path structure as `buy_max` / `buy_multiplier` / `buy_accelerator`:
/// high-end binary search above `BUYMAX` snaps the count without
/// subtracting the resource; the normal path brackets the affordable
/// count and walks the last few steps subtracting per-purchase.
#[must_use]
pub fn buy_particle_building(
    state: &mut ParticleBuildingsState,
    input: BuyParticleBuildingInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let starting_points = state.reincarnation_points;
    let original_cost = ORIGINAL_COSTS[usize::from(input.index - 1)];
    let cost_input = GetParticleCostInput {
        index: input.index,
        in_ascension_challenge_15: input.in_ascension_challenge_15,
    };

    let buy_start = state.owned(input.index);

    if buy_start >= BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        let log10_resource = state.reincarnation_points.log10().to_number();
        let log10_quadrillion_cost =
            get_cost_internal(original_cost, BUYMAX, input.in_ascension_challenge_15)
                .log10()
                .to_number();

        let mut hi = (BUYMAX
            * 1.0_f64.max((log10_resource / log10_quadrillion_cost).powf(diminishing_exponent)))
        .floor();
        let mut lo = BUYMAX;
        while hi - lo > 0.5 {
            let mid = (lo + (hi - lo) / 2.0).floor();
            if mid == lo || mid == hi {
                break;
            }
            if state.reincarnation_points < get_particle_cost(mid, cost_input) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let buyable = lo;
        let this_cost = get_particle_cost(buyable, cost_input);
        state.set_owned(input.index, buyable);
        state.set_cost(input.index, this_cost);

        if buyable > buy_start {
            events.push(CoreEvent::ParticleBuildingsPurchased {
                index: input.index,
                before: buy_start,
                after: buyable,
                spent: starting_points - state.reincarnation_points,
            });
        }
        return events;
    }

    // Start buying at the current amount bought + 1.
    let buydefault = buy_start + smallest_inc(buy_start);
    let mut buy_to = buydefault;

    let mut cash_to_buy = get_particle_cost(buy_to, cost_input);
    while state.reincarnation_points >= cash_to_buy {
        // Multiply target by 4 until cost just exceeds the available
        // budget.
        buy_to *= 4.0;
        cash_to_buy = get_particle_cost(buy_to, cost_input);
    }
    let mut stepdown = (buy_to / 8.0).floor();
    while stepdown >= smallest_inc(buy_to) {
        if get_particle_cost(buy_to - stepdown, cost_input) <= state.reincarnation_points {
            stepdown = (stepdown / 2.0).floor();
        } else {
            buy_to -= smallest_inc(buy_to).max(stepdown);
        }
    }

    if !input.autobuyer {
        let cap = input.particlebuyamount.as_f64() + buy_start;
        if cap < buy_to {
            buy_to = buy_start
                + input.particlebuyamount.as_f64()
                + smallest_inc(buy_start + input.particlebuyamount.as_f64());
        }
    }

    // Walk down 7 steps below the bracket top, then walk back up
    // subtracting per-purchase.
    let mut buy_from = (buy_to - 6.0 - smallest_inc(buy_to)).max(buydefault);
    let mut this_cost = get_particle_cost(buy_from, cost_input);
    while buy_from <= buy_to && state.reincarnation_points >= this_cost {
        state.reincarnation_points -= this_cost;
        state.set_owned(input.index, buy_from);
        buy_from += smallest_inc(buy_from);
        this_cost = get_particle_cost(buy_from, cost_input);
        state.set_cost(input.index, this_cost);
    }

    if state.owned(input.index) > buy_start {
        events.push(CoreEvent::ParticleBuildingsPurchased {
            index: input.index,
            before: buy_start,
            after: state.owned(input.index),
            spent: starting_points - state.reincarnation_points,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cost_input() -> GetParticleCostInput {
        GetParticleCostInput {
            index: 1,
            in_ascension_challenge_15: false,
        }
    }

    fn empty_state() -> ParticleBuildingsState {
        ParticleBuildingsState {
            reincarnation_points: Decimal::zero(),
            first_owned_particles: 0.0,
            first_cost_particles: get_particle_cost(1.0, cost_input()),
            first_generated_particles: Decimal::zero(),
            second_owned_particles: 0.0,
            second_cost_particles: get_particle_cost(
                1.0,
                GetParticleCostInput {
                    index: 2,
                    ..cost_input()
                },
            ),
            second_generated_particles: Decimal::zero(),
            third_owned_particles: 0.0,
            third_cost_particles: get_particle_cost(
                1.0,
                GetParticleCostInput {
                    index: 3,
                    ..cost_input()
                },
            ),
            third_generated_particles: Decimal::zero(),
            fourth_owned_particles: 0.0,
            fourth_cost_particles: get_particle_cost(
                1.0,
                GetParticleCostInput {
                    index: 4,
                    ..cost_input()
                },
            ),
            fourth_generated_particles: Decimal::zero(),
            fifth_owned_particles: 0.0,
            fifth_cost_particles: get_particle_cost(
                1.0,
                GetParticleCostInput {
                    index: 5,
                    ..cost_input()
                },
            ),
            fifth_generated_particles: Decimal::zero(),
        }
    }

    fn buy_input() -> BuyParticleBuildingInput {
        BuyParticleBuildingInput {
            index: 1,
            in_ascension_challenge_15: false,
            autobuyer: false,
            particlebuyamount: BuyAmount::HundredThousand,
        }
    }

    // ─── get_particle_cost ─────────────────────────────────────────────────

    #[test]
    fn tier_1_first_cost_is_one() {
        // base = 1, buying_to = 1 → 0 → 1 * 2^0 = 1
        let cost = get_particle_cost(1.0, cost_input());
        assert!((cost.to_number() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn tier_costs_double_per_step() {
        // base = 1; cost(N) = 2^(N-1)
        for n in 1..=10 {
            let cost = get_particle_cost(f64::from(n), cost_input());
            let expected = 2.0_f64.powi(n - 1);
            assert!(
                (cost.to_number() - expected).abs() < 1e-9,
                "n={n}: expected {expected}, got {}",
                cost.to_number()
            );
        }
    }

    #[test]
    fn tier_2_uses_correct_base_cost() {
        let inp = GetParticleCostInput {
            index: 2,
            ..cost_input()
        };
        // base = 100; cost(1) = 100
        assert!((get_particle_cost(1.0, inp).to_number() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn ascension_15_lowers_dr_threshold() {
        // Plain: dr = 325k → cost(1100) is unaffected by the dr-quadratic.
        // In asc15: dr = 1000 → cost(1100) has a tiny quadratic bonus.
        let plain = cost_input();
        let in_asc15 = GetParticleCostInput {
            in_ascension_challenge_15: true,
            ..plain
        };
        let plain_cost = get_particle_cost(1_100.0, plain);
        let asc15_cost = get_particle_cost(1_100.0, in_asc15);
        assert!(asc15_cost > plain_cost);
    }

    // ─── buy_particle_building ─────────────────────────────────────────────

    #[test]
    fn buy_is_noop_with_zero_reincarnation_points() {
        let mut state = empty_state();
        let events = buy_particle_building(&mut state, buy_input());
        assert_eq!(state.first_owned_particles, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_purchases_at_least_one_when_affordable() {
        // First particle building costs 1 reincarnation point.
        let mut state = ParticleBuildingsState {
            reincarnation_points: Decimal::from_finite(100.0),
            ..empty_state()
        };
        let baseline_points = state.reincarnation_points;
        let events = buy_particle_building(&mut state, buy_input());
        assert!(state.first_owned_particles > 0.0);
        assert!(state.reincarnation_points < baseline_points);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::ParticleBuildingsPurchased {
                index,
                before,
                after,
                ..
            } => {
                assert_eq!(*index, 1);
                assert_eq!(*before, 0.0);
                assert_eq!(*after, state.first_owned_particles);
            }
            other => panic!("expected ParticleBuildingsPurchased, got {other:?}"),
        }
    }

    #[test]
    fn buy_targets_only_the_requested_index() {
        let mut state = ParticleBuildingsState {
            reincarnation_points: Decimal::from_finite(1e20),
            ..empty_state()
        };
        let input = BuyParticleBuildingInput {
            index: 3,
            ..buy_input()
        };
        let _ = buy_particle_building(&mut state, input);
        assert!(state.third_owned_particles > 0.0);
        assert_eq!(state.first_owned_particles, 0.0);
        assert_eq!(state.second_owned_particles, 0.0);
        assert_eq!(state.fourth_owned_particles, 0.0);
        assert_eq!(state.fifth_owned_particles, 0.0);
    }

    #[test]
    fn buy_event_spent_matches_resource_delta() {
        let mut state = ParticleBuildingsState {
            reincarnation_points: Decimal::from_finite(1e8),
            ..empty_state()
        };
        let baseline_points = state.reincarnation_points;
        let events = buy_particle_building(&mut state, buy_input());
        let spent = baseline_points - state.reincarnation_points;
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::ParticleBuildingsPurchased {
                spent: ev_spent, ..
            } => {
                assert_eq!(*ev_spent, spent);
            }
            other => panic!("expected ParticleBuildingsPurchased, got {other:?}"),
        }
    }

    #[test]
    fn per_click_cap_limits_purchases() {
        let mut state = ParticleBuildingsState {
            reincarnation_points: Decimal::from_finite(1e20),
            ..empty_state()
        };
        let capped = BuyParticleBuildingInput {
            particlebuyamount: BuyAmount::One,
            ..buy_input()
        };
        let _ = buy_particle_building(&mut state, capped);
        // The cap shape is `buy_start + amount + smallest_inc(buy_start + amount)`,
        // so with start=0 and amount=1 we get 1 + smallest_inc(1) = 2.
        // Either way the result must be small (within a handful of units).
        assert!(
            state.first_owned_particles <= 2.0,
            "per-click cap exceeded: got {}",
            state.first_owned_particles
        );
    }

    #[test]
    fn autobuyer_bypasses_per_click_cap() {
        let mut state = ParticleBuildingsState {
            reincarnation_points: Decimal::from_finite(1e20),
            ..empty_state()
        };
        let auto = BuyParticleBuildingInput {
            autobuyer: true,
            particlebuyamount: BuyAmount::One,
            ..buy_input()
        };
        let _ = buy_particle_building(&mut state, auto);
        assert!(state.first_owned_particles > 1.0);
    }
}
