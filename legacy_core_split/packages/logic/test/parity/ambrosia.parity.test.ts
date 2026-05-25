// Parity tests for the ambrosia-family formulas lifted from
// packages/web_ui/src/Calculate.ts. Sweeps cross the digit-reduction
// thresholds (10000, 30000, 100000, 300000, ...), the blueberry-time
// 10000 power-kick boundary, the >=50000 / >=500000 quark-mult tier
// boundaries, the singularity-blueberry milestone steps (64/128/192/256/270),
// and the upgrade-composer array reducers.

import { describe, expect, it } from 'vitest'
import {
  calculateAmbrosiaCubeMult as newCubeMult,
  calculateAmbrosiaGenerationOcteractUpgrade as newAmbrosiaGenOct,
  calculateAmbrosiaGenerationSingularityUpgrade as newAmbrosiaGenSing,
  calculateAmbrosiaLuckOcteractUpgrade as newAmbrosiaLuckOct,
  calculateAmbrosiaLuckSingularityUpgrade as newAmbrosiaLuckSing,
  calculateAmbrosiaQuarkMult as newQuarkMult,
  calculateNumberOfThresholds as newNumThresholds,
  calculateRequiredBlueberryTime as newBlueberryTime,
  calculateRequiredRedAmbrosiaTime as newRedAmbrosiaTime,
  calculateSingularityMilestoneBlueberries as newSingMilestoneBlue,
  calculateToNextThreshold as newToNextThreshold
} from '../../src/mechanics/ambrosia'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

const oldDigitReduction = 4

const oldNumThresholds = (lifetimeAmbrosia: number): number => {
  const numDigits = lifetimeAmbrosia > 0 ? 1 + Math.floor(Math.log10(lifetimeAmbrosia)) : 0
  const matissa = Math.floor(lifetimeAmbrosia / Math.pow(10, numDigits - 1))
  const extraReduction = matissa >= 3 ? 1 : 0
  return Math.max(0, 2 * (numDigits - oldDigitReduction) - 1 + extraReduction)
}

const oldToNextThreshold = (lifetimeAmbrosia: number): number => {
  const numThresholds = oldNumThresholds(lifetimeAmbrosia)
  if (numThresholds === 0) {
    return 10000 - lifetimeAmbrosia
  }
  if (numThresholds % 2 === 0) {
    return Math.pow(10, numThresholds / 2 + oldDigitReduction) - lifetimeAmbrosia
  }
  return 3 * Math.pow(10, (numThresholds - 1) / 2 + oldDigitReduction) - lifetimeAmbrosia
}

const oldBlueberryTime = (timePerAmbrosia: number, lifetimeAmbrosia: number, acceleratorMult: number, brickOfLeadMult: number): number => {
  let val = timePerAmbrosia
  val += Math.floor(lifetimeAmbrosia / 300)
  val *= acceleratorMult
  val *= brickOfLeadMult
  if (lifetimeAmbrosia >= 10000) {
    const extraScalingPower = Math.log10(4)
    val *= Math.pow(lifetimeAmbrosia / 10000, extraScalingPower)
    return Math.ceil(val)
  }
  return val
}

const oldRedAmbrosiaTime = (timePerRedAmbrosia: number, lifetimeRedAmbrosia: number, barRequirementMultiplier: number): number => {
  let val = timePerRedAmbrosia
  val += 200 * lifetimeRedAmbrosia
  const max = 1e6 * barRequirementMultiplier
  val *= barRequirementMultiplier
  return Math.min(max, val)
}

const oldSingMilestoneBlue = (highestSingularityCount: number): number => {
  let val = 0
  if (highestSingularityCount >= 270) val = 5
  else if (highestSingularityCount >= 256) val = 4
  else if (highestSingularityCount >= 192) val = 3
  else if (highestSingularityCount >= 128) val = 2
  else if (highestSingularityCount >= 64) val = 1
  return val
}

