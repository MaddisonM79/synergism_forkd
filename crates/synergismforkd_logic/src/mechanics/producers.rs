//! Producer family cost formula and purchase loops.
//!
//! Verbatim port of `legacy_core_split/packages/logic/src/mechanics/producers.ts`.
//! Logic owns the pure cost formula AND the buy-max purchase loop; the
//! manual-click `buy_producer` loop is included too. The five-position,
//! four-family producer ladder backs every coin / Diamonds / Mythos /
//! Particles production tier.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::{CoreEvent, ProducerType};
use crate::math::smallest_inc::smallest_inc;
use crate::state::ProducerFamilyState;

/// Threshold past which the cost formula switches to the
/// "diminishing-returns" tail (mirror of `getCostAccelerator`'s BUYMAX).
const BUYMAX: f64 = 1e15;

/// Coin/exponent ceiling guard. Mirrors the original `buyMax`'s
/// `coinmax = 1e99` degenerate-case check — once the next cost's exponent
/// crosses this we bail rather than continue doubling `buy_inc` into
/// infinity.
const COIN_EXPONENT_CEILING: f64 = 1e99;

// ─── Stirling-approximation factorial helpers ──────────────────────────────

/// `log10(2 * π)` — pulled out so the Stirling helper avoids redundant
/// trig.
const MANTISSA_FACTORIAL_PART_EXTRA: f64 = 0.798_179_868_358_115_5; // log10(2π)
/// `log10(e)` — the Stirling helper subtracts this off the exponent term.
const EXPONENT_FACTORIAL_PART_EXTRA: f64 = std::f64::consts::LOG10_E;

/// Stirling-approximation `log10(n!)` operating on the post-increment
/// exponent. Used by the producer-cost formula in hot loops to avoid
/// constructing a full `Decimal` per factorial.
///
/// Verbatim port of `factorialByExponent` from the TS source. The inner
/// expression is
///
/// ```text
/// log10(fact * sqrt(fact * sinh(1/fact) + 1 / (810 * fact^6)))
/// ```
///
/// times `fact`, minus `fact * log10(e)`, plus
/// `(log10(2π) - log10(fact)) / 2`.
fn factorial_by_exponent(fact: f64) -> f64 {
    let fact = fact + 1.0;
    if fact == 0.0 {
        return 0.0;
    }
    let inner_sqrt = fact * (1.0_f64 / fact).sinh() + 1.0 / (810.0 * fact.powi(6));
    let log_term = (fact * inner_sqrt.sqrt()).log10() - EXPONENT_FACTORIAL_PART_EXTRA;
    log_term * fact + (MANTISSA_FACTORIAL_PART_EXTRA - fact.log10()) / 2.0
}

// ─── Per-family base cost arrays ───────────────────────────────────────────

/// Tier-1..5 base costs for the Coin family.
const COIN_BUILDING_COSTS: [f64; 5] = [100.0, 1_000.0, 2e4, 4e5, 8e6];
/// Tier-1..5 base costs for the Diamonds family.
const DIAMOND_BUILDING_COSTS: [f64; 5] = [100.0, 1e5, 1e15, 1e40, 1e100];
/// Tier-1..5 base costs shared by the Mythos and Particles families.
const MYTHOS_AND_PARTICLE_BUILDING_COSTS: [f64; 5] = [1.0, 1e2, 1e4, 1e8, 1e16];

/// `(originalCost, num)` for a given `(index, producer_type)` pair. `num`
/// is the producer-cost-ladder exponent base: Coin uses the position
/// directly, every other family uses the triangle-number
/// `index * (index + 1) / 2`.
fn get_original_cost_and_num(index: u8, producer_type: ProducerType) -> (f64, f64) {
    debug_assert!(
        matches!(index, 1..=5),
        "producer index out of range: {index}"
    );
    let array = match producer_type {
        ProducerType::Coin => &COIN_BUILDING_COSTS,
        ProducerType::Diamonds => &DIAMOND_BUILDING_COSTS,
        ProducerType::Mythos | ProducerType::Particles => &MYTHOS_AND_PARTICLE_BUILDING_COSTS,
    };
    let num = match producer_type {
        ProducerType::Coin => f64::from(index),
        _ => {
            let i = f64::from(index);
            i * (i + 1.0) / 2.0
        }
    };
    let idx = usize::from(index - 1);
    (array[idx], num)
}

