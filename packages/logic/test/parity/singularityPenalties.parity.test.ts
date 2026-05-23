// Parity tests for calculateEffectiveSingularities and calculateSingularityDebuff,
// lifted from packages/web_ui/src/singularity.ts. Each `oldXxx` transcribes the
// pre-migration body verbatim — including the local Exalt 4 helper that web_ui
// composed inline. Sweeps cover every singularityCount threshold boundary
// (10/25/36/50/100/150/200/215/230/269) and every debuff branch.

import { describe, expect, it } from 'vitest'
import {
  calculateEffectiveSingularities as newEffectiveSings,
  calculateSingularityDebuff as newSingDebuff,
  type SingularityDebuff
} from '../../src/mechanics/singularityPenalties'

// ─── Old implementations (verbatim from packages/web_ui/src/singularity.ts) ─

// The web_ui calculateEffectiveSingularities composed
// calculateExalt4EffectiveSingularityMultiplier(comps, false), and the
// Calculate.ts shim for that fed inExalt4 = player.singularityChallenges.noOcteracts.enabled.
// So the old behavior is `oldExalt4Mult(comps, false, inExalt4)`.
const oldExalt4Mult = (comps: number, force: boolean, inExalt4: boolean): number =>
  inExalt4 || force ? Math.pow(comps + 1, 3) : 1

interface OldEffectiveSingsInput {
  singularityCount: number
  noOcteractsCompletions: number
  inExalt4: boolean
  taxmanLastStandEnabled: boolean
  taxmanLastStandCompletions: number
  platonicUpgrade15: number
}

const oldEffectiveSings = (input: OldEffectiveSingsInput): number => {
  const singularityCount = input.singularityCount
  let effectiveSingularities = singularityCount
  effectiveSingularities *= Math.min(4.75, (0.75 * singularityCount) / 10 + 1)

  effectiveSingularities *= oldExalt4Mult(
    input.noOcteractsCompletions,
    false,
    input.inExalt4
  )

  if (singularityCount > 10) {
    effectiveSingularities *= 1.5
    effectiveSingularities *= Math.min(
      4,
      (1.25 * singularityCount) / 10 - 0.25
    )
  }
  if (singularityCount > 25) {
    effectiveSingularities *= 2.5
    effectiveSingularities *= Math.min(6, (1.5 * singularityCount) / 25 - 0.5)
  }
  if (singularityCount > 36) {
    effectiveSingularities *= 4
    effectiveSingularities *= Math.min(5, singularityCount / 18 - 1)
    effectiveSingularities *= Math.pow(
      1.1,
      Math.min(singularityCount - 36, 64)
    )
  }
  if (singularityCount > 50) {
    effectiveSingularities *= 5
    effectiveSingularities *= Math.min(8, (2 * singularityCount) / 50 - 1)
    effectiveSingularities *= Math.pow(
      1.1,
      Math.min(singularityCount - 50, 50)
    )
  }
  if (singularityCount > 100) {
    effectiveSingularities *= 2
    effectiveSingularities *= singularityCount / 25
    effectiveSingularities *= Math.pow(1.1, singularityCount - 100)
  }
  if (singularityCount > 150) {
    effectiveSingularities *= 2
    effectiveSingularities *= Math.pow(1.05, singularityCount - 150)
  }
  if (singularityCount > 200) {
    effectiveSingularities *= 1.5
    effectiveSingularities *= Math.pow(1.275, singularityCount - 200)
  }
  if (singularityCount > 215) {
    effectiveSingularities *= 1.25
    effectiveSingularities *= Math.pow(1.2, singularityCount - 215)
  }
  if (singularityCount > 230) {
    effectiveSingularities *= 2
  }
  if (singularityCount > 269) {
    effectiveSingularities *= 3
    effectiveSingularities *= Math.pow(3, singularityCount - 269)
  }

  if (
    input.taxmanLastStandEnabled
    && input.taxmanLastStandCompletions >= 8
    && input.platonicUpgrade15 === 0
  ) {
    effectiveSingularities = Math.pow(effectiveSingularities, 3 / 2)
  }

  return effectiveSingularities
}

interface OldSingDebuffInput {
  debuff: SingularityDebuff
  singularityCount: number
  antiquitiesRuneActive: boolean
  singularityReductions: number
  horseShoeMult: number
  noOcteractsCompletions: number
  inExalt4: boolean
  taxmanLastStandEnabled: boolean
  taxmanLastStandCompletions: number
  platonicUpgrade15: number
}

