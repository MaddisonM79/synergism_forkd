//! Per-upgrade effect formulas for shop upgrades.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/shopUpgrades.ts`.
//! The UI tier owns the `shopUpgrades` data table (i18n,
//! DOM-driven renderer, buy flow, price/maxLevel/type fields). The
//! cost progression already lives in `shop_costs.rs`. This module
//! owns the `effects(n, [key], ...)` field per upgrade for all 83
//! upgrades.
//!
//! Sixteen effects read state outside the logic tier. Each takes
//! the player-derived value as an extra parameter; the UI
//! data-table closure forwards it.

// ─── Constants / shared sub-formulas ──────────────────────────────────────

/// `offeringEX` / `obtainiumEX` share a `+6%/level` multiplier with
/// a stair-step `×1.08` every 10 levels.
fn ex_mult(n: f64) -> f64 {
    (1.0 + 0.06 * n) * 1.08_f64.powf((n / 10.0).floor())
}

/// `cubeToQuark` / `tesseractToQuark` / `hypercubeToQuark` share
/// the same piecewise: `1` below `n = 1`, then `1.5 + 0.5 × (1 -
/// 0.9^(n - 1))` once unlocked.
fn cube_quark_conversion(n: f64) -> f64 {
    if n >= 1.0 {
        1.5 + 0.5 * (1.0 - 0.9_f64.powf(n - 1.0))
    } else {
        1.0
    }
}

// Multi-key dispatchers are defined alongside their effect
// functions below — each variant maps to a reward key in the legacy
// `QuarkShopUpgradeRewards` type.

// ─── Effect functions (alphabetical-ish by family) ────────────────────────

/// `offeringPotion`: skip seconds — fixed 7200 (level ignored).
#[must_use]
pub fn offering_potion_effect(_n: f64) -> f64 {
    7_200.0
}

/// `obtainiumPotion`: skip seconds — fixed 7200 (level ignored).
#[must_use]
pub fn obtainium_potion_effect(_n: f64) -> f64 {
    7_200.0
}

/// `offeringEX`: shared EX multiplier.
#[must_use]
pub fn offering_ex_effect(n: f64) -> f64 {
    ex_mult(n)
}

/// `offeringEX2`: scales with singularity count.
#[must_use]
pub fn offering_ex_2_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + 0.01 * n * singularity_count
}

/// `offeringEX3` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfferingEX3Key {
    /// `offeringMult` — `1.012^n`.
    OfferingMult,
    /// `baseOfferings` — `floor(n / 25)`.
    BaseOfferings,
}

/// `offeringEX3`.
#[must_use]
pub fn offering_ex_3_effect(n: f64, key: OfferingEX3Key) -> f64 {
    match key {
        OfferingEX3Key::OfferingMult => 1.012_f64.powf(n),
        OfferingEX3Key::BaseOfferings => (n / 25.0).floor(),
    }
}

/// `obtainiumEX`: shared EX multiplier.
#[must_use]
pub fn obtainium_ex_effect(n: f64) -> f64 {
    ex_mult(n)
}

/// `obtainiumEX2`: scales with singularity count.
#[must_use]
pub fn obtainium_ex_2_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + 0.01 * n * singularity_count
}

/// `obtainiumEX3` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObtainiumEX3Key {
    /// `obtainiumMult` — `1.012^n`.
    ObtainiumMult,
    /// `immaculateObtainiuMult` — `1.06^floor(n/25)`.
    ImmaculateObtainiuMult,
}

/// `obtainiumEX3`.
#[must_use]
pub fn obtainium_ex_3_effect(n: f64, key: ObtainiumEX3Key) -> f64 {
    match key {
        ObtainiumEX3Key::ObtainiumMult => 1.012_f64.powf(n),
        ObtainiumEX3Key::ImmaculateObtainiuMult => 1.06_f64.powf((n / 25.0).floor()),
    }
}

/// `offeringAuto` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfferingAutoKey {
    /// `autoRune` unlock flag.
    AutoRune,
    /// `autoRuneSpeedMult` — `1 + 0.01n`.
    AutoRuneSpeedMult,
}

/// `offeringAuto` returns a tagged result.
#[derive(Debug, Clone, Copy)]
pub enum OfferingAutoValue {
    /// Unlock flag for `autoRune`.
    Unlock(bool),
    /// Speed mult.
    Mult(f64),
}

/// `offeringAuto`.
#[must_use]
pub fn offering_auto_effect(n: f64, key: OfferingAutoKey) -> OfferingAutoValue {
    match key {
        OfferingAutoKey::AutoRune => OfferingAutoValue::Unlock(n > 0.0),
        OfferingAutoKey::AutoRuneSpeedMult => OfferingAutoValue::Mult(1.0 + 0.01 * n),
    }
}

/// `obtainiumAuto` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObtainiumAutoKey {
    /// `autoResearch` unlock flag.
    AutoResearch,
    /// `researchCostMult` — `1 - 0.001n`.
    ResearchCostMult,
}

