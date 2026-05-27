//! Potion-consumption bonuses.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/potionBonuses.ts`
//! (lifted from the legacy `packages/web_ui/src/Calculate.ts`).
//! Both threshold-bonus functions count how many fixed potion-
//! consumption thresholds the player has crossed (via a binary
//! search) and return that count plus the distance to the next
//! threshold. The threshold arrays are pure data and move with the
//! formulas.
//!
//! Also exports [`calculate_potion_value`] (the actual resource award
//! a potion produces), which uses the same
//! `calculate_fast_forward_resources_global` helper internally.

use synergismforkd_bignum::Decimal;

/// Binary search returning the insertion index for `target` into
/// `array` (assumed sorted ascending). Reproduces the behavior of
/// the legacy `findInsertionIndex` (`Utility.ts`).
fn find_insertion_index(target: f64, array: &[f64]) -> usize {
    if array.is_empty() || target < array[0] {
        return 0;
    }
    if target >= array[array.len() - 1] {
        return array.len();
    }

    let mut low = 0_usize;
    let mut high = array.len() - 1;

    while low < high {
        let mid = low.midpoint(high + 1);
        if array[mid] <= target {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    low + 1
}

const OFFERING_POTION_THRESHOLDS: [f64; 20] = [
    1.0, 10.0, 25.0, 50.0, 100.0, 500.0, 1_000.0, 10_000.0, 5e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10,
    1e11, 1e12, 1e13, 1e14, 1e15,
];

const OBTAINIUM_POTION_THRESHOLDS: [f64; 15] = [
    1.0, 20.0, 50.0, 250.0, 1_000.0, 20_000.0, 4e5, 1e7, 4e8, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15,
];

/// Result of [`calculate_offering_potion_base_offerings`] and
/// [`calculate_obtainium_potion_base_obtainium`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PotionBonusResult {
    /// Number of crossed thresholds — drives the base offering /
    /// obtainium award.
    pub amount: f64,
    /// Lifetime potions of this type still required before the next
    /// threshold, or `f64::INFINITY` if all thresholds have been
    /// crossed.
    pub to_next: f64,
}

fn compute_potion_bonus(consumed: f64, thresholds: &[f64]) -> PotionBonusResult {
    let amount = find_insertion_index(consumed, thresholds);
    let to_next = if amount < thresholds.len() {
        thresholds[amount] - consumed
    } else {
        f64::INFINITY
    };
    PotionBonusResult {
        amount: amount as f64,
        to_next,
    }
}

/// Offering potion base award. Returns the count of crossed
/// `OFFERING_POTION_THRESHOLDS` plus the distance to the next one.
#[must_use]
pub fn calculate_offering_potion_base_offerings(consumed: f64) -> PotionBonusResult {
    compute_potion_bonus(consumed, &OFFERING_POTION_THRESHOLDS)
}

/// Obtainium potion base award. Same shape as
/// [`calculate_offering_potion_base_offerings`] with the
/// obtainium-specific threshold table.
#[must_use]
pub fn calculate_obtainium_potion_base_obtainium(consumed: f64) -> PotionBonusResult {
    compute_potion_bonus(consumed, &OBTAINIUM_POTION_THRESHOLDS)
}

// ─── Potion value ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct CalculateFastForwardResourcesGlobalInput {
    reset_time: f64,
    fast_forward_amount: Decimal,
    resource_mult: Decimal,
    base_resource: f64,
    half_mind_unlocked: bool,
    global_speed_mult: f64,
    reset_time_threshold: f64,
}

fn calculate_fast_forward_resources_global(
    input: CalculateFastForwardResourcesGlobalInput,
) -> Decimal {
    let delta_time = input.fast_forward_amount
        * Decimal::from_finite(if input.half_mind_unlocked {
            10.0
        } else {
            input.global_speed_mult
        });

    // Take the min of two derivative approximations: quadratic-penalty
    // branch (when reset time is below threshold) and linear-penalty
    // branch (above).
    let quadratic = delta_time * Decimal::from_finite(2.0 * input.reset_time)
        / Decimal::from_finite(input.reset_time_threshold.powi(2));
    let linear = delta_time / Decimal::from_finite(input.reset_time_threshold);
    let time_multiplier = quadratic.min(linear);

    // NOTE: preserved verbatim from the legacy — this multiplication
    // discards its result (the legacy code missed an assignment).
    // Treated as a no-op so parity stays exact; a real fix is out of
    // scope.
    let _no_op = time_multiplier
        * Decimal::from_finite(if input.half_mind_unlocked {
            input.global_speed_mult / 10.0
        } else {
            1.0
        });

    (input.fast_forward_amount * Decimal::from_finite(input.base_resource))
        .max(input.resource_mult * time_multiplier)
}