const oldSingDebuff = (input: OldSingDebuffInput): number => {
  if (input.singularityCount === 0 || input.antiquitiesRuneActive) {
    return (input.debuff === 'Salvage' || input.debuff === 'Ant ELO') ? 0 : 1
  }

  const constitutiveSingularityCount = input.singularityCount - input.singularityReductions
  if (constitutiveSingularityCount < 1) {
    return 1
  }

  const effectiveSingularities = oldEffectiveSings({
    singularityCount: constitutiveSingularityCount,
    noOcteractsCompletions: input.noOcteractsCompletions,
    inExalt4: input.inExalt4,
    taxmanLastStandEnabled: input.taxmanLastStandEnabled,
    taxmanLastStandCompletions: input.taxmanLastStandCompletions,
    platonicUpgrade15: input.platonicUpgrade15
  })

  let baseDebuffMultiplier = 1
  baseDebuffMultiplier *= input.horseShoeMult

  if (input.debuff === 'Offering') {
    const extraMult = Math.pow(1.02, constitutiveSingularityCount)
    return extraMult * baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 3 * (Math.sqrt(effectiveSingularities) + 1)
      : Math.pow(effectiveSingularities, 2 / 3) / 400)
  } else if (input.debuff === 'Salvage') {
    return -(4 * constitutiveSingularityCount
      + 4 * Math.max(0, constitutiveSingularityCount - 100)
      + 4 * Math.max(0, constitutiveSingularityCount - 200)
      + 3 * Math.max(0, constitutiveSingularityCount - 250)
      + 3 * Math.max(0, constitutiveSingularityCount - 270)
      + 2 * Math.max(0, constitutiveSingularityCount - 280))
  } else if (input.debuff === 'Ant ELO') {
    return -Math.min(1, 0.001 * constitutiveSingularityCount)
  } else if (input.debuff === 'Global Speed') {
    return baseDebuffMultiplier * (1 + Math.sqrt(effectiveSingularities) / 4)
  } else if (input.debuff === 'Obtainium') {
    const extraMult = Math.pow(1.02, constitutiveSingularityCount)
    return extraMult * baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 3 * (Math.sqrt(effectiveSingularities) + 1)
      : Math.pow(effectiveSingularities, 2 / 3) / 400)
  } else if (input.debuff === 'Researches') {
    return baseDebuffMultiplier * (1 + Math.sqrt(effectiveSingularities) / 2)
  } else if (input.debuff === 'Ascension Speed') {
    return baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 1 + Math.sqrt(effectiveSingularities) / 5
      : 1 + Math.pow(effectiveSingularities, 0.75) / 10000)
  } else if (input.debuff === 'Cubes') {
    const extraMult = constitutiveSingularityCount > 100
      ? 2 * Math.pow(1.03, constitutiveSingularityCount - 100)
      : 2
    return baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 3 * (1 + (Math.sqrt(effectiveSingularities) * extraMult) / 4)
      : 1 + (Math.pow(effectiveSingularities, 0.75) * extraMult) / 1000)
  } else if (input.debuff === 'Platonic Costs') {
    return baseDebuffMultiplier * (constitutiveSingularityCount > 36
      ? 1 + Math.pow(effectiveSingularities, 3 / 10) / 12
      : 1)
  } else if (input.debuff === 'Hepteract Costs') {
    return baseDebuffMultiplier * (constitutiveSingularityCount > 50
      ? 1 + Math.pow(effectiveSingularities, 11 / 50) / 25
      : 1)
  } else {
    return baseDebuffMultiplier * Math.cbrt(effectiveSingularities + 1)
  }
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Sweeps singularityCount across every step boundary (10, 25, 36, 50, 100,
// 150, 200, 215, 230, 269) plus mid-band and below-zero edge values.
const singGrid = [
  0,
  1,
  5,
  9,
  10,
  11,
  25,
  26,
  30,
  36,
  37,
  49,
  50,
  51,
  99,
  100,
  101,
  149,
  150,
  151,
  199,
  200,
  201,
  214,
  215,
  216,
  229,
  230,
  231,
  268,
  269,
  270,
  285
]

const noOctCompsGrid = [0, 1, 3, 8]
const platonic15Grid = [0, 1]

const allDebuffs: SingularityDebuff[] = [
  'Offering',
  'Obtainium',
  'Salvage',
  'Global Speed',
  'Researches',
  'Ant ELO',
  'Ascension Speed',
  'Cubes',
  'Cube Upgrades',
  'Platonic Costs',
  'Hepteract Costs'
]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateEffectiveSingularities (taxman OFF, exalt4 OFF)', () => {
  it.each(singGrid)('sing=%i', (singularityCount) => {
    const input = {
      singularityCount,
      noOcteractsCompletions: 0,
      inExalt4: false,
      taxmanLastStandEnabled: false,
      taxmanLastStandCompletions: 0,
      platonicUpgrade15: 0
    }
    const next = newEffectiveSings(input)
    const old = oldEffectiveSings(input)
    expect(closeEnough(next, old)).toBe(true)
  })
})

