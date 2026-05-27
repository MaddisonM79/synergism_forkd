//! Small singularity-related helpers.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/singularityHelpers.ts`.
//! None of these warrant their own module but they share the
//! "tiny pure helper called by singularity UI / cost flow" theme:
//!
//! - [`max_singularity_lookahead`]: buy-multi preview count, given
//!   the player's three lookahead-bonus upgrades.
//! - [`golden_quark_cost`]: wraps a GQ-cost result with a
//!   `cost_reduction` diff against the 10000-GQ baseline (for the
//!   UI's "you saved X GQ" badge).
//! - [`calculate_next_spike`]: finds the next singularity-penalty
//!   threshold the player will cross, accounting for shop/ambrosia
//!   reductions.

/// Singularity counts at which a new penalty tier activates.
/// Sorted ascending; [`calculate_next_spike`] walks the array and
/// returns the first threshold past the player's current adjusted
/// count.
const SINGULARITY_PENALTY_THRESHOLDS: &[f64] = &[
    11.0, 26.0, 37.0, 51.0, 101.0, 151.0, 201.0, 216.0, 230.0, 270.0,
];

/// Base cost of one golden quark, used as the baseline for the
/// "you saved X" display in the GQ-buy prompt.
const GOLDEN_QUARK_BASE_COST: f64 = 10_000.0;

/// Inputs to [`max_singularity_lookahead`].
#[derive(Debug, Clone, Copy)]
pub struct MaxSingularityLookaheadInput {
    /// True when the buy-multi prompt is in "show me what's
    /// possible" mode. The legacy `nonZero` parameter — when false,
    /// lookahead is hardcoded 0 (player is just viewing the current
    /// sing, not previewing forward).
    pub non_zero: bool,
    /// `getGQUpgradeEffect('singFastForward', 'lookahead')`.
    pub sing_fast_forward_lookahead: f64,
    /// `getGQUpgradeEffect('singFastForward2', 'lookahead')`.
    pub sing_fast_forward_2_lookahead: f64,
    /// `getOcteractUpgradeEffect('octeractFastForward', 'lookahead')`.
    pub octeract_fast_forward_lookahead: f64,
}

/// Max number of singularities the buy-multi prompt previews.
/// Always returns `0` when `non_zero` is false; otherwise sums the
/// three lookahead bonuses (default `1 + sum`).
#[must_use]
pub fn max_singularity_lookahead(input: &MaxSingularityLookaheadInput) -> f64 {
    if !input.non_zero {
        return 0.0;
    }
    1.0 + input.sing_fast_forward_lookahead
        + input.sing_fast_forward_2_lookahead
        + input.octeract_fast_forward_lookahead
}

/// Result of [`golden_quark_cost`].
#[derive(Debug, Clone, Copy)]
pub struct GoldenQuarkCostResult {
    /// The actual per-GQ cost (passed through).
    pub cost: f64,
    /// `max(0, 10000 - cost)` — how much cheaper than the 10000-GQ
    /// baseline the current cost is. Used by the UI to display
    /// "you saved X" badges.
    pub cost_reduction: f64,
}

/// Wraps a calculated GQ cost with its `cost_reduction` diff
/// against the 10000-GQ baseline. The reduction is floored at `0`
/// — if `cost > 10000` (e.g. inside a debuffing challenge), no
/// reduction is shown.
#[must_use]
pub fn golden_quark_cost(cost: f64) -> GoldenQuarkCostResult {
    GoldenQuarkCostResult {
        cost,
        cost_reduction: 0.0_f64.max(GOLDEN_QUARK_BASE_COST - cost),
    }
}

/// Inputs to [`calculate_next_spike`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateNextSpikeInput {
    /// The player's raw singularity count being evaluated.
    pub singularity_count: f64,
    /// Shop/ambrosia singularity reductions. Subtracted from each
    /// threshold when checking if the player has crossed it —
    /// matches the `constitutiveSingularityCount` logic used
    /// elsewhere in singularity math.
    pub singularity_reductions: f64,
}

/// Returns the next singularity-penalty threshold the player will
/// cross, or `-1` if they're past all of them. Each threshold is
/// offset by the player's `singularityReductions`, so the spike
/// fires later for players with reduction upgrades.
#[must_use]
pub fn calculate_next_spike(input: &CalculateNextSpikeInput) -> f64 {
    for &sing in SINGULARITY_PENALTY_THRESHOLDS {
        if sing + input.singularity_reductions > input.singularity_count {
            return sing + input.singularity_reductions;
        }
    }
    -1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_lookahead_returns_zero_when_non_zero_is_false() {
        let result = max_singularity_lookahead(&MaxSingularityLookaheadInput {
            non_zero: false,
            sing_fast_forward_lookahead: 5.0,
            sing_fast_forward_2_lookahead: 5.0,
            octeract_fast_forward_lookahead: 5.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn max_lookahead_sums_three_bonuses() {
        let result = max_singularity_lookahead(&MaxSingularityLookaheadInput {
            non_zero: true,
            sing_fast_forward_lookahead: 2.0,
            sing_fast_forward_2_lookahead: 3.0,
            octeract_fast_forward_lookahead: 1.0,
        });
        // 1 + 2 + 3 + 1 = 7
        assert_eq!(result, 7.0);
    }

    #[test]
    fn golden_quark_cost_reduction_floored_at_zero() {
        let result = golden_quark_cost(15_000.0);
        assert_eq!(result.cost, 15_000.0);
        assert_eq!(result.cost_reduction, 0.0);
    }

    #[test]
    fn golden_quark_cost_reduction_against_baseline() {
        let result = golden_quark_cost(5_000.0);
        assert_eq!(result.cost_reduction, 5_000.0);
    }

    #[test]
    fn next_spike_at_singularity_5_is_11() {
        let result = calculate_next_spike(&CalculateNextSpikeInput {
            singularity_count: 5.0,
            singularity_reductions: 0.0,
        });
        assert_eq!(result, 11.0);
    }

    #[test]
    fn next_spike_past_all_returns_neg_1() {
        let result = calculate_next_spike(&CalculateNextSpikeInput {
            singularity_count: 1_000.0,
            singularity_reductions: 0.0,
        });
        assert_eq!(result, -1.0);
    }

    #[test]
    fn next_spike_offset_by_reductions() {
        // Without reductions: at 30, next is 37
        // With reductions=10: thresholds become 21, 36, 47... at sing=30,
        // 21 < 30 so skip, 36 > 30 so return 36.
        let result = calculate_next_spike(&CalculateNextSpikeInput {
            singularity_count: 30.0,
            singularity_reductions: 10.0,
        });
        assert_eq!(result, 36.0);
    }
}
