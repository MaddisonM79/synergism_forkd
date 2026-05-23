// Parity tests for the total-octeract bonuses lifted from
// packages/web_ui/src/Calculate.ts. Sweeps across the 1000-octeract branching
// boundary, the small-value 1.00001 tolerance threshold, the octeractPow
// effect curve (the formula switches at 10 completions), and all combinations
// of the gate booleans.

import { describe, expect, it } from 'vitest'
import {
  calculateTotalOcteractCubeBonus as newCubeBonus,
  calculateTotalOcteractObtainiumBonus as newObtainiumBonus,
  calculateTotalOcteractOfferingBonus as newOfferingBonus,
  calculateTotalOcteractQuarkBonus as newQuarkBonus
} from '../../src/mechanics/octeractBonuses'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

const oldCubeBonus = (exalt4Enabled: boolean, totalWowOcteracts: number, octeractPow: number): number => {
  if (exalt4Enabled) {
    return 1
  }
  if (totalWowOcteracts < 1000) {
    const bonus = 1 + (2 / 1000) * totalWowOcteracts
    return bonus > 1.00001 ? bonus : 1
  } else {
    const power = 2 + octeractPow
    return 3 * Math.pow(Math.log10(totalWowOcteracts) - 2, power)
  }
}

const oldQuarkBonus = (exalt4Enabled: boolean, totalWowOcteracts: number): number => {
  if (exalt4Enabled) {
    return 1
  }
  if (totalWowOcteracts < 1000) {
    const bonus = 1 + (0.2 / 1000) * totalWowOcteracts
    return bonus > 1.00001 ? bonus : 1
  } else {
    return 1.1 + 0.1 * (Math.log10(totalWowOcteracts) - 2)
  }
}

const oldOfferingBonus = (offeringBonusEnabled: boolean, cubeBonus: number): number => {
  if (!offeringBonusEnabled) {
    return 1
  }
  return Math.pow(cubeBonus, 1.25)
}

const oldObtainiumBonus = (obtainiumBonusEnabled: boolean, cubeBonus: number): number => {
  if (!obtainiumBonusEnabled) {
    return 1
  }
  return Math.pow(cubeBonus, 1.25)
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Octeract count grid — crosses 0 (zero check), the small-value tolerance
// (1.00001 → totalWowOcteracts ~ 0.005), and the 1000 branching boundary.
const octeractGrid = [0, 0.001, 0.004, 0.005, 0.006, 1, 10, 100, 500, 999, 1000, 1001, 5000, 1e6, 1e9]

// Octeract pow grid — what getSingularityChallengeEffect returns for
// completions 0..15. Below 10 it's 0.02*n; at 10+ it's 0.2 + (n-10)/100.
const octeractPowGrid = [0, 0.02, 0.1, 0.18, 0.2, 0.21, 0.25, 0.3, 0.35]

// Cube bonus grid for the downstream offering/obtainium tests.
const cubeBonusGrid = [1, 1.5, 2, 3, 5, 10, 50, 100]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateTotalOcteractCubeBonus', () => {
  const enabledGrid = [true, false]
  for (const exalt4Enabled of enabledGrid) {
    for (const octeractPow of octeractPowGrid) {
      it.each(octeractGrid)(`exalt4=${exalt4Enabled} pow=${octeractPow} oct=%s`, (totalWowOcteracts) => {
        const next = newCubeBonus({ exalt4Enabled, totalWowOcteracts, octeractPow })
        const old = oldCubeBonus(exalt4Enabled, totalWowOcteracts, octeractPow)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})

describe('parity: calculateTotalOcteractQuarkBonus', () => {
  const enabledGrid = [true, false]
  for (const exalt4Enabled of enabledGrid) {
    it.each(octeractGrid)(`exalt4=${exalt4Enabled} oct=%s`, (totalWowOcteracts) => {
      const next = newQuarkBonus({ exalt4Enabled, totalWowOcteracts })
      const old = oldQuarkBonus(exalt4Enabled, totalWowOcteracts)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: calculateTotalOcteractOfferingBonus', () => {
  const enabledGrid = [true, false]
  for (const offeringBonusEnabled of enabledGrid) {
    it.each(cubeBonusGrid)(`enabled=${offeringBonusEnabled} cubeBonus=%s`, (cubeBonus) => {
      const next = newOfferingBonus({ offeringBonusEnabled, cubeBonus })
      const old = oldOfferingBonus(offeringBonusEnabled, cubeBonus)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: calculateTotalOcteractObtainiumBonus', () => {
  const enabledGrid = [true, false]
  for (const obtainiumBonusEnabled of enabledGrid) {
    it.each(cubeBonusGrid)(`enabled=${obtainiumBonusEnabled} cubeBonus=%s`, (cubeBonus) => {
      const next = newObtainiumBonus({ obtainiumBonusEnabled, cubeBonus })
      const old = oldObtainiumBonus(obtainiumBonusEnabled, cubeBonus)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})
