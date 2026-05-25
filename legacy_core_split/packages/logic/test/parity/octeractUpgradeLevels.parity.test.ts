// Parity tests for octeractUpgradeLevels — `octeractFreeLevelMultiplier`,
// `octeractFreeLevelSoftcap`, and the gated `actualOcteractUpgradeTotalLevels`,
// lifted from packages/web_ui/src/Octeracts.ts. Sweeps cover the softcap
// boundary (level == actualFreeLevels), both gate combinations, and the
// qualityOfLife escape hatch.

import { describe, expect, it } from 'vitest'
import {
  actualOcteractUpgradeTotalLevels as newActualLevels,
  octeractFreeLevelMultiplier as newFreeLevelMult,
  octeractFreeLevelSoftcap as newFreeLevelSoftcap
} from '../../src/mechanics/octeractUpgradeLevels'

// ─── Old implementations (verbatim from packages/web_ui/src/Octeracts.ts) ───

const oldFreeLevelMult = (cubeUpgrade78: number): number => 1 + 0.3 / 100 * cubeUpgrade78

const oldFreeLevelSoftcap = (freeLevel: number, freeLevelMult: number): number => freeLevel * freeLevelMult

interface OldActualLevelsInput {
  level: number
  freeLevel: number
  qualityOfLife: boolean
  cubeUpgrade78: number
  inNoOcteracts: boolean
  inSadisticPrequel: boolean
}

const oldActualLevels = (input: OldActualLevelsInput): number => {
  if ((input.inNoOcteracts || input.inSadisticPrequel) && !input.qualityOfLife) {
    return 0
  }
  const freeLevelMult = oldFreeLevelMult(input.cubeUpgrade78)
  const actualFreeLevels = oldFreeLevelSoftcap(input.freeLevel, freeLevelMult)
  if (input.level >= actualFreeLevels) {
    return actualFreeLevels + input.level
  }
  return 2 * Math.sqrt(actualFreeLevels * input.level)
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const cubeUpgrade78Grid = [0, 1, 10, 100, 1000]
const levelGrid = [0, 1, 5, 10, 50, 100, 500, 1000]
const freeLevelGrid = [0, 1, 5, 10, 50, 100, 500, 1000]
const boolGrid = [true, false]

describe('parity: octeractFreeLevelMultiplier', () => {
  it.each(cubeUpgrade78Grid)('cubeUpgrade78=%i', (c) => {
    expect(newFreeLevelMult(c)).toBe(oldFreeLevelMult(c))
  })
})

describe('parity: octeractFreeLevelSoftcap', () => {
  for (const freeLevel of freeLevelGrid) {
    for (const cubeUpgrade78 of cubeUpgrade78Grid) {
      const mult = oldFreeLevelMult(cubeUpgrade78)
      it(`freeLevel=${freeLevel} mult=${mult}`, () => {
        expect(closeEnough(newFreeLevelSoftcap(freeLevel, mult), oldFreeLevelSoftcap(freeLevel, mult))).toBe(true)
      })
    }
  }
})

describe('parity: actualOcteractUpgradeTotalLevels', () => {
  for (const inNoOcteracts of boolGrid) {
    for (const inSadisticPrequel of boolGrid) {
      for (const qualityOfLife of boolGrid) {
        for (const cubeUpgrade78 of cubeUpgrade78Grid) {
          for (const level of levelGrid) {
            for (const freeLevel of freeLevelGrid) {
              const input = {
                level,
                freeLevel,
                qualityOfLife,
                cubeUpgrade78,
                inNoOcteracts,
                inSadisticPrequel
              }
              it(`noOct=${inNoOcteracts} sadistic=${inSadisticPrequel} qol=${qualityOfLife} c78=${cubeUpgrade78} lv=${level} free=${freeLevel}`, () => {
                expect(closeEnough(newActualLevels(input), oldActualLevels(input))).toBe(true)
              })
            }
          }
        }
      }
    }
  }
})
