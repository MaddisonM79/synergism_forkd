// Bundled "head"-side timer composition for the per-tick body. Composes
// all 11 migrated per-case timer functions into a single logic call so
// the web_ui adapter makes one call instead of eleven switch dispatches.
//
// Mirrors the legacy `addTimers(...)` sequence in the per-tick body in
// packages/web_ui/src/Synergism.ts (`tack`), running in legacy order:
//
//   1. prestige      — advanceResetCounter(prestigecounter,    dt, globalTimeMultiplier)
//   2. transcension  — advanceResetCounter(transcendcounter,   dt, globalTimeMultiplier)
//   3. reincarnation — advanceResetCounter(reincarnationcounter, dt, globalTimeMultiplier)
//   4. ascension     — advanceAscensionTimer (timeMultiplier === 1, uses ascensionSpeedMulti)
//   5. quarks        — advanceQuarksTimer (timeMultiplier === 1, capped at maxQuarkTimer)
//   6. goldenQuarks  — advanceGoldenQuarksTimer (timeMultiplier === 1, capped + GQ-export gate)
//   7. octeracts     — advanceOcteractTimer (timeMultiplier === 1, with GQ-giveaway loop)
//   8. singularity   — advanceSingularityTimer (timeMultiplier === 1, uses singularitySpeedMulti)
//   9. autoPotion    — advanceAutoPotionTimer (timeMultiplier === 1, emits auto-potion-fired
//                       events — useConsumable runs in the UI dispatcher)
//  10. ambrosia      — advanceAmbrosiaTimer (timeMultiplier === 1, chunked + seeded RNG)
//  11. redAmbrosia   — advanceRedAmbrosiaTimer (timeMultiplier === 1, chunked + seeded RNG +
//                       bonus blueberry-time feedback fed into a recursive ambrosia advance)
//
// All 11 cases now run inside the bundle in their exact legacy
// positions. The `useConsumable` call autoPotion historically made
// inline now lives in the UI tier (via the `auto-potion-fired` event
// in tickEventHandlers.ts).
//
// The bonus-time feedback loop in case 11 (redAmbrosia → ambrosia) is
// handled internally: if redAmbrosia returns `bonusAmbrosiaTime > 0`,
// we recursively call advanceAmbrosiaTimer with that as `time` and
// `timeMultiplier === 1` (matching the legacy `addTimers('ambrosia',
// bonusAmbrosiaTime)` shim call which uses `timeMultiplier === 1`).
//
// Cases 5-11 use `timeMultiplier === 1` in legacy (see Helper.ts:67-76
// — the bundle reflects that by passing `1` to those cases regardless
// of the caller's `globalTimeMultiplier`). Only cases 1-3 read
// `globalTimeMultiplier` directly. Case 4 (ascension) uses the
// separately-evaluated `ascensionSpeedMulti`; case 8 (singularity)
// uses `singularitySpeedMulti`.

import type { CoreEvent } from '../events/types'
import {
  advanceAmbrosiaTimer,
  advanceAscensionTimer,
  advanceAutoPotionTimer,
  advanceGoldenQuarksTimer,
  advanceOcteractTimer,
  advanceQuarksTimer,
  advanceRedAmbrosiaTimer,
  advanceResetCounter,
  advanceSingularityTimer
} from './timers'

export interface AdvanceAllTimersInput {
  /** Tick delta (seconds). */
  dt: number
  /** Pre-evaluated `getGQUpgradeEffect('halfMind', 'unlocked') ? 10 :
   * calculateGlobalSpeedMult()`. Only the prestige/transcension/
   * reincarnation counters read this; everything else uses
   * `timeMultiplier === 1` in legacy. */
  globalTimeMultiplier: number

  // ─── 1. Prestige / Transcension / Reincarnation counters ───────────
  /** player.prestigecounter. */
  prestigecounter: number
  /** player.transcendcounter. */
  transcendcounter: number
  /** player.reincarnationcounter. */
  reincarnationcounter: number

  // ─── 4. Ascension ──────────────────────────────────────────────────
  /** player.ascensionCounter. */
  ascensionCounter: number
  /** player.ascensionCounterReal. */
  ascensionCounterReal: number
  /** Pre-evaluated `getGQUpgradeEffect('oneMind', 'unlocked') ? 10 :
   * calculateAscensionSpeedMult()`. */
  ascensionSpeedMulti: number

  // ─── 5. Quarks ─────────────────────────────────────────────────────
  /** player.quarkstimer. */
  quarkstimer: number
  /** Pre-evaluated `quarkHandler().maxTime`. */
  maxQuarkTimer: number

