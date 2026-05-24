// Per-tick resource generation. Lifted from packages/web_ui/src/Synergism.ts
// (resourceGain), minus the terminal challenge `resetCheck` dispatch which
// is async + modal-aware and stays in web_ui.
//
// Computes:
//   1. Coin gain (5 coin counters when produceTotal ≥ 0.001).
//   2. Per-tier point gains from upgrade-93 / upgrade-100 / cubeUpgrade-28.
//   3. Four producer cascades (Diamonds, Mythos, Particles, AscendBuildings)
//      — each computes its 5 G.produce*Tier fields then advances 4 generated
//      counters from the next tier's production.
//   4. Shard accumulation (prestige/transcend/reincarnation/ascend).
//   5. `awardAchievementGroup('constant')` gate (ascensionCount > 0).
//   6. Challenge 1-5 auto-completion (research-gated coin thresholds).
//
// Side effects surface as CoreEvents:
//   - achievement-group-awarded ('constant')
//   - challenge-auto-completed (one per c1-c5 increment)
//
// The caller orchestrates the pre-tick functions (calculateTotalAcceleratorBoost,
// updateAllTick, updateAllMultiplier, multipliers, calculatetax, resetCurrency)
// before invoking this so all G inputs are fresh.

import type { CoreEvent } from '../events/types'
import { Decimal } from '../math/bignum'

export interface ResourceGainInput {
  /** Tick delta in seconds (already scaled by globalSpeedMult by the caller). */
  dt: number

  // ─── Coin gain inputs (G.produceTotal / taxdivisor / maxexponent set by Phase 2) ─

  produceTotal: Decimal
  taxdivisor: Decimal
  taxdivisorcheck: Decimal
  maxexponent: number
  coins: Decimal
  coinsThisPrestige: Decimal
  coinsThisTranscension: Decimal
  coinsThisReincarnation: Decimal
  coinsTotal: Decimal

  // ─── Point gain inputs ──────────────────────────────────────────────

  /** player.upgrades[93] — when === 1 and coinsThisPrestige ≥ 1e16, drips prestigePoints. */
  upgrade93: number
  /** player.upgrades[100] — when === 1 and coinsThisTranscension ≥ 1e100, drips transcendPoints. */
  upgrade100: number
  /** player.cubeUpgrades[28] — when > 0 and transcendShards ≥ 1e300, drips reincarnationPoints. */
  cubeUpgrade28: number
  prestigePoints: Decimal
  transcendPoints: Decimal
  reincarnationPoints: Decimal
  /** Pre-evaluated G.prestigePointGain (set by resetCurrency earlier this tick). */
  prestigePointGain: Decimal
  /** Pre-evaluated G.transcendPointGain (set by resetCurrency earlier this tick). */
  transcendPointGain: Decimal
  /** Pre-evaluated G.reincarnationPointGain (set by resetCurrency earlier this tick). */
  reincarnationPointGain: Decimal

  // ─── Diamond cascade (5 generated + 5 owned + 5 produce values) ─────

  firstGeneratedDiamonds: Decimal
  secondGeneratedDiamonds: Decimal
  thirdGeneratedDiamonds: Decimal
  fourthGeneratedDiamonds: Decimal
  fifthGeneratedDiamonds: Decimal
  firstOwnedDiamonds: number
  secondOwnedDiamonds: number
  thirdOwnedDiamonds: number
  fourthOwnedDiamonds: number
  fifthOwnedDiamonds: number
  firstProduceDiamonds: number
  secondProduceDiamonds: number
  thirdProduceDiamonds: number
  fourthProduceDiamonds: number
  fifthProduceDiamonds: number
  /** Pre-evaluated G.globalCrystalMultiplier — multiplied into every diamond tier. */
  globalCrystalMultiplier: Decimal

  // ─── Mythos cascade (5 generated + 5 owned + 5 produce, plus 3 G mult inputs) ─

