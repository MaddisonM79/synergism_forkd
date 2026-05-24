// Per-tick reset/quark/singularity counter advancement. Lifted from
// packages/web_ui/src/Helper.ts (addTimers, simple counter cases).
//
// Covers 9 of the 11 addTimers cases:
//   prestige, transcension, reincarnation (shared shape: counter += time × mult)
//   ascension (dual counter + ascensionSpeedMulti)
//   singularity (triple counter + singularitySpeedMulti + challenge timer)
//   quarks (capped at maxQuarkTimer)
//   goldenQuarks (capped at 168 hours, gated by exportGQPerHour > 0)
//   ambrosia (chunked + seeded-random luck roll + recursive bar grant)
//   redAmbrosia (chunked + seeded-random luck roll + bonus blueberry time)
//
// The remaining two cases stay in web_ui:
//   octeracts — calculateGoldenQuarks called inside the giveaway loop
//     reads `player.quarksThisSingularity` that the loop mutates each
//     iteration; migrating cleanly requires inlining the goldenQuarks
//     stat dependency or restructuring the loop semantics.
//   autoPotion — fires `useConsumable(...)` which mutates player and
//     enqueues DOM/modal side effects; needs the consumable subsystem
//     migrated first.
//
// Caller pre-evaluates the per-tick globalTimeMultiplier
// (`getGQUpgradeEffect('halfMind', 'unlocked') ? 10 : calculateGlobalSpeedMult()`)
// and per-case speed multipliers (`ascensionSpeedMulti`, `singularitySpeedMulti`)
// and the various caps (`maxQuarkTimer`, `exportGQPerHour`).

// Shared counter advancement for prestige / transcension / reincarnation.
// Each of these three timers uses the same shape:
//   counter += time * globalTimeMultiplier
// — no caps, no conditional speed multipliers.
export function advanceResetCounter (
  counter: number,
  time: number,
  globalTimeMultiplier: number
): number {
  return counter + time * globalTimeMultiplier
}

export interface AdvanceAscensionTimerInput {
  /** Tick delta (seconds). */
  time: number
  /** player.ascensionCounter — advances by time × ascensionSpeedMulti. */
  ascensionCounter: number
  /** player.ascensionCounterReal — advances by time only (unaffected by ascension speed). */
  ascensionCounterReal: number
  /** Pre-evaluated ascension speed: `getGQUpgradeEffect('oneMind', 'unlocked') ? 10 : calculateAscensionSpeedMult()`. */
  ascensionSpeedMulti: number
}

export interface AdvanceAscensionTimerResult {
  ascensionCounter: number
  ascensionCounterReal: number
}

/**
 * Advance the ascension dual-counter. ascensionCounter scales with the
 * pre-evaluated `ascensionSpeedMulti`; ascensionCounterReal is raw wall
 * time. The ascension timer case in legacy addTimers passes `timeMultiplier
 * = 1` (not globalTimeMultiplier), so we don't take it as input.
 */
export function advanceAscensionTimer (input: AdvanceAscensionTimerInput): AdvanceAscensionTimerResult {
  return {
    ascensionCounter: input.ascensionCounter + input.time * input.ascensionSpeedMulti,
    ascensionCounterReal: input.ascensionCounterReal + input.time
  }
}

export interface AdvanceSingularityTimerInput {
  /** Tick delta (seconds). */
  time: number
  /** player.ascensionCounterRealReal — advances by raw `time`. */
  ascensionCounterRealReal: number
  /** player.singularityCounter — advances by time × singularitySpeedMulti. */
  singularityCounter: number
  /** player.singChallengeTimer — advances by time × singularitySpeedMulti
   * when insideSingularityChallenge, else reset to 0. */
  singChallengeTimer: number
  /** player.insideSingularityChallenge — gates the singChallengeTimer
   * accumulation vs. reset. */
  insideSingularityChallenge: boolean
  /** Pre-evaluated `getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'singularitySpeedMult')`. */
  singularitySpeedMulti: number
}

export interface AdvanceSingularityTimerResult {
  ascensionCounterRealReal: number
  singularityCounter: number
  singChallengeTimer: number
}

