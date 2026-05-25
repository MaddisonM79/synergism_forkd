import Decimal from 'break_infinity.js'
import {
  buyAccelerator as logicBuyAccelerator,
  buyCrystalUpgrades as logicBuyCrystalUpgrades,
  buyMax as logicBuyMax,
  buyMultiplier as logicBuyMultiplier,
  buyParticleBuilding as logicBuyParticleBuilding,
  buyProducer as logicBuyProducer,
  buyTesseractBuilding as logicBuyTesseractBuilding,
  buyUpgrades as logicBuyUpgrades,
  calculateTessBuildingsInBudget as logicCalculateTessBuildingsInBudget,
  type CrystalUpgradesState,
  getAcceleratorBoostCost as logicGetAcceleratorBoostCost,
  type GetAcceleratorBoostCostInput,
  type GetProducerCostInput,
  getProducerCost,
  getReductionValue as logicGetReductionValue,
  type ProducerFamilyState,
  type ProducerIndex,
  type ProducerType,
  type TesseractBuildings as LogicTesseractBuildings,
  type UpgradesState,
  type UpgradeTier
} from '@synergism/logic'
import { awardAchievementGroup } from './Achievements'
import { CalcECC } from './Challenges'
import { getAntUpgradeEffect } from './Features/Ants/AntUpgrades/lib/upgrade-effects'
import { AntUpgrades } from './Features/Ants/AntUpgrades/structs/structs'
import { reset } from './Reset'
import { getRuneBlessingEffect } from './RuneBlessings'
import { getRuneEffects } from './Runes'
import { player, updateAllMultiplier, updateAllTick } from './Synergism'
import type { FirstToFifth, OneToFive } from './types/Synergism'
import { crystalupgradedescriptions, upgradeRequirements, upgradeupdate } from './Upgrades'
import { smallestInc } from './Utility'
import { Globals as G, Upgrade } from './Variables'

// Cost-divisor `r` aggregator. The logic version of CalcECC is called inside
// logicGetReductionValue, so this shim only assembles the four external
// scalar contributions.
export const getReductionValue = () => {
  return logicGetReductionValue({
    thriftCostDelay: getRuneEffects('thrift', 'costDelay'),
    researchesSum: player.researches[56]
      + player.researches[57]
      + player.researches[58]
      + player.researches[59]
      + player.researches[60],
    challengeCompletions4: player.challengecompletions[4],
    antBuildingCostScale: getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingCostScale
  })
}

// Thin shim over @synergism/logic's pure buyAccelerator. Gathers the required
// computed inputs from player / Globals, applies the returned state slice back
// to the mutable player object, and runs the legacy post-buy side effects
// (display refresh + achievement check) that haven't migrated yet.
export const buyAccelerator = (autobuyer?: boolean) => {
  const { state } = logicBuyAccelerator(
    {
      acceleratorBought: player.acceleratorBought,
      acceleratorCost: player.acceleratorCost,
      coins: player.coins,
      prestigenoaccelerator: player.prestigenoaccelerator,
      transcendnoaccelerator: player.transcendnoaccelerator,
      reincarnatenoaccelerator: player.reincarnatenoaccelerator
    },
    {
      autobuyer: !!autobuyer,
      coinbuyamount: player.coinbuyamount,
      costDivisor: G.costDivisor,
      transcendECC: CalcECC('transcend', player.challengecompletions[4]),
      inTranscensionChallenge4: player.currentChallenge.transcension === 4,
      inReincarnationChallenge8: player.currentChallenge.reincarnation === 8
    }
  )

  player.acceleratorBought = state.acceleratorBought
  player.acceleratorCost = state.acceleratorCost
  player.coins = state.coins
  player.prestigenoaccelerator = state.prestigenoaccelerator
  player.transcendnoaccelerator = state.transcendnoaccelerator
  player.reincarnatenoaccelerator = state.reincarnatenoaccelerator

  updateAllTick()
  awardAchievementGroup('accelerators')
}

