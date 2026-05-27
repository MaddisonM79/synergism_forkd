//! Per-singularity-challenge effect / requirement / AP-value
//! formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/singularityChallenges.ts`.
//! The UI tier still owns the `singularityChallengeData` table.
//! This module owns the three pure-formula fields each challenge
//! has: `singularity_requirement(base_req, completions)`,
//! `achievement_point_value(n)`, and `effect(n, key)`.
//!
//! Effect functions return either booleans (for unlock keys) or
//! `f64` scalars. To keep the dispatch clean while preserving
//! variable return types, each challenge gets its own key enum and
//! its own tagged-result enum (with an `Unlock(bool)` and
//! `Scalar(f64)` variant). The caller pattern-matches the result.

// ─── Per-challenge singularityRequirement formulas ────────────────────────

/// `noSingularityUpgrades` — `+16/completion`, `+8` bonus past 9.
#[must_use]
pub fn no_singularity_upgrades_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    base_req + 16.0 * completions + 8.0 * if completions >= 9.0 { 1.0 } else { 0.0 }
}

/// `oneChallengeCap` — `+19/completion`, `-2` discount past 14.
#[must_use]
pub fn one_challenge_cap_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    base_req + 19.0 * completions - 2.0 * if completions >= 14.0 { 1.0 } else { 0.0 }
}

/// `noOcteracts` — `+13/comp` below 10, prefix + `+10/comp` past.
#[must_use]
pub fn no_octeracts_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    if completions < 10.0 {
        base_req + 13.0 * completions
    } else {
        base_req + 13.0 * 9.0 + 10.0 * (completions - 9.0)
    }
}

/// `limitedAscensions` — linear `+27/comp`.
#[must_use]
pub fn limited_ascensions_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    base_req + 27.0 * completions
}

/// `noAmbrosiaUpgrades` — `+12/comp` below 10, prefix + `+4/comp`
/// past.
#[must_use]
pub fn no_ambrosia_upgrades_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    if completions < 10.0 {
        base_req + 12.0 * completions
    } else {
        base_req + 12.0 * 9.0 + 4.0 * (completions - 9.0)
    }
}

/// `noQuarkUpgrades` — three-band piecewise. The (`completions - 6`)
/// offsets are verbatim from legacy (yes, the middle band
/// references a higher knee than its own band — pinned by parity
/// tests).
#[must_use]
pub fn no_quark_upgrades_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    if completions > 5.0 {
        base_req + 185.0 + 8.0 * (completions - 6.0)
    } else if completions > 2.0 {
        base_req + 70.0 + 9.0 * (completions - 6.0)
    } else {
        base_req + 15.0 * completions
    }
}

/// `limitedTime` — `+8/comp` below 10, hard `277 + 2*(comp - 10)`
/// past.
#[must_use]
pub fn limited_time_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    if completions > 9.0 {
        277.0 + 2.0 * (completions - 10.0)
    } else {
        base_req + 8.0 * completions
    }
}

/// `sadisticPrequel` — linear `+8/comp`.
#[must_use]
pub fn sadistic_prequel_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    base_req + 8.0 * completions
}

/// `taxmanLastStand` — linear `+4/comp`.
#[must_use]
pub fn taxman_last_stand_singularity_requirement(base_req: f64, completions: f64) -> f64 {
    base_req + 4.0 * completions
}

// ─── Per-challenge achievementPointValue formulas ─────────────────────────

/// `noSingularityUpgrades` AP value: `15n`.
#[must_use]
pub fn no_singularity_upgrades_achievement_point_value(n: f64) -> f64 {
    15.0 * n
}

/// `oneChallengeCap` AP value: `15n`.
#[must_use]
pub fn one_challenge_cap_achievement_point_value(n: f64) -> f64 {
    15.0 * n
}

/// `noOcteracts` AP value: `20n`.
#[must_use]
pub fn no_octeracts_achievement_point_value(n: f64) -> f64 {
    20.0 * n
}

