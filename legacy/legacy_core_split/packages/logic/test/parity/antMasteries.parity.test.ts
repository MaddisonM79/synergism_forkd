// Parity tests for the ant-mastery cluster. Old bodies transcribed verbatim
// from packages/web_ui/src/Features/Ants/AntMasteries/.
// Sweeps cover all 9 producers × representative mastery levels (0, 1, mid,
// max) × purchased counts, plus the can-buy / get-buyable matrix across
// the ELO-required and particle-cost edges.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  antMasteryData as newAntMasteryData,
  calculateSelfSpeedFromMastery as newCalculateSelfSpeed,
  canBuyAntMastery as newCanBuy,
  getBuyableAntMasteryLevels as newGetBuyable,
  MAX_ANT_MASTERY_LEVEL as newMaxLevel
} from '../../src/mechanics/antMasteries'

// ─── Old implementations (verbatim) ───────────────────────────────────────

const oldMaxLevel = 12

interface OldAntMasteryData {
  totalELORequirements: number[]
  particleCosts: Decimal[]
  selfSpeedMultipliers: Decimal[]
  selfPowerIncrement: number
}

const oldCalculateSelfSpeed = (
  antData: OldAntMasteryData,
  masteryLevel: number,
  purchased: number
): Decimal => {
  const selfPowerIncrement = masteryLevel * antData.selfPowerIncrement + 0.01 * Math.min(1, masteryLevel)
  const selfBaseMult = antData.selfSpeedMultipliers[masteryLevel]
  return Decimal.pow(1 + selfPowerIncrement, purchased).times(selfBaseMult)
}

const oldCanBuy = (
  antData: OldAntMasteryData,
  masteryLevel: number,
  maxLevel: number,
  currentELO: number,
  currentParticles: Decimal
): boolean => {
  if (masteryLevel >= maxLevel) return false
  const reqELO = antData.totalELORequirements[masteryLevel]
  const cost = antData.particleCosts[masteryLevel]
  return currentELO >= reqELO && currentParticles.gte(cost)
}

const oldGetBuyable = (
  antData: OldAntMasteryData,
  masteryLevel: number,
  maxLevel: number,
  currentELO: number,
  currentParticles: Decimal
): number => {
  let buyableLevels = 0
  while (masteryLevel + buyableLevels < maxLevel) {
    const reqELO = antData.totalELORequirements[masteryLevel + buyableLevels]
    const cost = antData.particleCosts[masteryLevel + buyableLevels]
    if (currentELO >= reqELO && currentParticles.gte(cost)) {
      buyableLevels++
    } else {
      break
    }
  }
  return buyableLevels
}

// ─── Data table structure ─────────────────────────────────────────────────

describe('parity: antMasteryData structure', () => {
  it('contains 9 entries (Workers..HolySpirit)', () => {
    expect(newAntMasteryData.length).toBe(9)
  })

  it('MAX_ANT_MASTERY_LEVEL is 12', () => {
    expect(newMaxLevel).toBe(oldMaxLevel)
  })

  for (let i = 0; i < 9; i++) {
    it(`producer ${i}: totalELORequirements has 12 entries`, () => {
      expect(newAntMasteryData[i].totalELORequirements.length).toBe(12)
    })
    it(`producer ${i}: particleCosts has 12 entries`, () => {
      expect(newAntMasteryData[i].particleCosts.length).toBe(12)
    })
    it(`producer ${i}: selfSpeedMultipliers has 13 entries (levels 0..12)`, () => {
      expect(newAntMasteryData[i].selfSpeedMultipliers.length).toBe(13)
    })
    it(`producer ${i}: selfPowerIncrement is a finite positive number`, () => {
      const inc = newAntMasteryData[i].selfPowerIncrement
      expect(Number.isFinite(inc)).toBe(true)
      expect(inc).toBeGreaterThan(0)
    })
  }
})

// Per-producer table-content sanity: spot-check first / mid / last entries.
describe('parity: antMasteryData spot-checks', () => {
  // Workers level 5 ELO requirement (transition from "free" to "costing ELO").
  it('Workers[5] ELO = 500', () => {
    expect(newAntMasteryData[0].totalELORequirements[5]).toBe(500)
  })
  // HolySpirit particleCosts: 1..12 sequence (intentionally trivial — sanity).
  it('HolySpirit particleCosts are 1..12 sequence', () => {
    for (let i = 0; i < 12; i++) {
      expect(newAntMasteryData[8].particleCosts[i].eq(new Decimal(i + 1))).toBe(true)
    }
  })
  // selfPowerIncrement: per-producer values from the legacy table.
  it.each([
    [0, 0.001],
    [1, 0.002],
    [2, 0.005],
    [3, 0.01],
    [4, 0.02],
    [5, 0.04],
    [6, 0.1],
    [7, 0.3],
    [8, 0.5]
  ])('producer %i selfPowerIncrement = %f', (i, expected) => {
    expect(newAntMasteryData[i].selfPowerIncrement).toBe(expected)
  })
})