/// Inputs to [`calculate_potion_value`].
#[derive(Debug, Clone, Copy)]
pub struct CalculatePotionValueInput {
    /// `player.{reset}counter` — current run time.
    pub reset_time: f64,
    /// Precomputed `calculateOfferings` or `calculateObtainium`
    /// value.
    pub resource_mult: Decimal,
    /// Precomputed `calculateBaseOfferings` /
    /// `calculateBaseObtainium`.
    pub base_resource: f64,
    /// `getGQUpgradeEffect('halfMind', 'unlocked')`.
    pub half_mind_unlocked: bool,
    /// `calculateGlobalSpeedMult()` in the legacy UI.
    pub global_speed_mult: f64,
    /// `resetTimeThreshold()` in the legacy UI.
    pub reset_time_threshold: f64,
    /// Product of the four potion-power multiplier effects:
    /// `potionBuff`, `potionBuff2`, `potionBuff3` (singularity GQ)
    /// and `octeractAutoPotionEfficiency` (octeract).
    pub potion_multipliers: f64,
}

/// Resource award for activating one potion. Combines a 7200-second
/// fast-forward (`= 2h`) computed against the current run time with
/// the stacked potion-power multipliers.
#[must_use]
pub fn calculate_potion_value(input: &CalculatePotionValueInput) -> Decimal {
    let potion_time_value = Decimal::from_finite(7_200.0);
    let fast_forward_mult =
        calculate_fast_forward_resources_global(CalculateFastForwardResourcesGlobalInput {
            reset_time: input.reset_time,
            fast_forward_amount: potion_time_value,
            resource_mult: input.resource_mult,
            base_resource: input.base_resource,
            half_mind_unlocked: input.half_mind_unlocked,
            global_speed_mult: input.global_speed_mult,
            reset_time_threshold: input.reset_time_threshold,
        });
    fast_forward_mult * Decimal::from_finite(input.potion_multipliers)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── threshold bonuses ─────────────────────────────────────────────────

    #[test]
    fn find_insertion_index_empty_array() {
        assert_eq!(find_insertion_index(5.0, &[]), 0);
    }

    #[test]
    fn find_insertion_index_below_first() {
        assert_eq!(find_insertion_index(0.5, &[1.0, 2.0, 3.0]), 0);
    }

    #[test]
    fn find_insertion_index_above_last() {
        assert_eq!(find_insertion_index(100.0, &[1.0, 2.0, 3.0]), 3);
    }

    #[test]
    fn find_insertion_index_at_value() {
        // target equals an entry → returns the index past it
        assert_eq!(find_insertion_index(2.0, &[1.0, 2.0, 3.0]), 2);
    }

    #[test]
    fn offering_potion_at_zero_crosses_no_thresholds() {
        let result = calculate_offering_potion_base_offerings(0.0);
        assert_eq!(result.amount, 0.0);
        // First threshold is 1 → need 1 more.
        assert_eq!(result.to_next, 1.0);
    }

    #[test]
    fn offering_potion_at_25_crosses_three_thresholds() {
        // 1, 10, 25 → all crossed since 25 >= 25 → index 3
        let result = calculate_offering_potion_base_offerings(25.0);
        assert_eq!(result.amount, 3.0);
        // Next threshold is 50 → 25 to go.
        assert_eq!(result.to_next, 25.0);
    }

    #[test]
    fn offering_potion_past_all_thresholds_returns_infinity() {
        // 1e16 > last threshold (1e15)
        let result = calculate_offering_potion_base_offerings(1e16);
        assert_eq!(result.amount, 20.0);
        assert_eq!(result.to_next, f64::INFINITY);
    }

    #[test]
    fn obtainium_potion_has_15_threshold_table() {
        let result = calculate_obtainium_potion_base_obtainium(1e16);
        assert_eq!(result.amount, 15.0);
    }

    // ─── potion value ──────────────────────────────────────────────────────

    fn potion_value_baseline() -> CalculatePotionValueInput {
        CalculatePotionValueInput {
            reset_time: 100.0,
            resource_mult: Decimal::from_finite(1e6),
            base_resource: 1.0,
            half_mind_unlocked: false,
            global_speed_mult: 1.0,
            reset_time_threshold: 10.0,
            potion_multipliers: 1.0,
        }
    }

    #[test]
    fn potion_value_is_positive() {
        let result = calculate_potion_value(&potion_value_baseline());
        assert!(result.to_number() > 0.0);
    }

    #[test]
    fn potion_value_scales_with_multipliers() {
        let baseline = calculate_potion_value(&potion_value_baseline());
        let doubled = calculate_potion_value(&CalculatePotionValueInput {
            potion_multipliers: 2.0,
            ..potion_value_baseline()
        });
        let ratio = doubled.to_number() / baseline.to_number();
        assert!((ratio - 2.0).abs() < 1e-9);
    }
}
