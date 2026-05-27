//! Per-tier coin production aggregation.
//!
//! Verbatim port of `calculateCoinProduction` from
//! `legacy/core_split/packages/logic/src/mechanics/coinProduction.ts`. Pure
//! `Decimal` math — each of the five coin-producer tiers contributes
//!
//! ```text
//! (generated + owned) * global_coin_multiplier * coin_multi * produce_coin
//! ```
//!
//! to the per-tick total. Per-tier outputs snap to 0 below
//! [`TIER_NOISE_FLOOR`] to suppress sub-noise contributions in the UI; the
//! aggregate `total` uses the pre-clamp values (matching the legacy
//! `G.produceTotal` semantics).

use synergismforkd_bignum::Decimal;

/// Below this threshold, a per-tier output snaps to 0 — suppresses
/// extremely-small-but-nonzero values that would otherwise pollute the
/// total before they're meaningful.
pub const TIER_NOISE_FLOOR: f64 = 0.0001;

/// Hardcoded 40 Hz tick-rate factor that maps total-per-tick to per-second
/// in the legacy display. Lives here rather than in the UI tier because it
/// is baked into the math contract callers depend on.
pub const TICKS_PER_SECOND: f64 = 40.0;

/// Per-tier inputs to [`calculate_coin_production`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerCoinTierInput {
    /// `player.<tier>GeneratedCoin` — the field guaranteed to be `Decimal`
    /// in the legacy schema.
    pub generated: Decimal,
    /// `player.<tier>OwnedCoin` — count of tier-N producers owned. Fed
    /// straight into the (`generated + owned`) term.
    pub owned: f64,
    /// `G.coin<Tier>Multi` — per-tier coin multiplier.
    pub coin_multi: Decimal,
    /// `player.<tier>ProduceCoin` — per-tier production scalar. Multiplied
    /// in after the global / coin-multi terms.
    pub produce_coin: f64,
}

/// Inputs to [`calculate_coin_production`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalculateCoinProductionInput {
    /// Tier-1 (first) coin-producer.
    pub first: PerCoinTierInput,
    /// Tier-2 (second) coin-producer.
    pub second: PerCoinTierInput,
    /// Tier-3 (third) coin-producer.
    pub third: PerCoinTierInput,
    /// Tier-4 (fourth) coin-producer.
    pub fourth: PerCoinTierInput,
    /// Tier-5 (fifth) coin-producer.
    pub fifth: PerCoinTierInput,
    /// `G.globalCoinMultiplier` — applied to every tier.
    pub global_coin_multiplier: Decimal,
}

/// Result of [`calculate_coin_production`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalculateCoinProductionResult {
    /// Tier-1 output, post-noise-clamp.
    pub first: Decimal,
    /// Tier-2 output, post-noise-clamp.
    pub second: Decimal,
    /// Tier-3 output, post-noise-clamp.
    pub third: Decimal,
    /// Tier-4 output, post-noise-clamp.
    pub fourth: Decimal,
    /// Tier-5 output, post-noise-clamp.
    pub fifth: Decimal,
    /// Sum of **pre-clamp** tier outputs — matches the legacy
    /// `G.produceTotal`. The clamps only affect the per-tier displays, not
    /// the aggregate.
    pub total: Decimal,
    /// `total * 40` — per-tick total scaled to per-second for display.
    pub per_second: Decimal,
}

fn tier_output(tier: PerCoinTierInput, global_coin_multiplier: Decimal) -> Decimal {
    (tier.generated + Decimal::from_finite(tier.owned))
        * global_coin_multiplier
        * tier.coin_multi
        * Decimal::from_finite(tier.produce_coin)
}

fn clamp_noise(value: Decimal) -> Decimal {
    if value <= Decimal::from_finite(TIER_NOISE_FLOOR) {
        Decimal::zero()
    } else {
        value
    }
}

