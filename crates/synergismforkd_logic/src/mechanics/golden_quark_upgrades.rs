//! Per-upgrade effect formulas for golden-quark (singularity)
//! upgrades.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/goldenQuarkUpgrades.ts`.
//! The UI tier owns the `goldenQuarkUpgrades` data table. Cost lives
//! in [`crate::mechanics::gq_upgrade_cost`]. This module owns the
//! per-upgrade `effect(n, [key])` field for all ~80 upgrades.
//!
//! Five effects read state outside the logic tier:
//! - `singOcteractPatreonBonus`: `getQuarkBonus()`
//! - `divinePack`: `player.corruptions.used.loadout`
//! - `platonicDelta` / `platonicPhi`: `player.singularityCounter` +
//!   `shopSingularitySpeedup` effect
//! - `favoriteUpgrade`: 9 sibling upgrades' levels/maxLevels
//!
//! Each takes the player-derived value(s) as extra parameter(s).

// ─── Helper: boolean unlock ───────────────────────────────────────────────

#[inline]
fn bool_unlock(n: f64) -> bool {
    n > 0.0
}

// ─── Basic GQ upgrade effects ─────────────────────────────────────────────

/// `goldenQuarks1`: `1 + 0.1n`.
#[must_use]
pub fn golden_quarks_1_effect(n: f64) -> f64 {
    1.0 + 0.1 * n
}

/// `goldenQuarks2`: piecewise cost reduction. Gentle linear below 250
/// (capped at 50%), then `1 / log2(n / 62.5)` decays slowly past.
#[must_use]
pub fn golden_quarks_2_effect(n: f64) -> f64 {
    if n > 250.0 {
        1.0 / (n / 62.5).log2()
    } else {
        1.0 - 0.5_f64.min(n / 500.0)
    }
}

/// `goldenQuarks3`: triangular numbers `n × (n + 1) / 2`.
#[must_use]
pub fn golden_quarks_3_effect(n: f64) -> f64 {
    n * (n + 1.0) / 2.0
}

/// `starterPack` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarterPackKey {
    /// `obtainiumMult` — `1 + 5n`.
    ObtainiumMult,
    /// `offeringMult` — `1 + 5n`.
    OfferingMult,
    /// `cubeMult` — `1 + 4n`.
    CubeMult,
}

/// `starterPack`.
#[must_use]
pub fn starter_pack_effect(n: f64, key: StarterPackKey) -> f64 {
    match key {
        StarterPackKey::ObtainiumMult | StarterPackKey::OfferingMult => 1.0 + 5.0 * n,
        StarterPackKey::CubeMult => 1.0 + 4.0 * n,
    }
}

// ─── Cookies family + simple boolean unlocks ──────────────────────────────

/// `cookies`: unlock at `n > 0`.
#[must_use]
pub fn cookies_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `cookies2`: unlock at `n > 0`.
#[must_use]
pub fn cookies_2_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `cookies3`: unlock at `n > 0`.
#[must_use]
pub fn cookies_3_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `cookies4`: unlock at `n > 0`.
#[must_use]
pub fn cookies_4_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `cookies5`: unlock at `n > 0`.
#[must_use]
pub fn cookies_5_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `ascensions`: `(1 + 2n/100) × (1 + floor(n/10)/100)`.
#[must_use]
pub fn ascensions_effect(n: f64) -> f64 {
    (1.0 + (2.0 * n) / 100.0) * (1.0 + (n / 10.0).floor() / 100.0)
}

/// `corruptionFourteen`: unlock at `n > 0`.
#[must_use]
pub fn corruption_fourteen_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `corruptionFifteen`: identity `n`.
#[must_use]
pub fn corruption_fifteen_effect(n: f64) -> f64 {
    n
}

// ─── Offering / obtainium / cubes 1-3 ─────────────────────────────────────

/// `singOfferings1`: `1 + 0.02n`.
#[must_use]
pub fn sing_offerings_1_effect(n: f64) -> f64 {
    1.0 + 0.02 * n
}