/// Tagged result for [`obtainium_auto_effect`].
#[derive(Debug, Clone, Copy)]
pub enum ObtainiumAutoValue {
    /// Unlock flag.
    Unlock(bool),
    /// Cost mult.
    Mult(f64),
}

/// `obtainiumAuto`.
#[must_use]
pub fn obtainium_auto_effect(n: f64, key: ObtainiumAutoKey) -> ObtainiumAutoValue {
    match key {
        ObtainiumAutoKey::AutoResearch => ObtainiumAutoValue::Unlock(n > 0.0),
        ObtainiumAutoKey::ResearchCostMult => ObtainiumAutoValue::Mult(1.0 - 0.001 * n),
    }
}

/// `cashGrab`: `1 + 0.01n` (both keys share value).
#[must_use]
pub fn cash_grab_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `cashGrab2`: `1 + 0.005n` (both keys share value).
#[must_use]
pub fn cash_grab_2_effect(n: f64) -> f64 {
    1.0 + 0.005 * n
}

/// `shopTalisman`: PCoin unlock 1 OR `n > 0`.
#[must_use]
pub fn shop_talisman_effect(n: f64, pcoin_instant_unlock_1: bool) -> bool {
    n > 0.0 || pcoin_instant_unlock_1
}

/// `infiniteAscent`: PCoin unlock 2 OR `n > 0`.
#[must_use]
pub fn infinite_ascent_effect(n: f64, pcoin_instant_unlock_2: bool) -> bool {
    n > 0.0 || pcoin_instant_unlock_2
}

/// `shopSadisticRune`: rune unlock at `n > 0`.
#[must_use]
pub fn shop_sadistic_rune_effect(n: f64) -> bool {
    n > 0.0
}

/// `antSpeed`: `4n`.
#[must_use]
pub fn ant_speed_effect(n: f64) -> f64 {
    4.0 * n
}

/// `instantChallenge` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstantChallengeKey {
    /// `unlocked` (`n > 0`).
    Unlocked,
    /// `extraCompPerTick` — `10n`.
    ExtraCompPerTick,
}

/// `instantChallenge` tagged result.
#[derive(Debug, Clone, Copy)]
pub enum InstantChallengeValue {
    /// Unlock flag.
    Unlock(bool),
    /// Per-tick comps.
    Scalar(f64),
}

/// `instantChallenge`.
#[must_use]
pub fn instant_challenge_effect(n: f64, key: InstantChallengeKey) -> InstantChallengeValue {
    match key {
        InstantChallengeKey::Unlocked => InstantChallengeValue::Unlock(n > 0.0),
        InstantChallengeKey::ExtraCompPerTick => InstantChallengeValue::Scalar(10.0 * n),
    }
}

/// `instantChallenge2` — same keys as `instantChallenge` but
/// `extraCompPerTick` scales with `highestSingularityCount`.
#[must_use]
pub fn instant_challenge_2_effect(
    n: f64,
    key: InstantChallengeKey,
    highest_singularity_count: f64,
) -> InstantChallengeValue {
    match key {
        InstantChallengeKey::Unlocked => InstantChallengeValue::Unlock(n > 0.0),
        InstantChallengeKey::ExtraCompPerTick => {
            InstantChallengeValue::Scalar(n * highest_singularity_count)
        }
    }
}

/// `challengeExtension`: `2n`.
#[must_use]
pub fn challenge_extension_effect(n: f64) -> f64 {
    2.0 * n
}

/// `challengeTome` / `challengeTome2` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeTomeKey {
    /// `c10RequirementReduction` — `2e7 × n`.
    C10RequirementReduction,
    /// `c9c10ScalingReduction` — `-n / 100`.
    C9C10ScalingReduction,
}

fn challenge_tome_body(n: f64, key: ChallengeTomeKey) -> f64 {
    match key {
        ChallengeTomeKey::C10RequirementReduction => 2e7 * n,
        ChallengeTomeKey::C9C10ScalingReduction => -n / 100.0,
    }
}

/// `challengeTome`.
#[must_use]
pub fn challenge_tome_effect(n: f64, key: ChallengeTomeKey) -> f64 {
    challenge_tome_body(n, key)
}

/// `challengeTome2`.
#[must_use]
pub fn challenge_tome_2_effect(n: f64, key: ChallengeTomeKey) -> f64 {
    challenge_tome_body(n, key)
}

/// `challenge15Auto`: `n > 0`.
#[must_use]
pub fn challenge_15_auto_effect(n: f64) -> bool {
    n > 0.0
}

/// `seasonPass`: `1 + 0.0225n`.
#[must_use]
pub fn season_pass_effect(n: f64) -> f64 {
    1.0 + 0.022_5 * n
}

