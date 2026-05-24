import {
  autoAscensionChallengeSweepUnlock as logicAutoAscChallengeSweepUnlock,
  CalcECC as logicCalcECC,
  challenge15ScoreMultiplier as logicChallenge15ScoreMultiplier,
  challengeRequirement as logicChallengeRequirement,
  challengeScoreDisplay as logicChallengeScoreDisplay,
  type CoreEvent,
  getMaxChallenges as logicGetMaxChallenges,
  getNextAscensionChallenge as logicGetNextAscChallenge,
  getNextRegularChallenge as logicGetNextRegularChallenge,
  type SweepStates,
  tickChallengeSweep as logicTickChallengeSweep
} from '@synergism/logic'
import Decimal from 'break_infinity.js'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { hepteractEffective } from './Hepteracts'
import { getShopUpgradeEffects } from './Shop'
import { getGQUpgradeEffect } from './singularity'
import { getSingularityChallengeEffect } from './SingularityChallenges'
import { format, player, resetCheck } from './Synergism'
import { AutoAscensionResetModes, toggleAutoChallengeModeText, toggleChallenges } from './Toggles'
import { Globals as G } from './Variables'

export type Challenge15Rewards =
  | 'cube1'
  | 'ascensions'
  | 'coinExponent'
  | 'taxes'
  | 'obtainium'
  | 'offering'
  | 'accelerator'
  | 'multiplier'
  | 'runeExp'
  | 'runeBonus'
  | 'cube2'
  | 'transcendChallengeReduction'
  | 'reincarnationChallengeReduction'
  | 'antSpeed'
  | 'bonusAntLevel'
  | 'cube3'
  | 'talismanBonus'
  | 'globalSpeed'
  | 'blessingBonus'
  | 'constantBonus'
  | 'cube4'
  | 'spiritBonus'
  | 'score'
  | 'quarks'
  | 'hepteractsUnlocked'
  | 'challengeHepteractUnlocked'
  | 'cube5'
  | 'powder'
  | 'abyssHepteractUnlocked'
  | 'exponent'
  | 'acceleratorHepteractUnlocked'
  | 'acceleratorBoostHepteractUnlocked'
  | 'multiplierHepteractUnlocked'
  | 'freeOrbs'
  | 'ascensionSpeed'
  | 'achievementUnlock'

export type Challenge15RewardsInformation = {
  value: number
  baseValue: number
  requirement: number
  HTMLColor?: string
  doNotUsePercentage?: boolean
}

export type Challenge15RewardObject = Record<Challenge15Rewards, Challenge15RewardsInformation>

export const getMaxChallenges = (i: number) =>
  logicGetMaxChallenges({
    challenge: i,
    oneChallengeCapEnabled: player.singularityChallenges.oneChallengeCap.enabled,
    infiniteTranscendResearch: player.researches[105],
    transcendResearchForChallenge: player.researches[65 + i],
    cubeUpgrade29: player.cubeUpgrades[29],
    challengeExtensionCap: getShopUpgradeEffects('challengeExtension', 'reincarnationChallengeCap'),
    gqReincarnationCapIncrease: getGQUpgradeEffect('singChallengeExtension', 'reincarnationCapIncrease')
      + getGQUpgradeEffect('singChallengeExtension2', 'reincarnationCapIncrease')
      + getGQUpgradeEffect('singChallengeExtension3', 'reincarnationCapIncrease'),
    singReincarnationCapIncrease: getSingularityChallengeEffect('oneChallengeCap', 'capIncrease')
      + getSingularityChallengeEffect('oneChallengeCap', 'reinCapIncrease2'),
    gqAscensionCapIncrease: getGQUpgradeEffect('singChallengeExtension', 'ascensionCapIncrease')
      + getGQUpgradeEffect('singChallengeExtension2', 'ascensionCapIncrease')
      + getGQUpgradeEffect('singChallengeExtension3', 'ascensionCapIncrease'),
    singAscensionCapIncrease: getSingularityChallengeEffect('oneChallengeCap', 'ascCapIncrease2'),
    platonicUpgrade5: player.platonicUpgrades[5],
    platonicUpgrade10: player.platonicUpgrades[10],
    platonicUpgrade15: player.platonicUpgrades[15]
  })

