//! Ant-mastery data + per-producer formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/antMasteries.ts`.
//! The data is indexed by ant-producer (`0..=8`, Workers..HolySpirit)
//! and by mastery level. ELO requirements and per-level particle
//! costs use indices `0..=11` (purchasing level `k+1` requires
//! `requirements[k]`); `self_speed_multipliers` uses indices
//! `0..=12` (level 0 is the multiplicative identity).
//!
//! Per-level Decimal values are returned by function (rather than a
//! const array) because they exceed `f64::MAX` (up to
//! `Decimal::from_mantissa_exponent(1.0, 1.0e10)`).
//!
//! All formulas are pure given the per-producer data + current state
//! inputs; the buy action itself stays in the UI tier because it
//! mutates player state.

use synergismforkd_bignum::Decimal;

/// Mastery cap (legacy `MAX_ANT_MASTERY_LEVEL`). Mastery levels run
/// `0..=12`; the cap is the max value `masteryLevel` can take.
pub const MAX_ANT_MASTERY_LEVEL: u8 = 12;

// ─── Per-producer ELO requirements ────────────────────────────────────────

/// Total reborn-ELO required to purchase mastery level `level + 1`
/// for `producer`. `level` is `0..=11`; the returned value gates the
/// transition from `level` to `level + 1`.
#[must_use]
pub fn ant_mastery_total_elo_requirement(producer: u8, level: u8) -> f64 {
    debug_assert!(
        matches!(producer, 0..=8),
        "producer out of range: {producer}"
    );
    debug_assert!(matches!(level, 0..=11), "level out of range: {level}");
    let table: [f64; 12] = match producer {
        0 => [
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            500.0,
            1_250.0,
            3_000.0,
            7_500.0,
            25_000.0,
            256_000.0,
            1_024_000.0,
        ],
        1 => [
            0.0,
            0.0,
            0.0,
            0.0,
            2_500.0,
            3_000.0,
            8_000.0,
            30_000.0,
            60_000.0,
            115_000.0,
            403_000.0,
            1_344_000.0,
        ],
        2 => [
            0.0,
            0.0,
            1_250.0,
            2_600.0,
            6_000.0,
            9_000.0,
            27_000.0,
            55_000.0,
            100_000.0,
            180_000.0,
            598_000.0,
            1_996_000.0,
        ],
        3 => [
            0.0,
            0.0,
            2_700.0,
            10_000.0,
            25_000.0,
            42_000.0,
            65_000.0,
            102_000.0,
            160_000.0,
            260_000.0,
            800_000.0,
            2_900_000.0,
        ],
        4 => [
            2_500.0,
            3_100.0,
            11_000.0,
            22_000.0,
            44_000.0,
            83_000.0,
            142_000.0,
            221_000.0,
            333_333.0,
            500_000.0,
            1_500_000.0,
            4_300_000.0,
        ],
        5 => [
            6_000.0,
            12_000.0,
            31_000.0,
            61_000.0,
            122_000.0,
            200_000.0,
            340_000.0,
            525_000.0,
            740_000.0,
            1_115_000.0,
            2_400_000.0,
            6_200_000.0,
        ],
        6 => [
            23_000.0,
            53_000.0,
            100_000.0,
            190_000.0,
            377_000.0,
            621_000.0,
            1_021_000.0,
            1_600_000.0,
            2_340_000.0,
            3_400_000.0,
            4_400_000.0,
            5_999_000.0,
        ],
        7 => [
            100_000.0,
            200_000.0,
            400_000.0,
            750_000.0,
            1_400_000.0,
            2_400_000.0,
            3_500_000.0,
            4_666_000.0,
            6_020_000.0,
            7_250_000.0,
            9_000_000.0,
            10_000_000.0,
        ],
        _ => [
            2_500_000.0,
            3_500_000.0,
            4_500_000.0,
            5_500_000.0,
            6_250_000.0,
            7_000_000.0,
            7_750_000.0,
            8_500_000.0,
            9_250_000.0,
            10_000_000.0,
            11_000_000.0,
            12_500_000.0,
        ],
    };
    table[level as usize]
}