/// `singOfferings2`: `1 + 0.08n`.
#[must_use]
pub fn sing_offerings_2_effect(n: f64) -> f64 {
    1.0 + 0.08 * n
}

/// `singOfferings3`: `1 + 0.04n`.
#[must_use]
pub fn sing_offerings_3_effect(n: f64) -> f64 {
    1.0 + 0.04 * n
}

/// `singObtainium1`: `1 + 0.02n`.
#[must_use]
pub fn sing_obtainium_1_effect(n: f64) -> f64 {
    1.0 + 0.02 * n
}

/// `singObtainium2`: `1 + 0.08n`.
#[must_use]
pub fn sing_obtainium_2_effect(n: f64) -> f64 {
    1.0 + 0.08 * n
}

/// `singObtainium3`: `1 + 0.04n`.
#[must_use]
pub fn sing_obtainium_3_effect(n: f64) -> f64 {
    1.0 + 0.04 * n
}

/// `singCubes1`: `1 + 0.006n`.
#[must_use]
pub fn sing_cubes_1_effect(n: f64) -> f64 {
    1.0 + 0.006 * n
}

/// `singCubes2`: `1 + 0.08n`.
#[must_use]
pub fn sing_cubes_2_effect(n: f64) -> f64 {
    1.0 + 0.08 * n
}

/// `singCubes3`: `1 + 0.04n`.
#[must_use]
pub fn sing_cubes_3_effect(n: f64) -> f64 {
    1.0 + 0.04 * n
}

// ─── Citadel ──────────────────────────────────────────────────────────────

#[inline]
fn citadel_mult(n: f64) -> f64 {
    (1.0 + 0.02 * n) * (1.0 + (n / 10.0).floor() / 100.0)
}

/// `singCitadel`: same value across the three reward keys.
#[must_use]
pub fn sing_citadel_effect(n: f64) -> f64 {
    citadel_mult(n)
}

/// `singCitadel2` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingCitadel2Key {
    /// `offeringMult` / `obtainiumMult` / `cubeMult` — citadel mult.
    Mult,
    /// `citadel1FreeLevels` — identity `n`.
    Citadel1FreeLevels,
}

/// `singCitadel2`.
#[must_use]
pub fn sing_citadel_2_effect(n: f64, key: SingCitadel2Key) -> f64 {
    match key {
        SingCitadel2Key::Mult => citadel_mult(n),
        SingCitadel2Key::Citadel1FreeLevels => n,
    }
}

// ─── Octeract / patreon ───────────────────────────────────────────────────

/// `octeractUnlock`: unlock at `n > 0`.
#[must_use]
pub fn octeract_unlock_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `singOcteractPatreonBonus`: gated boolean that scales with the
/// live quark bonus once unlocked. `quark_bonus` is in percent
/// (0..100), matching the legacy `getQuarkBonus()` return.
#[must_use]
pub fn sing_octeract_patreon_bonus_effect(n: f64, quark_bonus: f64) -> f64 {
    if n > 0.0 {
        1.0 + quark_bonus / 100.0
    } else {
        1.0
    }
}

/// `offeringAutomatic`: unlock at `n > 0`.
#[must_use]
pub fn offering_automatic_effect(n: f64) -> bool {
    bool_unlock(n)
}

// ─── Packs ────────────────────────────────────────────────────────────────

/// `intermediatePack` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntermediatePackKey {
    /// `globalSpeedMult` — `2` if `n > 0`, else `1`.
    GlobalSpeedMult,
    /// `ascensionSpeedMult` — `1.5` if `n > 0`, else `1`.
    AscensionSpeedMult,
    /// `packQuarkAdd` — `0.02` if `n > 0`, else `0`.
    PackQuarkAdd,
}