  // ─── 6. Golden Quarks ──────────────────────────────────────────────
  /** player.goldenQuarksTimer. */
  goldenQuarksTimer: number
  /** Pre-evaluated `getGQUpgradeEffect('goldenQuarks3', 'exportGQPerHour')` —
   * 0 disables the timer advance entirely. */
  exportGQPerHour: number

  // ─── 7. Octeracts ──────────────────────────────────────────────────
  /** Pre-evaluated `getGQUpgradeEffect('octeractUnlock', 'unlocked')`. */
  octeractUnlocked: boolean
  /** player.octeractTimer. */
  octeractTimer: number
  /** player.wowOcteracts. */
  wowOcteracts: number
  /** player.totalWowOcteracts. */
  totalWowOcteracts: number
  /** player.goldenQuarks — also written by the GQ-giveaway block in case 7. */
  goldenQuarks: number
  /** player.quarksThisSingularity — geometrically decayed inside the
   * giveaway loop. */
  quarksThisSingularity: number
  /** Pre-evaluated `calculateOcteractMultiplier()`. */
  octeractPerSecond: number
  /** player.highestSingularityCount — gates the GQ-giveaway block (≥ 160)
   * and feeds calculateBaseGoldenQuarks + actualLevel. */
  highestSingularityCount: number
  /** player.singularityCount — feeds calculateBaseGoldenQuarks. */
  singularityCount: number
  /** Pre-evaluated product of `allGoldenQuarkMultiplierStats` excluding
   * the qts-dependent base (stat 0). Caller computes this only when the
   * giveaway block will run (highestSingularityCount >= 160); when below
   * threshold, pass 1 — it's unused. */
  goldenQuarksMultiplierExcludingBase: number

  // ─── 8. Singularity ────────────────────────────────────────────────
  /** player.ascensionCounterRealReal. */
  ascensionCounterRealReal: number
  /** player.singularityCounter. */
  singularityCounter: number
  /** player.singChallengeTimer. */
  singChallengeTimer: number
  /** player.insideSingularityChallenge. */
  insideSingularityChallenge: boolean
  /** Pre-evaluated `getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead',
   * 'singularitySpeedMult')`. */
  singularitySpeedMulti: number

  // ─── 9. Auto Potion ────────────────────────────────────────────────
  /** player.autoPotionTimer. */
  autoPotionTimer: number
  /** player.autoPotionTimerObtainium. */
  autoPotionTimerObtainium: number
  /** player.toggles[42] — Fast Offering Potion expenditure. */
  autoPotionToggleOffering: boolean
  /** player.toggles[43] — Fast Obtainium Potion expenditure. */
  autoPotionToggleObtainium: boolean
  /** player.shopUpgrades.offeringPotion. */
  offeringPotionCount: number
  /** player.shopUpgrades.obtainiumPotion. */
  obtainiumPotionCount: number
  /** Pre-evaluated `getOcteractUpgradeEffect('octeractAutoPotionSpeed',
   * 'autoPotionSpeedMult')`. */
  autoPotionSpeedMult: number

  // ─── 10. Ambrosia ──────────────────────────────────────────────────
  /** player.singularityChallenges.noSingularityUpgrades.completions —
   * branch gate (> 0 to run). */
  noSingularityUpgradesCompletions: number
  /** Pre-evaluated `calculateAmbrosiaGenerationSpeed()`. */
  ambrosiaGenerationSpeed: number
  /** G.ambrosiaTimer — fractional 1/8s accumulator. */
  ambrosiaTimerG: number
  /** player.blueberryTime. */
  blueberryTime: number
  /** player.ambrosia. */
  ambrosia: number
  /** player.lifetimeAmbrosia. */
  lifetimeAmbrosia: number
  /** player.seed[Seed.Ambrosia]. */
  ambrosiaSeed: number
  /** Pre-evaluated `calculateAmbrosiaLuck()`. */
  ambrosiaLuck: number
  /** Pre-evaluated `getSingularityChallengeEffect('noAmbrosiaUpgrades',
   * 'bonusAmbrosia')`. */
  bonusAmbrosia: number
  /** G.TIME_PER_AMBROSIA. */
  timePerAmbrosia: number
  /** Pre-evaluated `getShopUpgradeEffects('shopAmbrosiaAccelerator',
   * 'ambrosiaPointRequirementMult')`. */
  ambrosiaAcceleratorMult: number
  /** Pre-evaluated `getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead',
   * 'barRequirementMult')`. */
  ambrosiaBrickOfLeadMult: number

