//! Golden-quark state slice — the GQ currency and the ~80 named
//! GQ upgrades.
//!
//! Mirrors `player.goldenQuarks` and `player.singularityUpgrades`
//! from the legacy schema. Backs [`crate::mechanics::gq_upgrade_cost`],
//! [`crate::mechanics::gq_upgrade_levels`], and
//! [`crate::mechanics::golden_quark_upgrades`].
//!
//! The legacy schema keys upgrades by name; this slice indexes them
//! by position (caller maintains the name-to-id mapping). Each
//! entry carries the full GQ-upgrade shape (level, freeLevel,
//! maxLevel, canExceedCap, qualityOfLife, specialCostForm) so the
//! cost / effect dispatchers don't need to look it up elsewhere.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Special-cost-form selector for one GQ upgrade — pinned here
/// alongside the state so the storage matches the dispatch shape
/// in [`crate::mechanics::gq_upgrade_cost::GQUpgradeSpecialCostForm`].
/// Stored as a `u8` for `Copy` + small footprint:
/// `0 = Exponential2, 1 = Cubic, 2 = Quadratic, 3 = None`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoredSpecialCostForm {
    /// `Exponential2` form — soft sqrt(overcap) × `2^level`.
    Exponential2,
    /// `Cubic` form — overcap × `((level+1)^3 - level^3)` delta.
    Cubic,
    /// `Quadratic` form — overcap × `((level+1)^2 - level^2)` delta.
    Quadratic,
    /// Default linear branch (no special form).
    #[default]
    None,
}

/// One GQ upgrade's per-player state. Mirrors the legacy
/// `player.singularityUpgrades.<name>` shape.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct GoldenQuarkUpgrade {
    /// Purchased level.
    pub level: f64,
    /// Accumulated free levels.
    pub free_level: f64,
    /// Base maxLevel (`-1` for unlimited).
    pub max_level: f64,
    /// Whether this upgrade benefits from overclock-perk cap
    /// expansion.
    pub can_exceed_cap: bool,
    /// Quality-of-life flag — when true, the upgrade survives
    /// `noSingularityUpgrades` and `sadisticPrequel`.
    pub quality_of_life: bool,
    /// Cost-formula shape.
    pub special_cost_form: StoredSpecialCostForm,
    /// Base coefficient (`costPerLevel`) — used by the cost formula.
    pub cost_per_level: f64,
}

/// Slice of `GameState` for the golden-quark feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoldenQuarksState {
    /// `player.goldenQuarks` — the currency balance.
    pub golden_quarks: Decimal,
    /// `player.quarksThisSingularity` — drives `calculate_base_golden_quarks`.
    pub quarks_this_singularity: f64,
    /// Per-upgrade state. The UI/tier maintains the name ↔ index
    /// mapping; this slice holds the values.
    pub upgrades: Vec<GoldenQuarkUpgrade>,
}

impl GoldenQuarksState {
    /// Build with `n_upgrades` upgrade slots, each at default
    /// values. The legacy synergism build has ~80 named GQ upgrades.
    #[must_use]
    pub fn new(n_upgrades: usize) -> Self {
        Self {
            golden_quarks: Decimal::zero(),
            quarks_this_singularity: 0.0,
            upgrades: vec![GoldenQuarkUpgrade::default(); n_upgrades],
        }
    }
}

impl Default for GoldenQuarksState {
    fn default() -> Self {
        Self::new(80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_80_upgrade_slots() {
        let s = GoldenQuarksState::default();
        assert_eq!(s.upgrades.len(), 80);
        assert_eq!(s.golden_quarks.to_number(), 0.0);
    }

    #[test]
    fn upgrade_default_is_zeroed() {
        let u = GoldenQuarkUpgrade::default();
        assert_eq!(u.level, 0.0);
        assert!(!u.can_exceed_cap);
        assert!(matches!(u.special_cost_form, StoredSpecialCostForm::None));
    }
}
