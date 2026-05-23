// Parity test for the 5 rune-spirit effect formulas. Old bodies transcribed
// verbatim from packages/web_ui/src/RuneSpirits.ts (lines 65-179). Four
// spirits share the `1 + level / 1e9` shape; prism alone returns
// `level / 1e9` with no `1 +` prefix.

import { describe, expect, it } from 'vitest'
import {
  duplicationRuneSpiritEffects as newDuplication,
  prismRuneSpiritEffects as newPrism,
  speedRuneSpiritEffects as newSpeed,
  superiorIntellectRuneSpiritEffects as newSI,
  thriftRuneSpiritEffects as newThrift
} from '../../src/mechanics/runeSpiritEffects'

const oldSpeed = (level: number) => ({ globalSpeed: 1 + level / 1e9 })
const oldDuplication = (level: number) => ({ wowCubes: 1 + level / 1e9 })
const oldPrism = (level: number) => ({ crystalCaps: level / 1e9 })
const oldThrift = (level: number) => ({ offerings: 1 + level / 1e9 })
const oldSI = (level: number) => ({ obtainium: 1 + level / 1e9 })

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const grid = [0, 1, 100, 1e4, 1e6, 1e8, 1e12]

describe('parity: rune-spirit effects', () => {
  it.each(grid)('speed level=%i', (level) => {
    expect(closeEnough(newSpeed(level).globalSpeed, oldSpeed(level).globalSpeed)).toBe(true)
  })
  it.each(grid)('duplication level=%i', (level) => {
    expect(closeEnough(newDuplication(level).wowCubes, oldDuplication(level).wowCubes)).toBe(true)
  })
  it.each(grid)('prism level=%i', (level) => {
    expect(closeEnough(newPrism(level).crystalCaps, oldPrism(level).crystalCaps)).toBe(true)
  })
  it.each(grid)('thrift level=%i', (level) => {
    expect(closeEnough(newThrift(level).offerings, oldThrift(level).offerings)).toBe(true)
  })
  it.each(grid)('superiorIntellect level=%i', (level) => {
    expect(closeEnough(newSI(level).obtainium, oldSI(level).obtainium)).toBe(true)
  })
})
