import {
  addObtainium as logicAddObtainium,
  addOfferings as logicAddOfferings,
  advanceAllTimers as logicAdvanceAllTimers,
  type AdvanceAllTimersInput as LogicAdvanceAllTimersInput,
  type AdvanceAllTimersResult as LogicAdvanceAllTimersResult,
  advanceAmbrosiaTimer as logicAdvanceAmbrosiaTimer,
  advanceAntSacrificeTimers as logicAdvanceAntSacrificeTimers,
  advanceAscensionTimer as logicAdvanceAscensionTimer,
  advanceAutoPotionTimer as logicAdvanceAutoPotionTimer,
  advanceGoldenQuarksTimer as logicAdvanceGoldenQuarksTimer,
  advanceOcteractTimer as logicAdvanceOcteractTimer,
  advanceQuarksTimer as logicAdvanceQuarksTimer,
  advanceRedAmbrosiaTimer as logicAdvanceRedAmbrosiaTimer,
  advanceResetCounter as logicAdvanceResetCounter,
  advanceRuneSacrifice as logicAdvanceRuneSacrifice,
  advanceSingularityTimer as logicAdvanceSingularityTimer,
  type AutoSacrificeMode as LogicAutoSacrificeMode,
  checkAntSacrificeReady as logicCheckAntSacrificeReady
} from '@synergism/logic'
import Decimal from 'break_infinity.js'
import { getAmbrosiaUpgradeEffects } from './BlueberryUpgrades'
import {
  calculateAmbrosiaGenerationSpeed,
  calculateAmbrosiaLuck,
  calculateAscensionSpeedMult,
  calculateGlobalSpeedMult,
  calculateOcteractMultiplier,
  calculateRedAmbrosiaGenerationSpeed,
  calculateRedAmbrosiaLuck,
  calculateResearchAutomaticObtainium
} from './Calculate'
import { antSacrificeRewards } from './Features/Ants/AntSacrifice/Rewards/calculate-rewards'
import { calculateAvailableRebornELO } from './Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/lib/calculate'
import { AutoSacrificeModes } from './Features/Ants/toggles/structs/sacrifice'
import { getLevelMilestone } from './Levels'
import { getOcteractUpgradeEffect } from './Octeracts'
import { quarkHandler } from './Quark'
import { getRedAmbrosiaUpgradeEffects } from './RedAmbrosiaUpgrades'
import { Seed } from './RNG'
import { dispatchTickEvent } from './tickEventHandlers'
import { buyAllBlessingLevels } from './RuneBlessings'
import { getNumberUnlockedRunes, indexToRune, type RuneKeys, runes, sacrificeOfferings } from './Runes'
import { buyAllSpiritLevels } from './RuneSpirits'
import { getShopUpgradeEffects } from './Shop'
import { allGoldenQuarkMultiplierStats } from './Statistics'
import { getGQUpgradeEffect } from './singularity'
import { getSingularityChallengeEffect } from './SingularityChallenges'
import { player } from './Synergism'
import { buyAllTalismanResources } from './Talismans'
import { Globals as G } from './Variables'

type TimerInput =
  | 'prestige'
  | 'transcension'
  | 'reincarnation'
  | 'ascension'
  | 'quarks'
  | 'goldenQuarks'
  | 'singularity'
  | 'octeracts'
  | 'autoPotion'
  | 'ambrosia'
  | 'redAmbrosia'

/**
 * addTimers will add (in milliseconds) time to the reset counters, and quark export timer
 * @param input
 * @param time
 */