/// Per-tier coin production with noise-floor clamping. The aggregate `total`
/// uses the pre-clamp values (matching legacy behavior); each per-tier field
/// in the result is post-clamp. `per_second` is the per-tick total scaled
/// by the hardcoded 40 Hz tick rate.
#[must_use]
pub fn calculate_coin_production(
    input: CalculateCoinProductionInput,
) -> CalculateCoinProductionResult {
    let first = tier_output(input.first, input.global_coin_multiplier);
    let second = tier_output(input.second, input.global_coin_multiplier);
    let third = tier_output(input.third, input.global_coin_multiplier);
    let fourth = tier_output(input.fourth, input.global_coin_multiplier);
    let fifth = tier_output(input.fifth, input.global_coin_multiplier);

    // Aggregate uses the pre-clamp values — the clamps only affect the
    // per-tier display, not the total.
    let total = first + second + third + fourth + fifth;

    CalculateCoinProductionResult {
        first: clamp_noise(first),
        second: clamp_noise(second),
        third: clamp_noise(third),
        fourth: clamp_noise(fourth),
        fifth: clamp_noise(fifth),
        total,
        per_second: total * Decimal::from_finite(TICKS_PER_SECOND),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tier() -> PerCoinTierInput {
        PerCoinTierInput {
            generated: Decimal::zero(),
            owned: 0.0,
            coin_multi: Decimal::one(),
            produce_coin: 1.0,
        }
    }

    fn unit_input() -> CalculateCoinProductionInput {
        CalculateCoinProductionInput {
            first: unit_tier(),
            second: unit_tier(),
            third: unit_tier(),
            fourth: unit_tier(),
            fifth: unit_tier(),
            global_coin_multiplier: Decimal::one(),
        }
    }

    #[test]
    fn all_zero_input_produces_zero_output() {
        let result = calculate_coin_production(unit_input());
        assert_eq!(result.first, Decimal::zero());
        assert_eq!(result.second, Decimal::zero());
        assert_eq!(result.third, Decimal::zero());
        assert_eq!(result.fourth, Decimal::zero());
        assert_eq!(result.fifth, Decimal::zero());
        assert_eq!(result.total, Decimal::zero());
        assert_eq!(result.per_second, Decimal::zero());
    }

    #[test]
    fn single_tier_output_uses_all_multipliers() {
        // first.generated = 2, owned = 3 → 5
        // coin_multi = 10, produce_coin = 4, global = 6
        // 5 * 6 * 10 * 4 = 1200
        let mut input = unit_input();
        input.first = PerCoinTierInput {
            generated: Decimal::from_finite(2.0),
            owned: 3.0,
            coin_multi: Decimal::from_finite(10.0),
            produce_coin: 4.0,
        };
        input.global_coin_multiplier = Decimal::from_finite(6.0);
        let result = calculate_coin_production(input);
        assert!((result.first.to_number() - 1200.0).abs() < 1e-9);
    }

    #[test]
    fn total_aggregates_all_five_tiers() {
        // Each tier produces 100 (post unit multipliers); five tiers → 500.
        let tier = PerCoinTierInput {
            generated: Decimal::from_finite(100.0),
            owned: 0.0,
            coin_multi: Decimal::one(),
            produce_coin: 1.0,
        };
        let input = CalculateCoinProductionInput {
            first: tier,
            second: tier,
            third: tier,
            fourth: tier,
            fifth: tier,
            global_coin_multiplier: Decimal::one(),
        };
        let result = calculate_coin_production(input);
        assert!((result.total.to_number() - 500.0).abs() < 1e-9);
    }

    #[test]
    fn per_second_is_total_times_40() {
        let tier = PerCoinTierInput {
            generated: Decimal::from_finite(10.0),
            owned: 0.0,
            coin_multi: Decimal::one(),
            produce_coin: 1.0,
        };
        let input = CalculateCoinProductionInput {
            first: tier,
            second: tier,
            third: tier,
            fourth: tier,
            fifth: tier,
            global_coin_multiplier: Decimal::one(),
        };
        let result = calculate_coin_production(input);
        assert!((result.per_second.to_number() - result.total.to_number() * 40.0).abs() < 1e-9);
        assert!((result.per_second.to_number() - 2000.0).abs() < 1e-9);
    }

    #[test]
    fn per_tier_output_clamps_to_zero_below_noise_floor() {
        // generated = 1e-6 → output 1e-6, below 1e-4 floor → snaps to 0
        let mut input = unit_input();
        input.first = PerCoinTierInput {
            generated: Decimal::from_finite(1e-6),
            ..unit_tier()
        };
        let result = calculate_coin_production(input);
        assert_eq!(result.first, Decimal::zero());
        // Total uses pre-clamp, so it should still show the 1e-6 contribution.
        assert!(result.total.to_number() > 0.0);
        assert!((result.total.to_number() - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn per_tier_output_at_noise_floor_clamps_to_zero() {
        // Boundary case: exactly TIER_NOISE_FLOOR → clamps to 0 (lte check).
        let mut input = unit_input();
        input.first = PerCoinTierInput {
            generated: Decimal::from_finite(TIER_NOISE_FLOOR),
            ..unit_tier()
        };
        let result = calculate_coin_production(input);
        assert_eq!(result.first, Decimal::zero());
    }

    #[test]
    fn per_tier_output_just_above_noise_floor_survives() {
        // Just above noise floor → preserved.
        let mut input = unit_input();
        input.first = PerCoinTierInput {
            generated: Decimal::from_finite(TIER_NOISE_FLOOR * 2.0),
            ..unit_tier()
        };
        let result = calculate_coin_production(input);
        assert!(result.first.to_number() > 0.0);
    }
}
