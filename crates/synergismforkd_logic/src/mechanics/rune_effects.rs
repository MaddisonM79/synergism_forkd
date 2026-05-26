//! Per-rune effect formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/runeEffects.ts`. Each
//! function maps `(effective rune level n, effect key) → effect value`.
//! These are the pure cores of the `runes.<rune>.effects` fields in the
//! legacy `packages/web_ui/src/Runes.ts`.
//!
//! Two runes (infinite-ascent and antiquities) read singularity state
//! for one of their keys; those values are hoisted into a small
//! `input` argument and the shim recomputes them per call.

// ─── speed ─────────────────────────────────────────────────────────────────

/// Effect key for [`speed_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedRuneKey {
    /// Accelerator-power additive contribution.
    AcceleratorPower,
    /// Multiplicative-accelerators multiplier.
    MultiplicativeAccelerators,
    /// Global-speed multiplier (cube-root-exponential approach to 2).
    GlobalSpeed,
}

/// Per-effect value for the speed rune at level `n`.
#[must_use]
pub fn speed_rune_effects(n: f64, key: SpeedRuneKey) -> f64 {
    match key {
        SpeedRuneKey::AcceleratorPower => 0.0002 * n,
        SpeedRuneKey::MultiplicativeAccelerators => 1.0 + n / 400.0,
        SpeedRuneKey::GlobalSpeed => 2.0 - (-n.cbrt() / 100.0).exp(),
    }
}

// ─── duplication ───────────────────────────────────────────────────────────

/// Effect key for [`duplication_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicationRuneKey {
    /// Multiplier-boost additive contribution.
    MultiplierBoosts,
    /// Multiplicative-multipliers multiplier.
    MultiplicativeMultipliers,
    /// Tax-reduction multiplier (cube-root-exponential).
    TaxReduction,
}

/// Per-effect value for the duplication rune at level `n`.
#[must_use]
pub fn duplication_rune_effects(n: f64, key: DuplicationRuneKey) -> f64 {
    match key {
        DuplicationRuneKey::MultiplierBoosts => n / 5.0,
        DuplicationRuneKey::MultiplicativeMultipliers => 1.0 + n / 400.0,
        DuplicationRuneKey::TaxReduction => 0.001 + 0.999 * (-n.cbrt() / 5.0).exp(),
    }
}

// ─── prism ─────────────────────────────────────────────────────────────────

/// Effect key for [`prism_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrismRuneKey {
    /// Production-log10 additive contribution.
    ProductionLog10,
    /// Cost-divisor-log10 additive contribution (floor of `n / 10`).
    CostDivisorLog10,
}

/// Per-effect value for the prism rune at level `n`.
#[must_use]
pub fn prism_rune_effects(n: f64, key: PrismRuneKey) -> f64 {
    match key {
        PrismRuneKey::ProductionLog10 => 0.0_f64
            .max(2.0 * (1.0 + n / 2.0).log10() + (n / 2.0) * 2.0_f64.log10() - 256.0_f64.log10()),
        PrismRuneKey::CostDivisorLog10 => (n / 10.0).floor(),
    }
}

// ─── thrift ────────────────────────────────────────────────────────────────

/// Effect key for [`thrift_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThriftRuneKey {
    /// Cost-delay multiplier (capped at 1e15).
    CostDelay,
    /// Salvage additive contribution (`2.5 * ln(1 + n / 10)`).
    Salvage,
    /// Tax-reduction multiplier (cube-root-exponential).
    TaxReduction,
}

/// Per-effect value for the thrift rune at level `n`.
#[must_use]
pub fn thrift_rune_effects(n: f64, key: ThriftRuneKey) -> f64 {
    match key {
        ThriftRuneKey::CostDelay => 1e15_f64.min(n / 125.0),
        ThriftRuneKey::Salvage => 2.5 * (1.0 + n / 10.0).ln(),
        ThriftRuneKey::TaxReduction => 0.01 + 0.99 * (-n.cbrt() / 10.0).exp(),
    }
}

// ─── superiorIntellect ─────────────────────────────────────────────────────

/// Effect key for [`superior_intellect_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuperiorIntellectRuneKey {
    /// Offering multiplier.
    OfferingMult,
    /// Obtainium multiplier.
    ObtainiumMult,
    /// Ant-speed multiplier (`(1 + n / 500)^2`).
    AntSpeed,
}