// Shim over @synergism/logic's pure buyMultiplier. Mirror of the
// buyAccelerator shim above.
export const buyMultiplier = (autobuyer?: boolean) => {
  const { state } = logicBuyMultiplier(
    {
      multiplierBought: player.multiplierBought,
      multiplierCost: player.multiplierCost,
      coins: player.coins,
      prestigenomultiplier: player.prestigenomultiplier,
      transcendnomultiplier: player.transcendnomultiplier,
      reincarnatenomultiplier: player.reincarnatenomultiplier
    },
    {
      autobuyer: !!autobuyer,
      coinbuyamount: player.coinbuyamount,
      costDivisor: G.costDivisor,
      transcendECC: CalcECC('transcend', player.challengecompletions[4]),
      inTranscensionChallenge4: player.currentChallenge.transcension === 4,
      inReincarnationChallenge8: player.currentChallenge.reincarnation === 8
    }
  )

  player.multiplierBought = state.multiplierBought
  player.multiplierCost = state.multiplierCost
  player.coins = state.coins
  player.prestigenomultiplier = state.prestigenomultiplier
  player.transcendnomultiplier = state.transcendnomultiplier
  player.reincarnatenomultiplier = state.reincarnatenomultiplier

  updateAllMultiplier()
  awardAchievementGroup('multipliers')
}

// Build the GetProducerCostInput from live player + globals state. Used by
// both the local getCost shim and buyMax — anything in this file that needs
// to evaluate the producer-cost formula.
const buildProducerCostInput = (r?: number): GetProducerCostInput => ({
  costDivisor: r ?? getReductionValue(),
  inTranscensionChallenge4: player.currentChallenge.transcension === 4,
  inReincarnationChallenge8: player.currentChallenge.reincarnation === 8,
  inReincarnationChallenge10: player.currentChallenge.reincarnation === 10,
  challengecompletions4: player.challengecompletions[4],
  challengecompletions8: player.challengecompletions[8]
})

// Shim over @synergism/logic's getProducerCost. Same signature as the
// pre-migration getCost so external callers (Synergism.ts autobuyer code)
// don't change.
export const getCost = (
  index: OneToFive,
  type: keyof typeof buyProducerTypes,
  buyingTo: number,
  r?: number
) => {
  return getProducerCost(index, type, buyingTo, buildProducerCostInput(r))
}

// Shim over @synergism/logic's pure buyMax. Gathers the live 10-field slice
// for the chosen producer family + its spend resource via template-literal
// accessors, calls logic, and writes the returned slice back to player.
export const buyMax = (index: OneToFive, type: ProducerType) => {
  const tag = buyProducerTypes[type][0]
  const costInput = buildProducerCostInput()

  const familyState: ProducerFamilyState = {
    resource: player[tag],
    firstOwned: player[`firstOwned${type}` as const],
    firstCost: player[`firstCost${type}` as const],
    secondOwned: player[`secondOwned${type}` as const],
    secondCost: player[`secondCost${type}` as const],
    thirdOwned: player[`thirdOwned${type}` as const],
    thirdCost: player[`thirdCost${type}` as const],
    fourthOwned: player[`fourthOwned${type}` as const],
    fourthCost: player[`fourthCost${type}` as const],
    fifthOwned: player[`fifthOwned${type}` as const],
    fifthCost: player[`fifthCost${type}` as const]
  }

  const { state } = logicBuyMax(familyState, { index, type, costInput })

  player[tag] = state.resource
  player[`firstOwned${type}` as const] = state.firstOwned
  player[`firstCost${type}` as const] = state.firstCost
  player[`secondOwned${type}` as const] = state.secondOwned
  player[`secondCost${type}` as const] = state.secondCost
  player[`thirdOwned${type}` as const] = state.thirdOwned
  player[`thirdCost${type}` as const] = state.thirdCost
  player[`fourthOwned${type}` as const] = state.fourthOwned
  player[`fourthCost${type}` as const] = state.fourthCost
  player[`fifthOwned${type}` as const] = state.fifthOwned
  player[`fifthCost${type}` as const] = state.fifthCost
}

const buyProducerTypes = {
  Diamonds: ['prestigePoints', 'crystal'],
  Mythos: ['transcendPoints', 'mythos'],
  Particles: ['reincarnationPoints', 'particle'],
  Coin: ['coins', 'coin']
} as const