const oldCubeMult = (noAmbrosiaUpgradesEnabled: boolean, lifetimeAmbrosia: number): number => {
  const effectiveAmbrosia = noAmbrosiaUpgradesEnabled ? 0 : lifetimeAmbrosia
  let multiplier = 1
  multiplier += Math.min(1.5, Math.floor(effectiveAmbrosia / 66) / 100)
  if (effectiveAmbrosia >= 10000) {
    multiplier += Math.min(1.5, Math.floor(effectiveAmbrosia / 666) / 100)
  }
  if (effectiveAmbrosia >= 100000) {
    multiplier += Math.floor(effectiveAmbrosia / 6666) / 100
  }
  return multiplier
}

const oldQuarkMult = (noAmbrosiaUpgradesEnabled: boolean, lifetimeAmbrosia: number): number => {
  const effectiveAmbrosia = noAmbrosiaUpgradesEnabled ? 0 : lifetimeAmbrosia
  let multiplier = 1
  multiplier += Math.min(0.3, Math.floor(effectiveAmbrosia / 1666) / 100)
  if (effectiveAmbrosia >= 50000) {
    multiplier += Math.min(0.3, Math.floor(effectiveAmbrosia / 16666) / 100)
  }
  if (effectiveAmbrosia >= 500000) {
    multiplier += Math.floor(effectiveAmbrosia / 166666) / 100
  }
  return multiplier
}

const oldAmbrosiaGen = (effects: number[]): number => effects[0] * effects[1] * effects[2] * effects[3]
const oldAmbrosiaLuck = (effects: number[]): number => effects[0] + effects[1] + effects[2] + effects[3]

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Lifetime ambrosia grid — crosses every threshold boundary plus a few in
// between. Thresholds (for digitReduction=4): 10000, 30000, 100000, 300000,
// 1e6, 3e6, ...
const lifetimeAmbrosiaGrid = [
  0, 1, 9, 65, 66, 67, 100, 9999, 10000, 10001, 29999, 30000, 30001,
  99999, 100000, 100001, 299999, 300000, 300001, 1e6, 3e6, 1e7
]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateNumberOfThresholds', () => {
  it.each(lifetimeAmbrosiaGrid)('lifetime=%i', (lifetime) => {
    expect(newNumThresholds(lifetime)).toBe(oldNumThresholds(lifetime))
  })
})

describe('parity: calculateToNextThreshold', () => {
  it.each(lifetimeAmbrosiaGrid)('lifetime=%i', (lifetime) => {
    expect(newToNextThreshold(lifetime)).toBe(oldToNextThreshold(lifetime))
  })
})

describe('parity: calculateRequiredBlueberryTime', () => {
  const timePerAmbrosiaGrid = [45]
  const multGrid = [0.5, 1, 1.5, 2]
  for (const acceleratorMult of multGrid) {
    for (const brickOfLeadMult of multGrid) {
      for (const timePerAmbrosia of timePerAmbrosiaGrid) {
        it.each(lifetimeAmbrosiaGrid)(
          `t/a=${timePerAmbrosia} acc=${acceleratorMult} brick=${brickOfLeadMult} lifetime=%i`,
          (lifetime) => {
            const next = newBlueberryTime({
              timePerAmbrosia,
              lifetimeAmbrosia: lifetime,
              acceleratorMult,
              brickOfLeadMult
            })
            const old = oldBlueberryTime(timePerAmbrosia, lifetime, acceleratorMult, brickOfLeadMult)
            expect(closeEnough(next, old)).toBe(true)
          }
        )
      }
    }
  }
})