export const addTimers = (input: TimerInput, time = 0) => {
  const globalTimeMultiplier = getGQUpgradeEffect('halfMind', 'unlocked')
    ? 10
    : calculateGlobalSpeedMult()

  const timeMultiplier = input === 'ascension'
      || input === 'quarks'
      || input === 'goldenQuarks'
      || input === 'singularity'
      || input === 'octeracts'
      || input === 'autoPotion'
      || input === 'ambrosia'
      || input === 'redAmbrosia'
    ? 1
    : globalTimeMultiplier

  switch (input) {
    case 'prestige': {
      player.prestigecounter = logicAdvanceResetCounter(player.prestigecounter, time, timeMultiplier)
      break
    }
    case 'transcension': {
      player.transcendcounter = logicAdvanceResetCounter(player.transcendcounter, time, timeMultiplier)
      break
    }
    case 'reincarnation': {
      player.reincarnationcounter = logicAdvanceResetCounter(player.reincarnationcounter, time, timeMultiplier)
      break
    }
    case 'ascension': {
      const ascensionSpeedMulti = getGQUpgradeEffect('oneMind', 'unlocked')
        ? 10
        : calculateAscensionSpeedMult()
      const r = logicAdvanceAscensionTimer({
        time,
        ascensionCounter: player.ascensionCounter,
        ascensionCounterReal: player.ascensionCounterReal,
        ascensionSpeedMulti
      })
      player.ascensionCounter = r.ascensionCounter
      player.ascensionCounterReal = r.ascensionCounterReal
      break
    }
    case 'singularity': {
      const singularitySpeedMulti = getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'singularitySpeedMult')
      const r = logicAdvanceSingularityTimer({
        time,
        ascensionCounterRealReal: player.ascensionCounterRealReal,
        singularityCounter: player.singularityCounter,
        singChallengeTimer: player.singChallengeTimer,
        insideSingularityChallenge: player.insideSingularityChallenge,
        singularitySpeedMulti
      })
      player.ascensionCounterRealReal = r.ascensionCounterRealReal
      player.singularityCounter = r.singularityCounter
      player.singChallengeTimer = r.singChallengeTimer
      break
    }
    case 'quarks': {
      player.quarkstimer = logicAdvanceQuarksTimer({
        time,
        quarkstimer: player.quarkstimer,
        maxQuarkTimer: quarkHandler().maxTime
      })
      break
    }
    case 'goldenQuarks': {
      player.goldenQuarksTimer = logicAdvanceGoldenQuarksTimer({
        time,
        goldenQuarksTimer: player.goldenQuarksTimer,
        exportGQPerHour: getGQUpgradeEffect('goldenQuarks3', 'exportGQPerHour')
      })
      break
    }
    case 'octeracts': {
      if (!getGQUpgradeEffect('octeractUnlock', 'unlocked')) {
        return
      }
      // Pre-eval the GQ multiplier product (stats 1..end, skipping the
      // qts-dependent base at index 0) only when the GQ-giveaway block
      // will run (≥ sing 160). Logic recomputes the base each iteration.
      let goldenQuarksMultiplierExcludingBase = 1
      if (player.highestSingularityCount >= 160) {
        const gqStats = allGoldenQuarkMultiplierStats.map(s => s.stat())
        goldenQuarksMultiplierExcludingBase = gqStats.slice(1).reduce((a, b) => a * b, 1)
      }
      const octeractResult = logicAdvanceOcteractTimer({
        time,
        timeMultiplier,
        octeractUnlocked: true,
        octeractTimer: player.octeractTimer,
        wowOcteracts: player.wowOcteracts,
        totalWowOcteracts: player.totalWowOcteracts,
        goldenQuarks: player.goldenQuarks,
        quarksThisSingularity: player.quarksThisSingularity,
        perSecond: calculateOcteractMultiplier(),
        highestSingularityCount: player.highestSingularityCount,
        singularityCount: player.singularityCount,
        goldenQuarksMultiplierExcludingBase
      })
      player.octeractTimer = octeractResult.octeractTimer
      player.wowOcteracts = octeractResult.wowOcteracts
      player.totalWowOcteracts = octeractResult.totalWowOcteracts
      player.goldenQuarks = octeractResult.goldenQuarks
      player.quarksThisSingularity = octeractResult.quarksThisSingularity
      for (const event of octeractResult.events) {
        dispatchTickEvent(event)
      }
      break
    }
    case 'autoPotion': {
      const r = logicAdvanceAutoPotionTimer({
        time,
        timeMultiplier,
        highestSingularityCount: player.highestSingularityCount,
        autoPotionTimer: player.autoPotionTimer,
        autoPotionTimerObtainium: player.autoPotionTimerObtainium,
        toggleOffering: player.toggles[42],
        toggleObtainium: player.toggles[43],
        offeringPotionCount: player.shopUpgrades.offeringPotion,
        obtainiumPotionCount: player.shopUpgrades.obtainiumPotion,
        autoPotionSpeedMult: getOcteractUpgradeEffect('octeractAutoPotionSpeed', 'autoPotionSpeedMult')
      })
      player.autoPotionTimer = r.autoPotionTimer
      player.autoPotionTimerObtainium = r.autoPotionTimerObtainium
      for (const event of r.events) {
        dispatchTickEvent(event)
      }
      break
    }
    case 'ambrosia': {
      // Cheap gate first — feature locked when completions === 0. Mirrors
      // logic's inner gate; avoids paying for the calc pre-evals every tick.
      if (player.singularityChallenges.noSingularityUpgrades.completions <= 0) {
        break
      }
      const ambrosiaResult = logicAdvanceAmbrosiaTimer({
        time,
        timeMultiplier,
        noSingularityUpgradesCompletions: player.singularityChallenges.noSingularityUpgrades.completions,
        ambrosiaGenerationSpeed: calculateAmbrosiaGenerationSpeed(),
        ambrosiaTimerG: G.ambrosiaTimer,
        blueberryTime: player.blueberryTime,
        ambrosia: player.ambrosia,
        lifetimeAmbrosia: player.lifetimeAmbrosia,
        seed: player.seed[Seed.Ambrosia],
        ambrosiaLuck: calculateAmbrosiaLuck(),
        bonusAmbrosia: getSingularityChallengeEffect('noAmbrosiaUpgrades', 'bonusAmbrosia'),
        timePerAmbrosia: G.TIME_PER_AMBROSIA,
        acceleratorMult: getShopUpgradeEffects('shopAmbrosiaAccelerator', 'ambrosiaPointRequirementMult'),
        brickOfLeadMult: getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'barRequirementMult')
      })
      G.ambrosiaTimer = ambrosiaResult.ambrosiaTimerG
      player.blueberryTime = ambrosiaResult.blueberryTime
      player.ambrosia = ambrosiaResult.ambrosia
      player.lifetimeAmbrosia = ambrosiaResult.lifetimeAmbrosia
      player.seed[Seed.Ambrosia] = ambrosiaResult.seed
      for (const event of ambrosiaResult.events) {
        dispatchTickEvent(event)
      }
      break
    }
    case 'redAmbrosia': {
      if (player.singularityChallenges.noAmbrosiaUpgrades.completions <= 0) {
        break
      }
      const redAmbrosiaResult = logicAdvanceRedAmbrosiaTimer({
        time,
        timeMultiplier,
        noAmbrosiaUpgradesCompletions: player.singularityChallenges.noAmbrosiaUpgrades.completions,
        redAmbrosiaGenerationSpeed: calculateRedAmbrosiaGenerationSpeed(),
        redAmbrosiaTimerG: G.redAmbrosiaTimer,
        redAmbrosiaTime: player.redAmbrosiaTime,
        redAmbrosia: player.redAmbrosia,
        lifetimeRedAmbrosia: player.lifetimeRedAmbrosia,
        seed: player.seed[Seed.RedAmbrosia],
        redAmbrosiaLuck: calculateRedAmbrosiaLuck(),
        ambrosiaTimePerRedAmbrosia: getRedAmbrosiaUpgradeEffects('redAmbrosiaAccelerator', 'ambrosiaTimePerRedAmbrosia'),
        timePerRedAmbrosia: G.TIME_PER_RED_AMBROSIA,
        barRequirementMultiplier: getSingularityChallengeEffect('limitedTime', 'barRequirementMultiplier')
      })
      G.redAmbrosiaTimer = redAmbrosiaResult.redAmbrosiaTimerG
      player.redAmbrosiaTime = redAmbrosiaResult.redAmbrosiaTime
      player.redAmbrosia = redAmbrosiaResult.redAmbrosia
      player.lifetimeRedAmbrosia = redAmbrosiaResult.lifetimeRedAmbrosia
      player.seed[Seed.RedAmbrosia] = redAmbrosiaResult.seed
      if (redAmbrosiaResult.bonusAmbrosiaTime > 0) {
        addTimers('ambrosia', redAmbrosiaResult.bonusAmbrosiaTime)
      }
      for (const event of redAmbrosiaResult.events) {
        dispatchTickEvent(event)
      }
      break
    }
  }
}