export const challengeDisplay = (i: number, changefocus = true) => {
  let quarksMultiplier = 1

  if (changefocus) {
    G.challengefocus = i
    DOMCacheGetOrSet('oneChallengeDetails').style.display = 'flex'
    DOMCacheGetOrSet('startChallenge').style.display = 'block'
    DOMCacheGetOrSet('retryChallenge').style.display = 'block'
    G.triggerChallenge = i
  }

  const maxChallenges = getMaxChallenges(i)
  if (i <= 5 && changefocus) {
    if (player.challengecompletions[i] >= 100) {
      DOMCacheGetOrSet('completionSoftcap').innerHTML = i18next.t('challenges.perCompletionBonus', {
        x: 100,
        y: format(CalcECC('transcend', player.challengecompletions[i]), 2, true)
      })
    } else {
      DOMCacheGetOrSet('completionSoftcap').textContent = i18next.t('challenges.perCompletionBonusEmpty')
    }
  }

  if (i > 5 && i <= 10) {
    quarksMultiplier = 10
    if (player.challengecompletions[i] >= 25 && changefocus) {
      DOMCacheGetOrSet('completionSoftcap').innerHTML = i18next.t('challenges.perCompletionBonus', {
        x: 25,
        y: format(CalcECC('reincarnation', player.challengecompletions[i]), 2, true)
      })
    } else {
      DOMCacheGetOrSet('completionSoftcap').textContent = i18next.t('challenges.perCompletionBonusEmpty')
    }
  }
  if (i > 10) {
    if (player.challengecompletions[i] >= 10) {
      DOMCacheGetOrSet('completionSoftcap').innerHTML = i18next.t('challenges.perCompletionBonus', {
        x: 10,
        y: format(CalcECC('ascension', player.challengecompletions[i]), 2, true)
      })
    } else {
      DOMCacheGetOrSet('completionSoftcap').textContent = i18next.t('challenges.perCompletionBonusEmpty')
    }
  }
  let descriptor = ''
  const a = DOMCacheGetOrSet('challengeName')
  const b = DOMCacheGetOrSet('challengeFlavor')
  const c = DOMCacheGetOrSet('challengeRestrictions')
  const d = DOMCacheGetOrSet('challengeGoal')
  const e = DOMCacheGetOrSet('challengePer1').childNodes[0]
  const f = DOMCacheGetOrSet('challengePer2').childNodes[0]
  const g = DOMCacheGetOrSet('challengePer3').childNodes[0]
  const h = DOMCacheGetOrSet('challengeFirst1')
  const j = DOMCacheGetOrSet('challengeQuarkBonus')
  const k = DOMCacheGetOrSet('startChallenge')
  const l = DOMCacheGetOrSet('challengeCurrent1')
  const m = DOMCacheGetOrSet('challengeCurrent2')
  const n = DOMCacheGetOrSet('challengeCurrent3')

  if (i === G.challengefocus) {
    const completions = `${format(player.challengecompletions[i])}/${format(maxChallenges)}`
    const special = (i >= 6 && i <= 10) || i === 15
    const goal = format(challengeRequirement(i, player.challengecompletions[i], special ? i : 0))

    let current1 = ''
    let current2 = ''
    let current3 = ''

    switch (i) {
      case 1: {
        current1 = format(2 * CalcECC('transcend', player.challengecompletions[1]))
        current2 = format(0.75 * CalcECC('transcend', player.challengecompletions[1]), 2, true)
        current3 = format(0.04 * CalcECC('transcend', player.challengecompletions[1]), 2, true)
        break
      }
      case 2: {
        current1 = current2 = format(5 * CalcECC('transcend', player.challengecompletions[2]))
        current3 = format(0.25 * CalcECC('transcend', player.challengecompletions[2]))
        break
      }
      case 3: {
        current1 = format(0.04 * CalcECC('transcend', player.challengecompletions[3]), 2, true)
        current2 = format(0.5 * CalcECC('transcend', player.challengecompletions[3]), 2, true)
        current3 = format(0.01 * CalcECC('transcend', player.challengecompletions[3]), 2, true)
        break
      }
      case 4: {
        current1 = format(5 * CalcECC('transcend', player.challengecompletions[4]))
        current2 = format(2 * CalcECC('transcend', player.challengecompletions[4]))
        current3 = format(0.5 * CalcECC('transcend', player.challengecompletions[4]), 2, true)
        break
      }
      case 5: {
        current1 = format(0.5 + CalcECC('transcend', player.challengecompletions[5]) / 100, 2, true)
        current2 = format(Math.pow(10, CalcECC('transcend', player.challengecompletions[5])))
        current3 = format(5 * CalcECC('transcend', player.challengecompletions[5]), 2, true)
        break
      }
      case 6: {
        current1 = format(Math.pow(0.965, CalcECC('reincarnation', player.challengecompletions[6])), 3, true)
        current2 = format(0.3 * CalcECC('reincarnation', player.challengecompletions[6]), 2, true)
        current3 = format(2 * CalcECC('reincarnation', player.challengecompletions[6]))
        break
      }
      case 7: {
        current1 = format(1 + 0.04 * CalcECC('reincarnation', player.challengecompletions[7]), 2, true)
        current2 = format(0.3 * CalcECC('reincarnation', player.challengecompletions[7]), 2, true)
        current3 = format(15 * CalcECC('reincarnation', player.challengecompletions[7]), 2, true)
        break
      }
      case 8: {
        current1 = format(0.25 * CalcECC('reincarnation', player.challengecompletions[8]), 2, true)
        current2 = format(0.4 * CalcECC('reincarnation', player.challengecompletions[8]), 2, true)
        current3 = format(4 * CalcECC('reincarnation', player.challengecompletions[8]), 2, true)
        break
      }
      case 9: {
        current1 = format(CalcECC('reincarnation', player.challengecompletions[9]))
        current2 = format(Math.pow(1.1, CalcECC('reincarnation', player.challengecompletions[9])), 2, true)
        current3 = format(0.5 * CalcECC('reincarnation', player.challengecompletions[9]), 2, true)
        break
      }
      case 10: {
        current1 = format(100 * CalcECC('reincarnation', player.challengecompletions[10]))
        current2 = format(2 * CalcECC('reincarnation', player.challengecompletions[10]))
        current3 = format(10 * CalcECC('reincarnation', player.challengecompletions[10]), 2, true)
        break
      }
      case 11: {
        current1 = format(12 * CalcECC('ascension', player.challengecompletions[11]))
        current2 = format(Decimal.pow(1e5, CalcECC('ascension', player.challengecompletions[11])))
        current3 = format(CalcECC('ascension', player.challengecompletions[11]))
        break
      }
      case 12: {
        current1 = format(50 * CalcECC('ascension', player.challengecompletions[12]))
        current2 = format(12 * CalcECC('ascension', player.challengecompletions[12]))
        current3 = format(20 * CalcECC('ascension', player.challengecompletions[12]))
        break
      }
      case 13: {
        current1 = format(100 - 100 * Math.pow(0.966, CalcECC('ascension', player.challengecompletions[13])), 3, true)
        current2 = format(6 * CalcECC('ascension', player.challengecompletions[13]))
        current3 = format(3 * CalcECC('ascension', player.challengecompletions[13]))
        break
      }
      case 14: {
        current1 = format(50 * CalcECC('ascension', player.challengecompletions[14]))
        current2 = format(CalcECC('ascension', player.challengecompletions[14]))
        current3 = format(1.5 * CalcECC('ascension', player.challengecompletions[14]))
        break
      }
    }

    a.textContent = i18next.t(`challenges.${i}.name`, {
      value: completions,
      completions: player.challengecompletions[i],
      max: maxChallenges
    })
    b.textContent = i18next.t(`challenges.${i}.flavor`)
    c.innerHTML = i18next.t(`challenges.${i}.restrictions`)
    d.textContent = i18next.t(`challenges.${i}.goal`, { value: goal })
    e.textContent = i18next.t(`challenges.${i}.per.1`)
    f.textContent = i18next.t(`challenges.${i}.per.2`)
    g.textContent = i18next.t(`challenges.${i}.per.3`)
    h.textContent = i18next.t(`challenges.${i}.first`)
    k.textContent = i18next.t(`challenges.${i}.start`)
    l.textContent = i18next.t(`challenges.${i}.current.1`, { value: current1 })
    m.textContent = i18next.t(`challenges.${i}.current.2`, { value: current2 })
    n.textContent = i18next.t(`challenges.${i}.current.3`, { value: current3 })
  }

  if (i === 15 && G.challengefocus === 15 && maxChallenges === 0) {
    d.textContent = i18next.t('challenges.15.noGoal')
  }

  const scoreDisplay = logicChallengeScoreDisplay(i, player.highestchallengecompletions[i])
  if (changefocus) {
    j.textContent = ''
  }
  if (player.ascensionCount === 0) {
    descriptor = 'Quarks'
    j.style.color = 'cyan'
  }
  if (
    player.challengecompletions[i] >= player.highestchallengecompletions[i]
    && player.highestchallengecompletions[i] < maxChallenges && changefocus && player.ascensionCount < 1
  ) {
    j.textContent = i18next.t(descriptor ? 'challenges.firstTimeBonusQuarks' : 'challenges.firstTimeBonus', {
      x: Math.floor(
        quarksMultiplier * player.highestchallengecompletions[i] / 10 + 1 + player.cubeUpgrades[1]
          + player.cubeUpgrades[11] + player.cubeUpgrades[21] + player.cubeUpgrades[31] + player.cubeUpgrades[41]
      )
    })
  }
  if (
    player.challengecompletions[i] >= player.highestchallengecompletions[i]
    && player.highestchallengecompletions[i] < maxChallenges && changefocus && player.ascensionCount >= 1
    && i <= 10
  ) {
    j.textContent = i18next.t('challenges.ascensionBankAdd', {
      x: i > 5 ? 2 : 1,
      y: scoreDisplay
    })
  }
  if (
    player.challengecompletions[i] >= player.highestchallengecompletions[i]
    && player.highestchallengecompletions[i] < 10 && i > 10
  ) {
    j.textContent = i18next.t('challenges.hypercubeOneTimeBonus')
  }

  if (changefocus) {
    const el = DOMCacheGetOrSet('toggleAutoChallengeIgnore')
    el.style.display = i <= (autoAscensionChallengeSweepUnlock() ? 15 : 10) && player.researches[150] > 0
      ? 'block'
      : 'none'
    el.style.border = player.autoChallengeToggles[i] ? '2px solid green' : '2px solid red'

    if (i >= 11 && i <= 15) {
      if (player.autoChallengeToggles[i]) {
        el.textContent = i18next.t('challenges.autoAscRunChalOn', { x: i })
      } else {
        el.textContent = i18next.t('challenges.autoAscRunChalOff', { x: i })
      }
    } else {
      if (player.autoChallengeToggles[i]) {
        el.textContent = i18next.t('challenges.autoRunChalOn', { x: i })
      } else {
        el.textContent = i18next.t('challenges.autoRunChalOff', { x: i })
      }
    }
  }

  const ella = DOMCacheGetOrSet('toggleAutoChallengeStart')
  if (player.autoChallengeRunning) {
    ella.textContent = i18next.t('challenges.autoChallengeSweepOn')
    ella.style.border = '2px solid gold'
  } else {
    ella.textContent = i18next.t('challenges.autoChallengeSweepOff')
    ella.style.border = '2px solid red'
  }
}

