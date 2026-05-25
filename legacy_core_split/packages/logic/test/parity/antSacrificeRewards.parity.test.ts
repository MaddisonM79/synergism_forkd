// Parity test for the ant-sacrifice talisman-item reward formula. Old body
// transcribed verbatim from
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/TalismanCraftItems/
//     calculate-talisman-items.ts
//
// Plus the two constant tables (talismanItemRequiredELO,
// talismanRewardMultipliers).

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  calculateAntSacrificeTalismanItem as newCalc,
  type TalismanCraftItem,
  talismanItemRequiredELO as newRequiredELO,
  talismanRewardMultipliers as newMults
} from '../../src/mechanics/antSacrificeRewards'

// ─── Old constants (verbatim) ─────────────────────────────────────────────

const oldRequiredELO: Record<TalismanCraftItem, number> = {
  shard: 0,
  commonFragment: 300,
  uncommonFragment: 600,
  rareFragment: 1200,
  epicFragment: 2000,
  legendaryFragment: 7500,
  mythicalFragment: 7500
}

const oldMults: Record<TalismanCraftItem, number> = {
  shard: 1,
  commonFragment: 0.4,
  uncommonFragment: 0.1,
  rareFragment: 0.06,
  epicFragment: 0.02,
  legendaryFragment: 0.0008,
  mythicalFragment: 0.0001
}

const oldCalc = (
  item: TalismanCraftItem,
  elo: number,
  rewardMult: Decimal,
  stageMult: number
): Decimal => {
  if (elo < oldRequiredELO[item]) {
    return Decimal.fromString('0')
  }
  return Decimal.fromDecimal(rewardMult)
    .times(elo - oldRequiredELO[item] + 1)
    .times(stageMult)
    .times(oldMults[item])
}

const items: TalismanCraftItem[] = [
  'shard',
  'commonFragment',
  'uncommonFragment',
  'rareFragment',
  'epicFragment',
  'legendaryFragment',
  'mythicalFragment'
]

describe('parity: talismanItemRequiredELO', () => {
  for (const item of items) {
    it(item, () => {
      expect(newRequiredELO[item]).toBe(oldRequiredELO[item])
    })
  }
})

describe('parity: talismanRewardMultipliers', () => {
  for (const item of items) {
    it(item, () => {
      expect(newMults[item]).toBe(oldMults[item])
    })
  }
})

describe('parity: calculateAntSacrificeTalismanItem', () => {
  // Sweep: below threshold (returns 0), exactly at threshold, above threshold,
  // across multiple stageMult / rewardMult values.
  const rewardMults = [new Decimal(1), new Decimal('1e10'), new Decimal('1e100')]
  const stageMults = [1, 1.05, 2, 10]
  const eloOffsets = [-1, 0, 1, 100, 10_000]

  for (const item of items) {
    const threshold = newRequiredELO[item]
    for (const rewardMult of rewardMults) {
      for (const stageMult of stageMults) {
        for (const offset of eloOffsets) {
          const elo = threshold + offset
          it(`item=${item} elo=${elo} rewardMult=${rewardMult.toString()} stageMult=${stageMult}`, () => {
            const newRes = newCalc({ item, elo, rewardMult, stageMult })
            const oldRes = oldCalc(item, elo, rewardMult, stageMult)
            expect(newRes.eq(oldRes)).toBe(true)
          })
        }
      }
    }
  }
})
