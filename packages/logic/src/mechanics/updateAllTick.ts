// Per-tick accelerator-state aggregator. Lifted from
// packages/web_ui/src/Synergism.ts (updateAllTick).
//
// Computes the full accelerator stack for the current tick:
//   - totalAccelerator / costDivisor (bookkeeping)
//   - freeUpgradeAccelerator (intermediate snapshot used elsewhere)
//   - freeAccelerator (from upgrades + boosts + runes + cubes + multipliers
//     + hepteracts + corruptions)
//   - acceleratorPower / acceleratorEffect / acceleratorEffectDisplay
//   - generatorPower
//
// All "effect" inputs (achievement rewards, rune effects, cube blessings,
// hepteract effects, viscosity power, challenge 15 reward) are pre-evaluated
// by web_ui and passed in. G.acceleratorMultiplier is also pre-evaluated —
// the web_ui shim calls calculateAcceleratorMultiplier() before invoking
// updateAllTick so this slot is fresh.

import { Decimal } from '../math/bignum'
import { CalcECC } from './challenges'

export interface UpdateAllTickInput {
  // ─── Direct player state ──────────────────────────────────────────────

  /** player.acceleratorBought — base for totalAccelerator. */
  acceleratorBought: number
  /** player.multiplierBought — feeds upgrade-8's `floor(mult/7)`. */
  multiplierBought: number
  /** player.upgrades[8] — non-zero enables the multiplier-bought-derived bonus. */
  upgrade8: number
  /** player.upgrades[11] — when > 0.5 and r-chal ≠ 7, enables generatorPower formula. */
  upgrade11: number
  /** player.upgrades[21] — non-zero adds +5. */
  upgrade21: number
  /** player.upgrades[22] — non-zero adds +4. */
  upgrade22: number
  /** player.upgrades[23] — non-zero adds +3. */
  upgrade23: number
  /** player.upgrades[24] — non-zero adds +2. */
  upgrade24: number
  /** player.upgrades[25] — non-zero adds +1. */
  upgrade25: number
  /** player.upgrades[32] — non-zero adds min(500, floor(log1e25(prestigePoints+1))). */
  upgrade32: number
  /** player.upgrades[45] — non-zero adds min(2500, floor(log10(transcendShards+1))). */
  upgrade45: number
  /** player.upgrades[46] — when > 0.5, sets tuSevenMulti to 1.05 (else 1). */
  upgrade46: number
  /** player.prestigePoints — log-base argument for upgrade-32. */
  prestigePoints: Decimal
  /** player.transcendShards — log-base argument for upgrade-45. */
  transcendShards: Decimal
  /** player.challengecompletions[1] — used in t-chal 1 acceleratorPower formula. */
  c1Completions: number
  /** player.challengecompletions[2] — fed through CalcECC by caller (transcend) for ecc2tr. */
  c2Completions: number
  /** player.challengecompletions[7] — fed through CalcECC by caller (reincarnation) for ecc7r. */
  c7Completions: number
  /** player.currentChallenge.transcension — controls acceleratorPower overrides for 1/2/3. */
  transcensionChallenge: number
  /** player.currentChallenge.reincarnation — controls acceleratorPower / acceleratorEffect overrides for 7/10. */
  reincarnationChallenge: number
  /** player.researches[18] — adds +2 per level to the totalAcceleratorBoost factor. */
  research18: number
  /** player.researches[19] — adds +2 per level to the totalAcceleratorBoost factor. */
  research19: number
  /** player.researches[20] — adds +3 per level to the totalAcceleratorBoost factor. */
  research20: number
  /** player.platonicUpgrades[6] — contributes (1 + n/30) to the exponent on `a`. */
  platonicUpgrade6: number
  /** player.unlocks.prestige — gates the multiplicative-accelerators rune effect. */
  prestigeUnlocked: boolean
  /** player.corruptions.used.viscosity — sources viscosityPower and triggers >=15 / >=16 cuts. */
  viscosityCorruptionLevel: number

