// Per-tick multiplier-state aggregator. Lifted from
// packages/web_ui/src/Synergism.ts (updateAllMultiplier).
//
// Computes the full multiplier stack for the current tick:
//   - freeUpgradeMultiplier (intermediate snapshot)
//   - freeMultiplier / totalMultiplier
//   - challengeOneLog (constant 3, surfaced for parity)
//   - totalMultiplierBoost
//   - multiplierPower / multiplierEffect
//
// Same pattern as updateAllTick — web_ui pre-evaluates every rune /
// hepteract / cube-blessing / achievement-reward / ant-effect input and
// passes plain values in.

import { Decimal } from '../math/bignum'
import { CalcECC } from './challenges'

export interface UpdateAllMultiplierInput {
  // ─── Direct player state ──────────────────────────────────────────────

  /** player.upgrades[7] — when > 0, adds min(4, 1 + log10(fifthOwnedCoin+1)). */
  upgrade7: number
  /** player.upgrades[9] — when > 0, adds floor(acceleratorBought / 10). */
  upgrade9: number
  /** player.upgrades[21..25] — each contributes +1 when > 0 and feeds the 1.01^count factor. */
  upgrade21: number
  upgrade22: number
  upgrade23: number
  upgrade24: number
  upgrade25: number
  /** player.upgrades[33] — when > 0, adds totalAcceleratorBoost. */
  upgrade33: number
  /** player.upgrades[34] — adds +0.03 per level to a multiplier. */
  upgrade34: number
  /** player.upgrades[35] — adds +0.02 per level to a multiplier. */
  upgrade35: number
  /** player.upgrades[49] — when > 0, adds min(50, log1e10(transcendPoints+1)). */
  upgrade49: number
  /** player.upgrades[50] — when > 0.5 inside a transcension/reincarnation challenge, multiplies by 1.25. */
  upgrade50: number
  /** player.upgrades[68] — when > 0, adds min(2500, floor(log10(taxdivisor)/1000)). */
  upgrade68: number
  /** player.acceleratorBought — feeds upgrade-9 contribution. */
  acceleratorBought: number
  /** player.multiplierBought — added directly to freeMultiplier for totalMultiplier. */
  multiplierBought: number
  /** player.fifthOwnedCoin — log10 base for upgrade-7. */
  fifthOwnedCoin: number
  /** player.challengecompletions[1] — adds +1 when > 0; fed through CalcECC for ecc1tr. */
  c1Completions: number
  /** player.challengecompletions[7] — fed through CalcECC for ecc7r; triggers c7 = 1.25 when > 0.5. */
  c7Completions: number
  /** player.challengecompletions[14] — fed through CalcECC for ecc14a. */
  c14Completions: number
  /** player.transcendPoints — log1e10 base for upgrade-49. */
  transcendPoints: Decimal
  /** player.transcendShards — log3 base for the b accumulator. */
  transcendShards: Decimal
  /** player.researches[2] — adds (1/5 * n * (1 + 1/2 * ecc14a)) to a multiplier. */
  research2: number
  /** player.researches[11..15] — additive bumps to a multiplier (1/20, 1/25, 1/40, 3/200, 1/200 per level). */
  research11: number
  research12: number
  research13: number
  research14: number
  research15: number
  /** player.researches[33..35] — each adds (11n/100) to b multiplier. */
  research33: number
  research34: number
  research35: number
  /** player.researches[87] — adds (1/20 * n) to a multiplier. */
  research87: number
  /** player.researches[89] — adds (n/5) to b multiplier. */
  research89: number
  /** player.researches[94] — multiplies the "20 * floor(sumOfRuneLevels/8)" additive. */
  research94: number
  /** player.researches[128, 143, 158, 173, 188, 200] — additive bumps to a multiplier. */
  research128: number
  research143: number
  research158: number
  research173: number
  research188: number
  research200: number
  /** player.cubeUpgrades[50] — adds (0.01/100 * n) to a multiplier. */
  cubeUpgrade50: number
  /** player.platonicUpgrades[6] — contributes (1 + n/30) to the exponent on `a`. */
  platonicUpgrade6: number
  /** player.currentChallenge.transcension — controls upgrade-50 gate + t-chal 1/2 multiplierPower overrides. */
  transcensionChallenge: number
  /** player.currentChallenge.reincarnation — controls upgrade-50 gate + r-chal 7/10 multiplierPower overrides. */
  reincarnationChallenge: number
  /** player.corruptions.used.viscosity — triggers >=15 / >=16 cuts on a. */
  viscosityCorruptionLevel: number

