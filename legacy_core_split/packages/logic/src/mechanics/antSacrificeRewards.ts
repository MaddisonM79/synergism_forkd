// Pure reward formulas for ant sacrifice. Lifted from:
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/
//     TalismanCraftItems/constants.ts
//     TalismanCraftItems/calculate-talisman-items.ts
//
// The offering / obtainium calculators stay in web_ui because they reduce a
// Statistics stat array (offeringObtainiumTimeModifiers) — display-coupled.
// The talisman-item calculator is fully pure given its inputs and moves here.

import { Decimal } from '../math/bignum'

/** Fixed list of craftable talisman fragment / shard items. Matches the
 * TalismanCraftItems union in web_ui's Talismans.ts. */
export type TalismanCraftItem =
  | 'shard'
  | 'commonFragment'
  | 'uncommonFragment'
  | 'rareFragment'
  | 'epicFragment'
  | 'legendaryFragment'
  | 'mythicalFragment'

/** Minimum effective ant ELO required for each talisman item to drop. */
export const talismanItemRequiredELO: Record<TalismanCraftItem, number> = {
  shard: 0,
  commonFragment: 300,
  uncommonFragment: 600,
  rareFragment: 1200,
  epicFragment: 2000,
  legendaryFragment: 7500,
  mythicalFragment: 7500
}

/** Per-item multiplier applied to the (elo − threshold + 1) × rewardMult ×
 * stageMult product. Lower-tier items drop in larger quantities. */
export const talismanRewardMultipliers: Record<TalismanCraftItem, number> = {
  shard: 1,
  commonFragment: 0.4,
  uncommonFragment: 0.1,
  rareFragment: 0.06,
  epicFragment: 0.02,
  legendaryFragment: 0.0008,
  mythicalFragment: 0.0001
}

export interface AntSacrificeTalismanItemInput {
  item: TalismanCraftItem
  /** Effective ant ELO at the sacrifice. */
  elo: number
  /** Reward-multiplier (antSacrificeMult × timeMultiplier — caller computes). */
  rewardMult: Decimal
  /** Per-stage talisman-fragment modifier from the reborn-ELO stage system. */
  stageMult: number
}

/**
 * Quantity of a specific talisman craft item awarded by a sacrifice.
 * Returns 0 if ELO is below the item's threshold. Otherwise:
 *   rewardMult × (elo − threshold + 1) × stageMult × talismanRewardMultipliers[item]
 */
export function calculateAntSacrificeTalismanItem (input: AntSacrificeTalismanItemInput): Decimal {
  const required = talismanItemRequiredELO[input.item]
  if (input.elo < required) {
    return Decimal.fromString('0')
  }
  return Decimal.fromDecimal(input.rewardMult)
    .times(input.elo - required + 1)
    .times(input.stageMult)
    .times(talismanRewardMultipliers[input.item])
}