/** Build the AdvanceAllTimersInput struct for `dt`. Pre-evaluates every
 * speed multiplier / cap / stat-derived input the 11 timer cases need.
 * Used by the tackBody orchestrator in Synergism.ts:tack(). */
export const buildHeadTimersInput = (dt: number): LogicAdvanceAllTimersInput => {
  const globalTimeMultiplier = getGQUpgradeEffect('halfMind', 'unlocked')
    ? 10
    : calculateGlobalSpeedMult()
  const ascensionSpeedMulti = getGQUpgradeEffect('oneMind', 'unlocked')
    ? 10
    : calculateAscensionSpeedMult()
  const singularitySpeedMulti = getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'singularitySpeedMult')
  const octeractUnlocked = getGQUpgradeEffect('octeractUnlock', 'unlocked')

  // Octeract pre-eval: only meaningful when the GQ-giveaway block will
  // run (highestSingularityCount >= 160). Below threshold we pass 1 —
  // the bundle ignores this value when the gate fails.
  let goldenQuarksMultiplierExcludingBase = 1
  if (octeractUnlocked && player.highestSingularityCount >= 160) {
    const gqStats = allGoldenQuarkMultiplierStats.map(s => s.stat())
    goldenQuarksMultiplierExcludingBase = gqStats.slice(1).reduce((a, b) => a * b, 1)
  }

  return {
    dt,
    globalTimeMultiplier,
    prestigecounter: player.prestigecounter,
    transcendcounter: player.transcendcounter,
    reincarnationcounter: player.reincarnationcounter,
    ascensionCounter: player.ascensionCounter,
    ascensionCounterReal: player.ascensionCounterReal,
    ascensionSpeedMulti,
    quarkstimer: player.quarkstimer,
    maxQuarkTimer: quarkHandler().maxTime,
    goldenQuarksTimer: player.goldenQuarksTimer,
    exportGQPerHour: getGQUpgradeEffect('goldenQuarks3', 'exportGQPerHour'),
    octeractUnlocked,
    octeractTimer: player.octeractTimer,
    wowOcteracts: player.wowOcteracts,
    totalWowOcteracts: player.totalWowOcteracts,
    goldenQuarks: player.goldenQuarks,
    quarksThisSingularity: player.quarksThisSingularity,
    octeractPerSecond: calculateOcteractMultiplier(),
    highestSingularityCount: player.highestSingularityCount,
    singularityCount: player.singularityCount,
    goldenQuarksMultiplierExcludingBase,
    ascensionCounterRealReal: player.ascensionCounterRealReal,
    singularityCounter: player.singularityCounter,
    singChallengeTimer: player.singChallengeTimer,
    insideSingularityChallenge: player.insideSingularityChallenge,
    singularitySpeedMulti,
    autoPotionTimer: player.autoPotionTimer,
    autoPotionTimerObtainium: player.autoPotionTimerObtainium,
    autoPotionToggleOffering: player.toggles[42],
    autoPotionToggleObtainium: player.toggles[43],
    offeringPotionCount: player.shopUpgrades.offeringPotion,
    obtainiumPotionCount: player.shopUpgrades.obtainiumPotion,
    autoPotionSpeedMult: getOcteractUpgradeEffect('octeractAutoPotionSpeed', 'autoPotionSpeedMult'),
    noSingularityUpgradesCompletions: player.singularityChallenges.noSingularityUpgrades.completions,
    ambrosiaGenerationSpeed: calculateAmbrosiaGenerationSpeed(),
    ambrosiaTimerG: G.ambrosiaTimer,
    blueberryTime: player.blueberryTime,
    ambrosia: player.ambrosia,
    lifetimeAmbrosia: player.lifetimeAmbrosia,
    ambrosiaSeed: player.seed[Seed.Ambrosia],
    ambrosiaLuck: calculateAmbrosiaLuck(),
    bonusAmbrosia: getSingularityChallengeEffect('noAmbrosiaUpgrades', 'bonusAmbrosia'),
    timePerAmbrosia: G.TIME_PER_AMBROSIA,
    ambrosiaAcceleratorMult: getShopUpgradeEffects('shopAmbrosiaAccelerator', 'ambrosiaPointRequirementMult'),
    ambrosiaBrickOfLeadMult: getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'barRequirementMult'),
    noAmbrosiaUpgradesCompletions: player.singularityChallenges.noAmbrosiaUpgrades.completions,
    redAmbrosiaGenerationSpeed: calculateRedAmbrosiaGenerationSpeed(),
    redAmbrosiaTimerG: G.redAmbrosiaTimer,
    redAmbrosiaTime: player.redAmbrosiaTime,
    redAmbrosia: player.redAmbrosia,
    lifetimeRedAmbrosia: player.lifetimeRedAmbrosia,
    redAmbrosiaSeed: player.seed[Seed.RedAmbrosia],
    redAmbrosiaLuck: calculateRedAmbrosiaLuck(),
    ambrosiaTimePerRedAmbrosia: getRedAmbrosiaUpgradeEffects('redAmbrosiaAccelerator', 'ambrosiaTimePerRedAmbrosia'),
    timePerRedAmbrosia: G.TIME_PER_RED_AMBROSIA,
    redAmbrosiaBarRequirementMultiplier: getSingularityChallengeEffect('limitedTime', 'barRequirementMultiplier')
  }
}