/// `intermediatePack`.
#[must_use]
pub fn intermediate_pack_effect(n: f64, key: IntermediatePackKey) -> f64 {
    let on = n > 0.0;
    match key {
        IntermediatePackKey::GlobalSpeedMult => {
            if on {
                2.0
            } else {
                1.0
            }
        }
        IntermediatePackKey::AscensionSpeedMult => {
            if on {
                1.5
            } else {
                1.0
            }
        }
        IntermediatePackKey::PackQuarkAdd => {
            if on {
                0.02
            } else {
                0.0
            }
        }
    }
}

/// `advancedPack` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancedPackKey {
    /// `packQuarkAdd` — `0.04` if `n > 0`, else `0`.
    PackQuarkAdd,
    /// `corruptionScoreIncrease` — `0.33` if `n > 0`, else `0`.
    CorruptionScoreIncrease,
}

/// `advancedPack`.
#[must_use]
pub fn advanced_pack_effect(n: f64, key: AdvancedPackKey) -> f64 {
    let on = n > 0.0;
    match key {
        AdvancedPackKey::PackQuarkAdd => {
            if on {
                0.04
            } else {
                0.0
            }
        }
        AdvancedPackKey::CorruptionScoreIncrease => {
            if on {
                0.33
            } else {
                0.0
            }
        }
    }
}

/// `expertPack` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpertPackKey {
    /// `packQuarkAdd` — `0.06` if `n > 0`, else `0`.
    PackQuarkAdd,
    /// `ascensionScoreMult` — `1.5` if `n > 0`, else `1`.
    AscensionScoreMult,
    /// `addCodeAscensionTimeMult` — `1.2` if `n > 0`, else `1`.
    AddCodeAscensionTimeMult,
}

/// `expertPack`.
#[must_use]
pub fn expert_pack_effect(n: f64, key: ExpertPackKey) -> f64 {
    let on = n > 0.0;
    match key {
        ExpertPackKey::PackQuarkAdd => {
            if on {
                0.06
            } else {
                0.0
            }
        }
        ExpertPackKey::AscensionScoreMult => {
            if on {
                1.5
            } else {
                1.0
            }
        }
        ExpertPackKey::AddCodeAscensionTimeMult => {
            if on {
                1.2
            } else {
                1.0
            }
        }
    }
}

/// `masterPack` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterPackKey {
    /// `packQuarkAdd` — `0.08` if `n > 0`, else `0`.
    PackQuarkAdd,
    /// `ascensionScoreMult` — `2` if `n > 0`, else `1`.
    AscensionScoreMult,
}

/// `masterPack`.
#[must_use]
pub fn master_pack_effect(n: f64, key: MasterPackKey) -> f64 {
    let on = n > 0.0;
    match key {
        MasterPackKey::PackQuarkAdd => {
            if on {
                0.08
            } else {
                0.0
            }
        }
        MasterPackKey::AscensionScoreMult => {
            if on {
                2.0
            } else {
                1.0
            }
        }
    }
}

/// `divinePack` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivinePackKey {
    /// `packQuarkAdd` — `0.1` if `n > 0`, else `0`.
    PackQuarkAdd,
    /// `octeractMult` — scales with corruption loadout
    /// (14 → ×1.25, 15 → ×1.3, 16 → ×1.4, else ×1).
    OcteractMult,
}

/// `divinePack`. `corruption_loadout` is
/// `player.corruptions.used.loadout` — slice of corruption type ids.
#[must_use]
pub fn divine_pack_effect(n: f64, key: DivinePackKey, corruption_loadout: &[u32]) -> f64 {
    match key {
        DivinePackKey::PackQuarkAdd => {
            if n > 0.0 {
                0.1
            } else {
                0.0
            }
        }
        DivinePackKey::OcteractMult => {
            if n == 0.0 {
                return 1.0;
            }
            corruption_loadout.iter().fold(1.0_f64, |acc, &curr| {
                let per = match curr {
                    16 => 1.4,
                    15 => 1.3,
                    14 => 1.25,
                    _ => 1.0,
                };
                acc * per
            })
        }
    }
}

// ─── WoW passes ───────────────────────────────────────────────────────────