describe('parity: calculateRequiredRedAmbrosiaTime', () => {
  const timePerGrid = [100000]
  const lifetimeRedGrid = [0, 1, 100, 1000, 4900, 5000, 5001, 10000, 1e6]
  const barMultGrid = [0.5, 1, 1.5, 2, 10]
  for (const timePerRedAmbrosia of timePerGrid) {
    for (const barRequirementMultiplier of barMultGrid) {
      it.each(lifetimeRedGrid)(
        `t/red=${timePerRedAmbrosia} barMult=${barRequirementMultiplier} lifetimeRed=%i`,
        (lifetimeRed) => {
          const next = newRedAmbrosiaTime({
            timePerRedAmbrosia,
            lifetimeRedAmbrosia: lifetimeRed,
            barRequirementMultiplier
          })
          const old = oldRedAmbrosiaTime(timePerRedAmbrosia, lifetimeRed, barRequirementMultiplier)
          expect(closeEnough(next, old)).toBe(true)
        }
      )
    }
  }
})

describe('parity: calculateSingularityMilestoneBlueberries', () => {
  // Grid crosses every step (64, 128, 192, 256, 270) plus -1/+1 around each.
  const grid = [0, 1, 63, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256, 257, 269, 270, 271, 300]
  it.each(grid)('highest=%i', (highest) => {
    expect(newSingMilestoneBlue(highest)).toBe(oldSingMilestoneBlue(highest))
  })
})

describe('parity: calculateAmbrosiaCubeMult', () => {
  const enabledGrid = [true, false]
  for (const noAmbrosiaUpgradesEnabled of enabledGrid) {
    it.each(lifetimeAmbrosiaGrid)(`enabled=${noAmbrosiaUpgradesEnabled} lifetime=%i`, (lifetime) => {
      const next = newCubeMult({ noAmbrosiaUpgradesEnabled, lifetimeAmbrosia: lifetime })
      const old = oldCubeMult(noAmbrosiaUpgradesEnabled, lifetime)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: calculateAmbrosiaQuarkMult', () => {
  const enabledGrid = [true, false]
  // Include grid points around 50000 / 500000 quark-mult tier boundaries.
  const grid = [
    ...lifetimeAmbrosiaGrid,
    49999, 50000, 50001, 499999, 500000, 500001
  ]
  for (const noAmbrosiaUpgradesEnabled of enabledGrid) {
    it.each(grid)(`enabled=${noAmbrosiaUpgradesEnabled} lifetime=%i`, (lifetime) => {
      const next = newQuarkMult({ noAmbrosiaUpgradesEnabled, lifetimeAmbrosia: lifetime })
      const old = oldQuarkMult(noAmbrosiaUpgradesEnabled, lifetime)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: ambrosia upgrade composers (sing + octeract)', () => {
  // Each row is wrapped in an outer array so vitest passes the inner array as
  // a single argument to the test function (rather than spreading it).
  const speedSets: [number[]][] = [
    [[1, 1, 1, 1]],
    [[1, 1.1, 1.2, 1.3]],
    [[2, 3, 5, 7]],
    [[0.5, 1, 1, 1]],
    [[1, 1, 1, 1.5]]
  ]
  const luckSets: [number[]][] = [
    [[0, 0, 0, 0]],
    [[1, 2, 3, 4]],
    [[10, 20, 30, 40]],
    [[100, 0, 0, 50]],
    [[5, 5, 5, 5]]
  ]

  it.each(speedSets)('AmbrosiaGenerationSingularityUpgrade %j', (effects) => {
    expect(closeEnough(newAmbrosiaGenSing(effects), oldAmbrosiaGen(effects))).toBe(true)
  })
  it.each(speedSets)('AmbrosiaGenerationOcteractUpgrade %j', (effects) => {
    expect(closeEnough(newAmbrosiaGenOct(effects), oldAmbrosiaGen(effects))).toBe(true)
  })
  it.each(luckSets)('AmbrosiaLuckSingularityUpgrade %j', (effects) => {
    expect(newAmbrosiaLuckSing(effects)).toBe(oldAmbrosiaLuck(effects))
  })
  it.each(luckSets)('AmbrosiaLuckOcteractUpgrade %j', (effects) => {
    expect(newAmbrosiaLuckOct(effects)).toBe(oldAmbrosiaLuck(effects))
  })
})
