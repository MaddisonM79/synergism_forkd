// Parity tests for calculateTax, lifted from packages/web_ui/src/Tax.ts.
// The function has dozens of inputs feeding a long multiplicative chain
// plus several override branches (C13/C9/C15, taxmanLastStand, 1e-300
// overflow guard). The sweep strategy:
//   1. Build a `baseline` input that produces a known non-degenerate
//      result. Verify parity.
//   2. For each branch / override, perturb the relevant inputs and
//      verify parity on the perturbed input.
//   3. Smoke-test the overtaxed-achievement flag at its three gates.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import { CalcECC } from '../../src/mechanics/challenges'
import { calculateTax as newCalcTax, type CalculateTaxInput } from '../../src/mechanics/tax'

// ─── Old implementation (verbatim from packages/web_ui/src/Tax.ts) ─────────

const oldCalcTax = (input: CalculateTaxInput) => {
  let exp = 1

  if (input.inReinc6) {
    exp = 3 * Math.pow(1 + input.c6Completions / 25, 2)
  }
  if (input.inReinc9) {
    exp = 0.005
  }
  if (input.inAscension15) {
    exp = 0.000005
  }

  const c13effcompletions = Math.max(
    0,
    input.totalChallengeCompletions
      - input.c11Completions
      - input.c12Completions
      - input.c13Completions
      - input.c14Completions
      - input.c15Completions
      - ((input.singularityCount >= 15) ? 4 : 0)
      - ((input.singularityCount >= 20) ? 1 : 0)
  )

  if (input.inAscension13) {
    exp *= 400 * (1 + 1 / 6 * input.c13Completions)
    exp *= Math.pow(1.05, c13effcompletions)
  }
  if (input.c6Completions > 0) {
    exp /= 1.075
  }

  let exponent = 1
  exponent *= exp
  exponent *= 1 - 0.06 * input.research51
  exponent *= 1 - 0.05 * input.research52
  exponent *= 1 - 0.05 * input.research53
  exponent *= 1 - 0.05 * input.research54
  exponent *= 1 - 0.05 * input.research55
  exponent *= input.taxReductionAchievement
  exponent *= Math.pow(0.965, CalcECC('reincarnation', input.c6Completions))
  exponent *= input.duplicationRuneTaxReduction
  exponent *= input.thriftRuneTaxReduction
  exponent *= input.antTaxReduction
  exponent *= 1
    / Math.pow(
      1 + Decimal.log(input.ascendShards.add(1), 10),
      1 + 1 / 300 * input.c10Completions * input.upgrade125 + 0.1 * input.platonicUpgrade5
        + 0.2 * input.platonicUpgrade10 + input.taxPlatonicBlessing
    )
  exponent *= 1 + input.exemptionTalismanTaxReduction
  exponent *= Math.pow(0.98, 3 / 5 * Decimal.log(input.rareFragments.add(1), 10) * input.research159)
  exponent *= Math.pow(0.966, CalcECC('ascension', input.c13Completions))
  exponent *= 1 - 0.666 * input.research200 / 100000
  exponent *= 1 - 0.666 * input.cubeUpgrade50 / 100000
  exponent *= input.challenge15TaxesReward
  exponent *= input.campaignTaxMultiplier
  if (input.upgrade121 > 0) {
    exponent *= 0.5
  }
  if (input.highestSingularityCount >= 281) {
    exponent *= 0.5
  }
  if (input.taxmanLastStandEnabled) {
    if (input.ascensionsUnlocked) {
      exponent *= 4
    }
    if (input.highestC14Completions > 0) {
      exponent *= 5
    }
  }

  if (exponent < 1e-300) {
    exponent = 1e-300
  }

  let flatMaxExponentIncrease = Decimal.log(input.fortunaeFormicidaeCoinMultiplier, 10)
  flatMaxExponentIncrease += Decimal.log(input.buildingPowerCoinMultiplier, 10)

  const maxexponent = Math.floor(275 / (Decimal.log(1.01, 10) * exponent)) - 1 + flatMaxExponentIncrease

  const exponentForDivisor = Math.max(
    0,
    Math.min(maxexponent, Math.floor(Decimal.log(input.produceTotal.add(1), 10))) - flatMaxExponentIncrease
  )
  const exponentForWarning = Math.max(0, maxexponent - flatMaxExponentIncrease)

  const shouldAwardOvertaxed = input.inAscension13
    && (maxexponent - flatMaxExponentIncrease) <= 99999
    && c13effcompletions >= 1

  const divisorExponent = 1 / 550 * Math.pow(exponentForDivisor, 2)
  const checkExponent = 1 / 550 * Math.pow(exponentForWarning, 2)

  const taxdivisor = Decimal.pow(1.01, divisorExponent * exponent)
  const taxdivisorcheck = Decimal.pow(1.01, checkExponent * exponent)

  return { exponent, maxexponent, taxdivisor, taxdivisorcheck, shouldAwardOvertaxed }
}