  // ─── Pre-evaluated effects / rewards / blessings ──────────────────────

  /** `+getAchievementReward('multipliers')` — added to base a. */
  multipliersAchievement: number
  /** `sumOfRuneLevels()` — used as `20 * research94 * floor(n/8)` additive. */
  sumOfRuneLevels: number
  /** `getRuneEffects('duplication', 'multiplicativeMultipliers')` — multiplies a. */
  multiplicativeMultipliersRune: number
  /** `getRuneEffects('duplication', 'multiplierBoosts')` — added to b accumulator. */
  multiplierBoostsRune: number
  /** `getRuneBlessingEffect('duplication').multiplierBoosts` — multiplies b. */
  multiplierBoostsRuneBlessing: number
  /** `getAntUpgradeEffect(AntUpgrades.Multipliers).multiplierMult` — multiplies a. */
  antMultiplierMult: number
  /** `calculateMultiplierCubeBlessing()` — multiplies a. */
  multiplierCubeBlessing: number
  /** `getHepteractEffects('multiplier').multiplier` — added to a after the viscosity-pow step. */
  hepteractMultiplier: number
  /** `getHepteractEffects('multiplier').multiplierMultiplier` — multiplies a after the additive. */
  hepteractMultiplierMult: number

  // ─── G inputs (pre-extracted by web_ui) ───────────────────────────────

  /** G.totalAcceleratorBoost — added via upgrade-33. */
  totalAcceleratorBoost: number
  /** G.taxdivisor — log10 base for the upgrade-68 contribution. */
  taxdivisor: Decimal
  /** G.viscosityPower[player.corruptions.used.viscosity] — exponent factor on a. */
  viscosityPower: number
  /** G.challenge15Rewards.multiplier.value — multiplies a after the hepteract additive. */
  challenge15RewardMultiplier: number
}

export interface UpdateAllMultiplierResult {
  /** G.freeUpgradeMultiplier — `a` snapshot right after upgrades/achievements are folded in. */
  freeUpgradeMultiplier: number
  /** G.freeMultiplier — final floored `a` after all multiplicative + corruption gating. */
  freeMultiplier: number
  /** G.totalMultiplier — freeMultiplier + multiplierBought. */
  totalMultiplier: number
  /** G.challengeOneLog — constant 3, surfaced for parity. */
  challengeOneLog: number
  /** G.totalMultiplierBoost — pow(floor(b), 1 + 0.04 * ecc7r). */
  totalMultiplierBoost: number
  /** G.multiplierPower — `2 + 0.02 * totalMultiplierBoost * c7` (with challenge overrides). */
  multiplierPower: number
  /** G.multiplierEffect — Decimal.pow(multiplierPower, totalMultiplier). */
  multiplierEffect: Decimal
}

/**
 * Per-tick multiplier-state aggregator. Mirrors the legacy updateAllMultiplier
 * body verbatim aside from input/output shape.
 */