/// `seasonPass2`: `1 + 0.015n`.
#[must_use]
pub fn season_pass_2_effect(n: f64) -> f64 {
    1.0 + 0.015 * n
}

/// `seasonPass3`: `1 + 0.015n`.
#[must_use]
pub fn season_pass_3_effect(n: f64) -> f64 {
    1.0 + 0.015 * n
}

/// `seasonPassY`: `1 + 0.0075n`.
#[must_use]
pub fn season_pass_y_effect(n: f64) -> f64 {
    1.0 + 0.007_5 * n
}

/// `seasonPassZ`: scales with singularity count.
#[must_use]
pub fn season_pass_z_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + 0.01 * n * singularity_count
}

/// `seasonPassLost`: `1 + 0.001n`.
#[must_use]
pub fn season_pass_lost_effect(n: f64) -> f64 {
    1.0 + 0.001 * n
}

/// `seasonPassInfinity` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonPassInfinityKey {
    /// `globalCubeMult` — `1.012^n`.
    GlobalCubeMult,
    /// `wowOcteractMult` — `1.012^(n × 1.25)`.
    WowOcteractMult,
}

/// `seasonPassInfinity`.
#[must_use]
pub fn season_pass_infinity_effect(n: f64, key: SeasonPassInfinityKey) -> f64 {
    match key {
        SeasonPassInfinityKey::GlobalCubeMult => 1.012_f64.powf(n),
        SeasonPassInfinityKey::WowOcteractMult => 1.012_f64.powf(n * 1.25),
    }
}

// ─── Calculator family ────────────────────────────────────────────────────

/// `calculator` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculatorKey {
    /// `autoAnswer` — `n > 0`.
    AutoAnswer,
    /// `addQuarkMult` — `1 + 0.14n`.
    AddQuarkMult,
    /// `autoFill` — `n == 5`.
    AutoFill,
}

/// Tagged result for the calculator family — booleans for unlock
/// keys, scalars otherwise.
#[derive(Debug, Clone, Copy)]
pub enum CalculatorValue {
    /// Unlock flag.
    Unlock(bool),
    /// Scalar value.
    Scalar(f64),
}

/// `calculator`.
#[must_use]
pub fn calculator_effect(n: f64, key: CalculatorKey) -> CalculatorValue {
    match key {
        CalculatorKey::AutoAnswer => CalculatorValue::Unlock(n > 0.0),
        CalculatorKey::AddQuarkMult => CalculatorValue::Scalar(1.0 + 0.14 * n),
        CalculatorKey::AutoFill => CalculatorValue::Unlock((n - 5.0).abs() < f64::EPSILON),
    }
}

/// `calculator2` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calculator2Key {
    /// `addCodeCapacity` — `2n`.
    AddCodeCapacity,
    /// `addQuarkMult` — `1.25` at `n == 12`, else `1`.
    AddQuarkMult,
}

/// `calculator2`.
#[must_use]
pub fn calculator_2_effect(n: f64, key: Calculator2Key) -> f64 {
    match key {
        Calculator2Key::AddCodeCapacity => 2.0 * n,
        Calculator2Key::AddQuarkMult => {
            if (n - 12.0).abs() < f64::EPSILON {
                1.25
            } else {
                1.0
            }
        }
    }
}

/// `calculator3` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calculator3Key {
    /// `addRewardVarianceMultiplier` — `1 - n/10`.
    AddRewardVarianceMultiplier,
    /// `ascensionTimerAdd` — `60n`.
    AscensionTimerAdd,
}

/// `calculator3`.
#[must_use]
pub fn calculator_3_effect(n: f64, key: Calculator3Key) -> f64 {
    match key {
        Calculator3Key::AddRewardVarianceMultiplier => 1.0 - n / 10.0,
        Calculator3Key::AscensionTimerAdd => 60.0 * n,
    }
}

/// `calculator4` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calculator4Key {
    /// `addCodeIntervalMult` — `1 - n/25`.
    AddCodeIntervalMult,
    /// `addCodeCapacity` — `32` at `n == 10`, else `0`.
    AddCodeCapacity,
}

/// `calculator4`.
#[must_use]
pub fn calculator_4_effect(n: f64, key: Calculator4Key) -> f64 {
    match key {
        Calculator4Key::AddCodeIntervalMult => 1.0 - n / 25.0,
        Calculator4Key::AddCodeCapacity => {
            if (n - 10.0).abs() < f64::EPSILON {
                32.0
            } else {
                0.0
            }
        }
    }
}

/// `calculator5` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calculator5Key {
    /// `importGQTimerAdd` — `6n`.
    ImportGQTimerAdd,
    /// `addCodeCapacity` — `floor(n / 10) + 6 if n == 100 else 0`.
    AddCodeCapacity,
}