/// `limitedAscensions` AP value: `30n`.
#[must_use]
pub fn limited_ascensions_achievement_point_value(n: f64) -> f64 {
    30.0 * n
}

/// `noAmbrosiaUpgrades` AP value: `25n`.
#[must_use]
pub fn no_ambrosia_upgrades_achievement_point_value(n: f64) -> f64 {
    25.0 * n
}

/// `noQuarkUpgrades` AP value: `20n`.
#[must_use]
pub fn no_quark_upgrades_achievement_point_value(n: f64) -> f64 {
    20.0 * n
}

/// `limitedTime` AP value: `30n`.
#[must_use]
pub fn limited_time_achievement_point_value(n: f64) -> f64 {
    30.0 * n
}

/// `sadisticPrequel` AP value: `40n`.
#[must_use]
pub fn sadistic_prequel_achievement_point_value(n: f64) -> f64 {
    40.0 * n
}

/// `taxmanLastStand` AP value: `50n`.
#[must_use]
pub fn taxman_last_stand_achievement_point_value(n: f64) -> f64 {
    50.0 * n
}

// ─── Tagged result type ───────────────────────────────────────────────────

/// Tagged result for the effect dispatchers. Each challenge mixes
/// `bool` unlock flags with `f64` scalars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SingularityEffectValue {
    /// Unlock flag.
    Unlock(bool),
    /// Scalar value.
    Scalar(f64),
}

/// Convert a `bool` into the unit scalar legacy uses (`+n` coerces
/// true → 1, false → 0).
#[inline]
fn b2f(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

// ─── Per-challenge effect functions ───────────────────────────────────────

/// `noSingularityUpgrades` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoSingularityUpgradesKey {
    /// `cubes` — `1 + n`.
    Cubes,
    /// `goldenQuarks` — `1 + 0.12 × (n > 0)`.
    GoldenQuarks,
    /// `blueberries` — `n > 0` coerced to 0/1.
    Blueberries,
    /// `shopUpgrade` — `n >= 10`.
    ShopUpgrade,
    /// `additiveLuckMult` — `0.05` once `n >= 15`, else `0`.
    AdditiveLuckMult,
    /// `shopUpgrade2` — `n >= 15`.
    ShopUpgrade2,
}

/// `noSingularityUpgrades` effect.
#[must_use]
pub fn no_singularity_upgrades_effect(
    n: f64,
    key: NoSingularityUpgradesKey,
) -> SingularityEffectValue {
    match key {
        NoSingularityUpgradesKey::Cubes => SingularityEffectValue::Scalar(1.0 + n),
        NoSingularityUpgradesKey::GoldenQuarks => {
            SingularityEffectValue::Scalar(1.0 + 0.12 * b2f(n > 0.0))
        }
        NoSingularityUpgradesKey::Blueberries => SingularityEffectValue::Scalar(b2f(n > 0.0)),
        NoSingularityUpgradesKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 10.0),
        NoSingularityUpgradesKey::AdditiveLuckMult => {
            SingularityEffectValue::Scalar(if n >= 15.0 { 0.05 } else { 0.0 })
        }
        NoSingularityUpgradesKey::ShopUpgrade2 => SingularityEffectValue::Unlock(n >= 15.0),
    }
}

/// `oneChallengeCap` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneChallengeCapKey {
    /// `corrScoreIncrease` — `0.05n`.
    CorrScoreIncrease,
    /// `blueberrySpeedMult` — `1 + n/60`.
    BlueberrySpeedMult,
    /// `capIncrease` — `3 × (n > 0)`.
    CapIncrease,
    /// `freeCorruptionLevel` — `(n >= 12) ? 1 : 0`.
    FreeCorruptionLevel,
    /// `shopUpgrade` — `n >= 12`.
    ShopUpgrade,
    /// `reinCapIncrease2` — `7 × (n >= 15)`.
    ReinCapIncrease2,
    /// `ascCapIncrease2` — `2 × (n >= 15)`.
    AscCapIncrease2,
}