  firstGeneratedMythos: Decimal
  secondGeneratedMythos: Decimal
  thirdGeneratedMythos: Decimal
  fourthGeneratedMythos: Decimal
  fifthGeneratedMythos: Decimal
  firstOwnedMythos: number
  secondOwnedMythos: number
  thirdOwnedMythos: number
  fourthOwnedMythos: number
  fifthOwnedMythos: number
  firstProduceMythos: number
  secondProduceMythos: number
  thirdProduceMythos: number
  fourthProduceMythos: number
  fifthProduceMythos: number
  /** Pre-evaluated G.globalMythosMultiplier — base for every mythos tier. */
  globalMythosMultiplier: Decimal
  /** Pre-evaluated G.grandmasterMultiplier — only the fifth tier multiplies by this. */
  grandmasterMultiplier: Decimal
  /** Pre-evaluated G.mythosupgrade13 — only the first tier multiplies by this. */
  mythosupgrade13: Decimal
  /** Pre-evaluated G.mythosupgrade14 — only the third tier multiplies by this. */
  mythosupgrade14: Decimal
  /** Pre-evaluated G.mythosupgrade15 — only the fifth tier multiplies by this. */
  mythosupgrade15: Decimal

  // ─── Particle cascade (5 generated + 5 owned + 5 produce + upgrade67 gate) ─

  firstGeneratedParticles: Decimal
  secondGeneratedParticles: Decimal
  thirdGeneratedParticles: Decimal
  fourthGeneratedParticles: Decimal
  fifthGeneratedParticles: Decimal
  firstOwnedParticles: number
  secondOwnedParticles: number
  thirdOwnedParticles: number
  fourthOwnedParticles: number
  fifthOwnedParticles: number
  firstProduceParticles: number
  secondProduceParticles: number
  thirdProduceParticles: number
  fourthProduceParticles: number
  fifthProduceParticles: number
  /** player.upgrades[67] — when > 0.5, applies pm = 1.03 ^ totalOwnedParticles to the first tier. */
  upgrade67: number

  // ─── Shards ─────────────────────────────────────────────────────────

  prestigeShards: Decimal
  transcendShards: Decimal
  reincarnationShards: Decimal
  ascendShards: Decimal

  // ─── AscendBuildings (5 generated + 5 owned, plus globalConstantMult) ─

  ascendBuilding1Generated: Decimal
  ascendBuilding2Generated: Decimal
  ascendBuilding3Generated: Decimal
  ascendBuilding4Generated: Decimal
  ascendBuilding5Generated: Decimal
  ascendBuilding1Owned: number
  ascendBuilding2Owned: number
  ascendBuilding3Owned: number
  ascendBuilding4Owned: number
  ascendBuilding5Owned: number
  /** Pre-evaluated G.globalConstantMult (set by multipliers earlier this tick). */
  globalConstantMult: Decimal

  // ─── Achievement + challenge auto-completion inputs ─────────────────

  /** player.ascensionCount — gates the awardAchievementGroup('constant') event. */
  ascensionCount: number
  /** player.currentChallenge.transcension — disables prestigeShards / transcendShards gains when === 3. */
  transcensionChallenge: number
  /** player.currentChallenge.reincarnation — also disables both shards branches when === 10. */
  reincarnationChallenge: number

  // Challenge 1-5 auto-completion gates
  research66: number
  research67_: number  // research[67] — name disambiguated from upgrade67
  research68: number
  research69: number
  research70: number
  research71: number
  research72: number
  research73: number
  research74: number
  research75: number
  research105: number
  c1Completions: number
  c2Completions: number
  c3Completions: number
  c4Completions: number
  c5Completions: number
  highestC1: number
  highestC2: number
  highestC3: number
  highestC4: number
  highestC5: number
  /** G.challengeBaseRequirements — 5-element array; passed by reference (read-only). */
  challengeBaseRequirements: readonly number[]
}