// ─── Per-producer particle costs ──────────────────────────────────────────

/// Particle cost of buying mastery level `level + 1` for `producer`.
/// Values past f64 range use [`Decimal::from_mantissa_exponent`].
#[must_use]
pub fn ant_mastery_particle_cost(producer: u8, level: u8) -> Decimal {
    debug_assert!(
        matches!(producer, 0..=8),
        "producer out of range: {producer}"
    );
    debug_assert!(matches!(level, 0..=11), "level out of range: {level}");
    let exps: [f64; 12] = match producer {
        0 => [
            700.0,
            1_200.0,
            2_600.0,
            5_000.0,
            12_500.0,
            20_000.0,
            32_000.0,
            100_000.0,
            800_000.0,
            3_000_000.0,
            10_000_000.0,
            100_000_000.0,
        ],
        1 => [
            1_000.0,
            2_000.0,
            3_000.0,
            12_500.0,
            40_000.0,
            100_000.0,
            1_500_000.0,
            5_000_000.0,
            10_000_000.0,
            17_500_000.0,
            175_000_000.0,
            1_750_000_000.0,
        ],
        2 => [
            2_500.0,
            6_000.0,
            20_000.0,
            60_000.0,
            400_000.0,
            1_000_000.0,
            2_000_000.0,
            4_000_000.0,
            8_000_000.0,
            20_000_000.0,
            40_000_000.0,
            300_000_000.0,
        ],
        3 => [
            6_000.0,
            15_000.0,
            70_000.0,
            2_000_000.0,
            5_000_000.0,
            10_000_000.0,
            25_000_000.0,
            60_000_000.0,
            120_000_000.0,
            300_000_000.0,
            1_000_000_000.0,
            5_000_000_000.0,
        ],
        4 => [
            50_000.0,
            100_000.0,
            1_200_000.0,
            5_000_000.0,
            8_000_000.0,
            12_500_000.0,
            30_000_000.0,
            80_000_000.0,
            200_000_000.0,
            500_000_000.0,
            2_000_000_000.0,
            8_000_000_000.0,
        ],
        5 => [
            40_000.0,
            100_000.0,
            250_000.0,
            600_000.0,
            1_250_000.0,
            3_000_000.0,
            8_000_000.0,
            20_000_000.0,
            50_000_000.0,
            150_000_000.0,
            600_000_000.0,
            1_250_000_000.0,
        ],
        6 => [
            100_000.0,
            400_000.0,
            1_250_000.0,
            4_000_000.0,
            6_000_000.0,
            13_000_000.0,
            26_000_000.0,
            50_000_000.0,
            150_000_000.0,
            400_000_000.0,
            1_000_000_000.0,
            3_000_000_000.0,
        ],
        7 => [
            400_000.0,
            1_250_000.0,
            4_000_000.0,
            6_000_000.0,
            13_000_000.0,
            26_000_000.0,
            50_000_000.0,
            150_000_000.0,
            400_000_000.0,
            1_000_000_000.0,
            3_000_000_000.0,
            10_000_000_000.0,
        ],
        // HolySpirit (8): unique — small integer costs 1..=12
        _ => return Decimal::from_finite(f64::from(level + 1)),
    };
    Decimal::from_mantissa_exponent(1.0, exps[level as usize])
}

// ─── Per-producer self-speed multipliers (13-entry, levels 0..=12) ────────