/// `oneChallengeCap` effect.
#[must_use]
pub fn one_challenge_cap_effect(n: f64, key: OneChallengeCapKey) -> SingularityEffectValue {
    match key {
        OneChallengeCapKey::CorrScoreIncrease => SingularityEffectValue::Scalar(0.05 * n),
        OneChallengeCapKey::BlueberrySpeedMult => SingularityEffectValue::Scalar(1.0 + n / 60.0),
        OneChallengeCapKey::CapIncrease => SingularityEffectValue::Scalar(3.0 * b2f(n > 0.0)),
        OneChallengeCapKey::FreeCorruptionLevel => SingularityEffectValue::Scalar(b2f(n >= 12.0)),
        OneChallengeCapKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 12.0),
        OneChallengeCapKey::ReinCapIncrease2 => {
            SingularityEffectValue::Scalar(7.0 * b2f(n >= 15.0))
        }
        OneChallengeCapKey::AscCapIncrease2 => SingularityEffectValue::Scalar(2.0 * b2f(n >= 15.0)),
    }
}

/// `noOcteracts` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoOcteractsKey {
    /// `octeractPow` — piecewise `0.02n` / `0.2 + (n-10)/100`.
    OcteractPow,
    /// `offeringBonus` — `n > 0`.
    OfferingBonus,
    /// `obtainiumBonus` — `n >= 10`.
    ObtainiumBonus,
    /// `shopUpgrade` — `n >= 10`.
    ShopUpgrade,
}

/// `noOcteracts` effect.
#[must_use]
pub fn no_octeracts_effect(n: f64, key: NoOcteractsKey) -> SingularityEffectValue {
    match key {
        NoOcteractsKey::OcteractPow => {
            let val = if n <= 10.0 {
                0.02 * n
            } else {
                0.2 + (n - 10.0) / 100.0
            };
            SingularityEffectValue::Scalar(val)
        }
        NoOcteractsKey::OfferingBonus => SingularityEffectValue::Unlock(n > 0.0),
        NoOcteractsKey::ObtainiumBonus => SingularityEffectValue::Unlock(n >= 10.0),
        NoOcteractsKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 10.0),
    }
}

/// `limitedAscensions` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitedAscensionsKey {
    /// `ascensionSpeedMult` — `1 + 0.25n/100`.
    AscensionSpeedMult,
    /// `hepteractCap` — `n > 0`.
    HepteractCap,
    /// `shopUpgrade` — `n >= 8`.
    ShopUpgrade,
    /// `shopUpgrade2` — `n >= 10`.
    ShopUpgrade2,
}

/// `limitedAscensions` effect.
#[must_use]
pub fn limited_ascensions_effect(n: f64, key: LimitedAscensionsKey) -> SingularityEffectValue {
    match key {
        LimitedAscensionsKey::AscensionSpeedMult => {
            SingularityEffectValue::Scalar(1.0 + 0.25 * n / 100.0)
        }
        LimitedAscensionsKey::HepteractCap => SingularityEffectValue::Unlock(n > 0.0),
        LimitedAscensionsKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 8.0),
        LimitedAscensionsKey::ShopUpgrade2 => SingularityEffectValue::Unlock(n >= 10.0),
    }
}

/// `noAmbrosiaUpgrades` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoAmbrosiaUpgradesKey {
    /// `bonusAmbrosia` — `n > 0` (0/1).
    BonusAmbrosia,
    /// `blueberries` — `floor(n/5) + (n > 0)` (stair-steps every 5).
    Blueberries,
    /// `additiveLuckMult` — `n / 200`.
    AdditiveLuckMult,
    /// `ambrosiaLuck` — `20n`.
    AmbrosiaLuck,
    /// `redLuck` — `4n`.
    RedLuck,
    /// `blueberrySpeedMult` — `1 + n/25`.
    BlueberrySpeedMult,
    /// `redSpeedMult` — `1 + 2n/100`.
    RedSpeedMult,
    /// `shopUpgrade` — `n >= 8`.
    ShopUpgrade,
    /// `shopUpgrade2` — `n >= 10`.
    ShopUpgrade2,
}