/**
 * Advance the singularity tri-counter. Same `time` input feeds all three.
 * `singChallengeTimer` accumulates only when `insideSingularityChallenge`,
 * otherwise it resets to 0 every tick.
 */
export function advanceSingularityTimer (input: AdvanceSingularityTimerInput): AdvanceSingularityTimerResult {
  const ascensionCounterRealReal = input.ascensionCounterRealReal + input.time
  const singularityCounter = input.singularityCounter + input.time * input.singularitySpeedMulti
  const singChallengeTimer = input.insideSingularityChallenge
    ? input.singChallengeTimer + input.time * input.singularitySpeedMulti
    : 0
  return { ascensionCounterRealReal, singularityCounter, singChallengeTimer }
}

export interface AdvanceQuarksTimerInput {
  /** Tick delta (seconds). */
  time: number
  /** player.quarkstimer — advances by raw `time`, clamped to `maxQuarkTimer`. */
  quarkstimer: number
  /** Pre-evaluated `quarkHandler().maxTime` — upper bound on quarkstimer. */
  maxQuarkTimer: number
}

/**
 * Advance the quark export timer, clamped at `maxQuarkTimer` (~25 hours,
 * extended by Research 8x20). Legacy uses `timeMultiplier = 1` here.
 */
export function advanceQuarksTimer (input: AdvanceQuarksTimerInput): number {
  const advanced = input.quarkstimer + input.time
  return advanced > input.maxQuarkTimer ? input.maxQuarkTimer : advanced
}

export interface AdvanceGoldenQuarksTimerInput {
  /** Tick delta (seconds). */
  time: number
  /** player.goldenQuarksTimer — advances by raw `time`, clamped to 168 hours
   * (`3600 * 168`). */
  goldenQuarksTimer: number
  /** Pre-evaluated `getGQUpgradeEffect('goldenQuarks3', 'exportGQPerHour')` —
   * when 0, the timer doesn't advance at all (return value === current value). */
  exportGQPerHour: number
}

const GOLDEN_QUARKS_TIMER_CAP_SECONDS = 3600 * 168

/**
 * Advance the golden-quark export timer, gated by the `goldenQuarks3`
 * GQ upgrade. When `exportGQPerHour === 0`, the timer is untouched.
 * Otherwise it accumulates raw `time` and clamps to the 168-hour cap.
 */
export function advanceGoldenQuarksTimer (input: AdvanceGoldenQuarksTimerInput): number {
  if (input.exportGQPerHour === 0) {
    return input.goldenQuarksTimer
  }
  const advanced = input.goldenQuarksTimer + input.time
  return advanced > GOLDEN_QUARKS_TIMER_CAP_SECONDS ? GOLDEN_QUARKS_TIMER_CAP_SECONDS : advanced
}

// ─── Ambrosia ──────────────────────────────────────────────────────────────

import type { CoreEvent } from '../events/types'
import { calculateRequiredBlueberryTime, calculateRequiredRedAmbrosiaTime } from '../mechanics/ambrosia'
import { seededRandom } from '../math/rng'

export interface AdvanceAmbrosiaTimerInput {
  /** Tick delta (seconds). */
  time: number
  /** Pre-evaluated per-tick globalTimeMultiplier (halfMind or globalSpeedMult). */
  timeMultiplier: number

  // ─── Gates ─────────────────────────────────────────────────────────
  /** player.singularityChallenges.noSingularityUpgrades.completions — the whole
   * branch is gated on `> 0`. */
  noSingularityUpgradesCompletions: number
  /** Pre-evaluated `calculateAmbrosiaGenerationSpeed()` — when 0, the branch
   * short-circuits before touching ambrosiaTimerG. (Legacy calls this twice
   * with `compute === 0` check then re-reads as `baseBlueberryTime`; the
   * function is pure for the tick so passing one value is fine.) */
  ambrosiaGenerationSpeed: number