/// Per-effect value for the SI rune at level `n`.
#[must_use]
pub fn superior_intellect_rune_effects(n: f64, key: SuperiorIntellectRuneKey) -> f64 {
    match key {
        SuperiorIntellectRuneKey::OfferingMult => 1.0 + n / 2_000.0,
        SuperiorIntellectRuneKey::ObtainiumMult => 1.0 + n / 200.0,
        SuperiorIntellectRuneKey::AntSpeed => (1.0 + n / 500.0).powi(2),
    }
}

// ─── infiniteAscent ────────────────────────────────────────────────────────

/// Effect key for [`infinite_ascent_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfiniteAscentRuneKey {
    /// Quark multiplier (`1 + n/500 + (n > 0 ? 0.1 : 0)`).
    QuarkMult,
    /// Cube multiplier (`1 + n/100`).
    CubeMult,
    /// Salvage additive (`n * 0.025 * salvage_perk_unlocked_count`).
    Salvage,
}

/// Inputs to [`infinite_ascent_rune_effects`].
#[derive(Debug, Clone, Copy)]
pub struct InfiniteAscentRuneInput {
    /// Number of salvage-perk thresholds unlocked. Legacy:
    /// `salvagePerkLevels.filter(x => x <= player.highestSingularityCount).length`
    /// where `salvagePerkLevels = [30, 40, 61, 81, 111, 131, 161, 191, 236, 260]`.
    /// The perk-levels table is a UI config, so callers compute the
    /// count.
    pub salvage_perk_unlocked_count: f64,
}

/// Per-effect value for the infinite-ascent rune at level `n`.
#[must_use]
pub fn infinite_ascent_rune_effects(
    n: f64,
    key: InfiniteAscentRuneKey,
    input: InfiniteAscentRuneInput,
) -> f64 {
    match key {
        InfiniteAscentRuneKey::QuarkMult => 1.0 + n / 500.0 + if n > 0.0 { 0.1 } else { 0.0 },
        InfiniteAscentRuneKey::CubeMult => 1.0 + n / 100.0,
        InfiniteAscentRuneKey::Salvage => n * 0.025 * input.salvage_perk_unlocked_count,
    }
}

// ─── antiquities ───────────────────────────────────────────────────────────

/// Effect key for [`antiquities_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiquitiesRuneKey {
    /// `addCode` cooldown reduction. `0.8 - 0.3 * (n - 1) / (n + 10)`
    /// when `n > 0`, else `1`.
    AddCodeCooldownReduction,
    /// Offering-log10 contribution (`round(300 * (1 - (1 - 1/300)^n))`).
    OfferingLog10,
    /// Obtainium-log10 contribution (same formula as `OfferingLog10`).
    ObtainiumLog10,
    /// Cube-bonus multiplier (`1.01^(min(5, n) * singularity_count)` when
    /// `n > 0`, else `1`).
    CubeBonus,
}

/// Inputs to [`antiquities_rune_effects`].
#[derive(Debug, Clone, Copy)]
pub struct AntiquitiesRuneInput {
    /// `player.singularityCount` — feeds the cube-bonus exponent.
    pub singularity_count: f64,
}

/// Per-effect value for the antiquities rune at level `n`.
#[must_use]
pub fn antiquities_rune_effects(
    n: f64,
    key: AntiquitiesRuneKey,
    input: AntiquitiesRuneInput,
) -> f64 {
    match key {
        AntiquitiesRuneKey::AddCodeCooldownReduction => {
            if n > 0.0 {
                0.8 - 0.3 * (n - 1.0) / (n + 10.0)
            } else {
                1.0
            }
        }
        AntiquitiesRuneKey::OfferingLog10 | AntiquitiesRuneKey::ObtainiumLog10 => {
            (300.0_f64 * (1.0 - (1.0 - 1.0 / 300.0_f64).powf(n))).round()
        }
        AntiquitiesRuneKey::CubeBonus => {
            if n > 0.0 {
                1.01_f64.powf(5.0_f64.min(n) * input.singularity_count)
            } else {
                1.0
            }
        }
    }
}

// ─── horseShoe ─────────────────────────────────────────────────────────────

/// Effect key for [`horse_shoe_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorseShoeRuneKey {
    /// Ambrosia-luck additive contribution (`= n`).
    AmbrosiaLuck,
    /// Red-luck additive contribution (`n / 5`).
    RedLuck,
    /// Red-luck-conversion additive (`-0.5 * n / (n + 50)`).
    RedLuckConversion,
}