// ─── Input + public cost entry ─────────────────────────────────────────────

/// Inputs to [`get_producer_cost`]. Mirrors `GetProducerCostInput` in the
/// TS source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetProducerCostInput {
    /// `G.costDivisor` at call time (= `getReductionValue()` in the legacy
    /// UI).
    pub cost_divisor: f64,
    /// `player.currentChallenge.transcension === 4`.
    pub in_transcension_challenge_4: bool,
    /// `player.currentChallenge.reincarnation === 8`.
    pub in_reincarnation_challenge_8: bool,
    /// `player.currentChallenge.reincarnation === 10`.
    pub in_reincarnation_challenge_10: bool,
    /// `player.challengecompletions[4]`.
    pub challengecompletions_4: f64,
    /// `player.challengecompletions[8]`.
    pub challengecompletions_8: f64,
}

/// Precision-loss threshold for the `+1` corrections inside the cost
/// formula. `log10(1.25) * n ≈ log10(x) + 16 ⇒ x ≈ 188.582 / n`. Below
/// this, the `+1` corrections matter; above it they round off into
/// noise.
const PRECISION16_LOSS_ADDITION_OF_ONES: f64 = 188.582;

/// `log10(1.25)` — appears as a multiplier all over the cost formula.
const LOG10_1_25: f64 = 0.096_910_013_008_056_42;
/// `log10(1.03)` — applied past `250000 * r` in the cost ladder.
const LOG10_1_03: f64 = 0.012_837_224_705_169_534;

/// Multiply `cost` by `10^delta` in-place. Equivalent to the TS pattern
/// `cost.exponent += delta` (which break_infinity.js applies directly to a
/// public field). break-eternity-rs uses a layered internal representation
/// whose `set_exponent` is not field-write semantics — Decimal arithmetic
/// gives the same end value with better numerical robustness.
fn shift_exponent(cost: &mut Decimal, delta: f64) {
    *cost *= Decimal::from_finite(10.0).pow(Decimal::from_finite(delta));
}