/// `calculator5`.
#[must_use]
pub fn calculator_5_effect(n: f64, key: Calculator5Key) -> f64 {
    match key {
        Calculator5Key::ImportGQTimerAdd => 6.0 * n,
        Calculator5Key::AddCodeCapacity => {
            let bump = if (n - 100.0).abs() < f64::EPSILON {
                6.0
            } else {
                0.0
            };
            (n / 10.0).floor() + bump
        }
    }
}

/// `calculator6` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calculator6Key {
    /// `octeractTimerAdd` — `n`.
    OcteractTimerAdd,
    /// `addCodeCapacity` — `24` at `n == 100`, else `0`.
    AddCodeCapacity,
}

/// `calculator6`.
#[must_use]
pub fn calculator_6_effect(n: f64, key: Calculator6Key) -> f64 {
    match key {
        Calculator6Key::OcteractTimerAdd => n,
        Calculator6Key::AddCodeCapacity => {
            if (n - 100.0).abs() < f64::EPSILON {
                24.0
            } else {
                0.0
            }
        }
    }
}

/// `calculator7` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calculator7Key {
    /// `blueberryTimerAdd` — `n`.
    BlueberryTimerAdd,
    /// `addCodeCapacity` — `48` at `n == 50`, else `0`.
    AddCodeCapacity,
}

/// `calculator7`.
#[must_use]
pub fn calculator_7_effect(n: f64, key: Calculator7Key) -> f64 {
    match key {
        Calculator7Key::BlueberryTimerAdd => n,
        Calculator7Key::AddCodeCapacity => {
            if (n - 50.0).abs() < f64::EPSILON {
                48.0
            } else {
                0.0
            }
        }
    }
}

// ─── Chronometer family ───────────────────────────────────────────────────

/// `chronometer`: `1 + 0.012n`.
#[must_use]
pub fn chronometer_effect(n: f64) -> f64 {
    1.0 + 0.012 * n
}

/// `chronometer2`: `1 + 0.006n`.
#[must_use]
pub fn chronometer_2_effect(n: f64) -> f64 {
    1.0 + 0.006 * n
}

/// `chronometer3`: `1 + 0.015n`.
#[must_use]
pub fn chronometer_3_effect(n: f64) -> f64 {
    1.0 + 0.015 * n
}

/// `chronometerZ`: scales with singularity count.
#[must_use]
pub fn chronometer_z_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + 0.001 * n * singularity_count
}

/// `shopChronometerS`: `1.01^(n × max(0, singularityCount - 200))`.
/// Same value for both reward keys (ascensionSpeedMult,
/// globalSpeedMult).
#[must_use]
pub fn shop_chronometer_s_effect(n: f64, singularity_count: f64) -> f64 {
    1.01_f64.powf(n * 0.0_f64.max(singularity_count - 200.0))
}

/// `chronometerInfinity` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChronometerInfinityKey {
    /// `ascensionSpeedMult` — `1.006^n`.
    AscensionSpeedMult,
    /// `exponentSpread` — `0.001 × floor(n / 40)`.
    ExponentSpread,
}

/// `chronometerInfinity`.
#[must_use]
pub fn chronometer_infinity_effect(n: f64, key: ChronometerInfinityKey) -> f64 {
    match key {
        ChronometerInfinityKey::AscensionSpeedMult => 1.006_f64.powf(n),
        ChronometerInfinityKey::ExponentSpread => 0.001 * (n / 40.0).floor(),
    }
}

// ─── Improved quark hept family ───────────────────────────────────────────

/// `improveQuarkHept`: `0.01n`.
#[must_use]
pub fn improve_quark_hept_effect(n: f64) -> f64 {
    0.01 * n
}

/// `improveQuarkHept2`: `0.01n`.
#[must_use]
pub fn improve_quark_hept_2_effect(n: f64) -> f64 {
    0.01 * n
}

/// `improveQuarkHept3`: `0.01n`.
#[must_use]
pub fn improve_quark_hept_3_effect(n: f64) -> f64 {
    0.01 * n
}

/// `improveQuarkHept4`: `0.01n`.
#[must_use]
pub fn improve_quark_hept_4_effect(n: f64) -> f64 {
    0.01 * n
}

/// `improveQuarkHept5`: `0.0001n`.
#[must_use]
pub fn improve_quark_hept_5_effect(n: f64) -> f64 {
    0.000_1 * n
}

// ─── Cube/tesseract/hypercube → quark conversion family ───────────────────

/// `cubeToQuark`.
#[must_use]
pub fn cube_to_quark_effect(n: f64) -> f64 {
    cube_quark_conversion(n)
}

/// `tesseractToQuark`.
#[must_use]
pub fn tesseract_to_quark_effect(n: f64) -> f64 {
    cube_quark_conversion(n)
}

/// `hypercubeToQuark`.
#[must_use]
pub fn hypercube_to_quark_effect(n: f64) -> f64 {
    cube_quark_conversion(n)
}

/// `cubeToQuarkAll`: `1 + 0.002n`.
#[must_use]
pub fn cube_to_quark_all_effect(n: f64) -> f64 {
    1.0 + 0.002 * n
}