// ─── Baseline input ─────────────────────────────────────────────────────

const baseline: CalculateTaxInput = {
  inReinc6: false,
  inReinc9: false,
  inAscension15: false,
  inAscension13: false,
  c6Completions: 0,
  c13Completions: 0,

  totalChallengeCompletions: 0,
  c11Completions: 0,
  c12Completions: 0,
  c14Completions: 0,
  c15Completions: 0,
  singularityCount: 0,

  research51: 0,
  research52: 0,
  research53: 0,
  research54: 0,
  research55: 0,
  research159: 0,
  research200: 0,
  cubeUpgrade50: 0,
  platonicUpgrade5: 0,
  platonicUpgrade10: 0,
  taxPlatonicBlessing: 0,
  upgrade121: 0,
  upgrade125: 0,
  c10Completions: 0,

  highestSingularityCount: 0,
  taxmanLastStandEnabled: false,
  ascensionsUnlocked: false,
  highestC14Completions: 0,

  taxReductionAchievement: 1,
  duplicationRuneTaxReduction: 1,
  thriftRuneTaxReduction: 1,
  antTaxReduction: 1,
  exemptionTalismanTaxReduction: 0,
  challenge15TaxesReward: 1,
  campaignTaxMultiplier: 1,

  ascendShards: new Decimal(0),
  rareFragments: new Decimal(0),
  fortunaeFormicidaeCoinMultiplier: new Decimal(1),
  buildingPowerCoinMultiplier: new Decimal(1),

  produceTotal: new Decimal(1)
}

const closeEnough = (a: number, b: number, rel = 1e-10): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const decimalClose = (a: Decimal, b: Decimal, rel = 1e-10): boolean => {
  if (a.eq(b)) return true
  if (a.eq(0) && b.eq(0)) return true
  const diff = a.minus(b).abs()
  const denom = Decimal.max(a.abs(), b.abs())
  return diff.div(denom).lt(rel)
}

const expectResultsEqual = (
  next: ReturnType<typeof newCalcTax>,
  old: ReturnType<typeof oldCalcTax>
) => {
  expect(closeEnough(next.exponent, old.exponent)).toBe(true)
  expect(closeEnough(next.maxexponent, old.maxexponent)).toBe(true)
  expect(decimalClose(next.taxdivisor, old.taxdivisor)).toBe(true)
  expect(decimalClose(next.taxdivisorcheck, old.taxdivisorcheck)).toBe(true)
  expect(next.shouldAwardOvertaxed).toBe(old.shouldAwardOvertaxed)
}

// ─── Baseline + per-branch perturbations ───────────────────────────────────

describe('parity: calculateTax (baseline)', () => {
  it('all-default input', () => {
    expectResultsEqual(newCalcTax(baseline), oldCalcTax(baseline))
  })
})