fn get_cost_internal(
    original_cost: f64,
    buying_to: f64,
    producer_type: ProducerType,
    num: f64,
    input: GetProducerCostInput,
) -> Decimal {
    let r = input.cost_divisor;
    // Off-by-one: formula is 0-indexed, callers pass 1-indexed.
    let buying_to = buying_to - 1.0;

    let mut cost = Decimal::from_finite(original_cost);
    // Accounts for the cumulative `* 1.25^num` `buying_to` times.
    let mut mlog10_125 = num * buying_to;
    // The +1 corrections (TS: `cost.mantissa += buyingTo / 10^cost.exponent`)
    // are equivalent to adding `buying_to` to the cost directly — only
    // matter below the precision floor.
    if buying_to < PRECISION16_LOSS_ADDITION_OF_ONES / num {
        cost += Decimal::from_finite(buying_to);
    }
    let mut fast_fact_mult_buy_to = 0.0_f64;

    let fr = (r * 1_000.0).floor();
    if buying_to >= r * 1_000.0 {
        fast_fact_mult_buy_to += 1.0;
        shift_exponent(&mut cost, -factorial_by_exponent(fr));
        shift_exponent(
            &mut cost,
            (-3.0 + (1.0 + num / 2.0).log10()) * (buying_to - fr),
        );
    }

    let fr = (r * 5_000.0).floor();
    if buying_to >= r * 5_000.0 {
        fast_fact_mult_buy_to += 1.0;
        shift_exponent(&mut cost, -factorial_by_exponent(fr));
        shift_exponent(
            &mut cost,
            ((10.0_f64 + num * 10.0).log10() + 1.0) * (buying_to - fr - 1.0) + 1.0,
        );
    }

    let fr = (r * 20_000.0).floor();
    if buying_to >= r * 20_000.0 {
        fast_fact_mult_buy_to += 3.0;
        shift_exponent(&mut cost, -factorial_by_exponent(fr) * 3.0);
        shift_exponent(
            &mut cost,
            ((100.0_f64 + 100.0 * num).log10() + 5.0) * (buying_to - fr),
        );
    }

    let fr = (r * 250_000.0).floor();
    if buying_to >= r * 250_000.0 {
        // 1.03^x * 1.03^y = 1.03^(x+y) — sum the power as a triangle number.
        shift_exponent(
            &mut cost,
            LOG10_1_03 * (buying_to - fr) * ((buying_to - fr + 1.0) / 2.0),
        );
    }
    // Apply the factorial corrections accumulated across the r-bracket regions.
    shift_exponent(
        &mut cost,
        factorial_by_exponent(buying_to) * fast_fact_mult_buy_to,
    );

    // Challenge-driven amplifiers — Coin / Diamonds in C4 transcension and
    // C10 reincarnation, separately accumulated.
    let mut fast_fact_mult_buy_to_100 = 0.0_f64;
    if input.in_transcension_challenge_4
        && matches!(producer_type, ProducerType::Coin | ProducerType::Diamonds)
    {
        fast_fact_mult_buy_to_100 += 1.0;
        if buying_to >= 1_000.0 - 10.0 * input.challengecompletions_4 {
            mlog10_125 += buying_to * (buying_to + 1.0) / 2.0;
        }
    }
    if input.in_reincarnation_challenge_10
        && matches!(producer_type, ProducerType::Coin | ProducerType::Diamonds)
    {
        fast_fact_mult_buy_to_100 += 1.0;
        if buying_to >= r * 25_000.0 {
            mlog10_125 += buying_to * (buying_to + 1.0) / 2.0;
        }
    }
    const FACT_100_EXPONENT: f64 = 157.970_004_352_587_45; // log10(9.332621544394e+157)
    shift_exponent(
        &mut cost,
        fast_fact_mult_buy_to_100
            * ((factorial_by_exponent(buying_to + 100.0) - FACT_100_EXPONENT + 2.0 * buying_to)
                * (1.25 + input.challengecompletions_4 / 4.0)),
    );
    shift_exponent(&mut cost, LOG10_1_25 * mlog10_125);

    // Reincarnation Challenge 8 — affects Coin / Diamonds / Mythos at high
    // counts.
    let fr = (r * 1_000.0 * input.challengecompletions_8).floor();
    if input.in_reincarnation_challenge_8
        && matches!(
            producer_type,
            ProducerType::Coin | ProducerType::Diamonds | ProducerType::Mythos
        )
        && buying_to >= 1_000.0 * input.challengecompletions_8 * r
    {
        const LOG10_2: f64 = std::f64::consts::LOG10_2;
        let exponent_addend = (LOG10_2 * ((buying_to - fr + 1.0) / 2.0)
            - (1.0 + input.challengecompletions_8 / 2.0).log10())
            * (buying_to - fr);
        shift_exponent(&mut cost, exponent_addend);
    }

    if buying_to > BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        // Off-by-one in the recursion: TS passes BUYMAX then decrements
        // inside; we pre-decrement above, so pass BUYMAX + 1.0.
        let quadrillion_cost =
            get_cost_internal(original_cost, BUYMAX + 1.0, producer_type, num, input);
        let new_cost = quadrillion_cost.pow(Decimal::from_finite(
            (buying_to / BUYMAX).powf(1.0 / diminishing_exponent),
        ));
        return cost.max(new_cost);
    }
    cost
}

/// Public entry point — looks up the `(original_cost, num)` pair for
/// `(index, producer_type)` and dispatches to the internal cost formula.
///
/// `index` is 1..=5 (1-based to match the legacy convention); out-of-range
/// triggers a debug assertion.
#[must_use]
pub fn get_producer_cost(
    index: u8,
    producer_type: ProducerType,
    buying_to: f64,
    input: GetProducerCostInput,
) -> Decimal {
    let (original_cost, num) = get_original_cost_and_num(index, producer_type);
    get_cost_internal(original_cost, buying_to, producer_type, num, input)
}