// ─── Improved daily family ────────────────────────────────────────────────

/// `shopImprovedDaily`: `1 + 0.05n`.
#[must_use]
pub fn shop_improved_daily_effect(n: f64) -> f64 {
    1.0 + 0.05 * n
}

/// Shared key selector for the `shopImprovedDaily2/3/4` family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopImprovedDailyKey {
    /// `freeSingularityUpgrades` — identity (`n`).
    FreeSingularityUpgrades,
    /// `dailyCodeGoldenQuarkMult` — `1 + mult × n` (mult depends on
    /// upgrade tier).
    DailyCodeGoldenQuarkMult,
}

fn shop_improved_daily_helper(n: f64, key: ShopImprovedDailyKey, mult: f64) -> f64 {
    match key {
        ShopImprovedDailyKey::FreeSingularityUpgrades => n,
        ShopImprovedDailyKey::DailyCodeGoldenQuarkMult => 1.0 + mult * n,
    }
}

/// `shopImprovedDaily2`: mult `0.2`.
#[must_use]
pub fn shop_improved_daily_2_effect(n: f64, key: ShopImprovedDailyKey) -> f64 {
    shop_improved_daily_helper(n, key, 0.2)
}

/// `shopImprovedDaily3`: mult `0.15`.
#[must_use]
pub fn shop_improved_daily_3_effect(n: f64, key: ShopImprovedDailyKey) -> f64 {
    shop_improved_daily_helper(n, key, 0.15)
}

/// `shopImprovedDaily4`: mult `1`.
#[must_use]
pub fn shop_improved_daily_4_effect(n: f64, key: ShopImprovedDailyKey) -> f64 {
    shop_improved_daily_helper(n, key, 1.0)
}

// ─── Misc late-game pure ──────────────────────────────────────────────────

/// `constantEX`: identity.
#[must_use]
pub fn constant_ex_effect(n: f64) -> f64 {
    n
}

/// `powderEX`: `1 + 0.02n`.
#[must_use]
pub fn powder_ex_effect(n: f64) -> f64 {
    1.0 + 0.02 * n
}

/// `powderAuto`: `0.01n`.
#[must_use]
pub fn powder_auto_effect(n: f64) -> f64 {
    0.01 * n
}

/// `autoWarp`: unlock at `n > 0`.
#[must_use]
pub fn auto_warp_effect(n: f64) -> bool {
    n > 0.0
}

/// `extraWarp`: identity.
#[must_use]
pub fn extra_warp_effect(n: f64) -> f64 {
    n
}

// ─── Ambrosia generation / luck family ────────────────────────────────────

/// `shopAmbrosiaGeneration1`: `1 + 0.01n`.
#[must_use]
pub fn shop_ambrosia_generation_1_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `shopAmbrosiaGeneration2`: `1 + 0.01n`.
#[must_use]
pub fn shop_ambrosia_generation_2_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `shopAmbrosiaGeneration3`: `1 + 0.01n`.
#[must_use]
pub fn shop_ambrosia_generation_3_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `shopAmbrosiaGeneration4`: `1 + 0.001n`.
#[must_use]
pub fn shop_ambrosia_generation_4_effect(n: f64) -> f64 {
    1.0 + 0.001 * n
}

/// `shopAmbrosiaAccelerator`: scales with `noAmbrosiaUpgrades`
/// completions. `1 - 0.006 × n × ex5_completions`.
#[must_use]
pub fn shop_ambrosia_accelerator_effect(n: f64, ex5_completions: f64) -> f64 {
    1.0 - 0.006 * n * ex5_completions
}

/// `shopAmbrosiaLuck1`: `2n`.
#[must_use]
pub fn shop_ambrosia_luck_1_effect(n: f64) -> f64 {
    2.0 * n
}

/// `shopAmbrosiaLuck2`: `2n`.
#[must_use]
pub fn shop_ambrosia_luck_2_effect(n: f64) -> f64 {
    2.0 * n
}

/// `shopAmbrosiaLuck3`: `2n`.
#[must_use]
pub fn shop_ambrosia_luck_3_effect(n: f64) -> f64 {
    2.0 * n
}

/// `shopAmbrosiaLuck4`: `0.6n`.
#[must_use]
pub fn shop_ambrosia_luck_4_effect(n: f64) -> f64 {
    0.6 * n
}

/// `shopAmbrosiaLuckMultiplier4`: `0.01n`.
#[must_use]
pub fn shop_ambrosia_luck_multiplier_4_effect(n: f64) -> f64 {
    0.01 * n
}

/// `shopOcteractAmbrosiaLuck`: `n × (1 + floor(max(0, log10(wowOcteracts))))`.
#[must_use]
pub fn shop_octeract_ambrosia_luck_effect(n: f64, wow_octeracts: f64) -> f64 {
    n * (1.0 + 0.0_f64.max(wow_octeracts.log10()).floor())
}

