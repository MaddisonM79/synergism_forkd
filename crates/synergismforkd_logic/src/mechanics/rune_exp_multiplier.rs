//! Universal rune-EXP-per-offering multiplier.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/runeEXPMultiplier.ts`
//! (lifted from the legacy `packages/web_ui/src/Runes.ts`).
//!
//! The shape: `(additive) × (product of all-rune multipliers) × (recycle)`
//!
//! Three groups of inputs:
//! 1. **Additive**: base `1`, plus C1 completion bonus + per-C1 scaling,
//!    researches 5x2 / 5x3, particle upgrade 3x1 (which scales with the
//!    caller's `purchased_levels` to discourage low-level
//!    dump-and-respec).
//! 2. **Multiplicative**: researches 4x16 / 4x17, cube upgrade 32 ×
//!    ascension counter, constant upgrade 8, challenge-15 rune-EXP
//!    reward multiplier.
//! 3. **Recycle**: the inverse of recycle/salvage chance — passed in
//!    from the legacy `calculateSalvageRuneEXPMultiplier`.

use synergismforkd_bignum::Decimal;

/// Inputs to [`universal_rune_exp_mult`]. Mirrors
/// `UniversalRuneEXPMultInput`.
#[derive(Debug, Clone, Copy)]
pub struct UniversalRuneEXPMultInput {
    /// `rune.level` — the per-rune `purchasedLevels` parameter. Feeds
    /// the particle-upgrade-3x1 additive contribution.
    pub purchased_levels: f64,
    /// `player.highestchallengecompletions[1]`. Bonus is
    /// `min(1, n) + 0.04 * n`.
    pub c1_completions: f64,
    /// `player.researches[22]`. `+0.6` per level.
    pub research_22: f64,
    /// `player.researches[23]`. `+0.3` per level.
    pub research_23: f64,
    /// `player.upgrades[71]` — Particle upgrade 3x1: adds
    /// `n * purchased_levels / 25`.
    pub upgrade_71: f64,
    /// `player.researches[91]`. `×(1 + n/20)`.
    pub research_91: f64,
    /// `player.researches[92]`. `×(1 + n/20)`.
    pub research_92: f64,
    /// `player.ascensionCounter` — seconds in current ascension. Feeds
    /// the cube-upgrade-32 bonus.
    pub ascension_counter: f64,
    /// `player.cubeUpgrades[32]`. Bonus `×(1 + ascension_counter / 1000 * n)`.
    pub cube_upgrade_32: f64,
    /// `player.constantUpgrades[8]`. `×(1 + n / 10)`.
    pub constant_upgrade_8: f64,
    /// `G.challenge15Rewards.runeExp.value` — number multiplier from
    /// the C15 reward formula.
    pub challenge_15_rune_exp_reward: f64,
    /// `calculateSalvageRuneEXPMultiplier()` — the recycle/salvage
    /// multiplier (the inverse of effective recycle chance). Passed in
    /// to avoid pulling the whole salvage math chain into this module.
    pub salvage_rune_exp_multiplier: Decimal,
}

/// Universal multiplier applied to the base EXP-per-offering for every
/// rune. Pure function over the input bundle.
///
/// Returns `additive × multiplicative × recycle`, where `additive` and
/// `multiplicative` are themselves a sum-of-contributions and a
/// product-of-contributions as documented above. Result type is
/// `Decimal` because the C15 reward and salvage multiplier can both
/// exceed `Number` range.
#[must_use]
pub fn universal_rune_exp_mult(input: UniversalRuneEXPMultInput) -> Decimal {
    let all_rune_exp_additive_multiplier = 1.0
        // C1 completion: +1 for any completion, +0.04 per completion
        + 1.0_f64.min(input.c1_completions)
        + (0.4 / 10.0) * input.c1_completions
        // Research 5x2
        + 0.6 * input.research_22
        // Research 5x3
        + 0.3 * input.research_23
        // Particle upgrade 3x1 — scales with purchased_levels
        + (input.upgrade_71 * input.purchased_levels) / 25.0;

    let all_rune_exp_multiplier = [
        // Research 4x16
        1.0 + input.research_91 / 20.0,
        // Research 4x17
        1.0 + input.research_92 / 20.0,
        // Cube Upgrade 32 × ascension time
        1.0 + (input.ascension_counter / 1000.0) * input.cube_upgrade_32,
        // Constant Upgrade 8
        1.0 + (1.0 / 10.0) * input.constant_upgrade_8,
        // Challenge 15 reward
        input.challenge_15_rune_exp_reward,
    ]
    .iter()
    .fold(Decimal::one(), |acc, &x| acc * Decimal::from_finite(x));

    all_rune_exp_multiplier
        * Decimal::from_finite(all_rune_exp_additive_multiplier)
        * input.salvage_rune_exp_multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> UniversalRuneEXPMultInput {
        UniversalRuneEXPMultInput {
            purchased_levels: 0.0,
            c1_completions: 0.0,
            research_22: 0.0,
            research_23: 0.0,
            upgrade_71: 0.0,
            research_91: 0.0,
            research_92: 0.0,
            ascension_counter: 0.0,
            cube_upgrade_32: 0.0,
            constant_upgrade_8: 0.0,
            challenge_15_rune_exp_reward: 1.0,
            salvage_rune_exp_multiplier: Decimal::one(),
        }
    }

    #[test]
    fn baseline_is_one() {
        let result = universal_rune_exp_mult(baseline());
        assert!((result.to_number() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn c1_completion_first_adds_one_plus_4_percent() {
        // c1 = 1 → +1 + 0.04 = +1.04 → additive = 2.04
        let input = UniversalRuneEXPMultInput {
            c1_completions: 1.0,
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        assert!((result.to_number() - 2.04).abs() < 1e-12);
    }

    #[test]
    fn c1_completion_caps_min_1_term() {
        // The `min(1, n)` term saturates at n=1 — additional completions
        // only contribute the 0.04 each.
        let input = UniversalRuneEXPMultInput {
            c1_completions: 100.0,
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        // additive = 1 + 1 + 0.04*100 = 6
        assert!((result.to_number() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn research_22_adds_0_6_per_level() {
        let input = UniversalRuneEXPMultInput {
            research_22: 1.0,
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        assert!((result.to_number() - 1.6).abs() < 1e-12);
    }

    #[test]
    fn upgrade_71_scales_with_purchased_levels() {
        // upgrade_71 = 1, purchased_levels = 25 → +25/25 = +1 → additive 2
        let input = UniversalRuneEXPMultInput {
            upgrade_71: 1.0,
            purchased_levels: 25.0,
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        assert!((result.to_number() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn research_91_multiplies_by_one_plus_one_twentieth() {
        let input = UniversalRuneEXPMultInput {
            research_91: 1.0,
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        // additive = 1, multiplicative = 1.05
        assert!((result.to_number() - 1.05).abs() < 1e-12);
    }

    #[test]
    fn cube_upgrade_32_scales_with_ascension_counter() {
        // cube_upgrade_32 = 1, ascension_counter = 1000 → 1 + (1000/1000)*1 = 2
        let input = UniversalRuneEXPMultInput {
            cube_upgrade_32: 1.0,
            ascension_counter: 1000.0,
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        assert!((result.to_number() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn salvage_multiplier_scales_result() {
        let input = UniversalRuneEXPMultInput {
            salvage_rune_exp_multiplier: Decimal::from_finite(7.0),
            ..baseline()
        };
        let result = universal_rune_exp_mult(input);
        assert!((result.to_number() - 7.0).abs() < 1e-12);
    }
}