  // ─── Pre-evaluated effects / rewards / blessings ──────────────────────

  /** `+getAchievementReward('accelerators')` — added to base `a` (after upgrades). */
  acceleratorsAchievement: number
  /** `+getAchievementReward('acceleratorPower')` — additive term inside acceleratorPower. */
  acceleratorPowerAchievement: number
  /** `getRuneEffects('speed', 'multiplicativeAccelerators')` — multiplies `a` when prestige unlocked. */
  multiplicativeAcceleratorsRune: number
  /** `getRuneEffects('speed', 'acceleratorPower')` — additive term inside acceleratorPower. */
  acceleratorPowerRune: number
  /** `calculateAcceleratorCubeBlessing()` — adds to the totalAcceleratorBoost factor. */
  acceleratorCubeBlessing: number
  /** `getHepteractEffects('accelerator').accelerators` — added to `a` after rune+multiplier+power steps. */
  hepteractAccelerators: number
  /** `getHepteractEffects('accelerator').acceleratorMultiplier` — multiplies `a` after the additive step. */
  hepteractAcceleratorMult: number

  // ─── G inputs (pre-extracted by web_ui) ───────────────────────────────

  /** G.totalAcceleratorBoost (set by calculateTotalAcceleratorBoost prior to this call). */
  totalAcceleratorBoost: number
  /** G.acceleratorMultiplier (set by calculateAcceleratorMultiplier prior to this call). */
  acceleratorMultiplier: number
  /** G.viscosityPower[player.corruptions.used.viscosity] — viscosity-corruption exponent factor. */
  viscosityPower: number
  /** G.challenge15Rewards.accelerator.value — multiplied into `a` after the hepteract additive. */
  challenge15RewardAccelerator: number
}

export interface UpdateAllTickResult {
  /** G.totalAccelerator (= acceleratorBought + freeAccelerator). */
  totalAccelerator: number
  /** G.costDivisor — always set to 1 here, surfaced for parity. */
  costDivisor: number
  /** G.freeUpgradeAccelerator — `a` snapshot right after upgrades/achievements/ECC are folded in. */
  freeUpgradeAccelerator: number
  /** G.freeAccelerator — final floored `a` after all multiplicative + corruption gating. */
  freeAccelerator: number
  /** G.tuSevenMulti — 1.05 when upgrade46 > 0.5, else 1. */
  tuSevenMulti: number
  /** G.acceleratorPower — used as exponent base for acceleratorEffect. */
  acceleratorPower: number
  /** G.acceleratorEffect — Decimal pow result; reads totalAccelerator (+ totalMultiplier when t-chal 1). */
  acceleratorEffect: Decimal
  /** G.acceleratorEffectDisplay — `acceleratorPower * 100 - 100` wrapped in Decimal. */
  acceleratorEffectDisplay: Decimal
  /** G.generatorPower — `1.02 ^ totalAccelerator` when upgrade11 active outside r-chal 7; 1 otherwise. */
  generatorPower: Decimal
}

/**
 * Per-tick accelerator-state aggregator. See file-level comment for the
 * computation sequence; mirrors the legacy updateAllTick body verbatim aside
 * from input/output shape.
 *
 * @param totalMultiplier  G.totalMultiplier at call time — read only for the
 *                         t-chal 1 acceleratorEffect exponent
 *                         (`acceleratorPower ^ (totalAccelerator + totalMultiplier)`).
 *                         Separate from `input` because it's set by updateAllMultiplier
 *                         in the legacy ordering and we want the parity test to
 *                         exercise both with/without that completed.
 */
