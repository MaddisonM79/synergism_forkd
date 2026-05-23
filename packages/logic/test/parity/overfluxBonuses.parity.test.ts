// Parity tests for the overflux-derived multipliers lifted from
// packages/web_ui/src/Calculate.ts. Powder grids hit the 10000 boundary;
// the cube-to-quark orbs sweep crosses every singularity-unlock threshold
// (1, 2, 5, 10, 15, 20, 25, 30, 35) and toggles `autoWarpCheck`.

import { describe, expect, it } from 'vitest'
import {
  calculateCubeMultFromPowder as newCubeMultPowder,
  calculateCubeQuarkMultiplier as newCubeQuark,
  calculateQuarkMultFromPowder as newQuarkMultPowder
} from '../../src/mechanics/overfluxBonuses'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

const oldSigmoid = (constant: number, factor: number, divisor: number): number => {
  return 1 + (constant - 1) * (1 - Math.pow(2, -factor / divisor))
}

const oldCubeMultPowder = (overfluxPowder: number): number => {
  return overfluxPowder > 10000
    ? 1 + (1 / 16) * Math.pow(Math.log10(overfluxPowder), 2)
    : 1 + (1 / 10000) * overfluxPowder
}

const oldQuarkMultPowder = (overfluxPowder: number): number => {
  return overfluxPowder > 10000
    ? 1 + (1 / 40) * Math.log10(overfluxPowder)
    : 1 + (1 / 100000) * overfluxPowder
}

const oldCubeQuark = (
  overfluxOrbs: number,
  highestSingularityCount: number,
  cubeToQuarkAllMult: number,
  autoWarpCheck: boolean,
  dailyPowderResetUses: number
): number => {
  return (
    (oldSigmoid(2, Math.pow(overfluxOrbs, 0.5), 40)
      + oldSigmoid(1.5, Math.pow(overfluxOrbs, 0.5), 160)
      + oldSigmoid(1.5, Math.pow(overfluxOrbs, 0.5), 640)
      + oldSigmoid(1.15, +(highestSingularityCount >= 1) * Math.pow(overfluxOrbs, 0.45), 2560)
      + oldSigmoid(1.15, +(highestSingularityCount >= 2) * Math.pow(overfluxOrbs, 0.4), 10000)
      + oldSigmoid(1.25, +(highestSingularityCount >= 5) * Math.pow(overfluxOrbs, 0.35), 40000)
      + oldSigmoid(1.25, +(highestSingularityCount >= 10) * Math.pow(overfluxOrbs, 0.32), 160000)
      + oldSigmoid(1.35, +(highestSingularityCount >= 15) * Math.pow(overfluxOrbs, 0.27), 640000)
      + oldSigmoid(1.45, +(highestSingularityCount >= 20) * Math.pow(overfluxOrbs, 0.24), 2e6)
      + oldSigmoid(1.55, +(highestSingularityCount >= 25) * Math.pow(overfluxOrbs, 0.21), 1e7)
      + oldSigmoid(1.85, +(highestSingularityCount >= 30) * Math.pow(overfluxOrbs, 0.18), 4e7)
      + oldSigmoid(3, +(highestSingularityCount >= 35) * Math.pow(overfluxOrbs, 0.15), 1e8)
      - 11)
    * cubeToQuarkAllMult
    * (autoWarpCheck ? 1 + dailyPowderResetUses : 1)
  )
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateCubeMultFromPowder', () => {
  const grid = [0, 1, 100, 1000, 9999, 10000, 10001, 1e5, 1e7, 1e10]
  it.each(grid)('powder=%s', (powder) => {
    expect(closeEnough(newCubeMultPowder(powder), oldCubeMultPowder(powder))).toBe(true)
  })
})

describe('parity: calculateQuarkMultFromPowder', () => {
  const grid = [0, 1, 100, 1000, 9999, 10000, 10001, 1e5, 1e7, 1e10]
  it.each(grid)('powder=%s', (powder) => {
    expect(closeEnough(newQuarkMultPowder(powder), oldQuarkMultPowder(powder))).toBe(true)
  })
})

describe('parity: calculateCubeQuarkMultiplier', () => {
  // Orbs grid — spans a few decades; small values hit the early sigmoids,
  // large values exercise all 12 contributors.
  const orbsGrid = [0, 1, 10, 100, 1000, 1e4, 1e6, 1e8, 1e10]
  // Singularity grid hits every gate boundary and just above/below.
  const highGrid = [0, 1, 2, 4, 5, 9, 10, 14, 15, 19, 20, 24, 25, 29, 30, 34, 35, 100]
  const cubeToQuarkAllGrid = [1, 1.25, 2]
  const autoWarpGrid = [true, false]
  const dailyPowderGrid = [0, 1, 5]

  for (const cubeToQuarkAllMult of cubeToQuarkAllGrid) {
    for (const autoWarpCheck of autoWarpGrid) {
      for (const dailyPowderResetUses of dailyPowderGrid) {
        for (const high of highGrid) {
          it.each(orbsGrid)(
            `c2q=${cubeToQuarkAllMult} warp=${autoWarpCheck} daily=${dailyPowderResetUses} high=${high} orbs=%s`,
            (orbs) => {
              const next = newCubeQuark({
                overfluxOrbs: orbs,
                highestSingularityCount: high,
                cubeToQuarkAllMult,
                autoWarpCheck,
                dailyPowderResetUses
              })
              const old = oldCubeQuark(orbs, high, cubeToQuarkAllMult, autoWarpCheck, dailyPowderResetUses)
              expect(closeEnough(next, old)).toBe(true)
            }
          )
        }
      }
    }
  }
})