// Maps FirstToFifth ordinal names to the numeric ProducerIndex the logic
// function expects. The `num` value the OLD signature took is derivable from
// (index, type) and is computed inside the logic function, so it's no longer
// a shim parameter.
const POSITION_TO_INDEX: Record<FirstToFifth, ProducerIndex> = {
  first: 1,
  second: 2,
  third: 3,
  fourth: 4,
  fifth: 5
}

// Shim over @synergism/logic's pure buyProducer. Gathers the 10-field family
// slice + the spend resource, pre-computes `r` from getReductionValue() (the
// rune/research/CalcECC/ant-upgrade aggregate), and writes the result back to
// player. Existing callers stop passing `num` — logic derives it.
export const buyProducer = (
  pos: FirstToFifth,
  type: ProducerType,
  autobuyer?: boolean
) => {
  const tag = buyProducerTypes[type][0]
  const amounttype = buyProducerTypes[type][1]
  const buyamount = player[`${amounttype}buyamount` as const]

  const familyState: ProducerFamilyState = {
    resource: player[tag],
    firstOwned: player[`firstOwned${type}` as const],
    firstCost: player[`firstCost${type}` as const],
    secondOwned: player[`secondOwned${type}` as const],
    secondCost: player[`secondCost${type}` as const],
    thirdOwned: player[`thirdOwned${type}` as const],
    thirdCost: player[`thirdCost${type}` as const],
    fourthOwned: player[`fourthOwned${type}` as const],
    fourthCost: player[`fourthCost${type}` as const],
    fifthOwned: player[`fifthOwned${type}` as const],
    fifthCost: player[`fifthCost${type}` as const]
  }

  const { state } = logicBuyProducer(familyState, {
    index: POSITION_TO_INDEX[pos],
    type,
    autobuyer: !!autobuyer,
    buyamount,
    r: getReductionValue(),
    inTranscensionChallenge4: player.currentChallenge.transcension === 4,
    inReincarnationChallenge8: player.currentChallenge.reincarnation === 8,
    challengecompletions4: player.challengecompletions[4],
    challengecompletions8: player.challengecompletions[8]
  })

  player[tag] = state.resource
  player[`firstOwned${type}` as const] = state.firstOwned
  player[`firstCost${type}` as const] = state.firstCost
  player[`secondOwned${type}` as const] = state.secondOwned
  player[`secondCost${type}` as const] = state.secondCost
  player[`thirdOwned${type}` as const] = state.thirdOwned
  player[`thirdCost${type}` as const] = state.thirdCost
  player[`fourthOwned${type}` as const] = state.fourthOwned
  player[`fourthCost${type}` as const] = state.fourthCost
  player[`fifthOwned${type}` as const] = state.fifthOwned
  player[`fifthCost${type}` as const] = state.fifthCost
}

// Maps the Upgrade enum (whose values are resource field names) to the tier
// label the logic function dispatches on. Implemented as a function — not a
// top-level keyed map — to avoid a temporal-dead-zone access of `Upgrade` at
// module-load time (Buy.ts and Variables.ts participate in a circular
// dependency cycle through Calculate.ts / Tabs.ts).
const upgradeToTier = (type: Upgrade): UpgradeTier => {
  if (type === Upgrade.coin) return 'coin'
  if (type === Upgrade.prestige) return 'prestige'
  if (type === Upgrade.transcend) return 'transcend'
  return 'reincarnation'
}

