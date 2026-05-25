// Parity tests for resourceGain. Old body transcribed verbatim from
// packages/web_ui/src/Synergism.ts (resourceGain, ~line 2709 pre-migration),
// minus the terminal challenge resetCheck dispatch (which stays in web_ui).

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import type { CoreEvent } from '../../src/events/types'
import {
  resourceGain as newResourceGain,
  type ResourceGainInput,
  type ResourceGainResult
} from '../../src/mechanics/resourceGain'

const oldResourceGain = (input: ResourceGainInput): ResourceGainResult => {
  const dt = input.dt
  const events: CoreEvent[] = []

  let coins = input.coins
  let coinsThisPrestige = input.coinsThisPrestige
  let coinsThisTranscension = input.coinsThisTranscension
  let coinsThisReincarnation = input.coinsThisReincarnation
  let coinsTotal = input.coinsTotal
  if (input.produceTotal.gte(0.001)) {
    const addcoin = Decimal.min(
      input.produceTotal.dividedBy(input.taxdivisor),
      Decimal.pow(10, input.maxexponent - Decimal.log(input.taxdivisorcheck, 10))
    ).times(dt / 0.025)
    coins = coins.add(addcoin)
    coinsThisPrestige = coinsThisPrestige.add(addcoin)
    coinsThisTranscension = coinsThisTranscension.add(addcoin)
    coinsThisReincarnation = coinsThisReincarnation.add(addcoin)
    coinsTotal = coinsTotal.add(addcoin)
  }

  let prestigePoints = input.prestigePoints
  let transcendPoints = input.transcendPoints
  let reincarnationPoints = input.reincarnationPoints
  if (input.upgrade93 === 1 && coinsThisPrestige.gte(1e16)) {
    prestigePoints = prestigePoints.add(
      Decimal.floor(input.prestigePointGain.dividedBy(4000).times(dt / 0.025))
    )
  }
  if (input.upgrade100 === 1 && coinsThisTranscension.gte(1e100)) {
    transcendPoints = transcendPoints.add(
      Decimal.floor(input.transcendPointGain.dividedBy(4000).times(dt / 0.025))
    )
  }
  if (input.cubeUpgrade28 > 0 && input.transcendShards.gte(1e300)) {
    reincarnationPoints = reincarnationPoints.add(
      Decimal.floor(input.reincarnationPointGain.dividedBy(4000).times(dt / 0.025))
    )
  }

  const produceFirstDiamonds = input.firstGeneratedDiamonds.add(input.firstOwnedDiamonds)
    .times(input.firstProduceDiamonds).times(input.globalCrystalMultiplier)
  const produceSecondDiamonds = input.secondGeneratedDiamonds.add(input.secondOwnedDiamonds)
    .times(input.secondProduceDiamonds).times(input.globalCrystalMultiplier)
  const produceThirdDiamonds = input.thirdGeneratedDiamonds.add(input.thirdOwnedDiamonds)
    .times(input.thirdProduceDiamonds).times(input.globalCrystalMultiplier)
  const produceFourthDiamonds = input.fourthGeneratedDiamonds.add(input.fourthOwnedDiamonds)
    .times(input.fourthProduceDiamonds).times(input.globalCrystalMultiplier)
  const produceFifthDiamonds = input.fifthGeneratedDiamonds.add(input.fifthOwnedDiamonds)
    .times(input.fifthProduceDiamonds).times(input.globalCrystalMultiplier)

  const fourthGeneratedDiamonds = input.fourthGeneratedDiamonds.add(produceFifthDiamonds.times(dt / 0.025))
  const thirdGeneratedDiamonds = input.thirdGeneratedDiamonds.add(produceFourthDiamonds.times(dt / 0.025))
  const secondGeneratedDiamonds = input.secondGeneratedDiamonds.add(produceThirdDiamonds.times(dt / 0.025))
  const firstGeneratedDiamonds = input.firstGeneratedDiamonds.add(produceSecondDiamonds.times(dt / 0.025))
  const produceDiamonds = produceFirstDiamonds

  let prestigeShards = input.prestigeShards
  if (input.transcensionChallenge !== 3 && input.reincarnationChallenge !== 10) {
    prestigeShards = prestigeShards.add(produceDiamonds.times(dt / 0.025))
  }

  const produceFifthMythos = input.fifthGeneratedMythos.add(input.fifthOwnedMythos)
    .times(input.fifthProduceMythos).times(input.globalMythosMultiplier)
    .times(input.grandmasterMultiplier).times(input.mythosupgrade15)
  const produceFourthMythos = input.fourthGeneratedMythos.add(input.fourthOwnedMythos)
    .times(input.fourthProduceMythos).times(input.globalMythosMultiplier)
  const produceThirdMythos = input.thirdGeneratedMythos.add(input.thirdOwnedMythos)
    .times(input.thirdProduceMythos).times(input.globalMythosMultiplier).times(input.mythosupgrade14)
  const produceSecondMythos = input.secondGeneratedMythos.add(input.secondOwnedMythos)
    .times(input.secondProduceMythos).times(input.globalMythosMultiplier)
  const produceFirstMythos = input.firstGeneratedMythos.add(input.firstOwnedMythos)
    .times(input.firstProduceMythos).times(input.globalMythosMultiplier).times(input.mythosupgrade13)
  const fourthGeneratedMythos = input.fourthGeneratedMythos.add(produceFifthMythos.times(dt / 0.025))
  const thirdGeneratedMythos = input.thirdGeneratedMythos.add(produceFourthMythos.times(dt / 0.025))
  const secondGeneratedMythos = input.secondGeneratedMythos.add(produceThirdMythos.times(dt / 0.025))
  const firstGeneratedMythos = input.firstGeneratedMythos.add(produceSecondMythos.times(dt / 0.025))

  const produceMythos = firstGeneratedMythos.add(input.firstOwnedMythos)
    .times(input.firstProduceMythos).times(input.globalMythosMultiplier).times(input.mythosupgrade13)
  const producePerSecondMythos = produceMythos.times(40)

  let pm = new Decimal('1')
  if (input.upgrade67 > 0.5) {
    pm = pm.times(Decimal.pow(
      1.03,
      input.firstOwnedParticles + input.secondOwnedParticles
        + input.thirdOwnedParticles + input.fourthOwnedParticles + input.fifthOwnedParticles
    ))
  }
  const produceFifthParticles = input.fifthGeneratedParticles.add(input.fifthOwnedParticles).times(input.fifthProduceParticles)
  const produceFourthParticles = input.fourthGeneratedParticles.add(input.fourthOwnedParticles).times(input.fourthProduceParticles)
  const produceThirdParticles = input.thirdGeneratedParticles.add(input.thirdOwnedParticles).times(input.thirdProduceParticles)
  const produceSecondParticles = input.secondGeneratedParticles.add(input.secondOwnedParticles).times(input.secondProduceParticles)
  const produceFirstParticles = input.firstGeneratedParticles.add(input.firstOwnedParticles).times(input.firstProduceParticles).times(pm)
  const fourthGeneratedParticles = input.fourthGeneratedParticles.add(produceFifthParticles.times(dt / 0.025))
  const thirdGeneratedParticles = input.thirdGeneratedParticles.add(produceFourthParticles.times(dt / 0.025))
  const secondGeneratedParticles = input.secondGeneratedParticles.add(produceThirdParticles.times(dt / 0.025))
  const firstGeneratedParticles = input.firstGeneratedParticles.add(produceSecondParticles.times(dt / 0.025))

  const produceParticles = firstGeneratedParticles.add(input.firstOwnedParticles).times(input.firstProduceParticles).times(pm)
  const producePerSecondParticles = produceParticles.times(40)

  let transcendShards = input.transcendShards
  if (input.transcensionChallenge !== 3 && input.reincarnationChallenge !== 10) {
    transcendShards = transcendShards.add(produceMythos.times(dt / 0.025))
  }
  const reincarnationShards = input.reincarnationShards.add(produceParticles.times(dt / 0.025))

  const ascendOwned = [input.ascendBuilding1Owned, input.ascendBuilding2Owned, input.ascendBuilding3Owned, input.ascendBuilding4Owned, input.ascendBuilding5Owned]
  const ascendGenerated = [input.ascendBuilding1Generated, input.ascendBuilding2Generated, input.ascendBuilding3Generated, input.ascendBuilding4Generated, input.ascendBuilding5Generated]
  const ascendProd = [new Decimal(0), new Decimal(0), new Decimal(0), new Decimal(0), new Decimal(0)]
  for (let j = 4; j >= 0; j--) {
    ascendProd[j] = ascendGenerated[j].add(ascendOwned[j]).times(0.05).times(input.globalConstantMult)
    if (j !== 0) {
      ascendGenerated[j - 1] = ascendGenerated[j - 1].add(ascendProd[j].times(dt))
    }
  }
  const ascendShards = input.ascendShards.add(ascendProd[0].times(dt))

  if (input.ascensionCount > 0) {
    events.push({ kind: 'achievement-group-awarded', group: 'constant' })
  }

  let c1Completions = input.c1Completions
  let c2Completions = input.c2Completions
  let c3Completions = input.c3Completions
  let c4Completions = input.c4Completions
  let c5Completions = input.c5Completions

  if (
    input.research71 > 0.5
    && c1Completions < Math.min(input.highestC1, 25 + 5 * input.research66 + 925 * input.research105)
    && coins.gte(Decimal.pow(10, 1.25 * input.challengeBaseRequirements[0] * Math.pow(1 + c1Completions, 2)))
  ) {
    c1Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 1, newCompletions: c1Completions })
  }
  if (
    input.research72 > 0.5
    && c2Completions < Math.min(input.highestC2, 25 + 5 * input.research67_ + 925 * input.research105)
    && coins.gte(Decimal.pow(10, 1.6 * input.challengeBaseRequirements[1] * Math.pow(1 + c2Completions, 2)))
  ) {
    c2Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 2, newCompletions: c2Completions })
  }
  if (
    input.research73 > 0.5
    && c3Completions < Math.min(input.highestC3, 25 + 5 * input.research68 + 925 * input.research105)
    && coins.gte(Decimal.pow(10, 1.7 * input.challengeBaseRequirements[2] * Math.pow(1 + c3Completions, 2)))
  ) {
    c3Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 3, newCompletions: c3Completions })
  }
  if (
    input.research74 > 0.5
    && c4Completions < Math.min(input.highestC4, 25 + 5 * input.research69 + 925 * input.research105)
    && coins.gte(Decimal.pow(10, 1.45 * input.challengeBaseRequirements[3] * Math.pow(1 + c4Completions, 2)))
  ) {
    c4Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 4, newCompletions: c4Completions })
  }
  if (
    input.research75 > 0.5
    && c5Completions < Math.min(input.highestC5, 25 + 5 * input.research70 + 925 * input.research105)
    && coins.gte(Decimal.pow(10, 2 * input.challengeBaseRequirements[4] * Math.pow(1 + c5Completions, 2)))
  ) {
    c5Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 5, newCompletions: c5Completions })
  }

  return {
    coins,
    coinsThisPrestige,
    coinsThisTranscension,
    coinsThisReincarnation,
    coinsTotal,
    prestigePoints,
    transcendPoints,
    reincarnationPoints,
    prestigeShards,
    transcendShards,
    reincarnationShards,
    ascendShards,
    firstGeneratedDiamonds,
    secondGeneratedDiamonds,
    thirdGeneratedDiamonds,
    fourthGeneratedDiamonds,
    firstGeneratedMythos,
    secondGeneratedMythos,
    thirdGeneratedMythos,
    fourthGeneratedMythos,
    firstGeneratedParticles,
    secondGeneratedParticles,
    thirdGeneratedParticles,
    fourthGeneratedParticles,
    ascendBuilding1Generated: ascendGenerated[0],
    ascendBuilding2Generated: ascendGenerated[1],
    ascendBuilding3Generated: ascendGenerated[2],
    ascendBuilding4Generated: ascendGenerated[3],
    c1Completions,
    c2Completions,
    c3Completions,
    c4Completions,
    c5Completions,
    produceFirstDiamonds,
    produceSecondDiamonds,
    produceThirdDiamonds,
    produceFourthDiamonds,
    produceFifthDiamonds,
    produceDiamonds,
    produceFirstMythos,
    produceSecondMythos,
    produceThirdMythos,
    produceFourthMythos,
    produceFifthMythos,
    produceMythos,
    producePerSecondMythos,
    produceFirstParticles,
    produceSecondParticles,
    produceThirdParticles,
    produceFourthParticles,
    produceFifthParticles,
    produceParticles,
    producePerSecondParticles,
    ascendBuildingProduction: { first: ascendProd[0], second: ascendProd[1], third: ascendProd[2], fourth: ascendProd[3], fifth: ascendProd[4] },
    events
  }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