/// `wowPass`: unlock at `n > 0`.
#[must_use]
pub fn wow_pass_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `wowPass2`: unlock at `n > 0`.
#[must_use]
pub fn wow_pass_2_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `wowPass3`: unlock at `n > 0`.
#[must_use]
pub fn wow_pass_3_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `wowPass4`: unlock at `n > 0`.
#[must_use]
pub fn wow_pass_4_effect(n: f64) -> bool {
    bool_unlock(n)
}

// ─── Potion buffs (with explicit floors) ──────────────────────────────────

/// `potionBuff`: `max(1, 10 × n²)`.
#[must_use]
pub fn potion_buff_effect(n: f64) -> f64 {
    1.0_f64.max(10.0 * n.powi(2))
}

/// `potionBuff2`: `max(1, 2n)`.
#[must_use]
pub fn potion_buff_2_effect(n: f64) -> f64 {
    1.0_f64.max(2.0 * n)
}

/// `potionBuff3`: `max(1, 1 + 0.5n)`.
#[must_use]
pub fn potion_buff_3_effect(n: f64) -> f64 {
    1.0_f64.max(1.0 + 0.5 * n)
}

// ─── Challenge extensions (same shape ×3) ─────────────────────────────────

/// Challenge-extension shared key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingChallengeExtensionKey {
    /// `reincarnationCapIncrease` — `2n`.
    ReincarnationCapIncrease,
    /// `ascensionCapIncrease` — `n`.
    AscensionCapIncrease,
}

fn sing_challenge_extension_body(n: f64, key: SingChallengeExtensionKey) -> f64 {
    match key {
        SingChallengeExtensionKey::ReincarnationCapIncrease => 2.0 * n,
        SingChallengeExtensionKey::AscensionCapIncrease => n,
    }
}

/// `singChallengeExtension`.
#[must_use]
pub fn sing_challenge_extension_effect(n: f64, key: SingChallengeExtensionKey) -> f64 {
    sing_challenge_extension_body(n, key)
}

/// `singChallengeExtension2`.
#[must_use]
pub fn sing_challenge_extension_2_effect(n: f64, key: SingChallengeExtensionKey) -> f64 {
    sing_challenge_extension_body(n, key)
}

/// `singChallengeExtension3`.
#[must_use]
pub fn sing_challenge_extension_3_effect(n: f64, key: SingChallengeExtensionKey) -> f64 {
    sing_challenge_extension_body(n, key)
}

// ─── Quark / hepteract / octeract gain ────────────────────────────────────

/// `singQuarkImprover1`: `1 + n / 200`.
#[must_use]
pub fn sing_quark_improver_1_effect(n: f64) -> f64 {
    1.0 + n / 200.0
}

/// `singQuarkHepteract`: `n / 100`.
#[must_use]
pub fn sing_quark_hepteract_effect(n: f64) -> f64 {
    n / 100.0
}

/// `singQuarkHepteract2`: `n / 100`.
#[must_use]
pub fn sing_quark_hepteract_2_effect(n: f64) -> f64 {
    n / 100.0
}

/// `singQuarkHepteract3`: `n / 200` (bigger denominator).
#[must_use]
pub fn sing_quark_hepteract_3_effect(n: f64) -> f64 {
    n / 200.0
}

/// `singOcteractGain`: `1 + 0.0125n`.
#[must_use]
pub fn sing_octeract_gain_effect(n: f64) -> f64 {
    1.0 + 0.012_5 * n
}

/// `singOcteractGain2`: `1 + 0.05n`.
#[must_use]
pub fn sing_octeract_gain_2_effect(n: f64) -> f64 {
    1.0 + 0.05 * n
}

/// `singOcteractGain3`: `1 + 0.025n`.
#[must_use]
pub fn sing_octeract_gain_3_effect(n: f64) -> f64 {
    1.0 + 0.025 * n
}

/// `singOcteractGain4`: `1 + 0.02n`.
#[must_use]
pub fn sing_octeract_gain_4_effect(n: f64) -> f64 {
    1.0 + 0.02 * n
}

