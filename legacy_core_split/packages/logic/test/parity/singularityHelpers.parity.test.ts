// Parity tests for the three singularity helpers, lifted from
// packages/web_ui/src/singularity.ts. Sweeps cover:
//   - maxSingularityLookahead: nonZero false vs true × non-zero
//     lookahead contributions
//   - goldenQuarkCost: below / at / above the 10000-GQ baseline
//   - calculateNextSpike: every singularityPenaltyThresholds boundary
//     × reduction offset

import { describe, expect, it } from 'vitest'
import {
  calculateNextSpike as newNextSpike,
  goldenQuarkCost as newGqCost,
  maxSingularityLookahead as newLookahead
} from '../../src/mechanics/singularityHelpers'

// ─── Old implementations (verbatim from packages/web_ui/src/singularity.ts) ─

const oldSingularityPenaltyThresholds = [11, 26, 37, 51, 101, 151, 201, 216, 230, 270]
const OLD_GOLDEN_QUARK_BASE_COST = 10000

interface OldLookaheadInput {
  nonZero: boolean
  singFastForwardLookahead: number
  singFastForward2Lookahead: number
  octeractFastForwardLookahead: number
}

const oldLookahead = (input: OldLookaheadInput): number => {
  if (!input.nonZero) {
    return 0
  }
  let maxLookahead = 1
  maxLookahead += input.singFastForwardLookahead
  maxLookahead += input.singFastForward2Lookahead
  maxLookahead += input.octeractFastForwardLookahead
  return maxLookahead
}

const oldGqCost = (cost: number) => ({
  cost,
  costReduction: Math.max(0, OLD_GOLDEN_QUARK_BASE_COST - cost)
})

interface OldNextSpikeInput {
  singularityCount: number
  singularityReductions: number
}

const oldNextSpike = (input: OldNextSpikeInput): number => {
  for (const sing of oldSingularityPenaltyThresholds) {
    if (sing + input.singularityReductions > input.singularityCount) {
      return sing + input.singularityReductions
    }
  }
  return -1
}

// ─── maxSingularityLookahead ──────────────────────────────────────────────

describe('parity: maxSingularityLookahead', () => {
  // nonZero=false short-circuits to 0 regardless of bonuses
  for (const bonus of [[0, 0, 0], [1, 1, 1], [5, 3, 2]]) {
    it(`nonZero=false bonuses=${bonus.join(',')}`, () => {
      const input = {
        nonZero: false,
        singFastForwardLookahead: bonus[0],
        singFastForward2Lookahead: bonus[1],
        octeractFastForwardLookahead: bonus[2]
      }
      expect(newLookahead(input)).toBe(oldLookahead(input))
    })
  }
  // nonZero=true sums everything
  for (const fwd1 of [0, 1, 5]) {
    for (const fwd2 of [0, 1, 5]) {
      for (const oct of [0, 1, 3]) {
        it(`nonZero=true fwd1=${fwd1} fwd2=${fwd2} oct=${oct}`, () => {
          const input = {
            nonZero: true,
            singFastForwardLookahead: fwd1,
            singFastForward2Lookahead: fwd2,
            octeractFastForwardLookahead: oct
          }
          expect(newLookahead(input)).toBe(oldLookahead(input))
        })
      }
    }
  }
})

// ─── goldenQuarkCost ──────────────────────────────────────────────────────

describe('parity: goldenQuarkCost', () => {
  const costGrid = [0, 1, 100, 1000, 5000, 9999, 10000, 10001, 50000, 1e6]
  for (const cost of costGrid) {
    it(`cost=${cost}`, () => {
      const next = newGqCost(cost)
      const old = oldGqCost(cost)
      expect(next.cost).toBe(old.cost)
      expect(next.costReduction).toBe(old.costReduction)
    })
  }
})

// ─── calculateNextSpike ───────────────────────────────────────────────────

describe('parity: calculateNextSpike', () => {
  // Sweep every threshold boundary in both directions for several reduction
  // values. -1 happens past the last threshold.
  const singGrid = [
    0,
    10,
    11,
    12,
    25,
    26,
    27,
    36,
    37,
    38,
    50,
    51,
    52,
    100,
    101,
    102,
    150,
    151,
    152,
    200,
    201,
    202,
    215,
    216,
    217,
    229,
    230,
    231,
    269,
    270,
    271,
    300
  ]
  const reductionGrid = [0, 1, 5, 10]
  for (const reductions of reductionGrid) {
    for (const sing of singGrid) {
      it(`sing=${sing} reductions=${reductions}`, () => {
        const input = { singularityCount: sing, singularityReductions: reductions }
        expect(newNextSpike(input)).toBe(oldNextSpike(input))
      })
    }
  }
})
