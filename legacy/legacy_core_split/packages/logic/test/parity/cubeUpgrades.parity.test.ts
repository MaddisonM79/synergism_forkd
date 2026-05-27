// Parity tests for getCubeMax + getCubeCost migrated from
// packages/web_ui/src/Cubes.ts. The OLD implementations are transcribed
// below with player/G reads lifted into explicit parameters.

import { describe, expect, it } from 'vitest'
import {
  calculateCubicSumData,
  calculateSummationNonLinear
} from '../../src/math/summations'
import {
  getCubeCost as newGetCubeCost,
  getCubeMax as newGetCubeMax,
  getCubeUpgradeBaseCost as newBaseCost
} from '../../src/mechanics/cubeUpgrades'

// dprint-ignore
const oldCubeBaseCost = [
  200, 200, 200, 500, 500, 500, 500, 500, 2000, 40000,
  5000, 1000, 10000, 20000, 40000, 10000, 4000, 1e4, 50000, 12500,
  5e4, 3e4, 3e4, 4e4, 2e5, 4e5, 1e5, 177777, 1e5, 1e6,
  5e5, 3e5, 2e6, 4e7, 4e7, 1e8, 1e8, 1e9, 2e9, 2e8,
  2e8, 5e8, 1e9, 2e9, 2e9, 5e8, 9876543210, 1e10, 42934819467, 1e8,
  1, 1e4, 1e8, 1e12, 1e16, 10, 1e5, 1e9, 1e13, 1e17,
  1e2, 1e6, 1e10, 1e14, 1e18, 1e20, 1e30, 1e40, 1e50, 1e60,
  1, 1, 1e8, 1e16, 1e30, 1e100, 1e100, 1e200, 1e250, 1e300
]

// dprint-ignore
const oldCubeMaxLevel = [
  3, 10, 5, 1, 1, 1, 1, 1, 1, 1,
  3, 10, 1, 10, 10, 10, 5, 1, 1, 1,
  5, 10, 1, 10, 10, 10, 1, 1, 5, 1,
  5, 1, 1, 10, 10, 10, 10, 1, 1, 10,
  5, 10, 10, 10, 10, 20, 1, 1, 1, 100000,
  1, 900, 100, 900, 900, 20, 1, 1, 400, 10000,
  100, 1, 1, 1, 1, 1, 1, 1000, 1, 100000,
  1, 1, 5, 1, 30, 2, 25, 30, 1, 1
]

const oldGetCubeMax = (i: number, cubeUpgrade57: number): number => {
  let baseValue = oldCubeMaxLevel[i - 1]
  if (cubeUpgrade57 > 0 && i < 50 && i % 10 === 1) {
    baseValue += 1
  }
  return baseValue
}

const oldGetCubeCost = (
  i: number,
  buyMax: boolean,
  currentLevel: number,
  cubeUpgrade57: number,
  wowCubes: number,
  singularityDebuff: number
) => {
  const linGrowth = i === 50 ? 0.01 : 0
  const cubic = i > 50
  const maxLevel = oldGetCubeMax(i, cubeUpgrade57)
  let amountToBuy = buyMax ? 1e5 : 1
  amountToBuy = Math.min(maxLevel - currentLevel, amountToBuy)
  // Original collapses singularityDebuff to 1 for i > 50; the wrapper does
  // the same, so we can take the parameter as authoritative here.
  if (cubic) {
    amountToBuy = buyMax ? maxLevel : Math.min(maxLevel, currentLevel + 1)
    return calculateCubicSumData(currentLevel, oldCubeBaseCost[i - 1], wowCubes, amountToBuy)
  }
  return calculateSummationNonLinear(
    currentLevel,
    oldCubeBaseCost[i - 1] * singularityDebuff,
    wowCubes,
    linGrowth,
    amountToBuy
  )
}

// ─── getCubeMax ────────────────────────────────────────────────────────────