/// `singOcteractGain5`: `1 + 0.01n`.
#[must_use]
pub fn sing_octeract_gain_5_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

// ─── Platonic family ──────────────────────────────────────────────────────

/// `platonicTau` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatonicTauKey {
    /// `unlocked` — `n > 0`.
    Unlocked,
    /// `tauPower` — `1.01` if `n > 0`, else `1`.
    TauPower,
}

/// `platonicTau` tagged result.
#[derive(Debug, Clone, Copy)]
pub enum PlatonicTauValue {
    /// Unlock flag.
    Unlock(bool),
    /// Tau power scalar.
    Scalar(f64),
}

/// `platonicTau`.
#[must_use]
pub fn platonic_tau_effect(n: f64, key: PlatonicTauKey) -> PlatonicTauValue {
    match key {
        PlatonicTauKey::Unlocked => PlatonicTauValue::Unlock(n > 0.0),
        PlatonicTauKey::TauPower => PlatonicTauValue::Scalar(if n > 0.0 { 1.01 } else { 1.0 }),
    }
}

/// `platonicAlpha`: unlock at `n > 0`.
#[must_use]
pub fn platonic_alpha_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `platonicDelta`: cube mult scales with
/// `min((singularityCounter + 1) × speedMult / (3600*24), 9)` once
/// unlocked. The `3600*24` divisor converts seconds → days.
#[must_use]
pub fn platonic_delta_effect(
    n: f64,
    singularity_counter: f64,
    singularity_upgrade_speed_mult: f64,
) -> f64 {
    if n <= 0.0 {
        return 1.0;
    }
    1.0 + 9.0_f64.min((singularity_counter + 1.0) * singularity_upgrade_speed_mult / 86_400.0)
}

/// `platonicPhi`: daily codes scale with
/// `floor(5 × min(singularityCounter × speedMult / (3600*24), 10))`.
/// No `+1` offset (differs from `platonicDelta`).
#[must_use]
pub fn platonic_phi_effect(
    n: f64,
    singularity_counter: f64,
    singularity_upgrade_speed_mult: f64,
) -> f64 {
    if n <= 0.0 {
        return 0.0;
    }
    (5.0 * 10.0_f64.min(singularity_counter * singularity_upgrade_speed_mult / 86_400.0)).floor()
}

// ─── Fast-forward / ascension speed / mind ────────────────────────────────

/// `singFastForward`: identity.
#[must_use]
pub fn sing_fast_forward_effect(n: f64) -> f64 {
    n
}

/// `singFastForward2`: identity.
#[must_use]
pub fn sing_fast_forward_2_effect(n: f64) -> f64 {
    n
}

/// `singAscensionSpeed`: `0.03` if `n > 0`, else `0`.
#[must_use]
pub fn sing_ascension_speed_effect(n: f64) -> f64 {
    if n > 0.0 {
        0.03
    } else {
        0.0
    }
}

/// `singAscensionSpeed2`: `0.001n`.
#[must_use]
pub fn sing_ascension_speed_2_effect(n: f64) -> f64 {
    0.001 * n
}

/// `ultimatePen`: platonic-powers unlock at `n > 0`.
#[must_use]
pub fn ultimate_pen_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `halfMind`: unlock at `n > 0`.
#[must_use]
pub fn half_mind_effect(n: f64) -> bool {
    bool_unlock(n)
}

/// `oneMind`: unlock at `n > 0`.
#[must_use]
pub fn one_mind_effect(n: f64) -> bool {
    bool_unlock(n)
}

// ─── Blueberries / ambrosia ───────────────────────────────────────────────

/// `blueberries`: identity.
#[must_use]
pub fn blueberries_effect(n: f64) -> f64 {
    n
}

/// `singAmbrosiaLuck`: `4n`.
#[must_use]
pub fn sing_ambrosia_luck_effect(n: f64) -> f64 {
    4.0 * n
}

/// `singAmbrosiaLuck2`: `2n`.
#[must_use]
pub fn sing_ambrosia_luck_2_effect(n: f64) -> f64 {
    2.0 * n
}