export interface ResourceGainResult {
  // ─── Player updates (every field is the post-tick value, even if unchanged) ─
  coins: Decimal
  coinsThisPrestige: Decimal
  coinsThisTranscension: Decimal
  coinsThisReincarnation: Decimal
  coinsTotal: Decimal
  prestigePoints: Decimal
  transcendPoints: Decimal
  reincarnationPoints: Decimal
  prestigeShards: Decimal
  transcendShards: Decimal
  reincarnationShards: Decimal
  ascendShards: Decimal
  firstGeneratedDiamonds: Decimal
  secondGeneratedDiamonds: Decimal
  thirdGeneratedDiamonds: Decimal
  fourthGeneratedDiamonds: Decimal
  firstGeneratedMythos: Decimal
  secondGeneratedMythos: Decimal
  thirdGeneratedMythos: Decimal
  fourthGeneratedMythos: Decimal
  firstGeneratedParticles: Decimal
  secondGeneratedParticles: Decimal
  thirdGeneratedParticles: Decimal
  fourthGeneratedParticles: Decimal
  /** Only first-fourth ascendBuilding generated values can change (5th doesn't have a "next tier" to feed it). */
  ascendBuilding1Generated: Decimal
  ascendBuilding2Generated: Decimal
  ascendBuilding3Generated: Decimal
  ascendBuilding4Generated: Decimal
  c1Completions: number
  c2Completions: number
  c3Completions: number
  c4Completions: number
  c5Completions: number

  // ─── G cache updates ────────────────────────────────────────────────
  produceFirstDiamonds: Decimal
  produceSecondDiamonds: Decimal
  produceThirdDiamonds: Decimal
  produceFourthDiamonds: Decimal
  produceFifthDiamonds: Decimal
  produceDiamonds: Decimal
  produceFirstMythos: Decimal
  produceSecondMythos: Decimal
  produceThirdMythos: Decimal
  produceFourthMythos: Decimal
  produceFifthMythos: Decimal
  produceMythos: Decimal
  producePerSecondMythos: Decimal
  produceFirstParticles: Decimal
  produceSecondParticles: Decimal
  produceThirdParticles: Decimal
  produceFourthParticles: Decimal
  produceFifthParticles: Decimal
  produceParticles: Decimal
  producePerSecondParticles: Decimal
  /** G.ascendBuildingProduction — { first, second, third, fourth, fifth } Decimal values. */
  ascendBuildingProduction: {
    first: Decimal
    second: Decimal
    third: Decimal
    fourth: Decimal
    fifth: Decimal
  }

  /** Events for the UI tier to dispatch (achievement notifications, challenge-auto-completion UI updates). */
  events: CoreEvent[]
}

/**
 * Per-tick resource generation. Pure given the input bundle; returns the
 * full post-tick player + G slice plus an event list.
 */
