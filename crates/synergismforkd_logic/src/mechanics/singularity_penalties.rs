//! Singularity-penalty math.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/singularityPenalties.ts`.
//! [`calculate_effective_singularities`] is the post-multiplier
//! singularity count used as the basis for every per-system debuff.
//! [`calculate_singularity_debuff`] switches on the system tag and
//! produces the actual multiplier (or subtractive amount, for
//! Salvage / Ant ELO). The UI side is responsible for sourcing
//! shop / ambrosia / antiquities / rune state and i18n display
//! strings — this module is pure math.

use crate::mechanics::exalt_penalties::{
    calculate_exalt_4_effective_singularity_multiplier,
    CalculateExalt4EffectiveSingularityMultiplierInput,
};

/// Which system the debuff applies to. Determines the formula
/// branch in [`calculate_singularity_debuff`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingularityDebuff {
    /// Offering generation.
    Offering,
    /// Obtainium generation.
    Obtainium,
    /// Salvage (subtractive).
    Salvage,
    /// Global game speed.
    GlobalSpeed,
    /// Research cost / time.
    Researches,
    /// Ant ELO (subtractive).
    AntELO,
    /// Ascension speed.
    AscensionSpeed,
    /// Cube award magnitude.
    Cubes,
    /// Cube-upgrade cost.
    CubeUpgrades,
    /// Platonic upgrade cost.
    PlatonicCosts,
    /// Hepteract cost.
    HepteractCosts,
}

/// Inputs to [`calculate_effective_singularities`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateEffectiveSingularitiesInput {
    /// The raw singularity count being evaluated. Callers usually
    /// pass the constitutive count (`singularityCount - reductions`),
    /// not the raw value.
    pub singularity_count: f64,
    /// `player.singularityChallenges.noOcteracts.completions` — feeds
    /// Exalt 4.
    pub no_octeracts_completions: f64,
    /// `player.singularityChallenges.noOcteracts.enabled` — gates
    /// Exalt 4.
    pub in_exalt_4: bool,
    /// `player.singularityChallenges.taxmanLastStand.enabled` —
    /// gates the cube root of the final value when the
    /// suppress-platonic-15 condition is met.
    pub taxman_last_stand_enabled: bool,
    /// `player.singularityChallenges.taxmanLastStand.completions` —
    /// must be `>= 8` alongside an unowned platonic 15 for the
    /// `^(3/2)` override.
    pub taxman_last_stand_completions: f64,
    /// `player.platonicUpgrades[15]` — when `> 0`, suppresses the
    /// taxman override.
    pub platonic_upgrade_15: f64,
}