describe('parity: calculateTax (challenge overrides)', () => {
  // Each override completely replaces the base `exp`. Test all three.
  const cases: { name: string; perturb: Partial<CalculateTaxInput> }[] = [
    { name: 'inReinc6 base', perturb: { inReinc6: true, c6Completions: 0 } },
    { name: 'inReinc6 c6=25', perturb: { inReinc6: true, c6Completions: 25 } },
    { name: 'inReinc6 c6=100', perturb: { inReinc6: true, c6Completions: 100 } },
    { name: 'inReinc9', perturb: { inReinc9: true } },
    { name: 'inAscension15', perturb: { inAscension15: true } },
    // Override precedence: reinc9 wins over reinc6
    { name: 'inReinc6 + inReinc9 (reinc9 wins)', perturb: { inReinc6: true, c6Completions: 10, inReinc9: true } },
    // asc15 wins over reinc9 (last branch in legacy precedence)
    { name: 'inReinc9 + inAscension15 (asc15 wins)', perturb: { inReinc9: true, inAscension15: true } }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const input = { ...baseline, ...c.perturb }
      expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    })
  }
})

describe('parity: calculateTax (C13 multiplier + c13effcompletions)', () => {
  const cases: { name: string; perturb: Partial<CalculateTaxInput> }[] = [
    { name: 'asc13 alone', perturb: { inAscension13: true, c13Completions: 0 } },
    { name: 'asc13 c13=10', perturb: { inAscension13: true, c13Completions: 10 } },
    {
      name: 'asc13 c13effcompletions positive',
      perturb: {
        inAscension13: true,
        c13Completions: 5,
        totalChallengeCompletions: 200,
        c11Completions: 10,
        c12Completions: 10,
        c14Completions: 5,
        c15Completions: 5
      }
    },
    {
      name: 'asc13 + singularityCount > 20 reductions',
      perturb: {
        inAscension13: true,
        totalChallengeCompletions: 50,
        singularityCount: 25
      }
    },
    // c13effcompletions clamped at 0
    {
      name: 'all-completions = high-tier (c13eff → 0)',
      perturb: {
        inAscension13: true,
        totalChallengeCompletions: 30,
        c11Completions: 10,
        c12Completions: 10,
        c13Completions: 10
      }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const input = { ...baseline, ...c.perturb }
      expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    })
  }
})

describe('parity: calculateTax (research / cube reductions)', () => {
  // Each research stacks multiplicatively. Sweep representative values.
  const cases: { name: string; perturb: Partial<CalculateTaxInput> }[] = [
    { name: 'research51 max', perturb: { research51: 16 } }, // 1 - 0.06*16 ≈ 0.04
    { name: 'research52..55 max', perturb: { research52: 20, research53: 20, research54: 20, research55: 20 } },
    { name: 'research200 max', perturb: { research200: 100000 } },
    { name: 'cubeUpgrade50 max', perturb: { cubeUpgrade50: 100000 } },
    { name: 'research159 with rareFragments', perturb: { research159: 1, rareFragments: new Decimal('1e10') } }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const input = { ...baseline, ...c.perturb }
      expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    })
  }
})

describe('parity: calculateTax (ascendShards divisor)', () => {
  // Exponent on the ascendShards divisor changes with c10/upgrade125/platonic/
  // taxPlatonicBlessing. Sweep across each contribution.
  const cases: { name: string; perturb: Partial<CalculateTaxInput> }[] = [
    { name: 'ascendShards alone', perturb: { ascendShards: new Decimal('1e50') } },
    {
      name: 'ascendShards + c10*upgrade125',
      perturb: { ascendShards: new Decimal('1e50'), c10Completions: 300, upgrade125: 1 }
    },
    { name: 'ascendShards + platonic5', perturb: { ascendShards: new Decimal('1e50'), platonicUpgrade5: 1 } },
    { name: 'ascendShards + platonic10', perturb: { ascendShards: new Decimal('1e50'), platonicUpgrade10: 1 } },
    {
      name: 'ascendShards + taxPlatonicBlessing',
      perturb: { ascendShards: new Decimal('1e50'), taxPlatonicBlessing: 0.5 }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const input = { ...baseline, ...c.perturb }
      expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    })
  }
})