/// Per-effect value for the horse-shoe rune at level `n`.
#[must_use]
pub fn horse_shoe_rune_effects(n: f64, key: HorseShoeRuneKey) -> f64 {
    match key {
        HorseShoeRuneKey::AmbrosiaLuck => n,
        HorseShoeRuneKey::RedLuck => n / 5.0,
        HorseShoeRuneKey::RedLuckConversion => -0.5 * n / (n + 50.0),
    }
}

// ─── finiteDescent ─────────────────────────────────────────────────────────

/// Effect key for [`finite_descent_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiniteDescentRuneKey {
    /// Ascension-score multiplier
    /// (`1.04 + 0.96 * (n - 1) / (n + 25)` when `n >= 1`, else `1`).
    AscensionScore,
    /// Corruption-free-levels contribution
    /// (`0.01 + 0.14 * (n - 1) / (n + 16)` when `n >= 1`, else `0`).
    CorruptionFreeLevels,
    /// Infinite-ascent free-level contribution (`floor(n / 2)`).
    InfiniteAscentFreeLevel,
}

/// Per-effect value for the finite-descent rune at level `n`.
#[must_use]
pub fn finite_descent_rune_effects(n: f64, key: FiniteDescentRuneKey) -> f64 {
    match key {
        FiniteDescentRuneKey::AscensionScore => {
            if n >= 1.0 {
                1.04 + 0.96 * (n - 1.0) / (n + 25.0)
            } else {
                1.0
            }
        }
        FiniteDescentRuneKey::CorruptionFreeLevels => {
            if n >= 1.0 {
                0.01 + 0.14 * (n - 1.0) / (n + 16.0)
            } else {
                0.0
            }
        }
        FiniteDescentRuneKey::InfiniteAscentFreeLevel => (n / 2.0).floor(),
    }
}

// ─── topHat ────────────────────────────────────────────────────────────────

/// Effect key for [`top_hat_rune_effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopHatRuneKey {
    /// Offering free-levels contribution
    /// (`round(200 * (1 - 0.995^n)) / 10`).
    FreeOfferingLevels,
    /// Obtainium free-levels contribution (same formula).
    FreeObtainiumLevels,
    /// Cube free-levels contribution
    /// (`round(150 * (1 - 0.997^n)) / 10`).
    FreeCubeLevels,
    /// Speed free-levels contribution (same formula).
    FreeSpeedLevels,
    /// Infinity free-levels contribution
    /// (`round(100 * (1 - 0.999^n)) / 10`).
    FreeInfinityLevels,
}