/// `noAmbrosiaUpgrades` effect.
#[must_use]
pub fn no_ambrosia_upgrades_effect(n: f64, key: NoAmbrosiaUpgradesKey) -> SingularityEffectValue {
    match key {
        NoAmbrosiaUpgradesKey::BonusAmbrosia => SingularityEffectValue::Scalar(b2f(n > 0.0)),
        NoAmbrosiaUpgradesKey::Blueberries => {
            SingularityEffectValue::Scalar((n / 5.0).floor() + b2f(n > 0.0))
        }
        NoAmbrosiaUpgradesKey::AdditiveLuckMult => SingularityEffectValue::Scalar(n / 200.0),
        NoAmbrosiaUpgradesKey::AmbrosiaLuck => SingularityEffectValue::Scalar(20.0 * n),
        NoAmbrosiaUpgradesKey::RedLuck => SingularityEffectValue::Scalar(4.0 * n),
        NoAmbrosiaUpgradesKey::BlueberrySpeedMult => SingularityEffectValue::Scalar(1.0 + n / 25.0),
        NoAmbrosiaUpgradesKey::RedSpeedMult => {
            SingularityEffectValue::Scalar(1.0 + 2.0 * n / 100.0)
        }
        NoAmbrosiaUpgradesKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 8.0),
        NoAmbrosiaUpgradesKey::ShopUpgrade2 => SingularityEffectValue::Unlock(n >= 10.0),
    }
}

/// `noQuarkUpgrades` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoQuarkUpgradesKey {
    /// `freeObtainiumLevels` — `n`.
    FreeObtainiumLevels,
    /// `freeOfferingLevels` — `n`.
    FreeOfferingLevels,
    /// `freeSpeedLevels` — `n`.
    FreeSpeedLevels,
    /// `freeCubeLevels` — `n`.
    FreeCubeLevels,
    /// `freeQuarkLevel` — `1` if `n >= 5`, else `0`.
    FreeQuarkLevel,
    /// `freeInfinityLevels` — `n`.
    FreeInfinityLevels,
    /// `shopUpgrade` — `n >= 1`.
    ShopUpgrade,
    /// `topHatUnlock` — `n >= 10`.
    TopHatUnlock,
}

/// `noQuarkUpgrades` effect.
#[must_use]
pub fn no_quark_upgrades_effect(n: f64, key: NoQuarkUpgradesKey) -> SingularityEffectValue {
    match key {
        NoQuarkUpgradesKey::FreeObtainiumLevels
        | NoQuarkUpgradesKey::FreeOfferingLevels
        | NoQuarkUpgradesKey::FreeSpeedLevels
        | NoQuarkUpgradesKey::FreeCubeLevels
        | NoQuarkUpgradesKey::FreeInfinityLevels => SingularityEffectValue::Scalar(n),
        NoQuarkUpgradesKey::FreeQuarkLevel => {
            SingularityEffectValue::Scalar(if n >= 5.0 { 1.0 } else { 0.0 })
        }
        NoQuarkUpgradesKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 1.0),
        NoQuarkUpgradesKey::TopHatUnlock => SingularityEffectValue::Unlock(n >= 10.0),
    }
}

/// `limitedTime` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitedTimeKey {
    /// `preserveQuarks` — `n > 0` (0/1).
    PreserveQuarks,
    /// `quarkMult` — `1 + 0.02n`.
    QuarkMult,
    /// `globalSpeed` — `1 + 0.12n`.
    GlobalSpeed,
    /// `ascensionSpeed` — `1 + 0.12n`.
    AscensionSpeed,
    /// `barRequirementMultiplier` — `1 - 0.02n`.
    BarRequirementMultiplier,
    /// `shopUpgrade` — `n >= 5`.
    ShopUpgrade,
    /// `shopUpgrade2` — `n >= 10`.
    ShopUpgrade2,
}