// ─── buy_max ───────────────────────────────────────────────────────────────

/// Inputs to [`buy_max`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyMaxInput {
    /// Tier index 1..=5.
    pub index: u8,
    /// Which family the buy targets.
    pub producer_type: ProducerType,
    /// Threaded into [`get_producer_cost`] for every cost query in the
    /// loop.
    pub cost_input: GetProducerCostInput,
}

/// Buy as many of the selected producer (5 positions × 4 families) as the
/// available resource allows. Same two-path structure as
/// [`crate::mechanics::multipliers::buy_multiplier`] /
/// [`crate::mechanics::accelerators::buy_accelerator`]: high-end binary
/// search above `BUYMAX` snaps the count without subtracting the resource;
/// the normal path brackets the affordable count and walks the last few
/// steps subtracting per-purchase.
#[must_use]
pub fn buy_max(
    state: &mut ProducerFamilyState,
    resource: &mut Decimal,
    input: BuyMaxInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let starting_resource = *resource;
    let buy_start = state.owned(input.index);

    let cost = |buying_to: f64| -> Decimal {
        get_producer_cost(
            input.index,
            input.producer_type,
            buying_to,
            input.cost_input,
        )
    };

    if buy_start >= BUYMAX {
        let diminishing_exponent = 1.0_f64 / 8.0;
        let log10_resource = resource.log10().to_number();
        let log10_quadrillion_cost = cost(BUYMAX).log10().to_number();

        let mut hi = (BUYMAX
            * 1.0_f64.max((log10_resource / log10_quadrillion_cost).powf(diminishing_exponent)))
        .floor();
        let mut lo = BUYMAX;
        while hi - lo > 0.5 {
            let mid = (lo + (hi - lo) / 2.0).floor();
            if mid == lo || mid == hi {
                break;
            }
            if *resource < cost(mid) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let buyable = lo;
        state.set_owned(input.index, buyable);
        state.set_cost(input.index, cost(buyable));
        if buyable > buy_start {
            events.push(CoreEvent::ProducersPurchased {
                producer_type: input.producer_type,
                index: input.index,
                before: buy_start,
                after: buyable,
                spent: starting_resource - *resource,
            });
        }
        return events;
    }

    // Normal path: exponential bracket, then refine, then walk the tail.
    let buydefault = buy_start + smallest_inc(buy_start);
    let mut buy_inc = 1.0_f64;

    let mut cash_to_buy = cost(buy_start + buy_inc);

    // Degenerate case: cost already past the exponent ceiling or
    // unaffordable.
    if cash_to_buy.exponent() >= COIN_EXPONENT_CEILING || *resource < cash_to_buy {
        return events;
    }

    while cash_to_buy.exponent() < COIN_EXPONENT_CEILING && *resource >= cash_to_buy {
        // Multiply target by 4 until cost just exceeds the available budget.
        buy_inc *= 4.0;
        cash_to_buy = cost(buy_start + buy_inc);
    }
    let mut stepdown = (buy_inc / 8.0).floor();
    while stepdown >= smallest_inc(buy_inc) {
        if cost(buy_start + buy_inc - stepdown) <= *resource {
            stepdown = (stepdown / 2.0).floor();
        } else {
            buy_inc -= smallest_inc(buy_inc).max(stepdown);
        }
    }

    // Snap to BUYMAX cap before the walk. The original commentary calls
    // this the "infamous autobuyer bug" fix — past BUYMAX we just write
    // the snapped state and stop.
    if buy_start + buy_inc >= BUYMAX {
        state.set_owned(input.index, BUYMAX);
        state.set_cost(input.index, cost(BUYMAX));
        events.push(CoreEvent::ProducersPurchased {
            producer_type: input.producer_type,
            index: input.index,
            before: buy_start,
            after: BUYMAX,
            spent: starting_resource - *resource,
        });
        return events;
    }

    let mut buy_from = (buy_start + buy_inc - 6.0 - smallest_inc(buy_inc)).max(buydefault);
    let mut this_cost = cost(buy_from);
    while buy_from <= buy_start + buy_inc && *resource >= this_cost {
        *resource -= this_cost;
        state.set_owned(input.index, buy_from);
        buy_from += smallest_inc(buy_from);
        this_cost = cost(buy_from);
        state.set_cost(input.index, this_cost);
    }

    if state.owned(input.index) > buy_start {
        events.push(CoreEvent::ProducersPurchased {
            producer_type: input.producer_type,
            index: input.index,
            before: buy_start,
            after: state.owned(input.index),
            spent: starting_resource - *resource,
        });
    }

    events
}

// ─── buy_producer (manual-click loop) ──────────────────────────────────────

/// Inputs to [`buy_producer`] — the manual-click purchase loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyProducerInput {
    /// Tier index 1..=5.
    pub index: u8,
    /// Which family the buy targets.
    pub producer_type: ProducerType,
    /// True when the autobuyer is driving — caps the loop at 500
    /// iterations.
    pub autobuyer: bool,
    /// Per-click cap from
    /// `player.{coin,crystal,mythos,particle}buyamount`.
    pub buyamount: f64,
    /// Reduction value — `getReductionValue()` in the UI. Shifts the
    /// per-step exponent thresholds (`1000*r`, `5000*r`, `20000*r`,
    /// `250000*r`) and the challenge-8 amplifier threshold.
    pub r: f64,
    /// `player.currentChallenge.transcension === 4`.
    pub in_transcension_challenge_4: bool,
    /// `player.currentChallenge.reincarnation === 8`.
    pub in_reincarnation_challenge_8: bool,
    /// `player.challengecompletions[4]`.
    pub challengecompletions_4: f64,
    /// `player.challengecompletions[8]`.
    pub challengecompletions_8: f64,
}

