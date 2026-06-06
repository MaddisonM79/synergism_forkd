//! Talisman rarity / level math.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/talismanLevels.ts`.
//! The talisman data table (i18n closures, `isUnlocked` predicates,
//! `baseMult` / `maxLevel` constants) stays in the UI tier — this
//! module owns the pure formulas that map `(level, max_level,
//! rarity)` → display rarity, the levels-until-next-rarity counter,
//! and the per-level affordability check.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use super::talisman_costs::TalismanCraftCosts;
use crate::events::CoreEvent;
use crate::state::TalismansState;

// ─── Rarity value table ────────────────────────────────────────────────────

/// Rune-bonus multiplier for the given rarity tier. Rarity `0` means
/// "talisman locked" and contributes nothing; rarities `1..=10` are
/// the displayed rarity tiers (Common → Mythic), with stacking
/// bonuses above rarity `7` awarded by crossing `2×`, `4×`, `8×` the
/// talisman's `max_level`.
///
/// Out-of-range inputs return `0` (matching the legacy
/// `Record<number, number>` fall-through).
#[must_use]
pub fn rarity_value(rarity: u8) -> f64 {
    match rarity {
        0 => 0.0,
        1 => 1.0,
        2 => 1.2,
        3 => 1.5,
        4 => 1.8,
        5 => 2.1,
        6 => 2.5,
        7 => 3.0,
        8 => 3.25,
        9 => 3.5,
        10 => 4.0,
        _ => 0.0,
    }
}

// ─── Rarity from level ─────────────────────────────────────────────────────

/// Inputs to [`compute_talisman_rarity`].
#[derive(Debug, Clone, Copy)]
pub struct ComputeTalismanRarityInput {
    /// `talismans[t].isUnlocked()`. When `false`, rarity is forced to
    /// `0`.
    pub is_unlocked: bool,
    /// `talismans[t].level`.
    pub level: f64,
    /// `talismans[t].maxLevel` — **not** the cap including
    /// `levelCapIncrease`. The raw `maxLevel` constant on the talisman
    /// data table; the rarity tier formula uses ratios of this value
    /// (`level / max_level ≥ 1, 2, 4, 8`).
    pub max_level: f64,
}

/// Display rarity for a talisman, `0..=10`. Locked talismans get `0`.
/// Unlocked talismans get
/// `1 + min(6, floor(6 * level / max_level)) + extra_rarity`, where
/// `extra_rarity` adds `+1` for each of the `2×`, `4×`, `8×`
/// `max_level` thresholds the talisman has crossed.
#[must_use]
pub fn compute_talisman_rarity(input: &ComputeTalismanRarityInput) -> u8 {
    if !input.is_unlocked {
        return 0;
    }
    let level_ratio = input.level / input.max_level;
    let mut extra_rarity = 0_u8;
    if level_ratio >= 1.0 {
        if level_ratio >= 2.0 {
            extra_rarity += 1;
        }
        if level_ratio >= 4.0 {
            extra_rarity += 1;
        }
        if level_ratio >= 8.0 {
            extra_rarity += 1;
        }
    }
    let band = (6.0 * level_ratio).floor().min(6.0) as u8;
    1 + band + extra_rarity
}

// ─── Levels until next rarity tier ─────────────────────────────────────────

/// Inputs to [`levels_until_talisman_rarity_increase`].
#[derive(Debug, Clone, Copy)]
pub struct LevelsUntilTalismanRarityIncreaseInput {
    /// `talismans[t].level`.
    pub level: f64,
    /// `talismans[t].maxLevel`.
    pub max_level: f64,
    /// `talismans[t].rarity` — current rarity tier.
    pub current_rarity: f64,
    /// `getTalismanLevelCap(t)` —
    /// `maxLevel + levelCapIncrease()`.
    pub level_cap: f64,
}

/// Levels remaining until the next rarity tier triggers. Once `level`
/// reaches `max_level` the rarity stops ratcheting via the
/// level-ratio thresholds (the `2×/4×/8×` extras still fire, but this
/// helper ignores them — the UI just buys up to the cap once you're
/// past the `max_level` mark).
#[must_use]
pub fn levels_until_talisman_rarity_increase(
    input: &LevelsUntilTalismanRarityIncreaseInput,
) -> f64 {
    if input.level >= input.max_level {
        return input.level_cap - input.level;
    }
    let level_req = (input.max_level * input.current_rarity / 6.0).ceil();
    level_req - input.level
}

// ─── Affordability check ───────────────────────────────────────────────────