  // ─── State accumulators ───────────────────────────────────────────
  /** G.ambrosiaTimer — fractional accumulator; processes at 1/8s
   * granularity (anything below 0.125 short-circuits). */
  ambrosiaTimerG: number
  /** player.blueberryTime — accumulates `floor(8 * ambrosiaTimerG)/8 *
   * baseBlueberryTime` per tick; loop consumes it via timeToAmbrosia. */
  blueberryTime: number
  /** player.ambrosia — credited by each loop iteration. */
  ambrosia: number
  /** player.lifetimeAmbrosia — same delta as ambrosia; feeds back into
   * `calculateRequiredBlueberryTime` for the next iteration's threshold. */
  lifetimeAmbrosia: number
  /** player.seed[Seed.Ambrosia] — RNG state advanced once per loop iteration. */
  seed: number

  // ─── Pre-evaluated per-tick lookups (stable across iterations) ────
  /** Pre-evaluated `calculateAmbrosiaLuck()` — drives ambrosiaMult + luckMult. */
  ambrosiaLuck: number
  /** Pre-evaluated `getSingularityChallengeEffect('noAmbrosiaUpgrades', 'bonusAmbrosia')`. */
  bonusAmbrosia: number

  // ─── Inputs for inner calculateRequiredBlueberryTime calls ─────────
  /** G.TIME_PER_AMBROSIA — constant base. */
  timePerAmbrosia: number
  /** getShopUpgradeEffects('shopAmbrosiaAccelerator', 'ambrosiaPointRequirementMult'). */
  acceleratorMult: number
  /** getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'barRequirementMult'). */
  brickOfLeadMult: number
}

export interface AdvanceAmbrosiaTimerResult {
  ambrosiaTimerG: number
  blueberryTime: number
  ambrosia: number
  lifetimeAmbrosia: number
  seed: number
  /** `ambrosia-gained` event with the total delta when any iteration ran;
   * empty otherwise. UI handler refreshes the ambrosia display. */
  events: CoreEvent[]
}

/**
 * Ambrosia case of addTimers. Accumulates G.ambrosiaTimer in 1/8s ticks,
 * adds blueberryTime, then loops crediting ambrosia bars when
 * blueberryTime meets the (mutating) `calculateRequiredBlueberryTime`
 * threshold. Each iteration rolls one seededRandom value for luck.
 *
 * Pre-tick gates that short-circuit (returning unchanged state, no event):
 *   - noSingularityUpgradesCompletions === 0 — feature locked.
 *   - ambrosiaGenerationSpeed === 0 — disabled (most paths).
 *   - ambrosiaTimerG + dt*mult < 0.125 — sub-tick threshold not met.
 */
export function advanceAmbrosiaTimer (input: AdvanceAmbrosiaTimerInput): AdvanceAmbrosiaTimerResult {
  if (input.noSingularityUpgradesCompletions <= 0) {
    return {
      ambrosiaTimerG: input.ambrosiaTimerG,
      blueberryTime: input.blueberryTime,
      ambrosia: input.ambrosia,
      lifetimeAmbrosia: input.lifetimeAmbrosia,
      seed: input.seed,
      events: []
    }
  }
  if (input.ambrosiaGenerationSpeed === 0) {
    return {
      ambrosiaTimerG: input.ambrosiaTimerG,
      blueberryTime: input.blueberryTime,
      ambrosia: input.ambrosia,
      lifetimeAmbrosia: input.lifetimeAmbrosia,
      seed: input.seed,
      events: []
    }
  }

  let ambrosiaTimerG = input.ambrosiaTimerG + input.time * input.timeMultiplier
  if (ambrosiaTimerG < 0.125) {
    return {
      ambrosiaTimerG,
      blueberryTime: input.blueberryTime,
      ambrosia: input.ambrosia,
      lifetimeAmbrosia: input.lifetimeAmbrosia,
      seed: input.seed,
      events: []
    }
  }

  let blueberryTime = input.blueberryTime + Math.floor(8 * ambrosiaTimerG) / 8 * input.ambrosiaGenerationSpeed
  ambrosiaTimerG %= 0.125

  let ambrosia = input.ambrosia
  let lifetimeAmbrosia = input.lifetimeAmbrosia
  let seed = input.seed
  let totalGained = 0

  let timeToAmbrosia = calculateRequiredBlueberryTime({
    timePerAmbrosia: input.timePerAmbrosia,
    lifetimeAmbrosia,
    acceleratorMult: input.acceleratorMult,
    brickOfLeadMult: input.brickOfLeadMult
  })

  while (blueberryTime >= timeToAmbrosia) {
    const rng = seededRandom(seed)
    seed = rng.newSeed
    const ambrosiaMult = Math.floor(input.ambrosiaLuck / 100)
    const luckMult = rng.value < input.ambrosiaLuck / 100 - Math.floor(input.ambrosiaLuck / 100) ? 1 : 0
    const ambrosiaToGain = (ambrosiaMult + luckMult) + input.bonusAmbrosia

    ambrosia += ambrosiaToGain
    lifetimeAmbrosia += ambrosiaToGain
    totalGained += ambrosiaToGain
    blueberryTime -= timeToAmbrosia

    timeToAmbrosia = calculateRequiredBlueberryTime({
      timePerAmbrosia: input.timePerAmbrosia,
      lifetimeAmbrosia,
      acceleratorMult: input.acceleratorMult,
      brickOfLeadMult: input.brickOfLeadMult
    })
  }

  return {
    ambrosiaTimerG,
    blueberryTime,
    ambrosia,
    lifetimeAmbrosia,
    seed,
    events: [{ kind: 'ambrosia-gained', amount: totalGained }]
  }
}