describe('parity: calculateTax (late-game halving)', () => {
  // upgrade121 and sing281 each halve. Verify in isolation and combined.
  for (const upgrade121 of [0, 1]) {
    for (const sing of [0, 280, 281, 500]) {
      it(`upgrade121=${upgrade121} sing=${sing}`, () => {
        const input = { ...baseline, upgrade121, highestSingularityCount: sing }
        expectResultsEqual(newCalcTax(input), oldCalcTax(input))
      })
    }
  }
})

describe('parity: calculateTax (taxmanLastStand)', () => {
  // Three on/off flags + the C14 completions gate. Eight cases total.
  for (const enabled of [true, false]) {
    for (const unlocked of [true, false]) {
      for (const c14 of [0, 1]) {
        it(`enabled=${enabled} ascUnlocked=${unlocked} c14=${c14}`, () => {
          const input = {
            ...baseline,
            taxmanLastStandEnabled: enabled,
            ascensionsUnlocked: unlocked,
            highestC14Completions: c14
          }
          expectResultsEqual(newCalcTax(input), oldCalcTax(input))
        })
      }
    }
  }
})

describe('parity: calculateTax (1e-300 overflow guard)', () => {
  // Stack enough reductions to drive exponent below 1e-300.
  it('extreme reduction inputs → clamped to 1e-300', () => {
    const input = {
      ...baseline,
      inReinc9: true, // base exp = 0.005
      research51: 16,
      research52: 20,
      research53: 20,
      research54: 20,
      research55: 20,
      duplicationRuneTaxReduction: 1e-100,
      thriftRuneTaxReduction: 1e-100
    }
    expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    expect(newCalcTax(input).exponent).toBe(1e-300)
  })
})

describe('parity: calculateTax (flatMaxExponentIncrease)', () => {
  // Ant Coins multiplier and building power both feed into the flat
  // max-exponent increase.
  const cases = [
    { fortunae: new Decimal('1e10'), building: new Decimal(1) },
    { fortunae: new Decimal(1), building: new Decimal('1e20') },
    { fortunae: new Decimal('1e10'), building: new Decimal('1e20') }
  ]
  for (let i = 0; i < cases.length; i++) {
    const c = cases[i]
    it(`fortunae=${c.fortunae.toString()} building=${c.building.toString()}`, () => {
      const input = {
        ...baseline,
        fortunaeFormicidaeCoinMultiplier: c.fortunae,
        buildingPowerCoinMultiplier: c.building
      }
      expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    })
  }
})

describe('parity: calculateTax (produceTotal feeds divisor)', () => {
  // produceTotal log10 caps the divisor exponent. Verify across orders of
  // magnitude.
  for (const exp of [0, 5, 10, 50, 100, 200]) {
    it(`produceTotal=1e${exp}`, () => {
      const input = { ...baseline, produceTotal: new Decimal(`1e${exp}`) }
      expectResultsEqual(newCalcTax(input), oldCalcTax(input))
    })
  }
})

describe('parity: calculateTax (overtaxed achievement gate)', () => {
  // Three gates: inAscension13 + maxexp gap ≤ 99999 + c13eff ≥ 1.
  // Build inputs that toggle each gate.
  it('asc13 + c13eff=0 → no award', () => {
    const input = { ...baseline, inAscension13: true }
    const res = newCalcTax(input)
    expect(res.shouldAwardOvertaxed).toBe(false)
  })
  it('asc13 + c13eff=1 + small maxexp gap → award', () => {
    const input = {
      ...baseline,
      inAscension13: true,
      c13Completions: 5,
      totalChallengeCompletions: 200,
      // Large exponent → small maxexp (since maxexp ~ 275 / exp).
      // Default exponent here will be large from the C13 multiplier so
      // maxexp comes out modest; let's check actual behavior matches.
      // Build power small so flatMaxExponentIncrease is 0.
      fortunaeFormicidaeCoinMultiplier: new Decimal(1),
      buildingPowerCoinMultiplier: new Decimal(1)
    }
    const res = newCalcTax(input)
    const old = oldCalcTax(input)
    expect(res.shouldAwardOvertaxed).toBe(old.shouldAwardOvertaxed)
  })
})