const baseInput: ResourceGainInput = {
  dt: 0.025,
  produceTotal: new Decimal(0),
  taxdivisor: new Decimal(1),
  taxdivisorcheck: new Decimal(1),
  maxexponent: 100,
  coins: new Decimal(0),
  coinsThisPrestige: new Decimal(0),
  coinsThisTranscension: new Decimal(0),
  coinsThisReincarnation: new Decimal(0),
  coinsTotal: new Decimal(0),

  upgrade93: 0, upgrade100: 0, cubeUpgrade28: 0,
  prestigePoints: new Decimal(0), transcendPoints: new Decimal(0), reincarnationPoints: new Decimal(0),
  prestigePointGain: new Decimal(0), transcendPointGain: new Decimal(0), reincarnationPointGain: new Decimal(0),

  firstGeneratedDiamonds: new Decimal(0), secondGeneratedDiamonds: new Decimal(0),
  thirdGeneratedDiamonds: new Decimal(0), fourthGeneratedDiamonds: new Decimal(0), fifthGeneratedDiamonds: new Decimal(0),
  firstOwnedDiamonds: 0, secondOwnedDiamonds: 0, thirdOwnedDiamonds: 0, fourthOwnedDiamonds: 0, fifthOwnedDiamonds: 0,
  firstProduceDiamonds: 0, secondProduceDiamonds: 0,
  thirdProduceDiamonds: 0, fourthProduceDiamonds: 0, fifthProduceDiamonds: 0,
  globalCrystalMultiplier: new Decimal(1),

  firstGeneratedMythos: new Decimal(0), secondGeneratedMythos: new Decimal(0),
  thirdGeneratedMythos: new Decimal(0), fourthGeneratedMythos: new Decimal(0), fifthGeneratedMythos: new Decimal(0),
  firstOwnedMythos: 0, secondOwnedMythos: 0, thirdOwnedMythos: 0, fourthOwnedMythos: 0, fifthOwnedMythos: 0,
  firstProduceMythos: 0, secondProduceMythos: 0,
  thirdProduceMythos: 0, fourthProduceMythos: 0, fifthProduceMythos: 0,
  globalMythosMultiplier: new Decimal(1), grandmasterMultiplier: new Decimal(1),
  mythosupgrade13: new Decimal(1), mythosupgrade14: new Decimal(1), mythosupgrade15: new Decimal(1),

  firstGeneratedParticles: new Decimal(0), secondGeneratedParticles: new Decimal(0),
  thirdGeneratedParticles: new Decimal(0), fourthGeneratedParticles: new Decimal(0), fifthGeneratedParticles: new Decimal(0),
  firstOwnedParticles: 0, secondOwnedParticles: 0, thirdOwnedParticles: 0, fourthOwnedParticles: 0, fifthOwnedParticles: 0,
  firstProduceParticles: 0, secondProduceParticles: 0,
  thirdProduceParticles: 0, fourthProduceParticles: 0, fifthProduceParticles: 0,
  upgrade67: 0,

  prestigeShards: new Decimal(0), transcendShards: new Decimal(0),
  reincarnationShards: new Decimal(0), ascendShards: new Decimal(0),

  ascendBuilding1Generated: new Decimal(0), ascendBuilding2Generated: new Decimal(0),
  ascendBuilding3Generated: new Decimal(0), ascendBuilding4Generated: new Decimal(0), ascendBuilding5Generated: new Decimal(0),
  ascendBuilding1Owned: 0, ascendBuilding2Owned: 0, ascendBuilding3Owned: 0, ascendBuilding4Owned: 0, ascendBuilding5Owned: 0,
  globalConstantMult: new Decimal(1),

  ascensionCount: 0,
  transcensionChallenge: 0, reincarnationChallenge: 0,

  research66: 0, research67_: 0, research68: 0, research69: 0, research70: 0,
  research71: 0, research72: 0, research73: 0, research74: 0, research75: 0,
  research105: 0,
  c1Completions: 0, c2Completions: 0, c3Completions: 0, c4Completions: 0, c5Completions: 0,
  highestC1: 0, highestC2: 0, highestC3: 0, highestC4: 0, highestC5: 0,
  challengeBaseRequirements: [10, 30, 100, 300, 1000]
}