/// Effective singularity count after stacking the staircase of
/// milestone multipliers (`×1.5` past 10, `×2.5` past 25, etc.).
/// The Exalt 4 multiplier and the taxman-last-stand `^(3/2)`
/// override are both applied here.
///
/// # ⚠️ Redesign pending — DO NOT extend
///
/// **Assayer audit BLOCKER 1**: the staircase below is an unbounded
/// reinforcing loop with no damping term. Numerical evaluation shows
/// growth crossing 1e36 by singularity 280; the only counterweight
/// is the antiquities-rune binary override in
/// [`calculate_singularity_debuff`] (audit BLOCKER 2), which makes
/// the two pathologies entangled — fixing the staircase requires
/// fixing the override, and vice versa.
///
/// Per the audit plan (Tier C item 16), further singularity-layer
/// porting is paused pending a Loom design pass on the replacement
/// curve. The current verbatim port is preserved so existing parity
/// fixtures stay green, but parity on this mechanic is
/// **informational, not blocking** (Tier C item 15) — a PR that
/// breaks the staircase numerics is *expected* once the redesign
/// lands.
#[must_use]
pub fn calculate_effective_singularities(input: &CalculateEffectiveSingularitiesInput) -> f64 {
    let singularity_count = input.singularity_count;
    let mut effective = singularity_count;
    effective *= 4.75_f64.min((0.75 * singularity_count) / 10.0 + 1.0);

    effective *= calculate_exalt_4_effective_singularity_multiplier(
        &CalculateExalt4EffectiveSingularityMultiplierInput {
            comps: input.no_octeracts_completions,
            force: false,
            in_exalt_4: input.in_exalt_4,
        },
    );

    if singularity_count > 10.0 {
        effective *= 1.5;
        effective *= 4.0_f64.min((1.25 * singularity_count) / 10.0 - 0.25);
    }
    if singularity_count > 25.0 {
        effective *= 2.5;
        effective *= 6.0_f64.min((1.5 * singularity_count) / 25.0 - 0.5);
    }
    if singularity_count > 36.0 {
        effective *= 4.0;
        effective *= 5.0_f64.min(singularity_count / 18.0 - 1.0);
        effective *= 1.1_f64.powf((singularity_count - 36.0).min(64.0));
    }
    if singularity_count > 50.0 {
        effective *= 5.0;
        effective *= 8.0_f64.min((2.0 * singularity_count) / 50.0 - 1.0);
        effective *= 1.1_f64.powf((singularity_count - 50.0).min(50.0));
    }
    if singularity_count > 100.0 {
        effective *= 2.0;
        effective *= singularity_count / 25.0;
        effective *= 1.1_f64.powf(singularity_count - 100.0);
    }
    if singularity_count > 150.0 {
        effective *= 2.0;
        effective *= 1.05_f64.powf(singularity_count - 150.0);
    }
    if singularity_count > 200.0 {
        effective *= 1.5;
        effective *= 1.275_f64.powf(singularity_count - 200.0);
    }
    if singularity_count > 215.0 {
        effective *= 1.25;
        effective *= 1.2_f64.powf(singularity_count - 215.0);
    }
    if singularity_count > 230.0 {
        effective *= 2.0;
    }
    if singularity_count > 269.0 {
        effective *= 3.0;
        effective *= 3.0_f64.powf(singularity_count - 269.0);
    }

    if input.taxman_last_stand_enabled
        && input.taxman_last_stand_completions >= 8.0
        && input.platonic_upgrade_15 == 0.0
    {
        effective = effective.powf(3.0 / 2.0);
    }

    effective
}

/// Inputs to [`calculate_singularity_debuff`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateSingularityDebuffInput {
    /// Which system the debuff applies to.
    pub debuff: SingularityDebuff,
    /// `player.singularityCount` — the raw count.
    pub singularity_count: f64,
    /// `runes.antiquities.level > 0` — when `true`, all penalties
    /// drop to their no-penalty value (`0` for Salvage / Ant ELO,
    /// `1` otherwise).
    pub antiquities_rune_active: bool,
    /// Sum of
    /// `shopSingularityPenaltyDebuff.singularityPenaltyReducers`
    /// and the appropriate ambrosia singularity-reduction (`1`
    /// outside sing challenges, `2` inside). Subtracted from
    /// `singularity_count`.
    pub singularity_reductions: f64,
    /// `getShopUpgradeEffects('shopHorseShoe', 'singularityPenaltyMult')`
    /// — applied to all multiplicative branches as
    /// `base_debuff_multiplier`. Not applied to Salvage / Ant ELO.
    pub horse_shoe_mult: f64,
    /// Pass-through to [`calculate_effective_singularities`].
    pub no_octeracts_completions: f64,
    /// Pass-through to [`calculate_effective_singularities`].
    pub in_exalt_4: bool,
    /// Pass-through to [`calculate_effective_singularities`].
    pub taxman_last_stand_enabled: bool,
    /// Pass-through to [`calculate_effective_singularities`].
    pub taxman_last_stand_completions: f64,
    /// Pass-through to [`calculate_effective_singularities`].
    pub platonic_upgrade_15: f64,
}