/// Per-level self-speed multiplier for `producer` at mastery `level`
/// (`0..=12`). Level `0` is the multiplicative identity.
#[must_use]
pub fn ant_mastery_self_speed_multiplier(producer: u8, level: u8) -> Decimal {
    debug_assert!(
        matches!(producer, 0..=8),
        "producer out of range: {producer}"
    );
    debug_assert!(matches!(level, 0..=12), "level out of range: {level}");
    // Encoded as `Decimal::from_mantissa_exponent(mantissa, exponent)`.
    // Workers (0) and Breeders (1) have small-integer multipliers at low
    // levels that don't fit the unit-mantissa pattern; encode mantissa
    // explicitly for those entries.
    //
    // Each arm maps 1:1 to a distinct entry in the legacy TS truth
    // table. Combining arms with `|` would obscure parity and make
    // future TS-side bug fixes harder to mirror — keep the table
    // enumerated.
    #[allow(clippy::match_same_arms)]
    let (mantissa, exp): (f64, f64) = match (producer, level) {
        // Workers (0) — Decimal.fromString('1' / '3' / '9' / '20' / '100' / '1e4' / …)
        (0, 0) => (1.0, 0.0),
        (0, 1) => (3.0, 0.0),
        (0, 2) => (9.0, 0.0),
        (0, 3) => (2.0, 1.0),
        (0, 4) => (1.0, 2.0),
        (0, 5) => (1.0, 4.0),
        (0, 6) => (1.0, 6.0),
        (0, 7) => (1.0, 9.0),
        (0, 8) => (1.0, 11.0),
        (0, 9) => (1.0, 20.0),
        (0, 10) => (1.0, 50.0),
        (0, 11) => (1.0, 200.0),
        (0, _) => (1.0, 1_000.0),
        // Breeders (1)
        (1, 0) => (1.0, 0.0),
        (1, 1) => (4.0, 0.0),
        (1, 2) => (1.6, 1.0),
        (1, 3) => (1.0, 2.0),
        (1, 4) => (2.0, 4.0),
        (1, 5) => (3.0, 8.0),
        (1, 6) => (3.0, 11.0),
        (1, 7) => (1.0, 25.0),
        (1, 8) => (1.0, 40.0),
        (1, 9) => (1.0, 70.0),
        (1, 10) => (1.0, 120.0),
        (1, 11) => (1.0, 400.0),
        (1, _) => (1.0, 1_400.0),
        // MetaBreeders (2)
        (2, 0) => (1.0, 0.0),
        (2, 1) => (1.0, 1.0),
        (2, 2) => (2.0, 2.0),
        (2, 3) => (1.0, 4.0),
        (2, 4) => (1.0, 7.0),
        (2, 5) => (1.0, 9.0),
        (2, 6) => (1.0, 16.0),
        (2, 7) => (1.0, 32.0),
        (2, 8) => (1.0, 55.0),
        (2, 9) => (1.0, 100.0),
        (2, 10) => (1.0, 160.0),
        (2, 11) => (1.0, 600.0),
        (2, _) => (1.0, 2_000.0),
        // MegaBreeders (3)
        (3, 0) => (1.0, 0.0),
        (3, 1) => (6.0, 1.0),
        (3, 2) => (1.0, 4.0),
        (3, 3) => (1.0, 8.0),
        (3, 4) => (1.0, 12.0),
        (3, 5) => (1.0, 24.0),
        (3, 6) => (1.0, 48.0),
        (3, 7) => (1.0, 80.0),
        (3, 8) => (1.0, 120.0),
        (3, 9) => (1.0, 200.0),
        (3, 10) => (1.0, 300.0),
        (3, 11) => (1.0, 800.0),
        (3, _) => (1.0, 3_000.0),
        // Queens (4)
        (4, 0) => (1.0, 0.0),
        (4, 1) => (1.0, 3.0),
        (4, 2) => (1.0, 7.0),
        (4, 3) => (1.0, 14.0),
        (4, 4) => (1.0, 24.0),
        (4, 5) => (1.0, 48.0),
        (4, 6) => (1.0, 80.0),
        (4, 7) => (1.0, 120.0),
        (4, 8) => (1.0, 200.0),
        (4, 9) => (1.0, 300.0),
        (4, 10) => (1.0, 600.0),
        (4, 11) => (1.0, 1_550.0),
        (4, _) => (1.0, 4_500.0),
        // LordRoyals (5)
        (5, 0) => (1.0, 0.0),
        (5, 1) => (1.0, 5.0),
        (5, 2) => (1.0, 12.0),
        (5, 3) => (1.0, 24.0),
        (5, 4) => (1.0, 48.0),
        (5, 5) => (1.0, 80.0),
        (5, 6) => (1.0, 120.0),
        (5, 7) => (1.0, 200.0),
        (5, 8) => (1.0, 300.0),
        (5, 9) => (1.0, 600.0),
        (5, 10) => (1.0, 1_000.0),
        (5, 11) => (1.0, 2_500.0),
        (5, _) => (1.0, 7_000.0),
        // Almighties (6)
        (6, 0) => (1.0, 0.0),
        (6, 1) => (1.0, 24.0),
        (6, 2) => (1.0, 48.0),
        (6, 3) => (1.0, 80.0),
        (6, 4) => (1.0, 120.0),
        (6, 5) => (1.0, 200.0),
        (6, 6) => (1.0, 300.0),
        (6, 7) => (1.0, 600.0),
        (6, 8) => (1.0, 1_000.0),
        (6, 9) => (1.0, 1_800.0),
        (6, 10) => (1.0, 3_000.0),
        (6, 11) => (1.0, 5_000.0),
        (6, _) => (1.0, 10_000.0),
        // Disciples (7)
        (7, 0) => (1.0, 0.0),
        (7, 1) => (1.0, 80.0),
        (7, 2) => (1.0, 120.0),
        (7, 3) => (1.0, 200.0),
        (7, 4) => (1.0, 300.0),
        (7, 5) => (1.0, 600.0),
        (7, 6) => (1.0, 1_000.0),
        (7, 7) => (1.0, 1_800.0),
        (7, 8) => (1.0, 3_000.0),
        (7, 9) => (1.0, 5_000.0),
        (7, 10) => (1.0, 11_000.0),
        (7, 11) => (1.0, 25_000.0),
        (7, _) => (1.0, 60_000.0),
        // HolySpirit (8)
        (_, 0) => (1.0, 0.0),
        (_, 1) => (1.0, 2_500.0),
        (_, 2) => (1.0, 4_200.0),
        (_, 3) => (1.0, 8_000.0),
        (_, 4) => (1.0, 13_000.0),
        (_, 5) => (1.0, 19_000.0),
        (_, 6) => (1.0, 26_000.0),
        (_, 7) => (1.0, 34_000.0),
        (_, 8) => (1.0, 43_000.0),
        (_, 9) => (1.0, 55_000.0),
        (_, 10) => (1.0, 70_000.0),
        (_, 11) => (1.0, 90_000.0),
        (_, _) => (1.0, 150_000.0),
    };
    Decimal::from_mantissa_exponent(mantissa, exp)
}