// ─── Red Ambrosia ─────────────────────────────────────────────────────────

export interface AdvanceRedAmbrosiaTimerInput {
  /** Tick delta (seconds). */
  time: number
  /** Pre-evaluated per-tick globalTimeMultiplier. */
  timeMultiplier: number

  // ─── Gates ─────────────────────────────────────────────────────────
  /** player.singularityChallenges.noAmbrosiaUpgrades.completions — branch
   * gated on `> 0`. */
  noAmbrosiaUpgradesCompletions: number
  /** Pre-evaluated `calculateRedAmbrosiaGenerationSpeed()`. */
  redAmbrosiaGenerationSpeed: number

  // ─── State accumulators ───────────────────────────────────────────
  /** G.redAmbrosiaTimer — fractional accumulator; same 1/8s chunking as
   * ambrosia. */
  redAmbrosiaTimerG: number
  /** player.redAmbrosiaTime — receives `floor(8 * timerG)/8 * speed`. */
  redAmbrosiaTime: number
  /** player.redAmbrosia — credited per loop iteration. */
  redAmbrosia: number
  /** player.lifetimeRedAmbrosia — same delta as redAmbrosia; feeds back
   * into `calculateRequiredRedAmbrosiaTime` for the next iteration. */
  lifetimeRedAmbrosia: number
  /** player.seed[Seed.RedAmbrosia] — advanced once per iteration. */
  seed: number

  // ─── Pre-evaluated per-tick lookups (stable across iterations) ────
  /** Pre-evaluated `calculateRedAmbrosiaLuck()` — drives the luck rolls.
   * Legacy calls this inside the loop, but it depends only on tick-stable
   * player state, so a single pre-eval is bug-for-bug equivalent. */
  redAmbrosiaLuck: number
  /** Pre-evaluated `getRedAmbrosiaUpgradeEffects('redAmbrosiaAccelerator',
   * 'ambrosiaTimePerRedAmbrosia')` — bonus blueberry time per red ambrosia. */
  ambrosiaTimePerRedAmbrosia: number

  // ─── Inputs for inner calculateRequiredRedAmbrosiaTime calls ───────
  /** G.TIME_PER_RED_AMBROSIA — constant base. */
  timePerRedAmbrosia: number
  /** Pre-evaluated `getSingularityChallengeEffect('limitedTime',
   * 'barRequirementMultiplier')`. */
  barRequirementMultiplier: number
}

