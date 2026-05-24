import {
  addObtainium as logicAddObtainium,
  addOfferings as logicAddOfferings,
  advanceAllTimers as logicAdvanceAllTimers,
  advanceAmbrosiaTimer as logicAdvanceAmbrosiaTimer,
  advanceAscensionTimer as logicAdvanceAscensionTimer,
  advanceGoldenQuarksTimer as logicAdvanceGoldenQuarksTimer,
  advanceOcteractTimer as logicAdvanceOcteractTimer,
  advanceQuarksTimer as logicAdvanceQuarksTimer,
  advanceRedAmbrosiaTimer as logicAdvanceRedAmbrosiaTimer,
  advanceResetCounter as logicAdvanceResetCounter,
  advanceSingularityTimer as logicAdvanceSingularityTimer
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
import { sacrificeAnts } from './Features/Ants/AntSacrifice/sacrifice'
import { canAutoSacrifice } from './Features/Ants/Automation/sacrifice'
import { getLevelMilestone } from './Levels'
import { getOcteractUpgradeEffect } from './Octeracts'
import { quarkHandler } from './Quark'
import { getRedAmbrosiaUpgradeEffects } from './RedAmbrosiaUpgrades'
import { Seed } from './RNG'
import { dispatchTickEvent } from './tickEventHandlers'
import { buyAllBlessingLevels } from './RuneBlessings'
import { getNumberUnlockedRunes, indexToRune, type RuneKeys, runes, sacrificeOfferings } from './Runes'
import { buyAllSpiritLevels } from './RuneSpirits'
import { getShopUpgradeEffects, useConsumable } from './Shop'
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
      if (player.highestSingularityCount < 6) {
        return
      } else {
        // player.toggles[42] enables FAST Offering Potion Expenditure, but actually spends the potion.
        // Hence, you need at least one potion to be able to use fast spend.
        const toggleOfferingOn = player.toggles[42] && player.shopUpgrades.offeringPotion > 0
        // player.toggles[43] enables FAST Obtainium Potion Expenditure, but actually spends the potion.
        const toggleObtainiumOn = player.toggles[43] && player.shopUpgrades.obtainiumPotion > 0

        player.autoPotionTimer += time * timeMultiplier
        player.autoPotionTimerObtainium += time * timeMultiplier

        const timerThreshold = (180 * Math.pow(1.03, -player.highestSingularityCount))
          / getOcteractUpgradeEffect('octeractAutoPotionSpeed', 'autoPotionSpeedMult')

        const effectiveOfferingThreshold = toggleOfferingOn
          ? Math.min(1, timerThreshold) / 20
          : timerThreshold
        const effectiveObtainiumThreshold = toggleObtainiumOn
          ? Math.min(1, timerThreshold) / 20
          : timerThreshold

        if (player.autoPotionTimer >= effectiveOfferingThreshold) {
          const amountOfPotions = (player.autoPotionTimer
            - (player.autoPotionTimer % effectiveOfferingThreshold))
            / effectiveOfferingThreshold
          player.autoPotionTimer %= effectiveOfferingThreshold
          useConsumable(
            'offeringPotion',
            true,
            amountOfPotions,
            toggleOfferingOn
          )
        }

        if (player.autoPotionTimerObtainium >= effectiveObtainiumThreshold) {
          const amountOfPotions = (player.autoPotionTimerObtainium
            - (player.autoPotionTimerObtainium % effectiveObtainiumThreshold))
            / effectiveObtainiumThreshold
          player.autoPotionTimerObtainium %= effectiveObtainiumThreshold
          useConsumable(
            'obtainiumPotion',
            true,
            amountOfPotions,
            toggleObtainiumOn
          )
        }
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

/**
 * Per-tick "head" timer bundle. Replaces the 10 logic-backed
 * `addTimers(...)` switch dispatches in `tack` (Synergism.ts) with a
 * single composed `advanceAllTimers` call from `@synergism/logic`.
 *
 * Pre-evaluates the same speed multipliers, caps, and stat-derived
 * inputs the per-case shims used to evaluate inline, threads them
 * through the bundle, then writes the result back to `player` / `G`
 * and dispatches the composed event list.
 *
 * `autoPotion` is NOT in this bundle — it uses `useConsumable(...)`
 * with DOM/modal side effects. The legacy sequence had autoPotion
 * between case 8 (singularity) and case 9 (ambrosia); the bundle runs
 * cases 9-10 contiguously and the caller now invokes
 * `addTimers('autoPotion', dt)` *after* this function. See
 * `packages/logic/src/tick/timersBundle.ts` header for the audit that
 * justifies the position shift as bug-for-bug equivalent.
 */
export const tackHeadTimers = (dt: number): void => {
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
  // the bundle ignores this value when the gate fails. Matches the
  // per-case `addTimers('octeracts', ...)` shim in this file.
  let goldenQuarksMultiplierExcludingBase = 1
  if (octeractUnlocked && player.highestSingularityCount >= 160) {
    const gqStats = allGoldenQuarkMultiplierStats.map(s => s.stat())
    goldenQuarksMultiplierExcludingBase = gqStats.slice(1).reduce((a, b) => a * b, 1)
  }

  const result = logicAdvanceAllTimers({
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
  })

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

  for (const event of result.events) {
    dispatchTickEvent(event)
  }
}

type AutoToolInput =
  | 'addObtainium'
  | 'addOfferings'
  | 'runeSacrifice'
  | 'antSacrifice'

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
    case 'runeSacrifice':
      // Every real life second this will trigger
      player.sacrificeTimer += time
      if (
        player.sacrificeTimer >= autoSacrificeInterval
        && player.offerings.gt(0)
      ) {
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
        player.sacrificeTimer = 0
      }
      break
    case 'antSacrifice': {
      const globalDelta = getGQUpgradeEffect('halfMind', 'unlocked') ? 10 : calculateGlobalSpeedMult()

      player.antSacrificeTimer += time * globalDelta
      player.antSacrificeTimerReal += time

      const timeElapsed = player.antSacrificeTimerReal
      const crumbs = player.ants.crumbsThisSacrifice
      const mode = player.ants.toggles.autoSacrificeMode
      if (
        canAutoSacrifice(crumbs, mode, timeElapsed)
      ) {
        sacrificeAnts()
      }
      break
    }
  }
}