/** Apply an AdvanceAllTimersResult to player + G. Does NOT dispatch
 * events — the orchestrator merges events from all bundles and
 * dispatches in a single pass. */
export const applyHeadTimersResult = (result: LogicAdvanceAllTimersResult): void => {
  player.prestigecounter = result.prestigecounter
  player.transcendcounter = result.transcendcounter
  player.reincarnationcounter = result.reincarnationcounter
  player.ascensionCounter = result.ascensionCounter
  player.ascensionCounterReal = result.ascensionCounterReal
  player.quarkstimer = result.quarkstimer
  player.goldenQuarksTimer = result.goldenQuarksTimer
  player.octeractTimer = result.octeractTimer
  player.wowOcteracts = result.wowOcteracts
  player.totalWowOcteracts = result.totalWowOcteracts
  player.goldenQuarks = result.goldenQuarks
  player.quarksThisSingularity = result.quarksThisSingularity
  player.ascensionCounterRealReal = result.ascensionCounterRealReal
  player.singularityCounter = result.singularityCounter
  player.singChallengeTimer = result.singChallengeTimer
  player.autoPotionTimer = result.autoPotionTimer
  player.autoPotionTimerObtainium = result.autoPotionTimerObtainium
  G.ambrosiaTimer = result.ambrosiaTimerG
  player.blueberryTime = result.blueberryTime
  player.ambrosia = result.ambrosia
  player.lifetimeAmbrosia = result.lifetimeAmbrosia
  player.seed[Seed.Ambrosia] = result.ambrosiaSeed
  G.redAmbrosiaTimer = result.redAmbrosiaTimerG
  player.redAmbrosiaTime = result.redAmbrosiaTime
  player.redAmbrosia = result.redAmbrosia
  player.lifetimeRedAmbrosia = result.lifetimeRedAmbrosia
  player.seed[Seed.RedAmbrosia] = result.redAmbrosiaSeed
}

