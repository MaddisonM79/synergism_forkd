// Parity test for the 10 rune effect formulas.
//
// Pre-migration source: packages/web_ui/src/Runes.ts at HEAD. Each OLD
// `effects(n, key)` from the rune table is transcribed below. The two impure
// runes (infiniteAscent, antiquities) have their player reads hoisted to
// explicit inputs.

import { describe, expect, it } from 'vitest'
import {
  antiquitiesRuneEffects as newAntiquities,
  duplicationRuneEffects as newDuplication,
  finiteDescentRuneEffects as newFiniteDescent,
  horseShoeRuneEffects as newHorseShoe,
  infiniteAscentRuneEffects as newInfiniteAscent,
  prismRuneEffects as newPrism,
  speedRuneEffects as newSpeed,
  superiorIntellectRuneEffects as newSuperiorIntellect,
  thriftRuneEffects as newThrift,
  topHatRuneEffects as newTopHat
} from '../../src/mechanics/runeEffects'

// ─── OLD reference impls (transcribed verbatim) ────────────────────────────

const oldSpeed = (n: number, key: 'acceleratorPower' | 'multiplicativeAccelerators' | 'globalSpeed'): number => {
  if (key === 'acceleratorPower') return 0.0002 * n
  if (key === 'multiplicativeAccelerators') return 1 + n / 400
  return 2 - Math.exp(-Math.cbrt(n) / 100)
}
const oldDuplication = (n: number, key: 'multiplierBoosts' | 'multiplicativeMultipliers' | 'taxReduction'): number => {
  if (key === 'multiplierBoosts') return n / 5
  if (key === 'multiplicativeMultipliers') return 1 + n / 400
  return 0.001 + .999 * Math.exp(-Math.cbrt(n) / 5)
}
const oldPrism = (n: number, key: 'productionLog10' | 'costDivisorLog10'): number => {
  if (key === 'productionLog10') return Math.max(0, 2 * Math.log10(1 + n / 2) + (n / 2) * Math.log10(2) - Math.log10(256))
  return Math.floor(n / 10)
}
const oldThrift = (n: number, key: 'costDelay' | 'salvage' | 'taxReduction'): number => {
  if (key === 'costDelay') return Math.min(1e15, n / 125)
  if (key === 'salvage') return 2.5 * Math.log(1 + n / 10)
  return 0.01 + 0.99 * Math.exp(-Math.cbrt(n) / 10)
}
const oldSI = (n: number, key: 'offeringMult' | 'obtainiumMult' | 'antSpeed'): number => {
  if (key === 'offeringMult') return 1 + n / 2000
  if (key === 'obtainiumMult') return 1 + n / 200
  return Math.pow(1 + n / 500, 2)
}
const oldInfiniteAscent = (
  n: number,
  key: 'quarkMult' | 'cubeMult' | 'salvage',
  salvagePerkUnlockedCount: number
): number => {
  if (key === 'quarkMult') return 1 + n / 500 + (n > 0 ? 0.1 : 0)
  if (key === 'cubeMult') return 1 + n / 100
  return n * 0.025 * salvagePerkUnlockedCount
}
const oldAntiquities = (
  n: number,
  key: 'addCodeCooldownReduction' | 'offeringLog10' | 'obtainiumLog10' | 'cubeBonus',
  singularityCount: number
): number => {
  if (key === 'addCodeCooldownReduction') return n > 0 ? 0.8 - 0.3 * (n - 1) / (n + 10) : 1
  if (key === 'offeringLog10') return Math.round(300 * (1 - Math.pow(1 - 1 / 300, n)))
  if (key === 'obtainiumLog10') return Math.round(300 * (1 - Math.pow(1 - 1 / 300, n)))
  return (n > 0) ? Math.pow(1.01, Math.min(5, n) * singularityCount) : 1
}
const oldHorseShoe = (n: number, key: 'ambrosiaLuck' | 'redLuck' | 'redLuckConversion'): number => {
  if (key === 'ambrosiaLuck') return n
  if (key === 'redLuck') return n / 5
  return -0.5 * n / (n + 50)
}
const oldFiniteDescent = (
  n: number,
  key: 'ascensionScore' | 'corruptionFreeLevels' | 'infiniteAscentFreeLevel'
): number => {
  if (key === 'ascensionScore') return n >= 1 ? 1.04 + 0.96 * (n - 1) / (n + 25) : 1
  if (key === 'corruptionFreeLevels') return n >= 1 ? 0.01 + 0.14 * (n - 1) / (n + 16) : 0
  return Math.floor(n / 2)
}
const oldTopHat = (
  n: number,
  key: 'freeOfferingLevels' | 'freeObtainiumLevels' | 'freeCubeLevels' | 'freeSpeedLevels' | 'freeInfinityLevels'
): number => {
  if (key === 'freeOfferingLevels') return Math.round(200 * (1 - Math.pow(0.995, n))) / 10
  if (key === 'freeObtainiumLevels') return Math.round(200 * (1 - Math.pow(0.995, n))) / 10
  if (key === 'freeCubeLevels') return Math.round(150 * (1 - Math.pow(0.997, n))) / 10
  if (key === 'freeSpeedLevels') return Math.round(150 * (1 - Math.pow(0.997, n))) / 10
  return Math.round(100 * (1 - Math.pow(0.999, n))) / 10
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const nGrid = [0, 1, 5, 50, 500, 5_000, 50_000, 1e6]
const singularityGrid = [0, 1, 10, 100, 500]
const perkCountGrid = [0, 1, 3, 5, 10]

describe('parity: rune effects', () => {
  describe('speed', () => {
    const keys: Array<'acceleratorPower' | 'multiplicativeAccelerators' | 'globalSpeed'> = [
      'acceleratorPower',
      'multiplicativeAccelerators',
      'globalSpeed'
    ]
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newSpeed(n, key), oldSpeed(n, key))).toBe(true)
      })
    }
  })

  describe('duplication', () => {
    const keys: Array<'multiplierBoosts' | 'multiplicativeMultipliers' | 'taxReduction'> = [
      'multiplierBoosts',
      'multiplicativeMultipliers',
      'taxReduction'
    ]
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newDuplication(n, key), oldDuplication(n, key))).toBe(true)
      })
    }
  })

  describe('prism', () => {
    const keys: Array<'productionLog10' | 'costDivisorLog10'> = ['productionLog10', 'costDivisorLog10']
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newPrism(n, key), oldPrism(n, key))).toBe(true)
      })
    }
  })

  describe('thrift', () => {
    const keys: Array<'costDelay' | 'salvage' | 'taxReduction'> = ['costDelay', 'salvage', 'taxReduction']
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newThrift(n, key), oldThrift(n, key))).toBe(true)
      })
    }
  })

  describe('superiorIntellect', () => {
    const keys: Array<'offeringMult' | 'obtainiumMult' | 'antSpeed'> = ['offeringMult', 'obtainiumMult', 'antSpeed']
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newSuperiorIntellect(n, key), oldSI(n, key))).toBe(true)
      })
    }
  })

  describe('infiniteAscent (salvage takes salvagePerkUnlockedCount)', () => {
    const keys: Array<'quarkMult' | 'cubeMult' | 'salvage'> = ['quarkMult', 'cubeMult', 'salvage']
    for (const key of keys) {
      for (const perks of perkCountGrid) {
        it.each(nGrid)(`${key}, perks=${perks}, n=%i`, (n) => {
          expect(closeEnough(
            newInfiniteAscent(n, key, { salvagePerkUnlockedCount: perks }),
            oldInfiniteAscent(n, key, perks)
          )).toBe(true)
        })
      }
    }
  })

  describe('antiquities (cubeBonus takes singularityCount)', () => {
    const keys: Array<'addCodeCooldownReduction' | 'offeringLog10' | 'obtainiumLog10' | 'cubeBonus'> = [
      'addCodeCooldownReduction',
      'offeringLog10',
      'obtainiumLog10',
      'cubeBonus'
    ]
    for (const key of keys) {
      for (const sing of singularityGrid) {
        it.each(nGrid)(`${key}, sing=${sing}, n=%i`, (n) => {
          expect(closeEnough(
            newAntiquities(n, key, { singularityCount: sing }),
            oldAntiquities(n, key, sing)
          )).toBe(true)
        })
      }
    }
  })

  describe('horseShoe', () => {
    const keys: Array<'ambrosiaLuck' | 'redLuck' | 'redLuckConversion'> = [
      'ambrosiaLuck',
      'redLuck',
      'redLuckConversion'
    ]
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newHorseShoe(n, key), oldHorseShoe(n, key))).toBe(true)
      })
    }
  })

  describe('finiteDescent', () => {
    const keys: Array<'ascensionScore' | 'corruptionFreeLevels' | 'infiniteAscentFreeLevel'> = [
      'ascensionScore',
      'corruptionFreeLevels',
      'infiniteAscentFreeLevel'
    ]
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newFiniteDescent(n, key), oldFiniteDescent(n, key))).toBe(true)
      })
    }
  })

  describe('topHat', () => {
    const keys: Array<
      'freeOfferingLevels' | 'freeObtainiumLevels' | 'freeCubeLevels' | 'freeSpeedLevels' | 'freeInfinityLevels'
    > = ['freeOfferingLevels', 'freeObtainiumLevels', 'freeCubeLevels', 'freeSpeedLevels', 'freeInfinityLevels']
    for (const key of keys) {
      it.each(nGrid)(`${key}, n=%i`, (n) => {
        expect(closeEnough(newTopHat(n, key), oldTopHat(n, key))).toBe(true)
      })
    }
  })
})