export const getChallengeConditions = (i?: number) => {
  if (player.currentChallenge.reincarnation === 9) {
    player.crystalUpgrades = [0, 0, 0, 0, 0, 0, 0, 0]
  }
  G.prestigePointGain = new Decimal('0')
  if (typeof i === 'number') {
    if (i >= 6) {
      G.transcendPointGain = new Decimal('0')
    }
    if (i >= 11) {
      G.reincarnationPointGain = new Decimal('0')
    }
  }
}

export const toggleRetryChallenges = () => {
  DOMCacheGetOrSet('retryChallenge').textContent = player.retrychallenges
    ? i18next.t('challenges.retryChallengesOff')
    : i18next.t('challenges.retryChallengesOn')

  player.retrychallenges = !player.retrychallenges
}

export const highestChallengeRewards = (chalNum: number, highestValue: number) => {
  let multiplier = 1 / 10
  if (chalNum >= 6) {
    multiplier = 1
  }
  if (player.ascensionCount === 0) {
    player.worlds.add(1 + Math.floor(highestValue * multiplier) * 100 / 100, true, true)
  }
}

// Re-export of @synergism/logic's pure CalcECC. ECC stands for "Effective
// Challenge Completions" — three piecewise linear curves keyed by tier.
export const CalcECC = logicCalcECC