  // ─── 11. Red Ambrosia ──────────────────────────────────────────────
  /** player.singularityChallenges.noAmbrosiaUpgrades.completions — branch
   * gate (> 0 to run). */
  noAmbrosiaUpgradesCompletions: number
  /** Pre-evaluated `calculateRedAmbrosiaGenerationSpeed()`. */
  redAmbrosiaGenerationSpeed: number
  /** G.redAmbrosiaTimer. */
  redAmbrosiaTimerG: number
  /** player.redAmbrosiaTime. */
  redAmbrosiaTime: number
  /** player.redAmbrosia. */
  redAmbrosia: number
  /** player.lifetimeRedAmbrosia. */
  lifetimeRedAmbrosia: number
  /** player.seed[Seed.RedAmbrosia]. */
  redAmbrosiaSeed: number
  /** Pre-evaluated `calculateRedAmbrosiaLuck()`. */
  redAmbrosiaLuck: number
  /** Pre-evaluated `getRedAmbrosiaUpgradeEffects('redAmbrosiaAccelerator',
   * 'ambrosiaTimePerRedAmbrosia')`. */
  ambrosiaTimePerRedAmbrosia: number
  /** G.TIME_PER_RED_AMBROSIA. */
  timePerRedAmbrosia: number
  /** Pre-evaluated `getSingularityChallengeEffect('limitedTime',
   * 'barRequirementMultiplier')`. */
  redAmbrosiaBarRequirementMultiplier: number
}

export interface AdvanceAllTimersResult {
  // ─── 1-3 ───────────────────────────────────────────────────────────
  prestigecounter: number
  transcendcounter: number
  reincarnationcounter: number

  // ─── 4 ─────────────────────────────────────────────────────────────
  ascensionCounter: number
  ascensionCounterReal: number

  // ─── 5 ─────────────────────────────────────────────────────────────
  quarkstimer: number

  // ─── 6 ─────────────────────────────────────────────────────────────
  goldenQuarksTimer: number

  // ─── 7 ─────────────────────────────────────────────────────────────
  octeractTimer: number
  wowOcteracts: number
  totalWowOcteracts: number
  goldenQuarks: number
  quarksThisSingularity: number

  // ─── 8 ─────────────────────────────────────────────────────────────
  ascensionCounterRealReal: number
  singularityCounter: number
  singChallengeTimer: number

  // ─── 9 ─────────────────────────────────────────────────────────────
  autoPotionTimer: number
  autoPotionTimerObtainium: number

  // ─── 10 ────────────────────────────────────────────────────────────
  ambrosiaTimerG: number
  blueberryTime: number
  ambrosia: number
  lifetimeAmbrosia: number
  ambrosiaSeed: number

  // ─── 11 ────────────────────────────────────────────────────────────
  redAmbrosiaTimerG: number
  redAmbrosiaTime: number
  redAmbrosia: number
  lifetimeRedAmbrosia: number
  redAmbrosiaSeed: number

  /** Composed event list — octeract / auto-potion / ambrosia /
   * red-ambrosia events in the same order they were produced. The
   * recursive ambrosia advance from the redAmbrosia bonus feedback may
   * add a second `ambrosia-gained` event after the
   * `red-ambrosia-gained` event. */
  events: CoreEvent[]
}

/**
 * Pure composition of the eleven migrated per-tick timer cases.
 * Mirrors the legacy `addTimers(...)` sweep in `tack` (Synergism.ts)
 * 1:1 — see the file header for the case-by-case mapping.
 *
 * No gating beyond what each per-case function does internally: an
 * octeract-locked save, an autoPotion-locked save (sing < 6), an
 * ambrosia-locked save, or a redAmbrosia-locked save will
 * short-circuit inside their respective cases and leave both state
 * and events untouched.
 */