export function updateAllMultiplier (input: UpdateAllMultiplierInput): UpdateAllMultiplierResult {
  let a = 0

  if (input.upgrade7 > 0) {
    a += Math.min(4, 1 + Math.floor(Decimal.log(input.fifthOwnedCoin + 1, 10)))
  }
  if (input.upgrade9 > 0) {
    a += Math.floor(input.acceleratorBought / 10)
  }
  if (input.upgrade21 > 0) a += 1
  if (input.upgrade22 > 0) a += 1
  if (input.upgrade23 > 0) a += 1
  if (input.upgrade24 > 0) a += 1
  if (input.upgrade25 > 0) a += 1
  if (input.upgrade33 > 0) {
    a += input.totalAcceleratorBoost
  }
  if (input.upgrade49 > 0) {
    a += Math.min(50, Math.floor(Decimal.log(input.transcendPoints.add(1), 1e10)))
  }
  if (input.upgrade68 > 0) {
    a += Math.min(2500, Math.floor((Decimal.log(input.taxdivisor, 10) * 1) / 1000))
  }
  if (input.c1Completions > 0) {
    a += 1
  }

  a += input.multipliersAchievement
  a += 20 * input.research94 * Math.floor(input.sumOfRuneLevels / 8)

  const freeUpgradeMultiplier = Math.min(1e100, a)

  const ecc14a = CalcECC('ascension', input.c14Completions)
  const ecc1tr = CalcECC('transcend', input.c1Completions)
  const ecc7r = CalcECC('reincarnation', input.c7Completions)

  a *= Math.pow(
    1.01,
    input.upgrade21 + input.upgrade22 + input.upgrade23 + input.upgrade24 + input.upgrade25
  )
  a *= 1 + 0.03 * input.upgrade34 + 0.02 * input.upgrade35
  a *= 1 + (1 / 5) * input.research2 * (1 + (1 / 2) * ecc14a)
  a *= 1
    + (1 / 20) * input.research11
    + (1 / 25) * input.research12
    + (1 / 40) * input.research13
    + (3 / 200) * input.research14
    + (1 / 200) * input.research15
  a *= input.multiplicativeMultipliersRune
  a *= 1 + (1 / 20) * input.research87
  a *= 1 + (1 / 100) * input.research128
  a *= 1 + (0.8 / 100) * input.research143
  a *= 1 + (0.6 / 100) * input.research158
  a *= 1 + (0.4 / 100) * input.research173
  a *= 1 + (0.2 / 100) * input.research188
  a *= 1 + (0.01 / 100) * input.research200
  a *= 1 + (0.01 / 100) * input.cubeUpgrade50
  a *= input.antMultiplierMult
  a *= input.multiplierCubeBlessing

  if (
    (input.transcensionChallenge !== 0 || input.reincarnationChallenge !== 0)
    && input.upgrade50 > 0.5
  ) {
    a *= 1.25
  }
  a = Math.pow(
    a,
    Math.min(1, (1 + input.platonicUpgrade6 / 30) * input.viscosityPower)
  )
  a += input.hepteractMultiplier
  a *= input.challenge15RewardMultiplier
  a *= input.hepteractMultiplierMult
  a = Math.floor(Math.min(1e100, a))

  if (input.viscosityCorruptionLevel >= 15) a = Math.pow(a, 0.2)
  if (input.viscosityCorruptionLevel >= 16) a = 1

  const freeMultiplier = a
  const totalMultiplier = freeMultiplier + input.multiplierBought
  const challengeOneLog = 3

  let b = 0
  b += Decimal.log(input.transcendShards.add(1), 3)
  b += input.multiplierBoostsRune
  b += 2 * ecc1tr
  b *= 1 + (11 * input.research33) / 100
  b *= 1 + (11 * input.research34) / 100
  b *= 1 + (11 * input.research35) / 100
  b *= 1 + input.research89 / 5
  b *= input.multiplierBoostsRuneBlessing

  const totalMultiplierBoost = Math.pow(Math.floor(b), 1 + ecc7r * 0.04)

  const c7 = input.c7Completions > 0.5 ? 1.25 : 1

  let multiplierPower = 2 + 0.02 * totalMultiplierBoost * c7

  if (
    input.reincarnationChallenge !== 7
    && input.reincarnationChallenge !== 10
  ) {
    if (input.transcensionChallenge === 1) {
      multiplierPower = 1
    }
    if (input.transcensionChallenge === 2) {
      multiplierPower = 1.25 + 0.0012 * b * c7
    }
  }
  multiplierPower = Math.min(1e300, multiplierPower)

  if (input.reincarnationChallenge === 7) multiplierPower = 1
  if (input.reincarnationChallenge === 10) multiplierPower = 1

  const multiplierEffect = Decimal.pow(multiplierPower, totalMultiplier)

  return {
    freeUpgradeMultiplier,
    freeMultiplier,
    totalMultiplier,
    challengeOneLog,
    totalMultiplierBoost,
    multiplierPower,
    multiplierEffect
  }
}