// Shim over @synergism/logic's pure buyUpgrades. Gathers the four reset-tier
// resources + the upgrades bitmap + the seven no-upgrades flags, then writes
// the result back. On a successful purchase the upgrade-purchased event
// triggers the leftover UI side effect (upgradeupdate).
export const buyUpgrades = (type: Upgrade, pos: number, state?: boolean) => {
  const slice: UpgradesState = {
    coins: player.coins,
    prestigePoints: player.prestigePoints,
    transcendPoints: player.transcendPoints,
    reincarnationPoints: player.reincarnationPoints,
    upgrades: player.upgrades,
    prestigenocoinupgrades: player.prestigenocoinupgrades,
    transcendnocoinupgrades: player.transcendnocoinupgrades,
    transcendnocoinorprestigeupgrades: player.transcendnocoinorprestigeupgrades,
    reincarnatenocoinupgrades: player.reincarnatenocoinupgrades,
    reincarnatenocoinorprestigeupgrades: player.reincarnatenocoinorprestigeupgrades,
    reincarnatenocoinprestigeortranscendupgrades: player.reincarnatenocoinprestigeortranscendupgrades,
    reincarnatenocoinprestigetranscendorgeneratorupgrades:
      player.reincarnatenocoinprestigetranscendorgeneratorupgrades
  }

  const { state: next, events } = logicBuyUpgrades(slice, {
    tier: upgradeToTier(type),
    pos,
    costExponent: G.upgradeCosts[pos],
    requirementExists: upgradeRequirements[pos] !== undefined
  })

  player.coins = next.coins
  player.prestigePoints = next.prestigePoints
  player.transcendPoints = next.transcendPoints
  player.reincarnationPoints = next.reincarnationPoints
  player.upgrades = next.upgrades
  player.prestigenocoinupgrades = next.prestigenocoinupgrades
  player.transcendnocoinupgrades = next.transcendnocoinupgrades
  player.transcendnocoinorprestigeupgrades = next.transcendnocoinorprestigeupgrades
  player.reincarnatenocoinupgrades = next.reincarnatenocoinupgrades
  player.reincarnatenocoinorprestigeupgrades = next.reincarnatenocoinorprestigeupgrades
  player.reincarnatenocoinprestigeortranscendupgrades = next.reincarnatenocoinprestigeortranscendupgrades
  player.reincarnatenocoinprestigetranscendorgeneratorupgrades =
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades

  for (const event of events) {
    if (event.kind === 'upgrade-purchased') {
      upgradeupdate(event.pos, state)
    }
  }
}

// Shim over @synergism/logic's pure buyCrystalUpgrades. Looks up the per-index
// cost constants from G, computes the prism-rune reduction and the
// reincarnation-challenge flag, calls logic, applies state. On a successful
// purchase in the manual (non-auto) path, fires the leftover UI side effect
// `crystalupgradedescriptions(i)`.
export const buyCrystalUpgrades = (i: number, auto = false) => {
  const u = i - 1
  const slice: CrystalUpgradesState = {
    prestigeShards: player.prestigeShards,
    crystalUpgrades: player.crystalUpgrades
  }

  const { state: next, events } = logicBuyCrystalUpgrades(slice, {
    i,
    auto,
    prismCostDivisorLog10: getRuneEffects('prism', 'costDivisorLog10'),
    crystalUpgradesCost: G.crystalUpgradesCost[u],
    crystalUpgradeCostIncrement: G.crystalUpgradeCostIncrement[u],
    upgrade73: player.upgrades[73],
    inAnyReincarnationChallenge: player.currentChallenge.reincarnation !== 0
  })

  player.prestigeShards = next.prestigeShards
  player.crystalUpgrades = next.crystalUpgrades

  if (!auto) {
    for (const event of events) {
      if (event.kind === 'crystal-upgrade-purchased') {
        crystalupgradedescriptions(event.i)
      }
    }
  }
}