describe('parity: calculateEffectiveSingularities (exalt4 active across comps)', () => {
  for (const comps of noOctCompsGrid) {
    it.each(singGrid)(`comps=${comps} sing=%i`, (singularityCount) => {
      const input = {
        singularityCount,
        noOcteractsCompletions: comps,
        inExalt4: true,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 0,
        platonicUpgrade15: 0
      }
      const next = newEffectiveSings(input)
      const old = oldEffectiveSings(input)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: calculateEffectiveSingularities (taxman override boundary)', () => {
  // The taxman ^(3/2) override fires only when enabled AND comps ≥ 8 AND platonic15 == 0.
  // Sweep both sides of those gates plus a few singularity values.
  const taxmanGrid = [
    { enabled: false, comps: 8, plat15: 0 }, // gate-off
    { enabled: true, comps: 7, plat15: 0 }, // sub-threshold comps
    { enabled: true, comps: 8, plat15: 1 }, // platonic15 suppresses
    { enabled: true, comps: 8, plat15: 0 }, // fires
    { enabled: true, comps: 50, plat15: 0 } // fires, big comps
  ]
  const localSings = [5, 50, 150, 250]
  for (const taxman of taxmanGrid) {
    for (const sing of localSings) {
      it(`taxman.enabled=${taxman.enabled} comps=${taxman.comps} plat15=${taxman.plat15} sing=${sing}`, () => {
        const input = {
          singularityCount: sing,
          noOcteractsCompletions: 0,
          inExalt4: false,
          taxmanLastStandEnabled: taxman.enabled,
          taxmanLastStandCompletions: taxman.comps,
          platonicUpgrade15: taxman.plat15
        }
        const next = newEffectiveSings(input)
        const old = oldEffectiveSings(input)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})

describe('parity: calculateSingularityDebuff (every branch × sweep)', () => {
  const reductionsGrid = [0, 1, 5]
  const horseShoeGrid = [1, 0.75]
  for (const debuff of allDebuffs) {
    for (const reductions of reductionsGrid) {
      for (const horseShoe of horseShoeGrid) {
        it.each(singGrid)(`${debuff} reductions=${reductions} hs=${horseShoe} sing=%i`, (sing) => {
          const input = {
            debuff,
            singularityCount: sing,
            antiquitiesRuneActive: false,
            singularityReductions: reductions,
            horseShoeMult: horseShoe,
            noOcteractsCompletions: 0,
            inExalt4: false,
            taxmanLastStandEnabled: false,
            taxmanLastStandCompletions: 0,
            platonicUpgrade15: 0
          }
          const next = newSingDebuff(input)
          const old = oldSingDebuff(input)
          expect(closeEnough(next, old)).toBe(true)
        })
      }
    }
  }
})

describe('parity: calculateSingularityDebuff (antiquities gate)', () => {
  for (const debuff of allDebuffs) {
    it(`${debuff} returns no-penalty value when antiquities active`, () => {
      const input = {
        debuff,
        singularityCount: 100,
        antiquitiesRuneActive: true,
        singularityReductions: 0,
        horseShoeMult: 1,
        noOcteractsCompletions: 0,
        inExalt4: false,
        taxmanLastStandEnabled: false,
        taxmanLastStandCompletions: 0,
        platonicUpgrade15: 0
      }
      const next = newSingDebuff(input)
      const old = oldSingDebuff(input)
      expect(next).toBe(old)
    })
  }
})

describe('parity: calculateSingularityDebuff (taxman override propagates through)', () => {
  // Exercises the rare path where the ^(3/2) effective override actually shows
  // up in the final debuff multiplier.
  for (const debuff of allDebuffs) {
    for (const plat15 of platonic15Grid) {
      it(`${debuff} taxman enabled+comps=10 platonic15=${plat15}`, () => {
        const input = {
          debuff,
          singularityCount: 200,
          antiquitiesRuneActive: false,
          singularityReductions: 0,
          horseShoeMult: 1,
          noOcteractsCompletions: 0,
          inExalt4: false,
          taxmanLastStandEnabled: true,
          taxmanLastStandCompletions: 10,
          platonicUpgrade15: plat15
        }
        const next = newSingDebuff(input)
        const old = oldSingDebuff(input)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})