export const challengeRequirement = (challenge: number, completion: number, special = 0) => {
  const c10Reduction = challenge === 10
    ? 1e8 * (player.researches[140] + player.researches[155] + player.researches[170] + player.researches[185])
      + getShopUpgradeEffects('challengeTome', 'c10RequirementReduction')
      + getShopUpgradeEffects('challengeTome2', 'c10RequirementReduction')
    : 0

  return logicChallengeRequirement({
    challenge,
    completion,
    special,
    challengeBaseRequirement: G.challengeBaseRequirements[challenge - 1],
    c10RequirementReduction: c10Reduction,
    hyperchallengeMultiplier: G.hyperchallengeMultiplier[player.corruptions.used.hyperchallenge],
    platonicUpgrade8: player.platonicUpgrades[8],
    challenge15TranscendReduction: G.challenge15Rewards.transcendChallengeReduction.value,
    challenge15ReincarnationReduction: G.challenge15Rewards.reincarnationChallengeReduction.value,
    challengeTomeC9C10ScalingReduction: getShopUpgradeEffects('challengeTome', 'c9c10ScalingReduction'),
    challengeTome2C9C10ScalingReduction: getShopUpgradeEffects('challengeTome2', 'c9c10ScalingReduction')
  })
}

// Challenge State Machine — pure logic lives in @synergism/logic
// (packages/logic/src/tick/challengeSweep.ts). This module keeps the two
// pieces of mutable bookkeeping (currentSweepState + timeSinceLastStateChange)
// in module-locals and threads them through each tick; side effects fire
// from the dispatcher below in response to challenge-sweep-transitioned events.