/// `limitedTime` effect.
#[must_use]
pub fn limited_time_effect(n: f64, key: LimitedTimeKey) -> SingularityEffectValue {
    match key {
        LimitedTimeKey::PreserveQuarks => SingularityEffectValue::Scalar(b2f(n > 0.0)),
        LimitedTimeKey::QuarkMult => SingularityEffectValue::Scalar(1.0 + 0.02 * n),
        LimitedTimeKey::GlobalSpeed | LimitedTimeKey::AscensionSpeed => {
            SingularityEffectValue::Scalar(1.0 + 0.12 * n)
        }
        LimitedTimeKey::BarRequirementMultiplier => SingularityEffectValue::Scalar(1.0 - 0.02 * n),
        LimitedTimeKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 5.0),
        LimitedTimeKey::ShopUpgrade2 => SingularityEffectValue::Unlock(n >= 10.0),
    }
}

/// `sadisticPrequel` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SadisticPrequelKey {
    /// `extraFree` — `50 × (n > 0)`.
    ExtraFree,
    /// `quarkMult` — `1 + 0.06n`.
    QuarkMult,
    /// `freeUpgradeMult` — `1 + 0.06n`.
    FreeUpgradeMult,
    /// `shopUpgrade` — `n >= 5`.
    ShopUpgrade,
    /// `shopUpgrade2` — `n >= 10`.
    ShopUpgrade2,
    /// `shopUpgrade3` — `n >= 15`.
    ShopUpgrade3,
}

/// `sadisticPrequel` effect.
#[must_use]
pub fn sadistic_prequel_effect(n: f64, key: SadisticPrequelKey) -> SingularityEffectValue {
    match key {
        SadisticPrequelKey::ExtraFree => SingularityEffectValue::Scalar(50.0 * b2f(n > 0.0)),
        SadisticPrequelKey::QuarkMult | SadisticPrequelKey::FreeUpgradeMult => {
            SingularityEffectValue::Scalar(1.0 + 0.06 * n)
        }
        SadisticPrequelKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 5.0),
        SadisticPrequelKey::ShopUpgrade2 => SingularityEffectValue::Unlock(n >= 10.0),
        SadisticPrequelKey::ShopUpgrade3 => SingularityEffectValue::Unlock(n >= 15.0),
    }
}

/// `taxmanLastStand` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaxmanLastStandKey {
    /// `horseShoeUnlock` — `n > 0`.
    HorseShoeUnlock,
    /// `shopUpgrade` — `n >= 5`.
    ShopUpgrade,
    /// `talismanUnlock` — `n >= 10`.
    TalismanUnlock,
    /// `talismanFreeLevel` — `25n`.
    TalismanFreeLevel,
    /// `talismanRuneEffect` — `0.03n`.
    TalismanRuneEffect,
    /// `antiquityOOM` — `(1/50) × n / 10`.
    AntiquityOOM,
    /// `horseShoeOOM` — `(1/20) × n / 10`.
    HorseShoeOOM,
}