/// Inputs to [`affordable_talisman_level`].
#[derive(Debug, Clone, Copy)]
pub struct AffordableTalismanLevelInput {
    /// Per-item cost for the next level — output of the talisman's
    /// cost progression.
    pub costs: TalismanCraftCosts,
    /// Per-item budget available. Same fields as `costs`. For real
    /// purchases this is the player's owned fragments; during
    /// save-loading it's the saved `fragmentsInvested` snapshot.
    pub budget: TalismanCraftCosts,
    /// Floating-point cushion applied to the budget. The legacy
    /// caller uses `1.0001` when re-deriving level from invested
    /// fragments after a save load (compensating for `Decimal`
    /// round-trip imprecision); `1.0` for live purchases.
    pub buffer_mult: f64,
}

/// Returns `true` iff every item in `costs` is `≤ budget[item] *
/// buffer_mult`. Walks the seven cost fields directly so unused tiers
/// (zero cost) don't affect the result — they're trivially satisfied.
#[must_use]
pub fn affordable_talisman_level(input: &AffordableTalismanLevelInput) -> bool {
    let buffer = Decimal::from_finite(input.buffer_mult);
    if input.costs.shard > input.budget.shard * buffer {
        return false;
    }
    if input.costs.common_fragment > input.budget.common_fragment * buffer {
        return false;
    }
    if input.costs.uncommon_fragment > input.budget.uncommon_fragment * buffer {
        return false;
    }
    if input.costs.rare_fragment > input.budget.rare_fragment * buffer {
        return false;
    }
    if input.costs.epic_fragment > input.budget.epic_fragment * buffer {
        return false;
    }
    if input.costs.legendary_fragment > input.budget.legendary_fragment * buffer {
        return false;
    }
    if input.costs.mythical_fragment > input.budget.mythical_fragment * buffer {
        return false;
    }
    true
}

// ─── Manual buy ────────────────────────────────────────────────────────────

/// Inputs to [`buy_talisman_level`].
#[derive(Debug, Clone, Copy)]
pub struct BuyTalismanLevelInput {
    /// Talisman index (`0..7`, via the `TALISMAN_*` constants). Out-of-range
    /// is a no-op.
    pub index: usize,
    /// Per-item cost for the next level — the caller evaluates the talisman's
    /// cost progression ([`regular_cost_progression`] /
    /// [`exponential_cost_progression`]) for the current level. The per-talisman
    /// `baseMult` / `ratio` / which-progression are UI-tier data.
    ///
    /// [`regular_cost_progression`]: super::talisman_costs::regular_cost_progression
    /// [`exponential_cost_progression`]: super::talisman_costs::exponential_cost_progression
    pub costs: TalismanCraftCosts,
    /// `getTalismanLevelCap` = `maxLevel + levelCapIncrease()` — the purchase
    /// cap (UI-tier). The buy stops at `level == level_cap`.
    pub level_cap: f64,
}

/// Buy one talisman level for talisman `index` by spending talisman shards and
/// the six fragment tiers — a port of the legacy `buyTalismanLevel`. The cost
/// map is caller-provided (evaluated from the ported cost progression for the
/// current level); affordability uses [`affordable_talisman_level`]. Emits
/// [`CoreEvent::TalismanLevelPurchased`].
///
/// Faithful-at-current-state deferrals:
/// - **unlock** (`isUnlocked()`) is UI-tier, so the caller gates on it; this
///   buy is ungated on unlock;
/// - the rarity recompute (`setTalismanRarity`) reads UI-tier `isUnlocked`
///   (rarity is `0` while locked, the reachable state today), so it is left to
///   the UI — `talisman_rarity` is not updated here;
/// - `fragmentsInvested` respec tracking has no logic-state field, so it is
///   not accumulated;
/// - the resource pools are `f64` in state, so the `Decimal` costs are widened
///   for the affordability check and narrowed on the spend.
#[must_use]
pub fn buy_talisman_level(
    talismans: &mut TalismansState,
    input: BuyTalismanLevelInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    if input.index >= talismans.talisman_levels.len() {
        return events;
    }
    let before = talismans.talisman_levels[input.index];
    if before >= input.level_cap {
        return events;
    }

    let budget = TalismanCraftCosts {
        shard: Decimal::from_finite(talismans.talisman_shards),
        common_fragment: Decimal::from_finite(talismans.common_fragments),
        uncommon_fragment: Decimal::from_finite(talismans.uncommon_fragments),
        rare_fragment: Decimal::from_finite(talismans.rare_fragments),
        epic_fragment: Decimal::from_finite(talismans.epic_fragments),
        legendary_fragment: Decimal::from_finite(talismans.legendary_fragments),
        mythical_fragment: Decimal::from_finite(talismans.mythical_fragments),
    };
    if !affordable_talisman_level(&AffordableTalismanLevelInput {
        costs: input.costs,
        budget,
        buffer_mult: 1.0,
    }) {
        return events;
    }

    talismans.talisman_shards -= input.costs.shard.to_number();
    talismans.common_fragments -= input.costs.common_fragment.to_number();
    talismans.uncommon_fragments -= input.costs.uncommon_fragment.to_number();
    talismans.rare_fragments -= input.costs.rare_fragment.to_number();
    talismans.epic_fragments -= input.costs.epic_fragment.to_number();
    talismans.legendary_fragments -= input.costs.legendary_fragment.to_number();
    talismans.mythical_fragments -= input.costs.mythical_fragment.to_number();
    talismans.talisman_levels[input.index] += 1.0;

    events.push(CoreEvent::TalismanLevelPurchased {
        index: input.index as u32,
        before,
        after: talismans.talisman_levels[input.index],
    });
    events
}