const cases: Array<{ name: string, input: ResourceGainInput }> = [
  { name: 'baseline (nothing happens)', input: baseInput },

  // ─── Coin gain ────────────────────────────────────────────────────────
  {
    name: 'coin gain when produceTotal ≥ 0.001',
    input: { ...baseInput, produceTotal: new Decimal(1e10), taxdivisor: new Decimal(100), maxexponent: 30, taxdivisorcheck: new Decimal(100), coins: new Decimal(1e20), coinsThisPrestige: new Decimal(1e20), coinsThisTranscension: new Decimal(1e20), coinsThisReincarnation: new Decimal(1e20), coinsTotal: new Decimal(1e20) }
  },
  {
    name: 'coin gain skipped when produceTotal < 0.001',
    input: { ...baseInput, produceTotal: new Decimal(0.0001) }
  },
  {
    name: 'coin gain maxexponent clamp',
    input: { ...baseInput, produceTotal: new Decimal('1e1000'), taxdivisor: new Decimal(100), taxdivisorcheck: new Decimal(100), maxexponent: 10 }
  },

  // ─── Point gains ──────────────────────────────────────────────────────
  {
    name: 'upgrade 93 prestige point drip',
    input: { ...baseInput, upgrade93: 1, coinsThisPrestige: new Decimal(1e20), prestigePointGain: new Decimal(8000) }
  },
  {
    name: 'upgrade 93 no drip without coin threshold',
    input: { ...baseInput, upgrade93: 1, coinsThisPrestige: new Decimal(1e10), prestigePointGain: new Decimal(8000) }
  },
  {
    name: 'upgrade 100 transcend point drip',
    input: { ...baseInput, upgrade100: 1, coinsThisTranscension: new Decimal('1e120'), transcendPointGain: new Decimal(8000) }
  },
  {
    name: 'cubeUpgrade 28 reincarnation point drip',
    input: { ...baseInput, cubeUpgrade28: 1, transcendShards: new Decimal('1e310'), reincarnationPointGain: new Decimal(8000) }
  },

  // ─── Diamond cascade ──────────────────────────────────────────────────
  {
    name: 'diamond cascade with all 5 tiers + crystal mult',
    input: {
      ...baseInput,
      firstGeneratedDiamonds: new Decimal(10), secondGeneratedDiamonds: new Decimal(5),
      thirdGeneratedDiamonds: new Decimal(2), fourthGeneratedDiamonds: new Decimal(1),
      fifthGeneratedDiamonds: new Decimal(1),
      firstOwnedDiamonds: 100, secondOwnedDiamonds: 50, thirdOwnedDiamonds: 25,
      fourthOwnedDiamonds: 10, fifthOwnedDiamonds: 5,
      firstProduceDiamonds: 1, secondProduceDiamonds: 1,
      thirdProduceDiamonds: 1, fourthProduceDiamonds: 1,
      fifthProduceDiamonds: 1,
      globalCrystalMultiplier: new Decimal(1e5)
    }
  },
  {
    name: 'prestigeShards gated off in t-chal 3',
    input: { ...baseInput, transcensionChallenge: 3, firstOwnedDiamonds: 100, firstProduceDiamonds: 1, globalCrystalMultiplier: new Decimal(1) }
  },
  {
    name: 'prestigeShards gated off in r-chal 10',
    input: { ...baseInput, reincarnationChallenge: 10, firstOwnedDiamonds: 100, firstProduceDiamonds: 1, globalCrystalMultiplier: new Decimal(1) }
  },

  // ─── Mythos cascade ───────────────────────────────────────────────────
  {
    name: 'mythos cascade with grandmaster + upgrades 13/14/15',
    input: {
      ...baseInput,
      firstGeneratedMythos: new Decimal(10), secondGeneratedMythos: new Decimal(5),
      thirdGeneratedMythos: new Decimal(2), fourthGeneratedMythos: new Decimal(1),
      fifthGeneratedMythos: new Decimal(1),
      firstOwnedMythos: 100, secondOwnedMythos: 50, thirdOwnedMythos: 25,
      fourthOwnedMythos: 10, fifthOwnedMythos: 5,
      firstProduceMythos: 1, secondProduceMythos: 1,
      thirdProduceMythos: 1, fourthProduceMythos: 1,
      fifthProduceMythos: 1,
      globalMythosMultiplier: new Decimal(1e5),
      grandmasterMultiplier: new Decimal(1.5),
      mythosupgrade13: new Decimal(2), mythosupgrade14: new Decimal(2), mythosupgrade15: new Decimal(2)
    }
  },
  {
    name: 'transcendShards gated off in t-chal 3 / r-chal 10',
    input: { ...baseInput, transcensionChallenge: 3, firstOwnedMythos: 100, firstProduceMythos: 1, globalMythosMultiplier: new Decimal(1) }
  },

  // ─── Particle cascade ─────────────────────────────────────────────────
  {
    name: 'particle cascade without upgrade 67',
    input: {
      ...baseInput,
      firstGeneratedParticles: new Decimal(10), secondGeneratedParticles: new Decimal(5),
      thirdGeneratedParticles: new Decimal(2), fourthGeneratedParticles: new Decimal(1),
      fifthGeneratedParticles: new Decimal(1),
      firstOwnedParticles: 100, secondOwnedParticles: 50, thirdOwnedParticles: 25,
      fourthOwnedParticles: 10, fifthOwnedParticles: 5,
      firstProduceParticles: 1, secondProduceParticles: 1,
      thirdProduceParticles: 1, fourthProduceParticles: 1,
      fifthProduceParticles: 1
    }
  },
  {
    name: 'particle cascade with upgrade 67 pm factor',
    input: {
      ...baseInput,
      upgrade67: 1,
      firstOwnedParticles: 50, secondOwnedParticles: 50, thirdOwnedParticles: 50,
      fourthOwnedParticles: 50, fifthOwnedParticles: 50,
      firstProduceParticles: 1
    }
  },

  // ─── AscendBuildings cascade ──────────────────────────────────────────
  {
    name: 'ascendBuilding cascade with globalConstantMult',
    input: {
      ...baseInput,
      ascendBuilding1Owned: 100, ascendBuilding2Owned: 50, ascendBuilding3Owned: 25,
      ascendBuilding4Owned: 10, ascendBuilding5Owned: 5,
      ascendBuilding1Generated: new Decimal(10), ascendBuilding2Generated: new Decimal(5),
      ascendBuilding3Generated: new Decimal(2), ascendBuilding4Generated: new Decimal(1),
      ascendBuilding5Generated: new Decimal(1),
      globalConstantMult: new Decimal(10),
      dt: 1
    }
  },

  // ─── Achievement event ───────────────────────────────────────────────
  {
    name: 'awardAchievementGroup constant emitted when ascensionCount > 0',
    input: { ...baseInput, ascensionCount: 5 }
  },

  // ─── Challenge auto-completion ────────────────────────────────────────
  {
    name: 'c1 auto-completes when research71 + coins + highestC1 satisfy',
    input: { ...baseInput, research71: 1, highestC1: 25, coins: new Decimal('1e30'), challengeBaseRequirements: [10, 30, 100, 300, 1000] }
  },
  {
    name: 'c1 does not auto-complete when at highest cap',
    input: { ...baseInput, research71: 1, highestC1: 10, c1Completions: 10, coins: new Decimal('1e100') }
  },
  {
    name: 'c2 auto-completes',
    input: { ...baseInput, research72: 1, highestC2: 25, coins: new Decimal('1e50') }
  },
  {
    name: 'c3 auto-completes',
    input: { ...baseInput, research73: 1, highestC3: 25, coins: new Decimal('1e200') }
  },
  {
    name: 'c4 auto-completes',
    input: { ...baseInput, research74: 1, highestC4: 25, coins: new Decimal('1e500') }
  },
  {
    name: 'c5 auto-completes',
    input: { ...baseInput, research75: 1, highestC5: 25, coins: new Decimal('1e3000') }
  },
  {
    name: 'all 5 challenges auto-complete in one tick',
    input: {
      ...baseInput,
      research71: 1, research72: 1, research73: 1, research74: 1, research75: 1,
      highestC1: 25, highestC2: 25, highestC3: 25, highestC4: 25, highestC5: 25,
      coins: new Decimal('1e10000'),
      ascensionCount: 1
    }
  },

  // ─── Combined late-game tick ─────────────────────────────────────────
  {
    name: 'big late-game tick: cascades + points + achievement + challenges',
    input: {
      ...baseInput,
      dt: 0.5,
      produceTotal: new Decimal('1e100'),
      taxdivisor: new Decimal(1000),
      taxdivisorcheck: new Decimal(1000),
      maxexponent: 300,
      coins: new Decimal('1e300'),
      coinsThisPrestige: new Decimal('1e300'),
      coinsThisTranscension: new Decimal('1e300'),
      coinsThisReincarnation: new Decimal('1e300'),
      coinsTotal: new Decimal('1e300'),
      upgrade93: 1, upgrade100: 1, cubeUpgrade28: 1,
      prestigePointGain: new Decimal('1e50'),
      transcendPointGain: new Decimal('1e30'),
      reincarnationPointGain: new Decimal('1e20'),
      transcendShards: new Decimal('1e310'),
      firstOwnedDiamonds: 100, fifthOwnedDiamonds: 50, fifthProduceDiamonds: 10, firstProduceDiamonds: 5,
      globalCrystalMultiplier: new Decimal(1e10),
      firstOwnedMythos: 100, fifthOwnedMythos: 50, fifthProduceMythos: 10, firstProduceMythos: 5,
      globalMythosMultiplier: new Decimal(1e10),
      mythosupgrade13: new Decimal(5), mythosupgrade14: new Decimal(5), mythosupgrade15: new Decimal(5),
      grandmasterMultiplier: new Decimal(2),
      upgrade67: 1,
      firstOwnedParticles: 100, fifthOwnedParticles: 50, fifthProduceParticles: 10, firstProduceParticles: 5,
      secondOwnedParticles: 75, thirdOwnedParticles: 50, fourthOwnedParticles: 25,
      ascendBuilding1Owned: 1000, ascendBuilding5Owned: 100,
      globalConstantMult: new Decimal(100),
      ascensionCount: 50,
      research71: 1, research72: 1, research73: 1, research74: 1, research75: 1,
      highestC1: 25, highestC2: 25, highestC3: 25, highestC4: 25, highestC5: 25,
      c1Completions: 10, c2Completions: 5
    }
  }
]