/// `shopAmbrosiaUltra`: `2n × exalt_completions_sum`.
#[must_use]
pub fn shop_ambrosia_ultra_effect(n: f64, exalt_completions_sum: f64) -> f64 {
    2.0 * n * exalt_completions_sum
}

// ─── Red luck family ──────────────────────────────────────────────────────

/// Shared key selector for the `shopRedLuck1/2/3` family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopRedLuckKey {
    /// `redLuck` — `redLuckMult × n`.
    RedLuck,
    /// `luckConversionRatio` — `-0.01 × floor(n / 20)`.
    LuckConversionRatio,
}

fn red_luck_body(n: f64, key: ShopRedLuckKey, red_luck_mult: f64) -> f64 {
    match key {
        ShopRedLuckKey::RedLuck => red_luck_mult * n,
        ShopRedLuckKey::LuckConversionRatio => -0.01 * (n / 20.0).floor(),
    }
}

/// `shopRedLuck1`: mult `0.05`.
#[must_use]
pub fn shop_red_luck_1_effect(n: f64, key: ShopRedLuckKey) -> f64 {
    red_luck_body(n, key, 0.05)
}

/// `shopRedLuck2`: mult `0.075`.
#[must_use]
pub fn shop_red_luck_2_effect(n: f64, key: ShopRedLuckKey) -> f64 {
    red_luck_body(n, key, 0.075)
}

/// `shopRedLuck3`: mult `0.1`.
#[must_use]
pub fn shop_red_luck_3_effect(n: f64, key: ShopRedLuckKey) -> f64 {
    red_luck_body(n, key, 0.1)
}

/// `shopHorseShoe` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopHorseShoeKey {
    /// `bonusHorseLevels` — `3n`.
    BonusHorseLevels,
    /// `singularityPenaltyMult` — clamped scaling with horseShoe
    /// rune.
    SingularityPenaltyMult,
}

/// `shopHorseShoe`. `singularityPenaltyMult` clamps at 300 of
/// `horseShoe × n`.
#[must_use]
pub fn shop_horse_shoe_effect(
    n: f64,
    key: ShopHorseShoeKey,
    horse_shoe_rune_effective_level: f64,
) -> f64 {
    match key {
        ShopHorseShoeKey::BonusHorseLevels => 3.0 * n,
        ShopHorseShoeKey::SingularityPenaltyMult => {
            1.0 - 300.0_f64.min(horse_shoe_rune_effective_level * n) / 1_000.0
        }
    }
}

/// `shopInfiniteShopUpgrades`: `floor(0.01 × n × exalt_completions_sum)`.
#[must_use]
pub fn shop_infinite_shop_upgrades_effect(n: f64, exalt_completions_sum: f64) -> f64 {
    (0.01 * n * exalt_completions_sum).floor()
}

/// `shopSingularityPenaltyDebuff`: identity.
#[must_use]
pub fn shop_singularity_penalty_debuff_effect(n: f64) -> f64 {
    n
}

/// `shopCashGrabUltra` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopCashGrabUltraKey {
    /// `ambrosiaGenerationMult`.
    AmbrosiaGenerationMult,
    /// `cubesMult`.
    CubesMult,
    /// `quarkMult`.
    QuarkMult,
}

/// `shopCashGrabUltra`. `ratio = min(1, cbrt(lifetime_ambrosia / 1e7))`,
/// then `1 + per_key_mult × n × ratio`.
#[must_use]
pub fn shop_cash_grab_ultra_effect(
    n: f64,
    key: ShopCashGrabUltraKey,
    lifetime_ambrosia: f64,
) -> f64 {
    let ratio = 1.0_f64.min((lifetime_ambrosia / 1e7).cbrt());
    match key {
        ShopCashGrabUltraKey::AmbrosiaGenerationMult => 1.0 + 0.15 * n * ratio,
        ShopCashGrabUltraKey::CubesMult => 1.0 + 1.2 * n * ratio,
        ShopCashGrabUltraKey::QuarkMult => 1.0 + 0.08 * n * ratio,
    }
}

/// `shopEXUltra`. Same value for all three reward keys.
#[must_use]
pub fn shop_ex_ultra_effect(n: f64, lifetime_ambrosia: f64) -> f64 {
    let ambrosia_mult = (125.0 * n).min(lifetime_ambrosia / 1_000.0) / 1_000.0;
    1.0 + ambrosia_mult
}

/// `shopSingularitySpeedup`: `50` when unlocked, else `1`.
#[must_use]
pub fn shop_singularity_speedup_effect(n: f64) -> f64 {
    if n > 0.0 {
        50.0
    } else {
        1.0
    }
}

/// `shopSingularityPotency`: `3.66` when unlocked, else `1`.
#[must_use]
pub fn shop_singularity_potency_effect(n: f64) -> f64 {
    if n > 0.0 {
        3.66
    } else {
        1.0
    }
}