export const boostAccelerator = (automated?: boolean) => {
  let buyamount = 1
  if (player.upgrades[46] === 1) {
    buyamount = automated ? 9999 : player.coinbuyamount
  }

  // Cost-delay multiplier is stable for the duration of this buy loop —
  // capture once and feed every getAcceleratorBoostCost call through it.
  const accelBoostCostDelay = getRuneBlessingEffect('thrift').accelBoostCostDelay
  const costInput: GetAcceleratorBoostCostInput = { accelBoostCostDelay }

  if (player.upgrades[46] < 1) {
    while (player.prestigePoints.gte(player.acceleratorBoostCost) && G.ticker < buyamount) {
      if (player.prestigePoints.gte(player.acceleratorBoostCost)) {
        player.acceleratorBoostBought += 1
        player.acceleratorBoostCost = player.acceleratorBoostCost.times(1e10).times(
          Decimal.pow(10, player.acceleratorBoostBought)
        )
        if (player.acceleratorBoostBought > (1000 * accelBoostCostDelay)) {
          player.acceleratorBoostCost = player.acceleratorBoostCost.times(
            Decimal.pow(
              10,
              Math.pow(player.acceleratorBoostBought - (1000 * accelBoostCostDelay), 2)
                / accelBoostCostDelay
            )
          )
        }
        player.transcendnoaccelerator = false
        player.reincarnatenoaccelerator = false
        if (player.upgrades[46] < 0.5) {
          for (let j = 21; j < 41; j++) {
            player.upgrades[j] = 0
          }
          reset('prestige')
          player.prestigePoints = new Decimal(0)
        }
      }
    }
  } else {
    const buyStart = player.acceleratorBoostBought
    const buymax = Math.pow(10, 15)
    // If at least buymax, we will use a different formulae
    if (buyStart >= buymax) {
      const diminishingExponent = 1 / 8

      const log10Resource = Decimal.log10(player.prestigePoints)
      const log10QuadrillionCost = Decimal.log10(logicGetAcceleratorBoostCost(buymax, costInput))

      let hi = Math.floor(buymax * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent)))
      let lo = buymax
      while (hi - lo > 0.5) {
        const mid = Math.floor(lo + (hi - lo) / 2)
        if (mid === lo || mid === hi) {
          break
        }
        if (!player.prestigePoints.gte(logicGetAcceleratorBoostCost(mid, costInput))) {
          hi = mid
        } else {
          lo = mid
        }
      }
      const buyable = lo
      const thisCost = logicGetAcceleratorBoostCost(buyable, costInput)

      player.acceleratorBoostBought = buyable
      player.acceleratorBoostCost = thisCost
      return
    }

    // Start buying at the current amount bought + 1
    const buydefault = buyStart + smallestInc(buyStart)
    let buyInc = 1

    let cost = logicGetAcceleratorBoostCost(buyStart + buyInc, costInput)
    while (player.prestigePoints.gte(cost)) {
      buyInc *= 4
      cost = logicGetAcceleratorBoostCost(buyStart + buyInc, costInput)
    }
    let stepdown = Math.floor(buyInc / 8)
    while (stepdown >= smallestInc(buyInc)) {
      // if step down would push it below out of expense range then divide step down by 2
      if (logicGetAcceleratorBoostCost(buyStart + buyInc - stepdown, costInput).lte(player.prestigePoints)) {
        stepdown = Math.floor(stepdown / 2)
      } else {
        buyInc = buyInc - Math.max(smallestInc(buyInc), stepdown)
      }
    }
    // go down by 7 steps below the last one able to be bought and spend the cost of 25 up to the one that you started with and stop if coin goes below requirement
    let buyFrom = Math.max(buyStart + buyInc - 6 - smallestInc(buyInc), buydefault)
    let thisCost = logicGetAcceleratorBoostCost(player.acceleratorBoostBought, costInput)
    while (
      buyFrom <= buyStart + buyInc
      && player.prestigePoints.gte(logicGetAcceleratorBoostCost(buyFrom, costInput))
    ) {
      player.prestigePoints = player.prestigePoints.sub(thisCost)
      if (buyFrom >= buymax) {
        buyFrom = buymax
      }
      player.acceleratorBoostBought = buyFrom
      buyFrom = buyFrom + smallestInc(buyFrom)
      thisCost = logicGetAcceleratorBoostCost(buyFrom, costInput)
      player.acceleratorBoostCost = thisCost

      player.transcendnoaccelerator = false
      player.reincarnatenoaccelerator = false
      if (buyFrom >= buymax) {
        return
      }
    }
  }

  G.ticker = 0
  awardAchievementGroup('acceleratorBoosts')
}