let currentSweepState: SweepStates = { kind: 'idle' }

function dispatchSweepTransition (from: SweepStates, to: SweepStates): void {
  // Exiting an active challenge — fire the corresponding async reset check.
  if (from.kind === 'active') {
    if (from.index <= 5) {
      void resetCheck('transcensionChallenge', undefined, true)
    } else {
      void resetCheck('reincarnationChallenge', undefined, true)
    }
  }

  switch (to.kind) {
    case 'idle':
      toggleAutoChallengeModeText('OFF')
      break
    case 'initial_wait':
      toggleAutoChallengeModeText('START')
      break
    case 'enter_wait':
      toggleAutoChallengeModeText('ENTER')
      break
    case 'active':
      toggleChallenges(to.index, true)
      toggleAutoChallengeModeText('CHALLENGE')
      break
    case 'c15_wait':
      toggleAutoChallengeModeText('WAIT')
      break
    case 'finished':
      toggleAutoChallengeModeText('COMPLETE')
      break
  }
}

export type AutoChallengeStates = 'OFF' | 'START' | 'CHALLENGE' | 'ENTER' | 'WAIT' | 'COMPLETE'

// Time (in seconds) that have been spent since the last state shift.
export let timeSinceLastStateChange = 0

function shouldRunSweep (): boolean {
  return player.researches[150] > 0 && player.autoChallengeRunning
}

export function clearStateChangeTimer (): void {
  timeSinceLastStateChange = 0
}

export function resetChallengeSweep (): void {
  if (currentSweepState.kind !== 'idle') {
    currentSweepState = { kind: 'idle' }
    timeSinceLastStateChange = 0
    toggleAutoChallengeModeText('OFF')
  }
}

