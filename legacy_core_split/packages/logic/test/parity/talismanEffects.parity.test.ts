// Parity test for the 11 talisman effect formulas.

import { describe, expect, it } from 'vitest'
import {
  achievementTalismanEffects as newAchievement,
  chronosTalismanEffects as newChronos,
  cookieGrandmaTalismanEffects as newCookieGrandma,
  exemptionTalismanEffects as newExemption,
  horseShoeTalismanEffects as newHorseShoe,
  metaphysicsTalismanEffects as newMetaphysics,
  midasTalismanEffects as newMidas,
  mortuusTalismanEffects as newMortuus,
  plasticTalismanEffects as newPlastic,
  polymathTalismanEffects as newPolymath,
  wowSquareTalismanEffects as newWowSquare
} from '../../src/mechanics/talismanEffects'

// ─── OLD reference impls (transcribed verbatim) ────────────────────────────

const exemptionVals = [0, -0.2, -0.3, -0.4, -0.45, -0.5, -0.55, -0.6, -0.61, -0.62, -0.65]
const chronosVals = [1, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.4]
const midasVals = [1, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.40]
const metaphysicsVals = [1, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2]
const polymathVals = [1, 1.04, 1.08, 1.12, 1.16, 1.20, 1.25, 1.30, 1.325, 1.35, 1.40]
const mortuusVals = [1, 1.05, 1.1, 1.15, 1.2, 1.3, 1.4, 1.5, 1.65, 1.8, 2]
const plasticVals = [1, 1.005, 1.01, 1.015, 1.02, 1.025, 1.03, 1.04, 1.045, 1.05, 1.0666]
const wowSquareVals = [1, 1.025, 1.05, 1.075, 1.1, 1.125, 1.15, 1.2, 1.225, 1.25, 1.30]
const achievementVals = [0, 0.001, 0.002, 0.003, 0.004, 0.006, 0.008, .01, .015, .02, .03]
const cookieVals = [0, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10]
const horseShoeVals = [0, 0.001, 0.002, 0.003, 0.004, 0.005, 0.007, 0.01, 0.012, 0.015, 0.02]

const oldExemption = (n: number) => ({
  taxReduction: exemptionVals[n] ?? 0,
  duplicationOOMBonus: n >= 6 ? 12 : 0
})
const oldChronos = (n: number) => ({
  globalSpeed: chronosVals[n] ?? 1,
  speedOOMBonus: n >= 6 ? 12 : 0
})
const oldMidas = (n: number) => ({
  blessingBonus: midasVals[n] ?? 1,
  thriftOOMBonus: n >= 6 ? 12 : 0
})
const oldMetaphysics = (n: number) => ({
  talismanEffect: metaphysicsVals[n] ?? 1,
  extraTalismanEffect: n >= 6 ? 1.07 : 1
})
const oldPolymath = (n: number) => ({
  ascensionSpeedBonus: polymathVals[n] ?? 1,
  SIOOMBonus: n >= 6 ? 12 : 0
})
const oldMortuus = (n: number) => ({
  antBonus: mortuusVals[n] ?? 1,
  prismOOMBonus: n >= 6 ? 12 : 0
})
const oldPlastic = (n: number) => ({ quarkBonus: plasticVals[n] ?? 1 })
const oldWowSquare = (n: number) => ({
  evenDimBonus: wowSquareVals[n] ?? 1,
  oddDimBonus: n >= 6 ? 1.20 : 1
})
const oldAchievement = (n: number) => ({
  positiveSalvageMult: achievementVals[n] ?? 1,
  negativeSalvageMult: n >= 6 ? -0.02 : 0
})
const oldCookie = (n: number) => ({
  freeCorruptionLevel: cookieVals[n] ?? 0,
  cookieSix: n >= 6
})
const oldHorseShoe = (n: number) => ({
  luckPercentage: horseShoeVals[n] ?? 0,
  redLuck: n >= 6 ? 40 : 0
})

const nGrid = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 50] // include past array end

describe('parity: talisman effects', () => {
  it.each(nGrid)('exemption n=%i', (n) => {
    expect(newExemption(n)).toEqual(oldExemption(n))
  })
  it.each(nGrid)('chronos n=%i', (n) => {
    expect(newChronos(n)).toEqual(oldChronos(n))
  })
  it.each(nGrid)('midas n=%i', (n) => {
    expect(newMidas(n)).toEqual(oldMidas(n))
  })
  it.each(nGrid)('metaphysics n=%i', (n) => {
    expect(newMetaphysics(n)).toEqual(oldMetaphysics(n))
  })
  it.each(nGrid)('polymath n=%i', (n) => {
    expect(newPolymath(n)).toEqual(oldPolymath(n))
  })
  it.each(nGrid)('mortuus n=%i', (n) => {
    expect(newMortuus(n)).toEqual(oldMortuus(n))
  })
  it.each(nGrid)('plastic n=%i', (n) => {
    expect(newPlastic(n)).toEqual(oldPlastic(n))
  })
  it.each(nGrid)('wowSquare n=%i', (n) => {
    expect(newWowSquare(n)).toEqual(oldWowSquare(n))
  })
  it.each(nGrid)('achievement n=%i', (n) => {
    expect(newAchievement(n)).toEqual(oldAchievement(n))
  })
  it.each(nGrid)('cookieGrandma n=%i', (n) => {
    expect(newCookieGrandma(n)).toEqual(oldCookie(n))
  })
  it.each(nGrid)('horseShoe n=%i', (n) => {
    expect(newHorseShoe(n)).toEqual(oldHorseShoe(n))
  })
})
