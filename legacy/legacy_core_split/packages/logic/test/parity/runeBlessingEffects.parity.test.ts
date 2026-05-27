// Parity test for the 5 rune-blessing effect formulas.

import { describe, expect, it } from 'vitest'
import {
  duplicationRuneBlessingEffects as newDuplication,
  prismRuneBlessingEffects as newPrism,
  speedRuneBlessingEffects as newSpeed,
  superiorIntellectRuneBlessingEffects as newSI,
  thriftRuneBlessingEffects as newThrift
} from '../../src/mechanics/runeBlessingEffects'

const oldSpeed = (level: number) => ({ globalSpeed: 1 + level / 1000000 })
const oldDuplication = (level: number) => ({ multiplierBoosts: 1 + level / 1000000 })
const oldPrism = (level: number) => ({ antSacrificeMult: 1 + level / 1000000 })
const oldThrift = (level: number) => ({ accelBoostCostDelay: 1 + level / 1000000 })
const oldSI = (level: number) => ({ obtToAntExponent: Math.log(1 + level / 1000000) })

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const grid = [0, 1, 100, 1e4, 1e6, 1e8, 1e12]

describe('parity: rune-blessing effects', () => {
  it.each(grid)('speed level=%i', (level) => {
    expect(closeEnough(newSpeed(level).globalSpeed, oldSpeed(level).globalSpeed)).toBe(true)
  })
  it.each(grid)('duplication level=%i', (level) => {
    expect(closeEnough(newDuplication(level).multiplierBoosts, oldDuplication(level).multiplierBoosts)).toBe(true)
  })
  it.each(grid)('prism level=%i', (level) => {
    expect(closeEnough(newPrism(level).antSacrificeMult, oldPrism(level).antSacrificeMult)).toBe(true)
  })
  it.each(grid)('thrift level=%i', (level) => {
    expect(closeEnough(newThrift(level).accelBoostCostDelay, oldThrift(level).accelBoostCostDelay)).toBe(true)
  })
  it.each(grid)('superiorIntellect level=%i', (level) => {
    expect(closeEnough(newSI(level).obtToAntExponent, oldSI(level).obtToAntExponent)).toBe(true)
  })
})