// ─── calculateSelfSpeedFromMastery parity ─────────────────────────────────

describe('parity: calculateSelfSpeedFromMastery', () => {
  const masteryLevels = [0, 1, 5, 11, 12] // includes 0 (no +0.01), 1 (gets +0.01), mid, last-buyable, cap
  const purchasedCounts = [0, 1, 10, 100, 1000]

  for (let producer = 0; producer < 9; producer++) {
    const antData = newAntMasteryData[producer]
    for (const masteryLevel of masteryLevels) {
      for (const purchased of purchasedCounts) {
        it(`producer=${producer} mastery=${masteryLevel} purchased=${purchased}`, () => {
          const newRes = newCalculateSelfSpeed({ antData, masteryLevel, purchased })
          const oldRes = oldCalculateSelfSpeed(antData, masteryLevel, purchased)
          expect(newRes.eq(oldRes)).toBe(true)
        })
      }
    }
  }
})

// ─── canBuyAntMastery parity ──────────────────────────────────────────────

describe('parity: canBuyAntMastery', () => {
  // Construct test cases that exercise: at-cap, ELO-blocked, particle-blocked,
  // both-pass, both-fail.
  const producers = [0, 4, 8] // Workers, Queens, HolySpirit
  for (const producer of producers) {
    const antData = newAntMasteryData[producer]
    const cases = [
      // at cap → never buyable
      { masteryLevel: 12, currentELO: 1e18, currentParticles: new Decimal('1e100'), label: 'at cap' },
      // mastery 0, both pass (huge balances)
      { masteryLevel: 0, currentELO: 1e18, currentParticles: new Decimal('1e10000000000'), label: 'mastery 0 rich' },
      // mastery 0, ELO blocked
      { masteryLevel: 0, currentELO: 0, currentParticles: new Decimal('1e10000000000'), label: 'mastery 0 elo blocked' },
      // mastery 5, particle blocked
      { masteryLevel: 5, currentELO: 1e18, currentParticles: new Decimal('0'), label: 'mastery 5 particle blocked' },
      // mastery 5, both pass
      { masteryLevel: 5, currentELO: 1e18, currentParticles: new Decimal('1e10000000000'), label: 'mastery 5 rich' }
    ]
    for (const c of cases) {
      it(`producer=${producer} ${c.label}`, () => {
        const newRes = newCanBuy({
          antData,
          masteryLevel: c.masteryLevel,
          maxLevel: newMaxLevel,
          currentELO: c.currentELO,
          currentParticles: c.currentParticles
        })
        const oldRes = oldCanBuy(antData, c.masteryLevel, oldMaxLevel, c.currentELO, c.currentParticles)
        expect(newRes).toBe(oldRes)
      })
    }
  }
})

// ─── getBuyableAntMasteryLevels parity ────────────────────────────────────

describe('parity: getBuyableAntMasteryLevels', () => {
  const producers = [0, 4, 8]
  for (const producer of producers) {
    const antData = newAntMasteryData[producer]
    const cases = [
      // mastery 0, no resources → 0 buyable
      { masteryLevel: 0, currentELO: 0, currentParticles: new Decimal('0'), label: 'no resources' },
      // mastery 0, infinite resources → 12 buyable
      {
        masteryLevel: 0,
        currentELO: Number.POSITIVE_INFINITY,
        currentParticles: new Decimal('1e100000000000'),
        label: 'infinite resources'
      },
      // at cap → 0 buyable
      {
        masteryLevel: 12,
        currentELO: Number.POSITIVE_INFINITY,
        currentParticles: new Decimal('1e100000000000'),
        label: 'at cap'
      },
      // mid mastery, infinite resources → can buy remaining levels
      {
        masteryLevel: 6,
        currentELO: Number.POSITIVE_INFINITY,
        currentParticles: new Decimal('1e100000000000'),
        label: 'mid mastery infinite resources'
      },
      // ELO-limited (only first N levels achievable)
      {
        masteryLevel: 0,
        currentELO: antData.totalELORequirements[3],
        currentParticles: new Decimal('1e100000000000'),
        label: 'ELO at level-3 threshold'
      },
      // particle-limited at a specific level
      {
        masteryLevel: 0,
        currentELO: Number.POSITIVE_INFINITY,
        currentParticles: antData.particleCosts[2],
        label: 'particles exactly cover level-3 cost'
      }
    ]
    for (const c of cases) {
      it(`producer=${producer} ${c.label}`, () => {
        const newRes = newGetBuyable({
          antData,
          masteryLevel: c.masteryLevel,
          maxLevel: newMaxLevel,
          currentELO: c.currentELO,
          currentParticles: c.currentParticles
        })
        const oldRes = oldGetBuyable(antData, c.masteryLevel, oldMaxLevel, c.currentELO, c.currentParticles)
        expect(newRes).toBe(oldRes)
      })
    }
  }
})