const expectEventsEqual = (a: CoreEvent[], b: CoreEvent[]): void => {
  expect(a.length).toBe(b.length)
  for (let i = 0; i < a.length; i++) {
    expect(a[i]).toEqual(b[i])
  }
}

describe('parity: resourceGain', () => {
  for (const c of cases) {
    it(c.name, () => {
      const newRes = newResourceGain(c.input)
      const oldRes = oldResourceGain(c.input)
      expect(decimalEq(newRes.coins, oldRes.coins)).toBe(true)
      expect(decimalEq(newRes.coinsThisPrestige, oldRes.coinsThisPrestige)).toBe(true)
      expect(decimalEq(newRes.coinsThisTranscension, oldRes.coinsThisTranscension)).toBe(true)
      expect(decimalEq(newRes.coinsThisReincarnation, oldRes.coinsThisReincarnation)).toBe(true)
      expect(decimalEq(newRes.coinsTotal, oldRes.coinsTotal)).toBe(true)
      expect(decimalEq(newRes.prestigePoints, oldRes.prestigePoints)).toBe(true)
      expect(decimalEq(newRes.transcendPoints, oldRes.transcendPoints)).toBe(true)
      expect(decimalEq(newRes.reincarnationPoints, oldRes.reincarnationPoints)).toBe(true)
      expect(decimalEq(newRes.prestigeShards, oldRes.prestigeShards)).toBe(true)
      expect(decimalEq(newRes.transcendShards, oldRes.transcendShards)).toBe(true)
      expect(decimalEq(newRes.reincarnationShards, oldRes.reincarnationShards)).toBe(true)
      expect(decimalEq(newRes.ascendShards, oldRes.ascendShards)).toBe(true)
      expect(decimalEq(newRes.firstGeneratedDiamonds, oldRes.firstGeneratedDiamonds)).toBe(true)
      expect(decimalEq(newRes.secondGeneratedDiamonds, oldRes.secondGeneratedDiamonds)).toBe(true)
      expect(decimalEq(newRes.thirdGeneratedDiamonds, oldRes.thirdGeneratedDiamonds)).toBe(true)
      expect(decimalEq(newRes.fourthGeneratedDiamonds, oldRes.fourthGeneratedDiamonds)).toBe(true)
      expect(decimalEq(newRes.firstGeneratedMythos, oldRes.firstGeneratedMythos)).toBe(true)
      expect(decimalEq(newRes.secondGeneratedMythos, oldRes.secondGeneratedMythos)).toBe(true)
      expect(decimalEq(newRes.thirdGeneratedMythos, oldRes.thirdGeneratedMythos)).toBe(true)
      expect(decimalEq(newRes.fourthGeneratedMythos, oldRes.fourthGeneratedMythos)).toBe(true)
      expect(decimalEq(newRes.firstGeneratedParticles, oldRes.firstGeneratedParticles)).toBe(true)
      expect(decimalEq(newRes.secondGeneratedParticles, oldRes.secondGeneratedParticles)).toBe(true)
      expect(decimalEq(newRes.thirdGeneratedParticles, oldRes.thirdGeneratedParticles)).toBe(true)
      expect(decimalEq(newRes.fourthGeneratedParticles, oldRes.fourthGeneratedParticles)).toBe(true)
      expect(decimalEq(newRes.ascendBuilding1Generated, oldRes.ascendBuilding1Generated)).toBe(true)
      expect(decimalEq(newRes.ascendBuilding2Generated, oldRes.ascendBuilding2Generated)).toBe(true)
      expect(decimalEq(newRes.ascendBuilding3Generated, oldRes.ascendBuilding3Generated)).toBe(true)
      expect(decimalEq(newRes.ascendBuilding4Generated, oldRes.ascendBuilding4Generated)).toBe(true)
      expect(newRes.c1Completions).toBe(oldRes.c1Completions)
      expect(newRes.c2Completions).toBe(oldRes.c2Completions)
      expect(newRes.c3Completions).toBe(oldRes.c3Completions)
      expect(newRes.c4Completions).toBe(oldRes.c4Completions)
      expect(newRes.c5Completions).toBe(oldRes.c5Completions)
      expect(decimalEq(newRes.produceFirstDiamonds, oldRes.produceFirstDiamonds)).toBe(true)
      expect(decimalEq(newRes.produceFifthDiamonds, oldRes.produceFifthDiamonds)).toBe(true)
      expect(decimalEq(newRes.produceDiamonds, oldRes.produceDiamonds)).toBe(true)
      expect(decimalEq(newRes.produceFirstMythos, oldRes.produceFirstMythos)).toBe(true)
      expect(decimalEq(newRes.produceFifthMythos, oldRes.produceFifthMythos)).toBe(true)
      expect(decimalEq(newRes.produceMythos, oldRes.produceMythos)).toBe(true)
      expect(decimalEq(newRes.producePerSecondMythos, oldRes.producePerSecondMythos)).toBe(true)
      expect(decimalEq(newRes.produceFirstParticles, oldRes.produceFirstParticles)).toBe(true)
      expect(decimalEq(newRes.produceFifthParticles, oldRes.produceFifthParticles)).toBe(true)
      expect(decimalEq(newRes.produceParticles, oldRes.produceParticles)).toBe(true)
      expect(decimalEq(newRes.producePerSecondParticles, oldRes.producePerSecondParticles)).toBe(true)
      expect(decimalEq(newRes.ascendBuildingProduction.first, oldRes.ascendBuildingProduction.first)).toBe(true)
      expect(decimalEq(newRes.ascendBuildingProduction.fifth, oldRes.ascendBuildingProduction.fifth)).toBe(true)
      expectEventsEqual(newRes.events, oldRes.events)
    })
  }
})