// ─── Per-producer self-power increment ────────────────────────────────────

/// Per-producer base increment applied to the
/// `(1 + selfPowerIncrement)^purchased` multiplier. Scales the bonus
/// from purchased-producer count once any mastery is owned.
#[must_use]
pub const fn ant_mastery_self_power_increment(producer: u8) -> f64 {
    debug_assert!(matches!(producer, 0..=8), "producer out of range");
    match producer {
        0 => 0.001,
        1 => 0.002,
        2 => 0.005,
        3 => 0.01,
        4 => 0.02,
        5 => 0.04,
        6 => 0.1,
        7 => 0.3,
        _ => 0.5,
    }
}

// ─── Speed-from-mastery formula ───────────────────────────────────────────

/// Inputs to [`calculate_self_speed_from_mastery`].
#[derive(Debug, Clone, Copy)]
pub struct SelfSpeedFromMasteryInput {
    /// Ant-producer index (`0..=8`).
    pub producer: u8,
    /// `player.ants.masteries[ant].mastery` — current mastery level
    /// (`0..=12`).
    pub mastery_level: u8,
    /// `player.ants.producers[ant].purchased` — exponentiates the
    /// per-level power-increment multiplier.
    pub purchased: f64,
}

/// Per-producer speed multiplier granted by mastery. Combines:
/// - `(1 + selfPowerIncrement)^purchased`, where `selfPowerIncrement`
///   is `mastery_level × per_producer_increment + 0.01 × min(1, mastery_level)`
///   (the `+0.01` is a flat bonus for owning any mastery, not per-level)
/// - `self_speed_multipliers[mastery_level]` (per-level base mult)
#[must_use]
pub fn calculate_self_speed_from_mastery(input: &SelfSpeedFromMasteryInput) -> Decimal {
    let base_inc = ant_mastery_self_power_increment(input.producer);
    let level_f = f64::from(input.mastery_level);
    let self_power_increment = level_f * base_inc + 0.01 * level_f.min(1.0);
    let self_base_mult = ant_mastery_self_speed_multiplier(input.producer, input.mastery_level);
    Decimal::from_finite(1.0 + self_power_increment).pow(Decimal::from_finite(input.purchased))
        * self_base_mult
}