export interface AdvanceRedAmbrosiaTimerResult {
  redAmbrosiaTimerG: number
  redAmbrosiaTime: number
  redAmbrosia: number
  lifetimeRedAmbrosia: number
  seed: number
  /** Bonus blueberry time accumulated this tick — caller must feed into
   * the ambrosia branch (`addTimers('ambrosia', bonusAmbrosiaTime)` in the
   * legacy shim). 0 when no iteration ran or the upgrade is inactive. */
  bonusAmbrosiaTime: number
  /** `red-ambrosia-gained` event when any iteration ran; empty otherwise. */
  events: CoreEvent[]
}

/**
 * Red Ambrosia case of addTimers. Same shape as the ambrosia case:
 * 1/8s chunked timer accumulation → blueberry-time-equivalent credit →
 * inner loop that mints red ambrosia when redAmbrosiaTime meets the
 * (mutating) `calculateRequiredRedAmbrosiaTime` threshold. Additionally
 * accumulates `bonusAmbrosiaTime` (`redAmbrosiaToGain * timeCoeff`) that
 * the caller feeds into the ambrosia timer afterward.
 */
export function advanceRedAmbrosiaTimer (input: AdvanceRedAmbrosiaTimerInput): AdvanceRedAmbrosiaTimerResult {
  if (input.noAmbrosiaUpgradesCompletions <= 0) {
    return {
      redAmbrosiaTimerG: input.redAmbrosiaTimerG,
      redAmbrosiaTime: input.redAmbrosiaTime,
      redAmbrosia: input.redAmbrosia,
      lifetimeRedAmbrosia: input.lifetimeRedAmbrosia,
      seed: input.seed,
      bonusAmbrosiaTime: 0,
      events: []
    }
  }

  let redAmbrosiaTimerG = input.redAmbrosiaTimerG + input.time * input.timeMultiplier
  if (redAmbrosiaTimerG < 0.125) {
    return {
      redAmbrosiaTimerG,
      redAmbrosiaTime: input.redAmbrosiaTime,
      redAmbrosia: input.redAmbrosia,
      lifetimeRedAmbrosia: input.lifetimeRedAmbrosia,
      seed: input.seed,
      bonusAmbrosiaTime: 0,
      events: []
    }
  }

  let redAmbrosiaTime = input.redAmbrosiaTime
    + Math.floor(8 * redAmbrosiaTimerG) / 8 * input.redAmbrosiaGenerationSpeed
  redAmbrosiaTimerG %= 0.125

  let redAmbrosia = input.redAmbrosia
  let lifetimeRedAmbrosia = input.lifetimeRedAmbrosia
  let seed = input.seed
  let totalGained = 0
  let bonusAmbrosiaTime = 0

  let timeToRedAmbrosia = calculateRequiredRedAmbrosiaTime({
    timePerRedAmbrosia: input.timePerRedAmbrosia,
    lifetimeRedAmbrosia,
    barRequirementMultiplier: input.barRequirementMultiplier
  })

  while (redAmbrosiaTime >= timeToRedAmbrosia) {
    const rng = seededRandom(seed)
    seed = rng.newSeed
    const redAmbrosiaMult = Math.floor(input.redAmbrosiaLuck / 100)
    const luckMult = rng.value < input.redAmbrosiaLuck / 100 - Math.floor(input.redAmbrosiaLuck / 100) ? 1 : 0
    const redAmbrosiaToGain = redAmbrosiaMult + luckMult

    redAmbrosia += redAmbrosiaToGain
    lifetimeRedAmbrosia += redAmbrosiaToGain
    totalGained += redAmbrosiaToGain
    bonusAmbrosiaTime += redAmbrosiaToGain * input.ambrosiaTimePerRedAmbrosia
    redAmbrosiaTime -= timeToRedAmbrosia

    timeToRedAmbrosia = calculateRequiredRedAmbrosiaTime({
      timePerRedAmbrosia: input.timePerRedAmbrosia,
      lifetimeRedAmbrosia,
      barRequirementMultiplier: input.barRequirementMultiplier
    })
  }

  return {
    redAmbrosiaTimerG,
    redAmbrosiaTime,
    redAmbrosia,
    lifetimeRedAmbrosia,
    seed,
    bonusAmbrosiaTime,
    events: [{ kind: 'red-ambrosia-gained', amount: totalGained }]
  }
}