/// `singAmbrosiaLuck3`: `3n`.
#[must_use]
pub fn sing_ambrosia_luck_3_effect(n: f64) -> f64 {
    3.0 * n
}

/// `singAmbrosiaLuck4`: `5n`.
#[must_use]
pub fn sing_ambrosia_luck_4_effect(n: f64) -> f64 {
    5.0 * n
}

/// `singAmbrosiaGeneration`: `1 + n / 100`.
#[must_use]
pub fn sing_ambrosia_generation_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `singAmbrosiaGeneration2`: `1 + n / 100`.
#[must_use]
pub fn sing_ambrosia_generation_2_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `singAmbrosiaGeneration3`: `1 + n / 100`.
#[must_use]
pub fn sing_ambrosia_generation_3_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `singAmbrosiaGeneration4`: `1 + 2n / 100`.
#[must_use]
pub fn sing_ambrosia_generation_4_effect(n: f64) -> f64 {
    1.0 + (2.0 * n) / 100.0
}

// ─── Bonus tokens ─────────────────────────────────────────────────────────

/// `singBonusTokens1`: identity.
#[must_use]
pub fn sing_bonus_tokens_1_effect(n: f64) -> f64 {
    n
}

/// `singBonusTokens2`: `1 + n / 100`.
#[must_use]
pub fn sing_bonus_tokens_2_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `singBonusTokens3`: `2n`.
#[must_use]
pub fn sing_bonus_tokens_3_effect(n: f64) -> f64 {
    2.0 * n
}

/// `singBonusTokens4`: `5n`.
#[must_use]
pub fn sing_bonus_tokens_4_effect(n: f64) -> f64 {
    5.0 * n
}

// ─── Misc late-game ───────────────────────────────────────────────────────

/// `singInfiniteShopUpgrades`: identity (infinity vouchers count).
#[must_use]
pub fn sing_infinite_shop_upgrades_effect(n: f64) -> f64 {
    n
}

/// `singTalismanBonusRunes1`: `n / 100`.
#[must_use]
pub fn sing_talisman_bonus_runes_1_effect(n: f64) -> f64 {
    n / 100.0
}

/// `singTalismanBonusRunes2`: `n / 100`.
#[must_use]
pub fn sing_talisman_bonus_runes_2_effect(n: f64) -> f64 {
    n / 100.0
}

/// `singTalismanBonusRunes3`: `n / 100`.
#[must_use]
pub fn sing_talisman_bonus_runes_3_effect(n: f64) -> f64 {
    n / 100.0
}

/// `singTalismanBonusRunes4`: `n / 100`.
#[must_use]
pub fn sing_talisman_bonus_runes_4_effect(n: f64) -> f64 {
    n / 100.0
}