export function advanceAllTimers (input: AdvanceAllTimersInput): AdvanceAllTimersResult {
  const events: CoreEvent[] = []

  // ─── 1. Prestige ───────────────────────────────────────────────────
  const prestigecounter = advanceResetCounter(input.prestigecounter, input.dt, input.globalTimeMultiplier)

  // ─── 2. Transcension ───────────────────────────────────────────────
  const transcendcounter = advanceResetCounter(input.transcendcounter, input.dt, input.globalTimeMultiplier)

  // ─── 3. Reincarnation ──────────────────────────────────────────────
  const reincarnationcounter = advanceResetCounter(input.reincarnationcounter, input.dt, input.globalTimeMultiplier)

  // ─── 4. Ascension ──────────────────────────────────────────────────
  const ascR = advanceAscensionTimer({
    time: input.dt,
    ascensionCounter: input.ascensionCounter,
    ascensionCounterReal: input.ascensionCounterReal,
    ascensionSpeedMulti: input.ascensionSpeedMulti
  })

  // ─── 5. Quarks ─────────────────────────────────────────────────────
  const quarkstimer = advanceQuarksTimer({
    time: input.dt,
    quarkstimer: input.quarkstimer,
    maxQuarkTimer: input.maxQuarkTimer
  })

  // ─── 6. Golden Quarks ──────────────────────────────────────────────
  const goldenQuarksTimer = advanceGoldenQuarksTimer({
    time: input.dt,
    goldenQuarksTimer: input.goldenQuarksTimer,
    exportGQPerHour: input.exportGQPerHour
  })

  // ─── 7. Octeracts ──────────────────────────────────────────────────
  // timeMultiplier === 1 (legacy Helper.ts case 'octeracts' is in the
  // == 1 list). Internal gate on octeractUnlocked.
  const octR = advanceOcteractTimer({
    time: input.dt,
    timeMultiplier: 1,
    octeractUnlocked: input.octeractUnlocked,
    octeractTimer: input.octeractTimer,
    wowOcteracts: input.wowOcteracts,
    totalWowOcteracts: input.totalWowOcteracts,
    goldenQuarks: input.goldenQuarks,
    quarksThisSingularity: input.quarksThisSingularity,
    perSecond: input.octeractPerSecond,
    highestSingularityCount: input.highestSingularityCount,
    singularityCount: input.singularityCount,
    goldenQuarksMultiplierExcludingBase: input.goldenQuarksMultiplierExcludingBase
  })
  for (const e of octR.events) events.push(e)

  // ─── 8. Singularity ────────────────────────────────────────────────
  const singR = advanceSingularityTimer({
    time: input.dt,
    ascensionCounterRealReal: input.ascensionCounterRealReal,
    singularityCounter: input.singularityCounter,
    singChallengeTimer: input.singChallengeTimer,
    insideSingularityChallenge: input.insideSingularityChallenge,
    singularitySpeedMulti: input.singularitySpeedMulti
  })

  // ─── 9. Auto Potion ────────────────────────────────────────────────
  // timeMultiplier === 1 in legacy. Internal gate on
  // highestSingularityCount < 6. Emits up to two `auto-potion-fired`
  // events (one per side: offering, obtainium) when the timers cross
  // threshold; the UI dispatcher translates them into useConsumable.
  const autoPotionR = advanceAutoPotionTimer({
    time: input.dt,
    timeMultiplier: 1,
    highestSingularityCount: input.highestSingularityCount,
    autoPotionTimer: input.autoPotionTimer,
    autoPotionTimerObtainium: input.autoPotionTimerObtainium,
    toggleOffering: input.autoPotionToggleOffering,
    toggleObtainium: input.autoPotionToggleObtainium,
    offeringPotionCount: input.offeringPotionCount,
    obtainiumPotionCount: input.obtainiumPotionCount,
    autoPotionSpeedMult: input.autoPotionSpeedMult
  })
  for (const e of autoPotionR.events) events.push(e)

  // ─── 10. Ambrosia ──────────────────────────────────────────────────
  // timeMultiplier === 1 in legacy. Internal gates on
  // noSingularityUpgradesCompletions === 0 and ambrosiaGenerationSpeed === 0.
  const ambR = advanceAmbrosiaTimer({
    time: input.dt,
    timeMultiplier: 1,
    noSingularityUpgradesCompletions: input.noSingularityUpgradesCompletions,
    ambrosiaGenerationSpeed: input.ambrosiaGenerationSpeed,
    ambrosiaTimerG: input.ambrosiaTimerG,
    blueberryTime: input.blueberryTime,
    ambrosia: input.ambrosia,
    lifetimeAmbrosia: input.lifetimeAmbrosia,
    seed: input.ambrosiaSeed,
    ambrosiaLuck: input.ambrosiaLuck,
    bonusAmbrosia: input.bonusAmbrosia,
    timePerAmbrosia: input.timePerAmbrosia,
    acceleratorMult: input.ambrosiaAcceleratorMult,
    brickOfLeadMult: input.ambrosiaBrickOfLeadMult
  })
  for (const e of ambR.events) events.push(e)

  // ─── 11. Red Ambrosia ──────────────────────────────────────────────
  // timeMultiplier === 1 in legacy. Internal gate on
  // noAmbrosiaUpgradesCompletions === 0. Returns bonusAmbrosiaTime that
  // we feed back into ambrosia below.
  const redR = advanceRedAmbrosiaTimer({
    time: input.dt,
    timeMultiplier: 1,
    noAmbrosiaUpgradesCompletions: input.noAmbrosiaUpgradesCompletions,
    redAmbrosiaGenerationSpeed: input.redAmbrosiaGenerationSpeed,
    redAmbrosiaTimerG: input.redAmbrosiaTimerG,
    redAmbrosiaTime: input.redAmbrosiaTime,
    redAmbrosia: input.redAmbrosia,
    lifetimeRedAmbrosia: input.lifetimeRedAmbrosia,
    seed: input.redAmbrosiaSeed,
    redAmbrosiaLuck: input.redAmbrosiaLuck,
    ambrosiaTimePerRedAmbrosia: input.ambrosiaTimePerRedAmbrosia,
    timePerRedAmbrosia: input.timePerRedAmbrosia,
    barRequirementMultiplier: input.redAmbrosiaBarRequirementMultiplier
  })
  for (const e of redR.events) events.push(e)

  // ─── 11b. Bonus-time feedback (redAmbrosia → ambrosia) ─────────────
  // Mirrors the legacy `addTimers('ambrosia', bonusAmbrosiaTime)` shim
  // call (Helper.ts). timeMultiplier === 1 in that recursive call. The
  // ambrosia state at this point already reflects the case-10 result,
  // so we re-enter with the post-case-10 state and the bonus time.
  let ambrosiaTimerG = ambR.ambrosiaTimerG
  let blueberryTime = ambR.blueberryTime
  let ambrosia = ambR.ambrosia
  let lifetimeAmbrosia = ambR.lifetimeAmbrosia
  let ambrosiaSeed = ambR.seed
  if (redR.bonusAmbrosiaTime > 0) {
    const bonusR = advanceAmbrosiaTimer({
      time: redR.bonusAmbrosiaTime,
      timeMultiplier: 1,
      noSingularityUpgradesCompletions: input.noSingularityUpgradesCompletions,
      ambrosiaGenerationSpeed: input.ambrosiaGenerationSpeed,
      ambrosiaTimerG,
      blueberryTime,
      ambrosia,
      lifetimeAmbrosia,
      seed: ambrosiaSeed,
      ambrosiaLuck: input.ambrosiaLuck,
      bonusAmbrosia: input.bonusAmbrosia,
      timePerAmbrosia: input.timePerAmbrosia,
      acceleratorMult: input.ambrosiaAcceleratorMult,
      brickOfLeadMult: input.ambrosiaBrickOfLeadMult
    })
    ambrosiaTimerG = bonusR.ambrosiaTimerG
    blueberryTime = bonusR.blueberryTime
    ambrosia = bonusR.ambrosia
    lifetimeAmbrosia = bonusR.lifetimeAmbrosia
    ambrosiaSeed = bonusR.seed
    for (const e of bonusR.events) events.push(e)
  }

  return {
    prestigecounter,
    transcendcounter,
    reincarnationcounter,
    ascensionCounter: ascR.ascensionCounter,
    ascensionCounterReal: ascR.ascensionCounterReal,
    quarkstimer,
    goldenQuarksTimer,
    octeractTimer: octR.octeractTimer,
    wowOcteracts: octR.wowOcteracts,
    totalWowOcteracts: octR.totalWowOcteracts,
    goldenQuarks: octR.goldenQuarks,
    quarksThisSingularity: octR.quarksThisSingularity,
    ascensionCounterRealReal: singR.ascensionCounterRealReal,
    singularityCounter: singR.singularityCounter,
    singChallengeTimer: singR.singChallengeTimer,
    autoPotionTimer: autoPotionR.autoPotionTimer,
    autoPotionTimerObtainium: autoPotionR.autoPotionTimerObtainium,
    ambrosiaTimerG,
    blueberryTime,
    ambrosia,
    lifetimeAmbrosia,
    ambrosiaSeed,
    redAmbrosiaTimerG: redR.redAmbrosiaTimerG,
    redAmbrosiaTime: redR.redAmbrosiaTime,
    redAmbrosia: redR.redAmbrosia,
    lifetimeRedAmbrosia: redR.lifetimeRedAmbrosia,
    redAmbrosiaSeed: redR.seed,
    events
  }
}