describe('parity: getCubeMax', () => {
  it('matches base table when cubeUpgrade57 = 0', () => {
    for (let i = 1; i <= 80; i++) {
      expect(newGetCubeMax({ cubeUpgradeIndex: i, cubeUpgrade57: 0 })).toBe(oldGetCubeMax(i, 0))
    }
  })

  it('cubeUpgrade57 > 0: +1 only on row leaders (i % 10 === 1, i < 50)', () => {
    for (let i = 1; i <= 80; i++) {
      const newVal = newGetCubeMax({ cubeUpgradeIndex: i, cubeUpgrade57: 1 })
      const oldVal = oldGetCubeMax(i, 1)
      expect(newVal).toBe(oldVal)
    }
  })

  it('row leader 1 bumps from 3 → 4 with cu57', () => {
    expect(newGetCubeMax({ cubeUpgradeIndex: 1, cubeUpgrade57: 1 })).toBe(4)
  })

  it('i=51 (above 50) does NOT bump even with cu57', () => {
    expect(newGetCubeMax({ cubeUpgradeIndex: 51, cubeUpgrade57: 1 })).toBe(1)
  })

  it('i=2 (not row leader) does NOT bump with cu57', () => {
    expect(newGetCubeMax({ cubeUpgradeIndex: 2, cubeUpgrade57: 1 })).toBe(10)
  })
})

// ─── getCubeUpgradeBaseCost ────────────────────────────────────────────────

describe('parity: getCubeUpgradeBaseCost', () => {
  it('returns the base cost table verbatim', () => {
    for (let i = 1; i <= 80; i++) {
      expect(newBaseCost(i)).toBe(oldCubeBaseCost[i - 1])
    }
  })
})

// ─── getCubeCost ───────────────────────────────────────────────────────────

describe('parity: getCubeCost — non-cubic (i ≤ 50)', () => {
  // Sample one upgrade from each of the row-1..5 tiers, plus i=50 (linear-growth).
  const indices = [1, 12, 23, 35, 50]
  const buyMaxValues = [false, true]
  const currentLevels = [0, 1, 2]
  const wowCubesValues = [0, 1e3, 1e10, 1e30]
  const debuffValues = [1, 1.5, 0.5]

  for (const i of indices) {
    for (const buyMax of buyMaxValues) {
      for (const currentLevel of currentLevels) {
        for (const wow of wowCubesValues) {
          for (const debuff of debuffValues) {
            it(`i=${i} buyMax=${buyMax} cur=${currentLevel} wow=${wow} debuff=${debuff}`, () => {
              const maxLevel = oldGetCubeMax(i, 0)
              if (currentLevel >= maxLevel) return
              const newRes = newGetCubeCost({
                cubeUpgradeIndex: i,
                buyMax,
                currentLevel,
                maxLevel,
                wowCubes: wow,
                singularityDebuff: debuff
              })
              const oldRes = oldGetCubeCost(i, buyMax, currentLevel, 0, wow, debuff)
              expect(newRes).toEqual(oldRes)
            })
          }
        }
      }
    }
  }
})

describe('parity: getCubeCost — cubic (i > 50)', () => {
  // Sample across cubic tiers including the 1e300 base cost extreme.
  const indices = [51, 60, 70, 80]
  const buyMaxValues = [false, true]
  const currentLevels = [0, 1]
  const wowCubesValues = [0, 1e10, 1e50, 1e200]

  for (const i of indices) {
    for (const buyMax of buyMaxValues) {
      for (const currentLevel of currentLevels) {
        for (const wow of wowCubesValues) {
          it(`i=${i} buyMax=${buyMax} cur=${currentLevel} wow=${wow}`, () => {
            const maxLevel = oldGetCubeMax(i, 0)
            if (currentLevel >= maxLevel) return
            const newRes = newGetCubeCost({
              cubeUpgradeIndex: i,
              buyMax,
              currentLevel,
              maxLevel,
              wowCubes: wow,
              singularityDebuff: 1 // ignored for cubic
            })
            const oldRes = oldGetCubeCost(i, buyMax, currentLevel, 0, wow, 1)
            expect(newRes).toEqual(oldRes)
          })
        }
      }
    }
  }
})

describe('parity: getCubeCost — cu57 row-leader bumps max', () => {
  it('row leader 1 with cu57 can buy past the original max', () => {
    const maxLevelWithCu57 = oldGetCubeMax(1, 1) // 4
    const newRes = newGetCubeCost({
      cubeUpgradeIndex: 1,
      buyMax: true,
      currentLevel: 3, // would have hit cap at 3 without cu57
      maxLevel: maxLevelWithCu57,
      wowCubes: 1e10,
      singularityDebuff: 1
    })
    const oldRes = oldGetCubeCost(1, true, 3, 1, 1e10, 1)
    expect(newRes).toEqual(oldRes)
  })
})