// ─── Buy-availability + buyable-count ─────────────────────────────────────

/// Inputs to [`can_buy_ant_mastery`] and
/// [`get_buyable_ant_mastery_levels`].
#[derive(Debug, Clone, Copy)]
pub struct CanBuyAntMasteryInput {
    /// Ant-producer index (`0..=8`).
    pub producer: u8,
    /// Current mastery level (`0..=12`).
    pub mastery_level: u8,
    /// Max mastery level — pass [`MAX_ANT_MASTERY_LEVEL`] unless an
    /// override is in play.
    pub max_level: u8,
    /// `player.ants.rebornELO`.
    pub current_elo: f64,
    /// `player.reincarnationPoints`.
    pub current_particles: Decimal,
}

/// Below cap *and* have enough ELO + particles for the next level?
#[must_use]
pub fn can_buy_ant_mastery(input: &CanBuyAntMasteryInput) -> bool {
    if input.mastery_level >= input.max_level {
        return false;
    }
    let req_elo = ant_mastery_total_elo_requirement(input.producer, input.mastery_level);
    let cost = ant_mastery_particle_cost(input.producer, input.mastery_level);
    input.current_elo >= req_elo && input.current_particles >= cost
}

/// How many additional mastery levels can be bought with the given
/// ELO + particle balance. Walks one level at a time until cap or
/// budget exhaustion.
///
/// Note: particle balance is checked per-level against the
/// **per-level** cost, not the cumulative cost — mirroring legacy
/// behavior where each individual buy operation re-evaluates the
/// balance. The actual sub happens at buy time and may underflow.
#[must_use]
pub fn get_buyable_ant_mastery_levels(input: &CanBuyAntMasteryInput) -> u8 {
    let mut buyable_levels: u8 = 0;
    while input.mastery_level + buyable_levels < input.max_level {
        let lvl = input.mastery_level + buyable_levels;
        let req_elo = ant_mastery_total_elo_requirement(input.producer, lvl);
        let cost = ant_mastery_particle_cost(input.producer, lvl);
        if input.current_elo >= req_elo && input.current_particles >= cost {
            buyable_levels += 1;
        } else {
            break;
        }
    }
    buyable_levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_mastery_level_constant() {
        assert_eq!(MAX_ANT_MASTERY_LEVEL, 12);
    }

    #[test]
    fn workers_elo_req_levels_0_through_4_are_zero() {
        for lvl in 0..=4 {
            assert_eq!(ant_mastery_total_elo_requirement(0, lvl), 0.0);
        }
    }

    #[test]
    fn workers_elo_req_level_5_is_500() {
        assert_eq!(ant_mastery_total_elo_requirement(0, 5), 500.0);
    }

    #[test]
    fn holy_spirit_particle_costs_are_integers_1_to_12() {
        for lvl in 0..=11 {
            let cost = ant_mastery_particle_cost(8, lvl);
            assert_eq!(cost.to_number(), f64::from(lvl + 1));
        }
    }

    #[test]
    fn workers_particle_cost_level_0_is_1e700() {
        let cost = ant_mastery_particle_cost(0, 0);
        assert!((cost.log10().to_number() - 700.0).abs() < 1e-9);
    }

    #[test]
    fn workers_self_speed_at_level_0_is_one() {
        assert_eq!(ant_mastery_self_speed_multiplier(0, 0).to_number(), 1.0);
    }

    #[test]
    fn workers_self_speed_at_level_12_is_1e1000() {
        let val = ant_mastery_self_speed_multiplier(0, 12);
        assert!((val.log10().to_number() - 1_000.0).abs() < 1e-9);
    }

    #[test]
    fn breeders_self_speed_at_level_2_is_16() {
        // Breeders, level 2: '16' = 1.6e1
        let val = ant_mastery_self_speed_multiplier(1, 2);
        assert!((val.to_number() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn self_power_increments_match_table() {
        assert_eq!(ant_mastery_self_power_increment(0), 0.001);
        assert_eq!(ant_mastery_self_power_increment(3), 0.01);
        assert_eq!(ant_mastery_self_power_increment(6), 0.1);
        assert_eq!(ant_mastery_self_power_increment(8), 0.5);
    }

    #[test]
    fn self_speed_from_mastery_at_level_0_returns_one() {
        let result = calculate_self_speed_from_mastery(&SelfSpeedFromMasteryInput {
            producer: 0,
            mastery_level: 0,
            purchased: 0.0,
        });
        // (1 + 0)^0 * 1 = 1
        assert_eq!(result.to_number(), 1.0);
    }

    #[test]
    fn self_speed_from_mastery_includes_purchased_multiplier() {
        // Workers, mastery 1: selfPowerIncrement = 1*0.001 + 0.01*1 = 0.011
        // base mult = 3, purchased = 100 → 1.011^100 * 3 ≈ 2.99 * 3 ≈ ~9
        let result = calculate_self_speed_from_mastery(&SelfSpeedFromMasteryInput {
            producer: 0,
            mastery_level: 1,
            purchased: 100.0,
        });
        let expected = 1.011_f64.powi(100) * 3.0;
        assert!((result.to_number() - expected).abs() < 1e-6);
    }

    #[test]
    fn can_buy_returns_false_at_max_level() {
        let input = CanBuyAntMasteryInput {
            producer: 0,
            mastery_level: MAX_ANT_MASTERY_LEVEL,
            max_level: MAX_ANT_MASTERY_LEVEL,
            current_elo: 1e18,
            current_particles: Decimal::from_mantissa_exponent(1.0, 1e9),
        };
        assert!(!can_buy_ant_mastery(&input));
    }

    #[test]
    fn can_buy_workers_level_1_with_zero_elo() {
        // Workers level 0 → 1 requires elo 0, cost 1e700
        let input = CanBuyAntMasteryInput {
            producer: 0,
            mastery_level: 0,
            max_level: MAX_ANT_MASTERY_LEVEL,
            current_elo: 0.0,
            current_particles: Decimal::from_mantissa_exponent(1.0, 1_000.0),
        };
        assert!(can_buy_ant_mastery(&input));
    }

    #[test]
    fn can_buy_fails_when_particles_below_cost() {
        let input = CanBuyAntMasteryInput {
            producer: 0,
            mastery_level: 0,
            max_level: MAX_ANT_MASTERY_LEVEL,
            current_elo: 0.0,
            current_particles: Decimal::from_finite(1e100),
        };
        assert!(!can_buy_ant_mastery(&input));
    }

    #[test]
    fn buyable_levels_walks_until_block() {
        // Workers: levels 0..=4 require 0 elo; level 5 requires 500.
        // Costs: 1e700, 1e1200, 1e2600, 1e5000, 1e12500, …
        // Give it elo=400 and enough particles → buys 4 levels (0..=3),
        // then level-4 cost (1e12500) blocks further if particles run out.
        let input = CanBuyAntMasteryInput {
            producer: 0,
            mastery_level: 0,
            max_level: MAX_ANT_MASTERY_LEVEL,
            current_elo: 400.0,
            // Just enough for first 4 levels (1e5000 covers up to level 3 → 4 buys)
            current_particles: Decimal::from_mantissa_exponent(1.0, 5_000.0),
        };
        let result = get_buyable_ant_mastery_levels(&input);
        assert_eq!(result, 4);
    }
}