/// Per-effect value for the top-hat rune at level `n`.
#[must_use]
pub fn top_hat_rune_effects(n: f64, key: TopHatRuneKey) -> f64 {
    match key {
        TopHatRuneKey::FreeOfferingLevels | TopHatRuneKey::FreeObtainiumLevels => {
            (200.0 * (1.0 - 0.995_f64.powf(n))).round() / 10.0
        }
        TopHatRuneKey::FreeCubeLevels | TopHatRuneKey::FreeSpeedLevels => {
            (150.0 * (1.0 - 0.997_f64.powf(n))).round() / 10.0
        }
        TopHatRuneKey::FreeInfinityLevels => (100.0 * (1.0 - 0.999_f64.powf(n))).round() / 10.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── speed ─────────────────────────────────────────────────────────────

    #[test]
    fn speed_accelerator_power_scales_linearly() {
        assert_eq!(speed_rune_effects(0.0, SpeedRuneKey::AcceleratorPower), 0.0);
        assert!((speed_rune_effects(5000.0, SpeedRuneKey::AcceleratorPower) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn speed_global_speed_approaches_two() {
        // At very high n, 2 - exp(-cbrt(n)/100) approaches 2. At n = 1e15
        // the exp term underflows to 0 and the result hits exactly 2.0.
        let result = speed_rune_effects(1e15, SpeedRuneKey::GlobalSpeed);
        assert!(result > 1.9 && result <= 2.0);
    }

    // ─── duplication ───────────────────────────────────────────────────────

    #[test]
    fn duplication_tax_reduction_starts_near_one() {
        // n = 0 → 0.001 + 0.999 * exp(0) = 0.001 + 0.999 = 1.0
        let result = duplication_rune_effects(0.0, DuplicationRuneKey::TaxReduction);
        assert!((result - 1.0).abs() < 1e-12);
    }

    // ─── prism ─────────────────────────────────────────────────────────────

    #[test]
    fn prism_production_log10_clamped_at_zero() {
        // At n=0: 2*log10(1) + 0 - log10(256) = -log10(256) ≈ -2.4 → clamped 0
        assert_eq!(prism_rune_effects(0.0, PrismRuneKey::ProductionLog10), 0.0);
    }

    #[test]
    fn prism_cost_divisor_floor() {
        assert_eq!(prism_rune_effects(0.0, PrismRuneKey::CostDivisorLog10), 0.0);
        assert_eq!(prism_rune_effects(9.0, PrismRuneKey::CostDivisorLog10), 0.0);
        assert_eq!(
            prism_rune_effects(10.0, PrismRuneKey::CostDivisorLog10),
            1.0
        );
        assert_eq!(
            prism_rune_effects(100.0, PrismRuneKey::CostDivisorLog10),
            10.0
        );
    }

    // ─── thrift ────────────────────────────────────────────────────────────

    #[test]
    fn thrift_cost_delay_capped() {
        // n / 125 capped at 1e15; n = 2e17 / 125 = 1.6e15 → capped to 1e15.
        let result = thrift_rune_effects(2e17, ThriftRuneKey::CostDelay);
        assert_eq!(result, 1e15);
    }

    // ─── infiniteAscent ────────────────────────────────────────────────────

    #[test]
    fn infinite_ascent_quark_mult_zero_level() {
        // n = 0 → 1 + 0 + 0 (since n > 0 is false) = 1
        let input = InfiniteAscentRuneInput {
            salvage_perk_unlocked_count: 0.0,
        };
        assert_eq!(
            infinite_ascent_rune_effects(0.0, InfiniteAscentRuneKey::QuarkMult, input),
            1.0
        );
    }

    #[test]
    fn infinite_ascent_quark_mult_one_level_includes_threshold_bonus() {
        // n = 1 → 1 + 1/500 + 0.1 = 1.102
        let input = InfiniteAscentRuneInput {
            salvage_perk_unlocked_count: 0.0,
        };
        let result = infinite_ascent_rune_effects(1.0, InfiniteAscentRuneKey::QuarkMult, input);
        assert!((result - 1.102).abs() < 1e-9);
    }

    // ─── antiquities ───────────────────────────────────────────────────────

    #[test]
    fn antiquities_add_code_cooldown_zero_at_zero() {
        // n = 0 → 1 (no reduction)
        let input = AntiquitiesRuneInput {
            singularity_count: 0.0,
        };
        assert_eq!(
            antiquities_rune_effects(0.0, AntiquitiesRuneKey::AddCodeCooldownReduction, input),
            1.0
        );
    }

    #[test]
    fn antiquities_cube_bonus_zero_level_is_one() {
        let input = AntiquitiesRuneInput {
            singularity_count: 100.0,
        };
        assert_eq!(
            antiquities_rune_effects(0.0, AntiquitiesRuneKey::CubeBonus, input),
            1.0
        );
    }

    #[test]
    fn antiquities_cube_bonus_clamps_n_at_5() {
        // Both n=5 and n=10 should produce the same value (since min(5, n)).
        let input = AntiquitiesRuneInput {
            singularity_count: 100.0,
        };
        let n5 = antiquities_rune_effects(5.0, AntiquitiesRuneKey::CubeBonus, input);
        let n10 = antiquities_rune_effects(10.0, AntiquitiesRuneKey::CubeBonus, input);
        assert_eq!(n5, n10);
    }

    // ─── horseShoe ─────────────────────────────────────────────────────────

    #[test]
    fn horse_shoe_red_luck_is_one_fifth() {
        assert_eq!(
            horse_shoe_rune_effects(50.0, HorseShoeRuneKey::RedLuck),
            10.0
        );
    }

    // ─── finiteDescent ─────────────────────────────────────────────────────

    #[test]
    fn finite_descent_ascension_score_below_one_is_one() {
        assert_eq!(
            finite_descent_rune_effects(0.0, FiniteDescentRuneKey::AscensionScore),
            1.0
        );
    }

    #[test]
    fn finite_descent_ascension_score_at_one_is_1_04() {
        // 1.04 + 0.96 * 0 / 26 = 1.04
        let result = finite_descent_rune_effects(1.0, FiniteDescentRuneKey::AscensionScore);
        assert!((result - 1.04).abs() < 1e-12);
    }

    // ─── topHat ────────────────────────────────────────────────────────────

    #[test]
    fn top_hat_free_offering_levels_at_zero_is_zero() {
        // round(200 * (1 - 0.995^0)) / 10 = round(0) / 10 = 0
        assert_eq!(
            top_hat_rune_effects(0.0, TopHatRuneKey::FreeOfferingLevels),
            0.0
        );
    }
}