export function resourceGain (input: ResourceGainInput): ResourceGainResult {
  const dt = input.dt
  const events: CoreEvent[] = []

  // ─── Coin gain ─────────────────────────────────────────────────────
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

  // ─── Point gains ───────────────────────────────────────────────────
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

  // ─── Diamond cascade ───────────────────────────────────────────────
  const produceFirstDiamonds = input.firstGeneratedDiamonds
    .add(input.firstOwnedDiamonds)
    .times(input.firstProduceDiamonds)
    .times(input.globalCrystalMultiplier)
  const produceSecondDiamonds = input.secondGeneratedDiamonds
    .add(input.secondOwnedDiamonds)
    .times(input.secondProduceDiamonds)
    .times(input.globalCrystalMultiplier)
  const produceThirdDiamonds = input.thirdGeneratedDiamonds
    .add(input.thirdOwnedDiamonds)
    .times(input.thirdProduceDiamonds)
    .times(input.globalCrystalMultiplier)
  const produceFourthDiamonds = input.fourthGeneratedDiamonds
    .add(input.fourthOwnedDiamonds)
    .times(input.fourthProduceDiamonds)
    .times(input.globalCrystalMultiplier)
  const produceFifthDiamonds = input.fifthGeneratedDiamonds
    .add(input.fifthOwnedDiamonds)
    .times(input.fifthProduceDiamonds)
    .times(input.globalCrystalMultiplier)

  const fourthGeneratedDiamonds = input.fourthGeneratedDiamonds.add(
    produceFifthDiamonds.times(dt / 0.025)
  )
  const thirdGeneratedDiamonds = input.thirdGeneratedDiamonds.add(
    produceFourthDiamonds.times(dt / 0.025)
  )
  const secondGeneratedDiamonds = input.secondGeneratedDiamonds.add(
    produceThirdDiamonds.times(dt / 0.025)
  )
  const firstGeneratedDiamonds = input.firstGeneratedDiamonds.add(
    produceSecondDiamonds.times(dt / 0.025)
  )
  const produceDiamonds = produceFirstDiamonds

  let prestigeShards = input.prestigeShards
  if (input.transcensionChallenge !== 3 && input.reincarnationChallenge !== 10) {
    prestigeShards = prestigeShards.add(produceDiamonds.times(dt / 0.025))
  }

  // ─── Mythos cascade ────────────────────────────────────────────────
  const produceFifthMythos = input.fifthGeneratedMythos
    .add(input.fifthOwnedMythos)
    .times(input.fifthProduceMythos)
    .times(input.globalMythosMultiplier)
    .times(input.grandmasterMultiplier)
    .times(input.mythosupgrade15)
  const produceFourthMythos = input.fourthGeneratedMythos
    .add(input.fourthOwnedMythos)
    .times(input.fourthProduceMythos)
    .times(input.globalMythosMultiplier)
  const produceThirdMythos = input.thirdGeneratedMythos
    .add(input.thirdOwnedMythos)
    .times(input.thirdProduceMythos)
    .times(input.globalMythosMultiplier)
    .times(input.mythosupgrade14)
  const produceSecondMythos = input.secondGeneratedMythos
    .add(input.secondOwnedMythos)
    .times(input.secondProduceMythos)
    .times(input.globalMythosMultiplier)
  const produceFirstMythos = input.firstGeneratedMythos
    .add(input.firstOwnedMythos)
    .times(input.firstProduceMythos)
    .times(input.globalMythosMultiplier)
    .times(input.mythosupgrade13)
  const fourthGeneratedMythos = input.fourthGeneratedMythos.add(
    produceFifthMythos.times(dt / 0.025)
  )
  const thirdGeneratedMythos = input.thirdGeneratedMythos.add(
    produceFourthMythos.times(dt / 0.025)
  )
  const secondGeneratedMythos = input.secondGeneratedMythos.add(
    produceThirdMythos.times(dt / 0.025)
  )
  const firstGeneratedMythos = input.firstGeneratedMythos.add(
    produceSecondMythos.times(dt / 0.025)
  )

  // produceMythos: recomputed after mutations using post-tick firstGeneratedMythos
  const produceMythos = firstGeneratedMythos
    .add(input.firstOwnedMythos)
    .times(input.firstProduceMythos)
    .times(input.globalMythosMultiplier)
    .times(input.mythosupgrade13)
  const producePerSecondMythos = produceMythos.times(40)

  // ─── Particle cascade ──────────────────────────────────────────────
  let pm = new Decimal('1')
  if (input.upgrade67 > 0.5) {
    pm = pm.times(
      Decimal.pow(
        1.03,
        input.firstOwnedParticles
          + input.secondOwnedParticles
          + input.thirdOwnedParticles
          + input.fourthOwnedParticles
          + input.fifthOwnedParticles
      )
    )
  }
  const produceFifthParticles = input.fifthGeneratedParticles
    .add(input.fifthOwnedParticles)
    .times(input.fifthProduceParticles)
  const produceFourthParticles = input.fourthGeneratedParticles
    .add(input.fourthOwnedParticles)
    .times(input.fourthProduceParticles)
  const produceThirdParticles = input.thirdGeneratedParticles
    .add(input.thirdOwnedParticles)
    .times(input.thirdProduceParticles)
  const produceSecondParticles = input.secondGeneratedParticles
    .add(input.secondOwnedParticles)
    .times(input.secondProduceParticles)
  const produceFirstParticles = input.firstGeneratedParticles
    .add(input.firstOwnedParticles)
    .times(input.firstProduceParticles)
    .times(pm)
  const fourthGeneratedParticles = input.fourthGeneratedParticles.add(
    produceFifthParticles.times(dt / 0.025)
  )
  const thirdGeneratedParticles = input.thirdGeneratedParticles.add(
    produceFourthParticles.times(dt / 0.025)
  )
  const secondGeneratedParticles = input.secondGeneratedParticles.add(
    produceThirdParticles.times(dt / 0.025)
  )
  const firstGeneratedParticles = input.firstGeneratedParticles.add(
    produceSecondParticles.times(dt / 0.025)
  )

  // produceParticles: recomputed after mutations using post-tick firstGeneratedParticles
  const produceParticles = firstGeneratedParticles
    .add(input.firstOwnedParticles)
    .times(input.firstProduceParticles)
    .times(pm)
  const producePerSecondParticles = produceParticles.times(40)

  // ─── Transcend / reincarnation shards ──────────────────────────────
  let transcendShards = input.transcendShards
  if (input.transcensionChallenge !== 3 && input.reincarnationChallenge !== 10) {
    transcendShards = transcendShards.add(produceMythos.times(dt / 0.025))
  }
  const reincarnationShards = input.reincarnationShards.add(produceParticles.times(dt / 0.025))

  // ─── AscendBuildings cascade (note: dt unscaled — legacy uses raw dt, not dt/0.025) ─
  const ascendOwned: readonly number[] = [
    input.ascendBuilding1Owned,
    input.ascendBuilding2Owned,
    input.ascendBuilding3Owned,
    input.ascendBuilding4Owned,
    input.ascendBuilding5Owned
  ]
  const ascendGenerated: Decimal[] = [
    input.ascendBuilding1Generated,
    input.ascendBuilding2Generated,
    input.ascendBuilding3Generated,
    input.ascendBuilding4Generated,
    input.ascendBuilding5Generated
  ]
  const ascendProd: Decimal[] = [
    new Decimal(0),
    new Decimal(0),
    new Decimal(0),
    new Decimal(0),
    new Decimal(0)
  ]
  for (let j = 4; j >= 0; j--) {
    ascendProd[j] = ascendGenerated[j]
      .add(ascendOwned[j])
      .times(0.05)
      .times(input.globalConstantMult)

    if (j !== 0) {
      ascendGenerated[j - 1] = ascendGenerated[j - 1].add(ascendProd[j].times(dt))
    }
  }

  const ascendShards = input.ascendShards.add(ascendProd[0].times(dt))

  // ─── awardAchievementGroup('constant') gate ────────────────────────
  if (input.ascensionCount > 0) {
    events.push({ kind: 'achievement-group-awarded', group: 'constant' })
  }

  // ─── Challenge 1-5 auto-completion ─────────────────────────────────
  let c1Completions = input.c1Completions
  let c2Completions = input.c2Completions
  let c3Completions = input.c3Completions
  let c4Completions = input.c4Completions
  let c5Completions = input.c5Completions

  if (
    input.research71 > 0.5
    && c1Completions < Math.min(input.highestC1, 25 + 5 * input.research66 + 925 * input.research105)
    && coins.gte(
      Decimal.pow(10, 1.25 * input.challengeBaseRequirements[0] * Math.pow(1 + c1Completions, 2))
    )
  ) {
    c1Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 1, newCompletions: c1Completions })
  }
  if (
    input.research72 > 0.5
    && c2Completions < Math.min(input.highestC2, 25 + 5 * input.research67_ + 925 * input.research105)
    && coins.gte(
      Decimal.pow(10, 1.6 * input.challengeBaseRequirements[1] * Math.pow(1 + c2Completions, 2))
    )
  ) {
    c2Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 2, newCompletions: c2Completions })
  }
  if (
    input.research73 > 0.5
    && c3Completions < Math.min(input.highestC3, 25 + 5 * input.research68 + 925 * input.research105)
    && coins.gte(
      Decimal.pow(10, 1.7 * input.challengeBaseRequirements[2] * Math.pow(1 + c3Completions, 2))
    )
  ) {
    c3Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 3, newCompletions: c3Completions })
  }
  if (
    input.research74 > 0.5
    && c4Completions < Math.min(input.highestC4, 25 + 5 * input.research69 + 925 * input.research105)
    && coins.gte(
      Decimal.pow(10, 1.45 * input.challengeBaseRequirements[3] * Math.pow(1 + c4Completions, 2))
    )
  ) {
    c4Completions += 1
    events.push({ kind: 'challenge-auto-completed', challengeIndex: 4, newCompletions: c4Completions })
  }
  if (
    input.research75 > 0.5
    && c5Completions < Math.min(input.highestC5, 25 + 5 * input.research70 + 925 * input.research105)
    && coins.gte(
      Decimal.pow(10, 2 * input.challengeBaseRequirements[4] * Math.pow(1 + c5Completions, 2))
    )
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
    ascendBuildingProduction: {
      first: ascendProd[0],
      second: ascendProd[1],
      third: ascendProd[2],
      fourth: ascendProd[3],
      fifth: ascendProd[4]
    },

    events
  }
}