// ─── shopPanthema ─────────────────────────────────────────────────────────

/// `shopPanthema` reads `bonusLevels()` across seven
/// shop-upgrade-type groups. Caller pre-computes each group's bonus
/// levels. Keys match the legacy `ShopUpgradeGroups` enum's role
/// names.
#[derive(Debug, Clone, Copy, Default)]
pub struct ShopPanthemaBonusLevels {
    /// Offering group bonus levels.
    pub offering: f64,
    /// Obtainium group bonus levels.
    pub obtainium: f64,
    /// Cubes group bonus levels.
    pub cubes: f64,
    /// Speed group bonus levels.
    pub speed: f64,
    /// Quark group bonus levels.
    pub quark: f64,
    /// Ambrosia luck group bonus levels.
    pub ambrosia_luck: f64,
    /// Red ambrosia luck group bonus levels.
    pub red_ambrosia_luck: f64,
    /// Ambrosia generation group bonus levels.
    pub ambrosia_generation: f64,
    /// Infinity upgrades group bonus levels (drives shared boost).
    pub infinity_upgrades: f64,
}

/// `shopPanthema` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopPanthemaKey {
    /// `offeringMult`.
    OfferingMult,
    /// `obtainiumMult`.
    ObtainiumMult,
    /// `cubeMult`.
    CubeMult,
    /// `quarkMult`.
    QuarkMult,
    /// `ascensionSpeedMult`.
    AscensionSpeedMult,
    /// `ambrosiaGenerationMult`.
    AmbrosiaGenerationMult,
    /// `ambrosiaLuck`.
    AmbrosiaLuck,
    /// `redLuck`.
    RedLuck,
    /// `infinityMetaBoost`.
    InfinityMetaBoost,
}