/**
 * Per-tick "head" timer bundle. Calls `buildHeadTimersInput` →
 * `logicAdvanceAllTimers` → `applyHeadTimersResult` → dispatch events.
 * Kept for any caller that wants the full standalone pass; the per-tick
 * tack body uses the build/apply pair directly via the tackBody
 * orchestrator (in Synergism.ts:tack).
 */
export const tackHeadTimers = (dt: number): void => {
  const result = logicAdvanceAllTimers(buildHeadTimersInput(dt))
  applyHeadTimersResult(result)
  for (const event of result.events) {
    dispatchTickEvent(event)
  }
}

type AutoToolInput =
  | 'addObtainium'
  | 'addOfferings'
  | 'runeSacrifice'
  | 'antSacrifice'

/** Translate the AutoSacrificeModes numeric enum (web_ui-side UI config)
 * to the string union the logic API uses. Bug-for-bug 1:1 mapping. */
export const autoSacrificeModeToLogic = (mode: AutoSacrificeModes): LogicAutoSacrificeMode => {
  switch (mode) {
    case AutoSacrificeModes.InGameTime:
      return 'InGameTime'
    case AutoSacrificeModes.RealTime:
      return 'RealTime'
    case AutoSacrificeModes.ImmortalELOGain:
      return 'ImmortalELOGain'
    case AutoSacrificeModes.MaxRebornELO:
      return 'MaxRebornELO'
  }
}