/// `num` derivation for a `(index, producer_type)` pair. Mirrors the
/// `numFor` helper in the TS source.
fn num_for(index: u8, producer_type: ProducerType) -> f64 {
    match producer_type {
        ProducerType::Coin => f64::from(index),
        _ => {
            let i = f64::from(index);
            i * (i + 1.0) / 2.0
        }
    }
}

/// `f64::MAX_SAFE_INTEGER` — JS's `Number.MAX_SAFE_INTEGER`. Used as the
/// per-iteration cap inside [`buy_producer`].
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// Manual-click producer purchase loop. Buys one producer per iteration,
/// subtracts current cost, then applies the per-iteration cost multiplier
/// ladder (`×1.25^num`, `+1` mantissa adjustment, threshold amplifiers at
/// `1000/5000/20000/250000 * r`, challenge-4 transcension, challenge-8
/// reincarnation). Loop caps at `buyamount` (or `500` when the autobuyer
/// is driving).
#[must_use]
pub fn buy_producer(
    state: &mut ProducerFamilyState,
    resource: &mut Decimal,
    input: BuyProducerInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let starting_resource = *resource;
    let buy_start = state.owned(input.index);
    let num = num_for(input.index, input.producer_type);
    let buythisamount = if input.autobuyer {
        500.0
    } else {
        input.buyamount
    };

    let mut t = 0.0_f64;
    while *resource >= state.cost(input.index)
        && t < buythisamount
        && state.owned(input.index) < MAX_SAFE_INTEGER
    {
        let current_cost = state.cost(input.index);
        *resource -= current_cost;
        state.set_owned(input.index, state.owned(input.index) + 1.0);

        let mut cost =
            state.cost(input.index) * Decimal::from_finite(1.25).pow(Decimal::from_finite(num));
        cost += Decimal::one();
        let owned = state.owned(input.index);

        // Per-step exponent threshold ladder. Each rung adds a one-off
        // cost multiplier once the cumulative count crosses the
        // `threshold * r` mark.
        if owned >= 1_000.0 * input.r {
            cost = cost * Decimal::from_finite(owned) / Decimal::from_finite(1_000.0)
                * Decimal::from_finite(1.0 + num / 2.0);
        }
        if owned >= 5_000.0 * input.r {
            cost *= Decimal::from_finite(owned)
                * Decimal::from_finite(10.0)
                * Decimal::from_finite(10.0 + num * 10.0);
        }
        if owned >= 20_000.0 * input.r {
            cost *= Decimal::from_finite(owned).pow(Decimal::from_finite(3.0))
                * Decimal::from_finite(100_000.0)
                * Decimal::from_finite(100.0 + num * 100.0);
        }
        if owned >= 250_000.0 * input.r {
            cost *=
                Decimal::from_finite(1.03).pow(Decimal::from_finite(owned - 250_000.0 * input.r));
        }

        // Challenge-4 (transcension) — amplifies Coin / Diamonds.
        if input.in_transcension_challenge_4
            && matches!(
                input.producer_type,
                ProducerType::Coin | ProducerType::Diamonds
            )
        {
            cost *= Decimal::from_finite(
                (100.0 * owned + 10_000.0).powf(1.25 + 0.25 * input.challengecompletions_4),
            );
            if owned >= 1_000.0 - 10.0 * input.challengecompletions_4 {
                cost *= Decimal::from_finite(1.25).pow(Decimal::from_finite(owned));
            }
        }

        // Challenge-8 (reincarnation) — amplifies Coin / Diamonds / Mythos
        // at high counts.
        if input.in_reincarnation_challenge_8
            && matches!(
                input.producer_type,
                ProducerType::Coin | ProducerType::Diamonds | ProducerType::Mythos
            )
            && owned >= 1_000.0 * input.challengecompletions_8 * input.r
        {
            cost *= Decimal::from_finite(2.0).pow(Decimal::from_finite(
                (owned - 1_000.0 * input.challengecompletions_8 * input.r)
                    / (1.0 + input.challengecompletions_8 / 2.0),
            ));
        }

        state.set_cost(input.index, cost);
        t += 1.0;
    }

    if state.owned(input.index) > buy_start {
        events.push(CoreEvent::ProducersPurchased {
            producer_type: input.producer_type,
            index: input.index,
            before: buy_start,
            after: state.owned(input.index),
            spent: starting_resource - *resource,
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cost_input() -> GetProducerCostInput {
        GetProducerCostInput {
            cost_divisor: 1.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
            in_reincarnation_challenge_10: false,
            challengecompletions_4: 0.0,
            challengecompletions_8: 0.0,
        }
    }

    fn empty_family() -> ProducerFamilyState {
        ProducerFamilyState {
            first_owned: 0.0,
            first_cost: get_producer_cost(1, ProducerType::Coin, 1.0, cost_input()),
            first_generated: Decimal::zero(),
            second_owned: 0.0,
            second_cost: get_producer_cost(2, ProducerType::Coin, 1.0, cost_input()),
            second_generated: Decimal::zero(),
            third_owned: 0.0,
            third_cost: get_producer_cost(3, ProducerType::Coin, 1.0, cost_input()),
            third_generated: Decimal::zero(),
            fourth_owned: 0.0,
            fourth_cost: get_producer_cost(4, ProducerType::Coin, 1.0, cost_input()),
            fourth_generated: Decimal::zero(),
            fifth_owned: 0.0,
            fifth_cost: get_producer_cost(5, ProducerType::Coin, 1.0, cost_input()),
            fifth_generated: Decimal::zero(),
        }
    }

    // ─── get_producer_cost ─────────────────────────────────────────────────

    #[test]
    fn coin_tier1_first_cost_is_base_array_value() {
        let cost = get_producer_cost(1, ProducerType::Coin, 1.0, cost_input());
        assert!((cost.to_number() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn coin_tier_costs_match_base_array() {
        let bases = [100.0, 1_000.0, 2e4, 4e5, 8e6];
        for (i, base) in bases.iter().enumerate() {
            let idx = u8::try_from(i + 1).unwrap();
            let cost = get_producer_cost(idx, ProducerType::Coin, 1.0, cost_input());
            assert!(
                (cost.to_number() - base).abs() / base < 1e-9,
                "tier {idx}: expected {base}, got {}",
                cost.to_number()
            );
        }
    }

    #[test]
    fn diamond_tier1_first_cost_is_base_array_value() {
        let cost = get_producer_cost(1, ProducerType::Diamonds, 1.0, cost_input());
        assert!((cost.to_number() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn cost_strictly_increases_with_buying_to() {
        let inp = cost_input();
        let a = get_producer_cost(1, ProducerType::Coin, 5.0, inp);
        let b = get_producer_cost(1, ProducerType::Coin, 6.0, inp);
        let c = get_producer_cost(1, ProducerType::Coin, 50.0, inp);
        assert!(
            a < b,
            "expected cost(5) < cost(6); got a={}, b={}",
            a.to_number(),
            b.to_number()
        );
        assert!(
            b < c,
            "expected cost(6) < cost(50); got b={}, c={}",
            b.to_number(),
            c.to_number()
        );
    }

    #[test]
    fn higher_tier_costs_more_at_same_buying_to() {
        let inp = cost_input();
        // At buying_to = 10, every tier follows the same scaling but the
        // base costs differ. Tier 5 should dominate tier 1.
        let t1 = get_producer_cost(1, ProducerType::Coin, 10.0, inp);
        let t5 = get_producer_cost(5, ProducerType::Coin, 10.0, inp);
        assert!(t5 > t1);
    }

    #[test]
    fn higher_cost_divisor_reduces_cost() {
        let lo = GetProducerCostInput {
            cost_divisor: 1.0,
            ..cost_input()
        };
        let hi = GetProducerCostInput {
            cost_divisor: 10.0,
            ..cost_input()
        };
        // Higher cost_divisor pushes back the threshold rungs, so at the
        // same buying_to past the first rung the cost should be lower.
        let lo_cost = get_producer_cost(1, ProducerType::Coin, 2_000.0, lo);
        let hi_cost = get_producer_cost(1, ProducerType::Coin, 2_000.0, hi);
        assert!(hi_cost < lo_cost);
    }

    #[test]
    fn transcension_challenge_4_amplifies_coin_cost() {
        let plain = cost_input();
        let in_c4 = GetProducerCostInput {
            in_transcension_challenge_4: true,
            ..plain
        };
        let plain_cost = get_producer_cost(1, ProducerType::Coin, 10.0, plain);
        let c4_cost = get_producer_cost(1, ProducerType::Coin, 10.0, in_c4);
        assert!(c4_cost > plain_cost);
    }

    #[test]
    fn challenge_4_does_not_amplify_mythos() {
        // C4 / C10 amplifiers only fire for Coin / Diamonds.
        let plain = cost_input();
        let in_c4 = GetProducerCostInput {
            in_transcension_challenge_4: true,
            ..plain
        };
        let plain_cost = get_producer_cost(1, ProducerType::Mythos, 10.0, plain);
        let c4_cost = get_producer_cost(1, ProducerType::Mythos, 10.0, in_c4);
        assert_eq!(plain_cost, c4_cost);
    }

    #[test]
    fn cost_grows_past_threshold_rungs() {
        // The 1000 * r threshold fires at buying_to >= 1000.
        let inp = cost_input();
        let before = get_producer_cost(1, ProducerType::Coin, 999.0, inp);
        let after = get_producer_cost(1, ProducerType::Coin, 1_001.0, inp);
        assert!(after > before);
    }

    // ─── buy_max ───────────────────────────────────────────────────────────

    fn buy_max_input() -> BuyMaxInput {
        BuyMaxInput {
            index: 1,
            producer_type: ProducerType::Coin,
            cost_input: cost_input(),
        }
    }

    #[test]
    fn buy_max_is_noop_with_zero_resource() {
        let mut state = empty_family();
        let mut resource = Decimal::zero();
        let events = buy_max(&mut state, &mut resource, buy_max_input());
        assert_eq!(state.first_owned, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_max_purchases_when_resource_covers_first_cost() {
        let mut state = empty_family();
        let mut resource = Decimal::from_finite(1e6);
        let baseline_resource = resource;
        let events = buy_max(&mut state, &mut resource, buy_max_input());
        assert!(state.first_owned > 0.0);
        assert!(resource < baseline_resource);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::ProducersPurchased {
                producer_type,
                index,
                before,
                after,
                ..
            } => {
                assert_eq!(*producer_type, ProducerType::Coin);
                assert_eq!(*index, 1);
                assert_eq!(*before, 0.0);
                assert_eq!(*after, state.first_owned);
            }
            other => panic!("expected ProducersPurchased, got {other:?}"),
        }
    }

    #[test]
    fn buy_max_targets_only_the_requested_index() {
        let mut state = empty_family();
        let mut resource = Decimal::from_finite(1e15);
        let baseline_first_cost = state.first_cost;
        let baseline_fifth_cost = state.fifth_cost;
        let input = BuyMaxInput {
            index: 3,
            ..buy_max_input()
        };
        let _ = buy_max(&mut state, &mut resource, input);
        assert!(state.third_owned > 0.0);
        assert_eq!(state.first_owned, 0.0);
        assert_eq!(state.second_owned, 0.0);
        assert_eq!(state.fourth_owned, 0.0);
        assert_eq!(state.fifth_owned, 0.0);
        // Untouched cost fields are preserved.
        assert_eq!(state.first_cost, baseline_first_cost);
        assert_eq!(state.fifth_cost, baseline_fifth_cost);
    }

    #[test]
    fn buy_max_event_spent_matches_resource_delta() {
        let mut state = empty_family();
        let mut resource = Decimal::from_finite(1e8);
        let baseline_resource = resource;
        let events = buy_max(&mut state, &mut resource, buy_max_input());
        let spent = baseline_resource - resource;
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::ProducersPurchased {
                spent: ev_spent, ..
            } => {
                assert_eq!(*ev_spent, spent);
            }
            other => panic!("expected ProducersPurchased, got {other:?}"),
        }
    }

    #[test]
    fn buy_max_works_for_each_producer_type() {
        for ty in [
            ProducerType::Coin,
            ProducerType::Diamonds,
            ProducerType::Mythos,
            ProducerType::Particles,
        ] {
            let mut state = ProducerFamilyState {
                first_cost: get_producer_cost(1, ty, 1.0, cost_input()),
                ..empty_family()
            };
            let mut resource = Decimal::from_finite(1e20);
            let input = BuyMaxInput {
                producer_type: ty,
                ..buy_max_input()
            };
            let _ = buy_max(&mut state, &mut resource, input);
            assert!(state.first_owned > 0.0, "tier-1 {ty:?} did not advance");
        }
    }

    // ─── buy_producer ──────────────────────────────────────────────────────

    fn buy_producer_input() -> BuyProducerInput {
        BuyProducerInput {
            index: 1,
            producer_type: ProducerType::Coin,
            autobuyer: false,
            buyamount: 100.0,
            r: 1.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
            challengecompletions_4: 0.0,
            challengecompletions_8: 0.0,
        }
    }

    #[test]
    fn buy_producer_is_noop_with_zero_resource() {
        let mut state = empty_family();
        let mut resource = Decimal::zero();
        let events = buy_producer(&mut state, &mut resource, buy_producer_input());
        assert_eq!(state.first_owned, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_producer_caps_at_buyamount() {
        // First-tier Coin starts at 100 coins; with 1e6 coins available the
        // loop could go a long way, but `buyamount = 1` caps at one.
        let mut state = empty_family();
        let mut resource = Decimal::from_finite(1e6);
        let capped = BuyProducerInput {
            buyamount: 1.0,
            ..buy_producer_input()
        };
        let events = buy_producer(&mut state, &mut resource, capped);
        assert_eq!(state.first_owned, 1.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_producer_autobuyer_caps_at_500() {
        // Massive resource pool — without the autobuyer cap the loop would
        // run far more than 500 iterations.
        let mut state = empty_family();
        let mut resource = Decimal::from_mantissa_exponent(1.0, 30.0);
        let auto = BuyProducerInput {
            autobuyer: true,
            buyamount: 100_000.0, // ignored when autobuyer is true
            ..buy_producer_input()
        };
        let _ = buy_producer(&mut state, &mut resource, auto);
        assert!(
            state.first_owned <= 500.0,
            "autobuyer ran past 500 iterations: got {}",
            state.first_owned
        );
    }
}