export function updateAllTick (
  input: UpdateAllTickInput,
  totalMultiplier: number
): UpdateAllTickResult {
  let a = 0

  const totalAcceleratorInit = input.acceleratorBought
  const costDivisor = 1

  if (input.upgrade8 !== 0) {
    a += Math.floor(input.multiplierBought / 7)
  }
  if (input.upgrade21 !== 0) {
    a += 5
  }
  if (input.upgrade22 !== 0) {
    a += 4
  }
  if (input.upgrade23 !== 0) {
    a += 3
  }
  if (input.upgrade24 !== 0) {
    a += 2
  }
  if (input.upgrade25 !== 0) {
    a += 1
  }
  if (input.upgrade32 !== 0) {
    a += Math.min(500, Math.floor(Decimal.log(input.prestigePoints.add(1), 1e25)))
  }
  if (input.upgrade45 !== 0) {
    a += Math.min(2500, Math.floor(Decimal.log(input.transcendShards.add(1), 10)))
  }
  a += input.acceleratorsAchievement

  const ecc2tr = CalcECC('transcend', input.c2Completions)
  const ecc7r = CalcECC('reincarnation', input.c7Completions)

  a += 5 * ecc2tr
  const freeUpgradeAccelerator = a

  a += input.totalAcceleratorBoost
    * (5
      + 2 * input.research18
      + 2 * input.research19
      + 3 * input.research20
      + input.acceleratorCubeBlessing)

  if (input.prestigeUnlocked) {
    a *= input.multiplicativeAcceleratorsRune
  }

  a *= input.acceleratorMultiplier
  a = Math.pow(
    a,
    Math.min(1, (1 + input.platonicUpgrade6 / 30) * input.viscosityPower)
  )
  a += input.hepteractAccelerators
  a *= input.challenge15RewardAccelerator
  a *= input.hepteractAcceleratorMult
  a = Math.floor(Math.min(1e100, a))

  if (input.viscosityCorruptionLevel >= 15) {
    a = Math.pow(a, 0.2)
  }
  if (input.viscosityCorruptionLevel >= 16) {
    a = 1
  }

  const freeAccelerator = a
  const totalAccelerator = totalAcceleratorInit + freeAccelerator

  const tuSevenMulti = input.upgrade46 > 0.5 ? 1.05 : 1

  let acceleratorPower = Math.pow(
    1.1
      + input.acceleratorPowerRune
      + 1 / 400 * ecc2tr
      + input.acceleratorPowerAchievement
      + tuSevenMulti
        * (input.totalAcceleratorBoost / 100)
        * (1 + ecc2tr / 20),
    1 + 0.04 * ecc7r
  )

  // No-MA and Sadistic challenges overwrite the transcension overrides
  if (
    input.reincarnationChallenge !== 7
    && input.reincarnationChallenge !== 10
  ) {
    if (input.transcensionChallenge === 1) {
      acceleratorPower *= 25 / (50 + input.c1Completions)
      acceleratorPower += 0.55
      acceleratorPower = Math.max(1, acceleratorPower)
    }
    if (input.transcensionChallenge === 2) {
      acceleratorPower = 1
    }
    if (input.transcensionChallenge === 3) {
      acceleratorPower = 1 + acceleratorPower / 2
    }
  }
  acceleratorPower = Math.min(1e300, acceleratorPower)
  if (input.reincarnationChallenge === 7) {
    acceleratorPower = 1
  }
  if (input.reincarnationChallenge === 10) {
    acceleratorPower = 1
  }

  let acceleratorEffect: Decimal
  if (input.transcensionChallenge !== 1) {
    acceleratorEffect = Decimal.pow(acceleratorPower, totalAccelerator)
  } else {
    acceleratorEffect = Decimal.pow(acceleratorPower, totalAccelerator + totalMultiplier)
  }
  const acceleratorEffectDisplay = new Decimal(acceleratorPower * 100 - 100)
  if (input.reincarnationChallenge === 10) {
    acceleratorEffect = new Decimal(1)
  }

  let generatorPower = new Decimal(1)
  if (input.upgrade11 > 0.5 && input.reincarnationChallenge !== 7) {
    generatorPower = Decimal.pow(1.02, totalAccelerator)
  }

  return {
    totalAccelerator,
    costDivisor,
    freeUpgradeAccelerator,
    freeAccelerator,
    tuSevenMulti,
    acceleratorPower,
    acceleratorEffect,
    acceleratorEffectDisplay,
    generatorPower
  }
}