const calculateAutoSacrificeInterval = () => {
  let interval = 1
  interval /= getShopUpgradeEffects('offeringAuto', 'autoRuneSpeedMult')
  if (player.cubeUpgrades[20] > 0) {
    interval /= 2
  }
  interval /= getLevelMilestone('runeAutobuyImprover')
  return interval
}
let autoSacrificeInterval = 1

/** Read the cached autoSacrificeInterval. Refreshed inside
 * `executeRuneAutoSacrifice` each time a rune sacrifice fires; the tack
 * body / tackMiddle bundle passes this value into logic on every tick. */
export const getAutoSacrificeInterval = (): number => autoSacrificeInterval

/**
 * Assortment of tools which are used when actions are automated.
 * @param input
 * @param time
 */
export const automaticTools = (input: AutoToolInput, time: number) => {
  switch (input) {
    case 'addObtainium': {
      const obtainiumResult = logicAddObtainium({
        obtainium: player.obtainium,
        obtainiumGain: calculateResearchAutomaticObtainium(time),
        ascensionChallenge: player.currentChallenge.ascension,
        taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled,
        taxmanLastStandCompletions: player.singularityChallenges.taxmanLastStand.completions
      })
      player.obtainium = obtainiumResult.obtainium
      for (const event of obtainiumResult.events) {
        dispatchTickEvent(event)
      }
      break
    }
    case 'addOfferings': {
      // This counter can be increased through challenge 3 reward
      // As well as cube upgrade 1x2 (2).
      const offeringsResult = logicAddOfferings({
        time,
        autoOfferingCounter: G.autoOfferingCounter,
        offerings: player.offerings
      })
      G.autoOfferingCounter = offeringsResult.autoOfferingCounter
      player.offerings = offeringsResult.offerings
      break
    }
    case 'runeSacrifice': {
      const runeR = logicAdvanceRuneSacrifice({
        time,
        sacrificeTimer: player.sacrificeTimer,
        autoSacrificeInterval,
        offerings: player.offerings
      })
      player.sacrificeTimer = runeR.sacrificeTimer
      for (const event of runeR.events) {
        dispatchTickEvent(event)
      }
      break
    }
    case 'antSacrifice': {
      const timerR = logicAdvanceAntSacrificeTimers({
        time,
        globalDelta: getGQUpgradeEffect('halfMind', 'unlocked') ? 10 : calculateGlobalSpeedMult(),
        antSacrificeTimer: player.antSacrificeTimer,
        antSacrificeTimerReal: player.antSacrificeTimerReal
      })
      player.antSacrificeTimer = timerR.antSacrificeTimer
      player.antSacrificeTimerReal = timerR.antSacrificeTimerReal

      // Only pre-evaluate the ImmortalELOGain mode's lookup when that
      // mode is active — antSacrificeRewards() pulls in calculate-rewards
      // and is comparatively expensive. Other modes don't read it.
      const mode = autoSacrificeModeToLogic(player.ants.toggles.autoSacrificeMode)
      const immortalELOGain = mode === 'ImmortalELOGain'
        ? antSacrificeRewards().immortalELO
        : 0
      const checkR = logicCheckAntSacrificeReady({
        mode,
        crumbsThisSacrifice: player.ants.crumbsThisSacrifice,
        antSacrificeTimerReal: player.antSacrificeTimerReal,
        autoSacrificeEnabled: player.ants.toggles.autoSacrificeEnabled,
        availableRebornELO: calculateAvailableRebornELO(),
        onlySacrificeMaxRebornELO: player.ants.toggles.onlySacrificeMaxRebornELO,
        alwaysSacrificeMaxRebornELO: player.ants.toggles.alwaysSacrificeMaxRebornELO,
        antSacrificeTimer: player.antSacrificeTimer,
        autoSacrificeThreshold: player.ants.toggles.autoSacrificeThreshold,
        immortalELOGain,
        immortalELO: player.ants.immortalELO,
        rebornELO: player.ants.rebornELO
      })
      for (const event of checkR.events) {
        dispatchTickEvent(event)
      }
      break
    }
  }
}

