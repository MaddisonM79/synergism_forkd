// Per-tick reset/quark/singularity counter advancement. Lifted from
// packages/web_ui/src/Helper.ts (addTimers, simple counter cases).
//
// Covers the 7 cases of addTimers that are pure counter math with no
// side effects, no RNG, and no consumable orchestration:
//   prestige, transcension, reincarnation (shared shape: counter += time × mult)
//   ascension (dual counter + ascensionSpeedMulti)
//   singularity (triple counter + singularitySpeedMulti + challenge timer)
//   quarks (capped at maxQuarkTimer)
//   goldenQuarks (capped at 168 hours, gated by exportGQPerHour > 0)
//
// The four complex cases (octeracts, autoPotion, ambrosia, redAmbrosia)
// stay in web_ui for now — each involves modal/consumable side effects or
// the seededRandom RNG, which need their own migration scaffolding.
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