/// `shopPanthema` effect — each reward key scales with a different
/// shop-upgrade group's bonus levels, prefixed by the shared
/// `infinityBoost = 1 + 0.01 × n × bonusLevels.infinityUpgrades`.
#[must_use]
pub fn shop_panthema_effect(
    n: f64,
    key: ShopPanthemaKey,
    bonus_levels: &ShopPanthemaBonusLevels,
) -> f64 {
    let infinity_boost = 1.0 + 0.01 * n * bonus_levels.infinity_upgrades;
    match key {
        ShopPanthemaKey::InfinityMetaBoost => infinity_boost,
        ShopPanthemaKey::OfferingMult => 1.0 + 0.01 * n * bonus_levels.offering * infinity_boost,
        ShopPanthemaKey::ObtainiumMult => 1.0 + 0.01 * n * bonus_levels.obtainium * infinity_boost,
        ShopPanthemaKey::CubeMult => 1.0 + 0.005 * n * bonus_levels.cubes * infinity_boost,
        ShopPanthemaKey::AscensionSpeedMult => {
            1.0 + 0.005 * n * bonus_levels.speed * infinity_boost
        }
        ShopPanthemaKey::QuarkMult => 1.0 + 0.001 * n * bonus_levels.quark * infinity_boost,
        ShopPanthemaKey::AmbrosiaGenerationMult => {
            1.0 + 0.001 * n * bonus_levels.ambrosia_generation * infinity_boost
        }
        ShopPanthemaKey::AmbrosiaLuck => 0.2 * n * bonus_levels.ambrosia_luck * infinity_boost,
        ShopPanthemaKey::RedLuck => 0.05 * n * bonus_levels.red_ambrosia_luck * infinity_boost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offering_potion_is_constant() {
        assert_eq!(offering_potion_effect(0.0), 7_200.0);
        assert_eq!(offering_potion_effect(100.0), 7_200.0);
    }

    #[test]
    fn ex_mult_stair_steps_at_10() {
        // n=10: (1 + 0.6) * 1.08^1 = 1.6 * 1.08 = 1.728
        let v = offering_ex_effect(10.0);
        assert!((v - 1.728).abs() < 1e-9);
    }

    #[test]
    fn cube_quark_conversion_below_1_is_1() {
        assert_eq!(cube_to_quark_effect(0.0), 1.0);
        assert_eq!(cube_to_quark_effect(0.5), 1.0);
    }

    #[test]
    fn cube_quark_conversion_at_1_is_1p5() {
        // 1.5 + 0.5 * (1 - 0.9^0) = 1.5 + 0 = 1.5
        assert!((cube_to_quark_effect(1.0) - 1.5).abs() < 1e-12);
    }

    #[test]
    fn cube_quark_conversion_asymptotes_at_2() {
        let huge = cube_to_quark_effect(1e6);
        assert!(huge > 1.999 && huge <= 2.0);
    }

    #[test]
    fn challenge_tome_negative_scaling() {
        // c10: 2e7 * 5 = 1e8
        // c9c10: -5 / 100 = -0.05
        assert_eq!(
            challenge_tome_effect(5.0, ChallengeTomeKey::C10RequirementReduction),
            1e8
        );
        assert!(
            (challenge_tome_effect(5.0, ChallengeTomeKey::C9C10ScalingReduction) + 0.05).abs()
                < 1e-12
        );
    }

    #[test]
    fn shop_chronometer_s_zero_below_200_sing() {
        // n=10, sing=100 → 1.01^(10 * max(0, -100)) = 1.01^0 = 1
        assert_eq!(shop_chronometer_s_effect(10.0, 100.0), 1.0);
    }

    #[test]
    fn shop_chronometer_s_kicks_in_past_200() {
        // n=1, sing=300 → 1.01^(1 * 100) = 1.01^100
        let v = shop_chronometer_s_effect(1.0, 300.0);
        assert!((v - 1.01_f64.powi(100)).abs() < 1e-9);
    }

    #[test]
    fn calculator_auto_fill_only_at_n_5() {
        assert!(matches!(
            calculator_effect(4.0, CalculatorKey::AutoFill),
            CalculatorValue::Unlock(false)
        ));
        assert!(matches!(
            calculator_effect(5.0, CalculatorKey::AutoFill),
            CalculatorValue::Unlock(true)
        ));
        assert!(matches!(
            calculator_effect(6.0, CalculatorKey::AutoFill),
            CalculatorValue::Unlock(false)
        ));
    }

    #[test]
    fn calculator_2_add_quark_mult_steps_at_12() {
        assert_eq!(calculator_2_effect(11.0, Calculator2Key::AddQuarkMult), 1.0);
        assert_eq!(
            calculator_2_effect(12.0, Calculator2Key::AddQuarkMult),
            1.25
        );
        assert_eq!(calculator_2_effect(13.0, Calculator2Key::AddQuarkMult), 1.0);
    }

    #[test]
    fn shop_cash_grab_ultra_ratio_caps_at_1() {
        // lifetime_ambrosia=1e8 → cbrt(10) > 1 → capped at 1
        let v = shop_cash_grab_ultra_effect(1.0, ShopCashGrabUltraKey::CubesMult, 1e8);
        // 1 + 1.2 * 1 * 1 = 2.2
        assert!((v - 2.2).abs() < 1e-12);
    }

    #[test]
    fn shop_ex_ultra_caps_at_125n() {
        // n=1, lifetime=1e9 → min(125, 1e6) = 125 → /1000 = 0.125 → 1.125
        let v = shop_ex_ultra_effect(1.0, 1e9);
        assert!((v - 1.125).abs() < 1e-12);
    }

    #[test]
    fn shop_singularity_speedup_50x_when_unlocked() {
        assert_eq!(shop_singularity_speedup_effect(0.0), 1.0);
        assert_eq!(shop_singularity_speedup_effect(1.0), 50.0);
    }

    #[test]
    fn shop_horse_shoe_singularity_penalty_clamps() {
        // horseShoe=1000, n=1 → min(300, 1000) = 300 → 1 - 0.3 = 0.7
        let v = shop_horse_shoe_effect(1.0, ShopHorseShoeKey::SingularityPenaltyMult, 1_000.0);
        assert!((v - 0.7).abs() < 1e-12);
    }

    #[test]
    fn shop_red_luck_2_uses_0p075_mult() {
        assert_eq!(shop_red_luck_2_effect(10.0, ShopRedLuckKey::RedLuck), 0.75);
    }

    #[test]
    fn shop_red_luck_conversion_ratio_steps_at_20() {
        // n=19 → floor(19/20)=0 → 0
        // n=20 → floor=1 → -0.01
        assert_eq!(
            shop_red_luck_1_effect(19.0, ShopRedLuckKey::LuckConversionRatio),
            0.0
        );
        assert!(
            (shop_red_luck_1_effect(20.0, ShopRedLuckKey::LuckConversionRatio) + 0.01).abs()
                < 1e-12
        );
    }

    #[test]
    fn shop_panthema_infinity_meta_boost() {
        let levels = ShopPanthemaBonusLevels {
            infinity_upgrades: 10.0,
            ..ShopPanthemaBonusLevels::default()
        };
        let v = shop_panthema_effect(5.0, ShopPanthemaKey::InfinityMetaBoost, &levels);
        // 1 + 0.01 * 5 * 10 = 1.5
        assert!((v - 1.5).abs() < 1e-12);
    }

    #[test]
    fn shop_panthema_offering_includes_infinity_boost() {
        let levels = ShopPanthemaBonusLevels {
            offering: 10.0,
            infinity_upgrades: 0.0,
            ..ShopPanthemaBonusLevels::default()
        };
        // boost = 1, mult = 1 + 0.01*1*10*1 = 1.1
        let v = shop_panthema_effect(1.0, ShopPanthemaKey::OfferingMult, &levels);
        assert!((v - 1.1).abs() < 1e-12);
    }
}