/// `favoriteUpgrade`: quark mult scales with the count of 9 specific
/// sibling upgrades that have hit their max level. Caller pre-
/// computes the count (`0..=9`).
#[must_use]
pub fn favorite_upgrade_effect(n: f64, sum_of_maxed_sibling_upgrades: f64) -> f64 {
    1.0 + n / 5_000.0 * (sum_of_maxed_sibling_upgrades + 6.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_quarks_1_linear() {
        assert_eq!(golden_quarks_1_effect(0.0), 1.0);
        assert_eq!(golden_quarks_1_effect(10.0), 2.0);
    }

    #[test]
    fn golden_quarks_2_below_250_linear_capped_at_50pct() {
        // n=100 → 1 - min(0.5, 100/500) = 1 - 0.2 = 0.8
        assert_eq!(golden_quarks_2_effect(100.0), 0.8);
        // n=250 → 1 - 0.5 = 0.5
        assert_eq!(golden_quarks_2_effect(250.0), 0.5);
    }

    #[test]
    fn golden_quarks_2_above_250_log_branch() {
        // n=500 → 1 / log2(500/62.5) = 1 / log2(8) = 1/3
        let result = golden_quarks_2_effect(500.0);
        assert!((result - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn golden_quarks_3_triangular() {
        // 5 → 5*6/2 = 15
        assert_eq!(golden_quarks_3_effect(5.0), 15.0);
    }

    #[test]
    fn starter_pack_cube_uses_4x() {
        assert_eq!(starter_pack_effect(2.0, StarterPackKey::CubeMult), 9.0);
        assert_eq!(
            starter_pack_effect(2.0, StarterPackKey::ObtainiumMult),
            11.0
        );
    }

    #[test]
    fn ascensions_compounds_milestone() {
        // n=10 → (1 + 0.2) * (1 + 0.01) = 1.2 * 1.01 = 1.212
        let result = ascensions_effect(10.0);
        assert!((result - 1.212).abs() < 1e-12);
    }

    #[test]
    fn citadel_mult_stair_steps() {
        // n=0 → 1*1 = 1
        // n=10 → 1.2 * 1.01 = 1.212
        assert_eq!(sing_citadel_effect(0.0), 1.0);
        assert!((sing_citadel_effect(10.0) - 1.212).abs() < 1e-12);
    }

    #[test]
    fn intermediate_pack_global_speed_doubles_when_unlocked() {
        let off = intermediate_pack_effect(0.0, IntermediatePackKey::GlobalSpeedMult);
        let on = intermediate_pack_effect(1.0, IntermediatePackKey::GlobalSpeedMult);
        assert_eq!(off, 1.0);
        assert_eq!(on, 2.0);
    }

    #[test]
    fn divine_pack_octeract_mult_corruption_loadout() {
        // n=1, loadout=[14, 15, 16, 0, 0] → 1.25 * 1.3 * 1.4 * 1 * 1 = 2.275
        let result = divine_pack_effect(1.0, DivinePackKey::OcteractMult, &[14, 15, 16, 0, 0]);
        assert!((result - 2.275).abs() < 1e-9);
    }

    #[test]
    fn divine_pack_octeract_mult_zero_at_n_zero() {
        let result = divine_pack_effect(0.0, DivinePackKey::OcteractMult, &[14, 15, 16]);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn potion_buff_floors_at_1() {
        assert_eq!(potion_buff_effect(0.0), 1.0);
        // n=1 → 10, but with floor it's 10
        assert_eq!(potion_buff_effect(1.0), 10.0);
    }

    #[test]
    fn platonic_delta_capped_at_9_plus_1() {
        // singularity=1e6 days → very large → cap at 9 → 1 + 9 = 10
        let result = platonic_delta_effect(1.0, 1e10, 1.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn platonic_delta_locked_returns_1() {
        let result = platonic_delta_effect(0.0, 1e10, 1.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn platonic_phi_locked_returns_0() {
        assert_eq!(platonic_phi_effect(0.0, 1e10, 1.0), 0.0);
    }

    #[test]
    fn platonic_phi_capped_at_50() {
        // singularity → cap at 10 → floor(5*10) = 50
        let result = platonic_phi_effect(1.0, 1e10, 1.0);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn favorite_upgrade_baseline_six() {
        // n=10, sum=0 → 1 + 10/5000 * 6 = 1 + 0.012 = 1.012
        let result = favorite_upgrade_effect(10.0, 0.0);
        assert!((result - 1.012).abs() < 1e-12);
    }

    #[test]
    fn favorite_upgrade_with_all_siblings() {
        // n=10, sum=9 → 1 + 10/5000 * 15 = 1 + 0.03 = 1.03
        let result = favorite_upgrade_effect(10.0, 9.0);
        assert!((result - 1.03).abs() < 1e-12);
    }

    #[test]
    fn sing_octeract_patreon_bonus_applies_quark_bonus_when_unlocked() {
        let unlocked = sing_octeract_patreon_bonus_effect(1.0, 50.0);
        assert_eq!(unlocked, 1.5);
        let locked = sing_octeract_patreon_bonus_effect(0.0, 50.0);
        assert_eq!(locked, 1.0);
    }
}
