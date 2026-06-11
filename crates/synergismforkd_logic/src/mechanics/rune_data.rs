//! Static rune / blessing / spirit cost data.
//!
//! Numeric `costCoefficient` + base `levelsPerOOM` for the five core runes
//! (Speed..Superior Intellect) and their blessings and spirits, ported from
//! legacy `Runes.ts`, `RuneBlessings.ts`, and `RuneSpirits.ts`. Kept in the
//! logic tier (not UI-only) because the save migration also needs the
//! blessing/spirit coefficients to re-derive EXP from a stored level.
//!
//! Indices match the `RUNE_*` constants (`crate::state::runes`):
//! `0` Speed, `1` Duplication, `2` Prism, `3` Thrift, `4` Superior Intellect.
//! Runes 5..=9 (singularity/platonic-era) and the OOM-increase terms are not
//! covered here yet — they are `0`/identity at pre-ascension progression.

use synergismforkd_bignum::Decimal;

/// Number of runes that have blessings/spirits and are surfaced in the UI.
pub const CORE_RUNE_COUNT: usize = 5;

/// `rune.costCoefficient` (Runes.ts:343/387/431/469/509).
pub const RUNE_COST_COEFFICIENT: [f64; CORE_RUNE_COUNT] = [50.0, 20_000.0, 5e5, 2.5e7, 1e12];
/// `rune.levelsPerOOM` base slope (Runes.ts) — the OOM-increase term is `0`
/// until ascension-era research/challenges, so the base is exact for now.
pub const RUNE_LEVELS_PER_OOM: [f64; CORE_RUNE_COUNT] = [150.0, 120.0, 90.0, 60.0, 30.0];

/// `runeBlessings[*].costCoefficient` (RuneBlessings.ts:62/88/114/140/166).
pub const BLESSING_COST_COEFFICIENT: [f64; CORE_RUNE_COUNT] = [1e6, 1e7, 1e9, 1e12, 1e15];
/// `runeBlessings[*].levelsPerOOM` (all `4`).
pub const BLESSING_LEVELS_PER_OOM: [f64; CORE_RUNE_COUNT] = [4.0; CORE_RUNE_COUNT];

/// `runeSpirits[*].costCoefficient` (RuneSpirits.ts:63/87/111/135/159).
pub const SPIRIT_COST_COEFFICIENT: [f64; CORE_RUNE_COUNT] = [1e45, 1e52, 1e60, 1e72, 1e85];
/// `runeSpirits[*].levelsPerOOM` (all `2`).
pub const SPIRIT_LEVELS_PER_OOM: [f64; CORE_RUNE_COUNT] = [2.0; CORE_RUNE_COUNT];

/// Which rune-upgrade family a cost lookup targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuneUpgradeKind {
    /// Top-level rune.
    Rune,
    /// Rune blessing.
    Blessing,
    /// Rune spirit.
    Spirit,
}

impl RuneUpgradeKind {
    /// `costCoefficient` for core rune `index` (`0..5`) in this family.
    /// Out-of-range returns `1` (a harmless no-op coefficient).
    #[must_use]
    pub fn cost_coefficient(self, index: usize) -> Decimal {
        let table = match self {
            RuneUpgradeKind::Rune => &RUNE_COST_COEFFICIENT,
            RuneUpgradeKind::Blessing => &BLESSING_COST_COEFFICIENT,
            RuneUpgradeKind::Spirit => &SPIRIT_COST_COEFFICIENT,
        };
        Decimal::from_finite(table.get(index).copied().unwrap_or(1.0))
    }

    /// Base `levelsPerOOM` for core rune `index` in this family. Out-of-range
    /// returns `1`.
    #[must_use]
    pub fn levels_per_oom(self, index: usize) -> f64 {
        let table = match self {
            RuneUpgradeKind::Rune => &RUNE_LEVELS_PER_OOM,
            RuneUpgradeKind::Blessing => &BLESSING_LEVELS_PER_OOM,
            RuneUpgradeKind::Spirit => &SPIRIT_LEVELS_PER_OOM,
        };
        table.get(index).copied().unwrap_or(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_core_rune_sized() {
        assert_eq!(RUNE_COST_COEFFICIENT.len(), CORE_RUNE_COUNT);
        assert_eq!(BLESSING_LEVELS_PER_OOM.len(), CORE_RUNE_COUNT);
        assert_eq!(SPIRIT_COST_COEFFICIENT.len(), CORE_RUNE_COUNT);
    }

    #[test]
    fn kind_lookups_match_tables() {
        assert_eq!(RuneUpgradeKind::Rune.cost_coefficient(0).to_number(), 50.0);
        assert_eq!(
            RuneUpgradeKind::Blessing.cost_coefficient(4).to_number(),
            1e15
        );
        assert_eq!(RuneUpgradeKind::Spirit.levels_per_oom(0), 2.0);
        // Out-of-range is a harmless identity.
        assert_eq!(RuneUpgradeKind::Rune.cost_coefficient(99).to_number(), 1.0);
    }
}