export function tickChallengeSweep (dt: number): void {
  // Pre-evaluate the transition-lookup inputs that the logic state
  // machine needs, scoped to the current state's possible transitions
  // (skipping work that wouldn't be consulted this tick).
  let initialIndex = 1
  let nextRegularChallengeFromInitial = -1
  if (currentSweepState.kind === 'initial_wait') {
    if (player.highestSingularityCount >= 2 && player.currentChallenge.ascension !== 0) {
      initialIndex = 10
    }
    nextRegularChallengeFromInitial = getNextRegularChallenge(initialIndex, new Set())
  }

  let nextRegularChallengeFromActive = -1
  let chal15Check = false
  if (currentSweepState.kind === 'active') {
    nextRegularChallengeFromActive = getNextRegularChallenge(currentSweepState.index, currentSweepState.explored)
    chal15Check = challenge15AutoExponentCheck()
  }

  let isFinishedStillValid = false
  if (currentSweepState.kind === 'finished') {
    isFinishedStillValid = player.highestchallengecompletions[1] === getMaxChallenges(1)
      && player.highestchallengecompletions[6] === getMaxChallenges(6)
  }

  const result = logicTickChallengeSweep({
    dt,
    state: currentSweepState,
    timeSinceLastStateChange,
    shouldRunSweep: shouldRunSweep(),
    timerStart: player.autoChallengeTimer.start,
    timerExit: player.autoChallengeTimer.exit,
    timerEnter: player.autoChallengeTimer.enter,
    initialIndex,
    nextRegularChallengeFromInitial,
    nextRegularChallengeFromActive,
    challenge15AutoExponentCheck: chal15Check,
    isFinishedStillValid
  })

  currentSweepState = result.state
  timeSinceLastStateChange = result.timeSinceLastStateChange

  for (const event of result.events) {
    dispatchSweepEvent(event)
  }
}

function dispatchSweepEvent (event: CoreEvent): void {
  if (event.kind !== 'challenge-sweep-transitioned') return
  dispatchSweepTransition(event.from, event.to)
}

export const autoAscensionChallengeSweepUnlock = () =>
  logicAutoAscChallengeSweepUnlock(
    player.highestSingularityCount,
    getShopUpgradeEffects('instantChallenge2', 'unlocked')
  )

const challenge15AutoExponentCheck = () => {
  return autoAscensionChallengeSweepUnlock()
    && player.currentChallenge.ascension === 15
    && !getShopUpgradeEffects('challenge15Auto', 'unlocked')
    && player.autoAscend
    && player.cubeUpgrades[10] > 0
    && player.autoAscendMode === AutoAscensionResetModes.realAscensionTime
    && player.ascensionCounterRealReal >= Math.max(0.1, player.autoAscendThreshold - 5)
}

export const challenge15ScoreMultiplier = () =>
  logicChallenge15ScoreMultiplier({
    c15Bonus: player.campaigns.c15Bonus,
    challengeHepteractEffective: hepteractEffective('challenge'),
    platonicUpgrade15: player.platonicUpgrades[15]
  })

// "Regular" just means not ascension challenge
export const getNextRegularChallenge = (startIndex: number, explored: Set<number>) => {
  const maxChallenges: number[] = []
  for (let i = 1; i <= 10; i++) maxChallenges[i] = getMaxChallenges(i)
  return logicGetNextRegularChallenge({
    startIndex,
    explored,
    maxChallenges,
    highestCompletions: player.highestchallengecompletions,
    autoChallengeToggles: player.autoChallengeToggles
  })
}

// Ascension Challenge 'next' Check. We don't have access to explored so we can't just use the same logic again. Sad!
export const getNextAscensionChallenge = (startIndex: number) => {
  const maxChallenges: number[] = []
  for (let i = 11; i <= 14; i++) maxChallenges[i] = getMaxChallenges(i)
  return logicGetNextAscChallenge({
    startIndex,
    maxChallenges,
    highestCompletions: player.highestchallengecompletions,
    autoChallengeToggles: player.autoChallengeToggles
  })
}