// ─── Sum of rarities ───────────────────────────────────────────────────────

/// Sum of all talisman rarities. Used by the achievement-points
/// formula for the rarity-based progressive achievement. Trivial fold,
/// lifted to logic so the UI side doesn't have to iterate the
/// talismans map every time.
#[must_use]
pub fn sum_of_talisman_rarities(rarities: &[u8]) -> f64 {
    rarities.iter().map(|&r| f64::from(r)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rarity_value_locked_is_zero() {
        assert_eq!(rarity_value(0), 0.0);
    }

    #[test]
    fn rarity_value_mythic_is_4() {
        assert_eq!(rarity_value(10), 4.0);
    }

    #[test]
    fn rarity_value_out_of_range_is_zero() {
        assert_eq!(rarity_value(11), 0.0);
        assert_eq!(rarity_value(255), 0.0);
    }

    #[test]
    fn compute_rarity_locked_returns_zero() {
        let input = ComputeTalismanRarityInput {
            is_unlocked: false,
            level: 1_000.0,
            max_level: 100.0,
        };
        assert_eq!(compute_talisman_rarity(&input), 0);
    }

    #[test]
    fn compute_rarity_at_max_level_is_7() {
        // level / max_level = 1; floor(6*1) = 6; 1 + 6 = 7. extra_rarity = 0
        // (we're at 1×, not 2×).
        let input = ComputeTalismanRarityInput {
            is_unlocked: true,
            level: 100.0,
            max_level: 100.0,
        };
        assert_eq!(compute_talisman_rarity(&input), 7);
    }

    #[test]
    fn compute_rarity_at_2x_max_adds_one() {
        let input = ComputeTalismanRarityInput {
            is_unlocked: true,
            level: 200.0,
            max_level: 100.0,
        };
        // band = min(6, floor(12)) = 6; extra = 1 (>= 2×); 1 + 6 + 1 = 8
        assert_eq!(compute_talisman_rarity(&input), 8);
    }

    #[test]
    fn compute_rarity_at_8x_max_adds_three() {
        let input = ComputeTalismanRarityInput {
            is_unlocked: true,
            level: 800.0,
            max_level: 100.0,
        };
        // band = 6; extra = 3 (2× + 4× + 8×); 1 + 6 + 3 = 10
        assert_eq!(compute_talisman_rarity(&input), 10);
    }

    #[test]
    fn levels_until_rarity_increase_above_max_returns_cap_minus_level() {
        let input = LevelsUntilTalismanRarityIncreaseInput {
            level: 150.0,
            max_level: 100.0,
            current_rarity: 7.0,
            level_cap: 200.0,
        };
        assert_eq!(levels_until_talisman_rarity_increase(&input), 50.0);
    }

    #[test]
    fn levels_until_rarity_increase_below_max_uses_ratio_step() {
        // max_level = 60, current_rarity = 1 → level_req = ceil(60 * 1 / 6) = 10
        // level = 5 → need 5 more
        let input = LevelsUntilTalismanRarityIncreaseInput {
            level: 5.0,
            max_level: 60.0,
            current_rarity: 1.0,
            level_cap: 60.0,
        };
        assert_eq!(levels_until_talisman_rarity_increase(&input), 5.0);
    }

    fn zero_costs() -> TalismanCraftCosts {
        TalismanCraftCosts {
            shard: Decimal::zero(),
            common_fragment: Decimal::zero(),
            uncommon_fragment: Decimal::zero(),
            rare_fragment: Decimal::zero(),
            epic_fragment: Decimal::zero(),
            legendary_fragment: Decimal::zero(),
            mythical_fragment: Decimal::zero(),
        }
    }

    #[test]
    fn affordable_zero_cost_is_affordable() {
        let input = AffordableTalismanLevelInput {
            costs: zero_costs(),
            budget: zero_costs(),
            buffer_mult: 1.0,
        };
        assert!(affordable_talisman_level(&input));
    }

    #[test]
    fn affordable_higher_cost_than_budget_fails() {
        let costs = TalismanCraftCosts {
            shard: Decimal::from_finite(100.0),
            ..zero_costs()
        };
        let budget = TalismanCraftCosts {
            shard: Decimal::from_finite(99.0),
            ..zero_costs()
        };
        let input = AffordableTalismanLevelInput {
            costs,
            budget,
            buffer_mult: 1.0,
        };
        assert!(!affordable_talisman_level(&input));
    }

    #[test]
    fn affordable_buffer_mult_extends_budget() {
        let costs = TalismanCraftCosts {
            shard: Decimal::from_finite(100.0),
            ..zero_costs()
        };
        let budget = TalismanCraftCosts {
            shard: Decimal::from_finite(99.0),
            ..zero_costs()
        };
        // budget * 1.02 = 100.98 → 100 fits
        let input = AffordableTalismanLevelInput {
            costs,
            budget,
            buffer_mult: 1.02,
        };
        assert!(affordable_talisman_level(&input));
    }

    #[test]
    fn sum_of_talisman_rarities_is_simple_sum() {
        let rarities = [1_u8, 3, 5, 7, 10];
        assert_eq!(sum_of_talisman_rarities(&rarities), 26.0);
    }

    // ─── Manual buy ───────────────────────────────────────────────────────

    /// A cost map of `shard` shards + `common` common-fragments, the rest 0.
    fn cost_map(shard: f64, common: f64) -> TalismanCraftCosts {
        TalismanCraftCosts {
            shard: Decimal::from_finite(shard),
            common_fragment: Decimal::from_finite(common),
            uncommon_fragment: Decimal::zero(),
            rare_fragment: Decimal::zero(),
            epic_fragment: Decimal::zero(),
            legendary_fragment: Decimal::zero(),
            mythical_fragment: Decimal::zero(),
        }
    }

    fn talismans_with(shards: f64, common: f64) -> TalismansState {
        TalismansState {
            talisman_shards: shards,
            common_fragments: common,
            ..TalismansState::default()
        }
    }

    #[test]
    fn buy_talisman_level_spends_and_levels() {
        // Talisman 0 (Exemption) at level 0, next level costs 10 shards.
        let mut t = talismans_with(100.0, 0.0);
        let events = buy_talisman_level(
            &mut t,
            BuyTalismanLevelInput {
                index: 0,
                costs: cost_map(10.0, 0.0),
                level_cap: 100.0,
            },
        );
        assert_eq!(t.talisman_levels[0], 1.0);
        assert!((t.talisman_shards - 90.0).abs() < 1e-9);
        assert!(matches!(
            events.as_slice(),
            [CoreEvent::TalismanLevelPurchased { index: 0, .. }]
        ));
    }

    #[test]
    fn buy_talisman_level_unaffordable_is_noop() {
        let mut t = talismans_with(5.0, 0.0);
        let events = buy_talisman_level(
            &mut t,
            BuyTalismanLevelInput {
                index: 0,
                costs: cost_map(10.0, 0.0),
                level_cap: 100.0,
            },
        );
        assert_eq!(t.talisman_levels[0], 0.0);
        assert_eq!(t.talisman_shards, 5.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_talisman_level_multi_fragment_spends_all() {
        // 10 shards + 20 common fragments; budget covers both.
        let mut t = talismans_with(100.0, 50.0);
        let events = buy_talisman_level(
            &mut t,
            BuyTalismanLevelInput {
                index: 0,
                costs: cost_map(10.0, 20.0),
                level_cap: 100.0,
            },
        );
        assert_eq!(t.talisman_levels[0], 1.0);
        assert!((t.talisman_shards - 90.0).abs() < 1e-9);
        assert!((t.common_fragments - 30.0).abs() < 1e-9);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_talisman_level_maxed_is_noop() {
        let mut t = talismans_with(1e9, 0.0);
        t.talisman_levels[0] = 100.0;
        let events = buy_talisman_level(
            &mut t,
            BuyTalismanLevelInput {
                index: 0,
                costs: cost_map(10.0, 0.0),
                level_cap: 100.0,
            },
        );
        assert_eq!(t.talisman_levels[0], 100.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_talisman_level_out_of_range_is_noop() {
        let mut t = talismans_with(1e9, 0.0);
        let out_of_range = t.talisman_levels.len();
        let events = buy_talisman_level(
            &mut t,
            BuyTalismanLevelInput {
                index: out_of_range,
                costs: cost_map(10.0, 0.0),
                level_cap: 100.0,
            },
        );
        assert!(events.is_empty());
    }
}
