//! Pure reward formulas for ant sacrifice.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/antSacrificeRewards.ts`.
//! The offering / obtainium calculators stay in the UI tier because
//! they reduce a Statistics stat array (display-coupled). The
//! talisman-item calculator is fully pure given its inputs and lives
//! here.

use synergismforkd_bignum::Decimal;

/// Craftable talisman fragment / shard items. Matches the
/// `TalismanCraftItems` union in the legacy `Talismans.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalismanCraftItem {
    /// Talisman shard (lowest tier).
    Shard,
    /// Common fragment.
    CommonFragment,
    /// Uncommon fragment.
    UncommonFragment,
    /// Rare fragment.
    RareFragment,
    /// Epic fragment.
    EpicFragment,
    /// Legendary fragment.
    LegendaryFragment,
    /// Mythical fragment.
    MythicalFragment,
}

/// Minimum effective ant ELO required for each talisman item to drop.
#[must_use]
pub fn talisman_item_required_elo(item: TalismanCraftItem) -> f64 {
    match item {
        TalismanCraftItem::Shard => 0.0,
        TalismanCraftItem::CommonFragment => 300.0,
        TalismanCraftItem::UncommonFragment => 600.0,
        TalismanCraftItem::RareFragment => 1_200.0,
        TalismanCraftItem::EpicFragment => 2_000.0,
        TalismanCraftItem::LegendaryFragment | TalismanCraftItem::MythicalFragment => 7_500.0,
    }
}

/// Per-item multiplier applied to the
/// `(elo - threshold + 1) * reward_mult * stage_mult` product.
/// Lower-tier items drop in larger quantities.
#[must_use]
pub fn talisman_reward_multipliers(item: TalismanCraftItem) -> f64 {
    match item {
        TalismanCraftItem::Shard => 1.0,
        TalismanCraftItem::CommonFragment => 0.4,
        TalismanCraftItem::UncommonFragment => 0.1,
        TalismanCraftItem::RareFragment => 0.06,
        TalismanCraftItem::EpicFragment => 0.02,
        TalismanCraftItem::LegendaryFragment => 0.0008,
        TalismanCraftItem::MythicalFragment => 0.0001,
    }
}

/// Inputs to [`calculate_ant_sacrifice_talisman_item`].
#[derive(Debug, Clone, Copy)]
pub struct AntSacrificeTalismanItemInput {
    /// Which talisman item is being awarded.
    pub item: TalismanCraftItem,
    /// Effective ant ELO at the sacrifice.
    pub elo: f64,
    /// Reward-multiplier (`antSacrificeMult * timeMultiplier` —
    /// caller computes).
    pub reward_mult: Decimal,
    /// Per-stage talisman-fragment modifier from the reborn-ELO
    /// stage system.
    pub stage_mult: f64,
}

/// Quantity of a specific talisman craft item awarded by a
/// sacrifice. Returns `0` if ELO is below the item's threshold.
/// Otherwise:
///
/// ```text
/// reward_mult × (elo - threshold + 1) × stage_mult
///   × talisman_reward_multipliers[item]
/// ```
#[must_use]
pub fn calculate_ant_sacrifice_talisman_item(input: &AntSacrificeTalismanItemInput) -> Decimal {
    let required = talisman_item_required_elo(input.item);
    if input.elo < required {
        return Decimal::zero();
    }
    input.reward_mult
        * Decimal::from_finite(input.elo - required + 1.0)
        * Decimal::from_finite(input.stage_mult)
        * Decimal::from_finite(talisman_reward_multipliers(input.item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_always_drops() {
        let result = calculate_ant_sacrifice_talisman_item(&AntSacrificeTalismanItemInput {
            item: TalismanCraftItem::Shard,
            elo: 0.0,
            reward_mult: Decimal::from_finite(1.0),
            stage_mult: 1.0,
        });
        // 1 * (0 - 0 + 1) * 1 * 1 = 1
        assert_eq!(result.to_number(), 1.0);
    }

    #[test]
    fn common_fragment_gated_at_300_elo() {
        let below = calculate_ant_sacrifice_talisman_item(&AntSacrificeTalismanItemInput {
            item: TalismanCraftItem::CommonFragment,
            elo: 299.0,
            reward_mult: Decimal::from_finite(1.0),
            stage_mult: 1.0,
        });
        let at = calculate_ant_sacrifice_talisman_item(&AntSacrificeTalismanItemInput {
            item: TalismanCraftItem::CommonFragment,
            elo: 300.0,
            reward_mult: Decimal::from_finite(1.0),
            stage_mult: 1.0,
        });
        assert_eq!(below, Decimal::zero());
        // 1 * (300 - 300 + 1) * 1 * 0.4 = 0.4
        assert!((at.to_number() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn mythical_and_legendary_share_threshold() {
        assert_eq!(
            talisman_item_required_elo(TalismanCraftItem::LegendaryFragment),
            7_500.0
        );
        assert_eq!(
            talisman_item_required_elo(TalismanCraftItem::MythicalFragment),
            7_500.0
        );
    }

    #[test]
    fn epic_fragment_at_high_elo() {
        // elo = 10000, threshold = 2000 → 8001 * mult * 0.02
        let result = calculate_ant_sacrifice_talisman_item(&AntSacrificeTalismanItemInput {
            item: TalismanCraftItem::EpicFragment,
            elo: 10_000.0,
            reward_mult: Decimal::from_finite(2.0),
            stage_mult: 1.5,
        });
        // 2 * 8001 * 1.5 * 0.02 = 480.06
        assert!((result.to_number() - 480.06).abs() < 1e-9);
    }
}