// Shim over @synergism/logic's pure buyParticleBuilding. Same shape as the
// buyAccelerator / buyMultiplier shims above: gather inputs, call logic, apply
// returned state back to player. The logic function operates on all five
// positions in one slice — only the position selected by `index` actually
// changes — so we read/write the full slice each call.
export const buyParticleBuilding = (
  index: OneToFive,
  autobuyer = false
) => {
  const { state } = logicBuyParticleBuilding(
    {
      reincarnationPoints: player.reincarnationPoints,
      firstOwnedParticles: player.firstOwnedParticles,
      firstCostParticles: player.firstCostParticles,
      secondOwnedParticles: player.secondOwnedParticles,
      secondCostParticles: player.secondCostParticles,
      thirdOwnedParticles: player.thirdOwnedParticles,
      thirdCostParticles: player.thirdCostParticles,
      fourthOwnedParticles: player.fourthOwnedParticles,
      fourthCostParticles: player.fourthCostParticles,
      fifthOwnedParticles: player.fifthOwnedParticles,
      fifthCostParticles: player.fifthCostParticles
    },
    {
      index,
      autobuyer,
      particlebuyamount: player.particlebuyamount,
      inAscensionChallenge15: player.currentChallenge.ascension === 15
    }
  )

  player.reincarnationPoints = state.reincarnationPoints
  player.firstOwnedParticles = state.firstOwnedParticles
  player.firstCostParticles = state.firstCostParticles
  player.secondOwnedParticles = state.secondOwnedParticles
  player.secondCostParticles = state.secondCostParticles
  player.thirdOwnedParticles = state.thirdOwnedParticles
  player.thirdCostParticles = state.thirdCostParticles
  player.fourthOwnedParticles = state.fourthOwnedParticles
  player.fourthCostParticles = state.fourthCostParticles
  player.fifthOwnedParticles = state.fifthOwnedParticles
  player.fifthCostParticles = state.fifthCostParticles
}

// Re-export tesseract-building types and helpers from @synergism/logic so
// existing call sites (Synergism.ts, Reset.ts) keep importing them from
// './Buy' without disruption. The actual implementation lives in
// packages/logic/src/mechanics/tesseractBuildings.ts.
export type TesseractBuildings = LogicTesseractBuildings
export const calculateTessBuildingsInBudget = logicCalculateTessBuildingsInBudget

// Shim over @synergism/logic's pure buyTesseractBuilding. Gathers the live
// state slice for all five ascendBuildings + the wowTesseracts numeric value,
// calls logic, writes the resulting owned/cost back to player, and routes the
// tesseracts delta through the WowTesseracts wrapper's mutating .sub() so the
// wrapper instance stays the same reference.
export const buyTesseractBuilding = (index: OneToFive, amount: number = player.tesseractbuyamount) => {
  const wowBefore = Number(player.wowTesseracts)
  const { state } = logicBuyTesseractBuilding(
    {
      wowTesseracts: wowBefore,
      ascendBuilding1: { owned: player.ascendBuilding1.owned, cost: player.ascendBuilding1.cost },
      ascendBuilding2: { owned: player.ascendBuilding2.owned, cost: player.ascendBuilding2.cost },
      ascendBuilding3: { owned: player.ascendBuilding3.owned, cost: player.ascendBuilding3.cost },
      ascendBuilding4: { owned: player.ascendBuilding4.owned, cost: player.ascendBuilding4.cost },
      ascendBuilding5: { owned: player.ascendBuilding5.owned, cost: player.ascendBuilding5.cost }
    },
    { index, amount }
  )

  player.ascendBuilding1.owned = state.ascendBuilding1.owned
  player.ascendBuilding1.cost = state.ascendBuilding1.cost
  player.ascendBuilding2.owned = state.ascendBuilding2.owned
  player.ascendBuilding2.cost = state.ascendBuilding2.cost
  player.ascendBuilding3.owned = state.ascendBuilding3.owned
  player.ascendBuilding3.cost = state.ascendBuilding3.cost
  player.ascendBuilding4.owned = state.ascendBuilding4.owned
  player.ascendBuilding4.cost = state.ascendBuilding4.cost
  player.ascendBuilding5.owned = state.ascendBuilding5.owned
  player.ascendBuilding5.cost = state.ascendBuilding5.cost
  player.wowTesseracts.sub(wowBefore - state.wowTesseracts)
}