/// Per-system singularity penalty.
///
/// Returns `0` (Salvage / Ant ELO) or `1` (everything else) when:
/// - `singularity_count == 0`, OR
/// - the antiquities rune is active, OR
/// - constitutive count (`singularity_count - reductions`) is below
///   `1`.
///
/// Otherwise switches on `debuff` to pick a formula. Salvage and
/// Ant ELO return subtractive amounts (negative of the magnitude);
/// the rest return multiplicative penalties.
///
/// # ⚠️ Redesign pending — DO NOT extend
///
/// **Assayer audit BLOCKER 2**: the `antiquities_rune_active` check
/// at the top of the function is a Sirlin-degenerate binary override
/// — there is no skill gradient between "no antiquities rune" and
/// "antiquities rune"; the choice geometry collapses to a single
/// boolean. Entangled with the staircase in
/// [`calculate_effective_singularities`] (audit BLOCKER 1).
///
/// Per the audit plan (Tier C item 16), further singularity-layer
/// porting is paused pending a Loom design pass. Parity on this
/// mechanic is **informational, not blocking** (Tier C item 15).
#[must_use]
pub fn calculate_singularity_debuff(input: &CalculateSingularityDebuffInput) -> f64 {
    if input.singularity_count == 0.0 || input.antiquities_rune_active {
        return if matches!(
            input.debuff,
            SingularityDebuff::Salvage | SingularityDebuff::AntELO
        ) {
            0.0
        } else {
            1.0
        };
    }

    let constitutive = input.singularity_count - input.singularity_reductions;
    if constitutive < 1.0 {
        return 1.0;
    }

    let effective = calculate_effective_singularities(&CalculateEffectiveSingularitiesInput {
        singularity_count: constitutive,
        no_octeracts_completions: input.no_octeracts_completions,
        in_exalt_4: input.in_exalt_4,
        taxman_last_stand_enabled: input.taxman_last_stand_enabled,
        taxman_last_stand_completions: input.taxman_last_stand_completions,
        platonic_upgrade_15: input.platonic_upgrade_15,
    });

    let base_mult = input.horse_shoe_mult;

    match input.debuff {
        SingularityDebuff::Offering | SingularityDebuff::Obtainium => {
            let extra_mult = 1.02_f64.powf(constitutive);
            let core = if constitutive < 150.0 {
                3.0 * (effective.sqrt() + 1.0)
            } else {
                effective.powf(2.0 / 3.0) / 400.0
            };
            extra_mult * base_mult * core
        }
        SingularityDebuff::Salvage => {
            -(4.0 * constitutive
                + 4.0 * (constitutive - 100.0).max(0.0)
                + 4.0 * (constitutive - 200.0).max(0.0)
                + 3.0 * (constitutive - 250.0).max(0.0)
                + 3.0 * (constitutive - 270.0).max(0.0)
                + 2.0 * (constitutive - 280.0).max(0.0))
        }
        SingularityDebuff::AntELO => -(1.0_f64.min(0.001 * constitutive)),
        SingularityDebuff::GlobalSpeed => base_mult * (1.0 + effective.sqrt() / 4.0),
        SingularityDebuff::Researches => base_mult * (1.0 + effective.sqrt() / 2.0),
        SingularityDebuff::AscensionSpeed => {
            base_mult
                * if constitutive < 150.0 {
                    1.0 + effective.sqrt() / 5.0
                } else {
                    1.0 + effective.powf(0.75) / 10_000.0
                }
        }
        SingularityDebuff::Cubes => {
            let extra_mult = if constitutive > 100.0 {
                2.0 * 1.03_f64.powf(constitutive - 100.0)
            } else {
                2.0
            };
            base_mult
                * if constitutive < 150.0 {
                    3.0 * (1.0 + (effective.sqrt() * extra_mult) / 4.0)
                } else {
                    1.0 + (effective.powf(0.75) * extra_mult) / 1_000.0
                }
        }
        SingularityDebuff::PlatonicCosts => {
            base_mult
                * if constitutive > 36.0 {
                    1.0 + effective.powf(3.0 / 10.0) / 12.0
                } else {
                    1.0
                }
        }
        SingularityDebuff::HepteractCosts => {
            base_mult
                * if constitutive > 50.0 {
                    1.0 + effective.powf(11.0 / 50.0) / 25.0
                } else {
                    1.0
                }
        }
        SingularityDebuff::CubeUpgrades => base_mult * (effective + 1.0).cbrt(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_singularities_below_first_step_uses_floor_mult() {
        // sing=5: 5 * min(4.75, 0.375 + 1) * 1 = 5 * 1.375 = 6.875
        let result = calculate_effective_singularities(&CalculateEffectiveSingularitiesInput {
            singularity_count: 5.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        assert!((result - 6.875).abs() < 1e-9);
    }

    #[test]
    fn effective_singularities_taxman_3_2_override() {
        // sing=5 → base=6.875; with override → 6.875^1.5 ≈ 18.05
        let result = calculate_effective_singularities(&CalculateEffectiveSingularitiesInput {
            singularity_count: 5.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: true,
            taxman_last_stand_completions: 8.0,
            platonic_upgrade_15: 0.0,
        });
        assert!((result - 6.875_f64.powf(1.5)).abs() < 1e-9);
    }

    #[test]
    fn singularity_debuff_zero_sing_offering_is_one() {
        let result = calculate_singularity_debuff(&CalculateSingularityDebuffInput {
            debuff: SingularityDebuff::Offering,
            singularity_count: 0.0,
            antiquities_rune_active: false,
            singularity_reductions: 0.0,
            horse_shoe_mult: 1.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn singularity_debuff_antiquities_zeros_salvage_and_antelo() {
        let salvage = calculate_singularity_debuff(&CalculateSingularityDebuffInput {
            debuff: SingularityDebuff::Salvage,
            singularity_count: 100.0,
            antiquities_rune_active: true,
            singularity_reductions: 0.0,
            horse_shoe_mult: 1.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        let ant_elo = calculate_singularity_debuff(&CalculateSingularityDebuffInput {
            debuff: SingularityDebuff::AntELO,
            singularity_count: 100.0,
            antiquities_rune_active: true,
            singularity_reductions: 0.0,
            horse_shoe_mult: 1.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        assert_eq!(salvage, 0.0);
        assert_eq!(ant_elo, 0.0);
    }

    #[test]
    fn singularity_debuff_offering_uses_extra_mult_below_150() {
        // sing=5, reductions=0 → constitutive=5; below 150
        // effective at 5 = 6.875
        // extra = 1.02^5 ≈ 1.10408
        // core = 3 * (sqrt(6.875) + 1) ≈ 3 * 3.622 = 10.866
        // total ≈ 1.10408 * 1.0 * 10.866 ≈ 11.998
        let result = calculate_singularity_debuff(&CalculateSingularityDebuffInput {
            debuff: SingularityDebuff::Offering,
            singularity_count: 5.0,
            antiquities_rune_active: false,
            singularity_reductions: 0.0,
            horse_shoe_mult: 1.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        let expected = 1.02_f64.powf(5.0) * 1.0 * 3.0 * (6.875_f64.sqrt() + 1.0);
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn singularity_debuff_salvage_is_subtractive() {
        let result = calculate_singularity_debuff(&CalculateSingularityDebuffInput {
            debuff: SingularityDebuff::Salvage,
            singularity_count: 50.0,
            antiquities_rune_active: false,
            singularity_reductions: 0.0,
            horse_shoe_mult: 1.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        // constitutive=50 → -(4*50 + 0 + 0 + 0 + 0 + 0) = -200
        assert_eq!(result, -200.0);
    }

    #[test]
    fn singularity_debuff_ant_elo_caps_at_neg_1() {
        let result = calculate_singularity_debuff(&CalculateSingularityDebuffInput {
            debuff: SingularityDebuff::AntELO,
            singularity_count: 10_000.0,
            antiquities_rune_active: false,
            singularity_reductions: 0.0,
            horse_shoe_mult: 1.0,
            no_octeracts_completions: 0.0,
            in_exalt_4: false,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            platonic_upgrade_15: 0.0,
        });
        // -min(1, 0.001 * 10000) = -min(1, 10) = -1
        assert_eq!(result, -1.0);
    }
}