/**
 * Execute the rune auto-sacrifice fan-out + refresh the module-local
 * autoSacrificeInterval cache. Invoked by the `rune-sacrifice-triggered`
 * event handler in tickEventHandlers.ts when logic's advanceRuneSacrifice
 * decides the gate fires this tick.
 *
 * Each call reads the latest player state (singularity-count gates, cube
 * upgrade gates, autoSacrifice rune index, etc.) — the event itself
 * carries no payload because every fan-out branch depends on un-migrated
 * subsystems (RuneBlessings, RuneSpirits, Talismans, sacrificeOfferings).
 */
export const executeRuneAutoSacrifice = (): void => {
  // Automatic purchase of Blessings
  if (player.highestSingularityCount >= 15) {
    if (player.toggles[36]) {
      buyAllBlessingLevels(player.offerings.div(2))
    }
    if (player.toggles[37]) {
      buyAllSpiritLevels(player.offerings.div(2))
    }
  }
  if (
    player.autoBuyFragment
    && player.highestSingularityCount >= 40
    && player.cubeUpgrades[51] > 0
  ) {
    buyAllTalismanResources()
  }

  // If you bought cube upgrade 2x10 then it sacrifices to all runes equally
  if (player.cubeUpgrades[20] === 1) {
    let numUnlocked = getNumberUnlockedRunes()

    // Do not purchase AoAG under s50
    if (player.highestSingularityCount < 50 && runes.antiquities.isUnlocked()) {
      numUnlocked -= 1
    }

    // Do not purchase IA under s30
    if (player.highestSingularityCount < 30 && runes.infiniteAscent.isUnlocked()) {
      numUnlocked -= 1
    }

    const offeringPerRune = Decimal.floor(player.offerings.mul(0.5).div(numUnlocked))

    for (const key of Object.keys(player.runes)) {
      const runeKey = key as RuneKeys
      sacrificeOfferings(runeKey, offeringPerRune, true)
    }
  } else {
    // If you did not buy cube upgrade 2x10 it sacrifices to selected rune.
    const rune = player.autoSacrifice
    if (rune !== 0) {
      sacrificeOfferings(indexToRune[rune], player.offerings, true)
    }
  }
  autoSacrificeInterval = calculateAutoSacrificeInterval()
}