/// `taxmanLastStand` effect.
#[must_use]
pub fn taxman_last_stand_effect(n: f64, key: TaxmanLastStandKey) -> SingularityEffectValue {
    match key {
        TaxmanLastStandKey::HorseShoeUnlock => SingularityEffectValue::Unlock(n > 0.0),
        TaxmanLastStandKey::ShopUpgrade => SingularityEffectValue::Unlock(n >= 5.0),
        TaxmanLastStandKey::TalismanUnlock => SingularityEffectValue::Unlock(n >= 10.0),
        TaxmanLastStandKey::TalismanFreeLevel => SingularityEffectValue::Scalar(25.0 * n),
        TaxmanLastStandKey::TalismanRuneEffect => SingularityEffectValue::Scalar(0.03 * n),
        TaxmanLastStandKey::AntiquityOOM => SingularityEffectValue::Scalar((1.0 / 50.0) * n / 10.0),
        TaxmanLastStandKey::HorseShoeOOM => SingularityEffectValue::Scalar((1.0 / 20.0) * n / 10.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_singularity_upgrades_requirement_below_knee() {
        // base=100, comps=5 → 100 + 80 + 0 = 180
        assert_eq!(
            no_singularity_upgrades_singularity_requirement(100.0, 5.0),
            180.0
        );
    }

    #[test]
    fn no_singularity_upgrades_requirement_at_knee() {
        // base=100, comps=9 → 100 + 144 + 8 = 252
        assert_eq!(
            no_singularity_upgrades_singularity_requirement(100.0, 9.0),
            252.0
        );
    }

    #[test]
    fn no_octeracts_requirement_two_band() {
        // comps=15 → base + 13*9 + 10*6 = base + 117 + 60 = base + 177
        let result = no_octeracts_singularity_requirement(100.0, 15.0);
        assert_eq!(result, 277.0);
    }

    #[test]
    fn limited_time_caps_at_277_past_9() {
        // comps=10 → 277 + 0 = 277
        // comps=15 → 277 + 10 = 287
        assert_eq!(limited_time_singularity_requirement(0.0, 10.0), 277.0);
        assert_eq!(limited_time_singularity_requirement(0.0, 15.0), 287.0);
    }

    #[test]
    fn no_quark_upgrades_three_band_piecewise() {
        // comps=1 → base + 15 (band 0..=2)
        assert_eq!(no_quark_upgrades_singularity_requirement(100.0, 1.0), 115.0);
        // comps=4 → base + 70 + 9*-2 = 100 + 70 - 18 = 152 (band 3..=5)
        assert_eq!(no_quark_upgrades_singularity_requirement(100.0, 4.0), 152.0);
        // comps=10 → base + 185 + 8*4 = 100 + 185 + 32 = 317 (band 6+)
        assert_eq!(
            no_quark_upgrades_singularity_requirement(100.0, 10.0),
            317.0
        );
    }

    #[test]
    fn achievement_point_values_are_linear() {
        assert_eq!(no_singularity_upgrades_achievement_point_value(2.0), 30.0);
        assert_eq!(taxman_last_stand_achievement_point_value(3.0), 150.0);
        assert_eq!(sadistic_prequel_achievement_point_value(1.0), 40.0);
    }

    #[test]
    fn no_singularity_upgrades_cubes_effect() {
        let v = no_singularity_upgrades_effect(5.0, NoSingularityUpgradesKey::Cubes);
        assert!(matches!(v, SingularityEffectValue::Scalar(s) if (s - 6.0).abs() < 1e-12));
    }

    #[test]
    fn no_octeracts_octeract_pow_kicks_in_past_10() {
        let below = no_octeracts_effect(10.0, NoOcteractsKey::OcteractPow);
        // 0.02 * 10 = 0.2 (uses ≤ branch)
        assert!(matches!(below, SingularityEffectValue::Scalar(s) if (s - 0.2).abs() < 1e-12));
        let above = no_octeracts_effect(15.0, NoOcteractsKey::OcteractPow);
        // 0.2 + 5/100 = 0.25
        assert!(matches!(above, SingularityEffectValue::Scalar(s) if (s - 0.25).abs() < 1e-12));
    }

    #[test]
    fn no_ambrosia_upgrades_blueberries_stair_step() {
        // n=4 → floor(4/5) + 1 = 1
        // n=5 → floor(5/5) + 1 = 2
        let four = no_ambrosia_upgrades_effect(4.0, NoAmbrosiaUpgradesKey::Blueberries);
        let five = no_ambrosia_upgrades_effect(5.0, NoAmbrosiaUpgradesKey::Blueberries);
        assert!(matches!(four, SingularityEffectValue::Scalar(s) if (s - 1.0).abs() < 1e-12));
        assert!(matches!(five, SingularityEffectValue::Scalar(s) if (s - 2.0).abs() < 1e-12));
    }

    #[test]
    fn taxman_last_stand_talisman_unlock_at_10() {
        let below = taxman_last_stand_effect(9.0, TaxmanLastStandKey::TalismanUnlock);
        let at = taxman_last_stand_effect(10.0, TaxmanLastStandKey::TalismanUnlock);
        assert!(matches!(below, SingularityEffectValue::Unlock(false)));
        assert!(matches!(at, SingularityEffectValue::Unlock(true)));
    }
}
