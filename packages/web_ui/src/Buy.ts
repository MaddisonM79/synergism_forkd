import Decimal from 'break_infinity.js'
import {
  buyAccelerator as logicBuyAccelerator,
  buyMax as logicBuyMax,
  buyMultiplier as logicBuyMultiplier,
  buyParticleBuilding as logicBuyParticleBuilding,
  buyTesseractBuilding as logicBuyTesseractBuilding,
  calculateTessBuildingsInBudget as logicCalculateTessBuildingsInBudget,
  getAcceleratorBoostCost as logicGetAcceleratorBoostCost,
  type GetAcceleratorBoostCostInput,
  type GetProducerCostInput,
  getProducerCost,
  type ProducerFamilyState,
  type ProducerType,
  type TesseractBuildings as LogicTesseractBuildings
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

export const getReductionValue = () => {
  let reduction = 1
  reduction += getRuneEffects('thrift', 'costDelay')
  reduction += (player.researches[56] + player.researches[57] + player.researches[58] + player.researches[59]
    + player.researches[60]) / 200
  reduction += CalcECC('transcend', player.challengecompletions[4]) / 200
  reduction += getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingCostScale
  return reduction
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

export const buyProducer = (
  pos: FirstToFifth,
  type: keyof typeof buyProducerTypes,
  num: number,
  autobuyer?: boolean
) => {
  const [tag, amounttype] = buyProducerTypes[type]
  const buythisamount = autobuyer ? 500 : player[`${amounttype}buyamount` as const]
  let r = 1
  r += getRuneEffects('thrift', 'costDelay')
  r += (player.researches[56] + player.researches[57] + player.researches[58] + player.researches[59]
    + player.researches[60]) / 200
  r += CalcECC('transcend', player.challengecompletions[4]) / 200
  r += getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingCostScale

  const posCostType = `${pos}Cost${type}` as const
  const posOwnedType = `${pos}Owned${type}` as const

  while (
    player[tag].gte(player[posCostType]) && G.ticker < buythisamount && player[posOwnedType] < Number.MAX_SAFE_INTEGER
  ) {
    player[tag] = player[tag].sub(player[posCostType])
    player[posOwnedType] += 1
    player[posCostType] = player[posCostType].times(Decimal.pow(1.25, num))
    player[posCostType] = player[posCostType].add(1)
    if (player[posOwnedType] >= (1000 * r)) {
      player[posCostType] = player[posCostType].times(player[posOwnedType]).dividedBy(1000).times(1 + num / 2)
    }
    if (player[posOwnedType] >= (5000 * r)) {
      player[posCostType] = player[posCostType].times(player[posOwnedType]).times(10).times(10 + num * 10)
    }
    if (player[posOwnedType] >= (20000 * r)) {
      player[posCostType] = player[posCostType].times(Decimal.pow(player[posOwnedType], 3)).times(100000).times(
        100 + num * 100
      )
    }
    if (player[posOwnedType] >= (250000 * r)) {
      player[posCostType] = player[posCostType].times(Decimal.pow(1.03, player[posOwnedType] - 250000 * r))
    }
    if (player.currentChallenge.transcension === 4 && (type === 'Coin' || type === 'Diamonds')) {
      player[posCostType] = player[posCostType].times(
        Math.pow(100 * player[posOwnedType] + 10000, 1.25 + 1 / 4 * player.challengecompletions[4])
      )
      if (player[posOwnedType] >= 1000 - (10 * player.challengecompletions[4])) {
        player[posCostType] = player[posCostType].times(Decimal.pow(1.25, player[posOwnedType]))
      }
    }
    if (
      player.currentChallenge.reincarnation === 8 && (type === 'Coin' || type === 'Diamonds' || type === 'Mythos')
      && player[posOwnedType] >= (1000 * player.challengecompletions[8] * r)
    ) {
      player[posCostType] = player[posCostType].times(
        Decimal.pow(
          2,
          (player[posOwnedType] - (1000 * player.challengecompletions[8] * r))
            / (1 + (player.challengecompletions[8] / 2))
        )
      )
    }
    G.ticker += 1
  }
  G.ticker = 0
}

export const buyUpgrades = (type: Upgrade, pos: number, state?: boolean) => {
  if (!upgradeRequirements[pos]) {
    return
  }

  const currency = type
  if (player[currency].gte(Decimal.pow(10, G.upgradeCosts[pos])) && player.upgrades[pos] === 0) {
    player[currency] = player[currency].sub(Decimal.pow(10, G.upgradeCosts[pos]))
    player.upgrades[pos] = 1
    upgradeupdate(pos, state)
  }

  if (type === Upgrade.transcend) {
    player.reincarnatenocoinprestigeortranscendupgrades = false
    player.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  if (type === Upgrade.prestige) {
    player.transcendnocoinorprestigeupgrades = false
    player.reincarnatenocoinorprestigeupgrades = false
    player.reincarnatenocoinprestigeortranscendupgrades = false
    player.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  if (type === Upgrade.coin) {
    player.prestigenocoinupgrades = false
    player.transcendnocoinupgrades = false
    player.transcendnocoinorprestigeupgrades = false
    player.reincarnatenocoinupgrades = false
    player.reincarnatenocoinorprestigeupgrades = false
    player.reincarnatenocoinprestigeortranscendupgrades = false
    player.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
}

const calculateCrystalBuy = (i: number) => {
  const u = i - 1
  const exponent = Decimal.log(player.prestigeShards.add(1), 10)
  const exponentCostReduction = getRuneEffects('prism', 'costDivisorLog10')
  const toBuy = Math.floor(
    Math.pow(
      Math.max(
        0,
        2 * (exponent + exponentCostReduction - G.crystalUpgradesCost[u]) / G.crystalUpgradeCostIncrement[u] + 1 / 4
      ),
      1 / 2
    )
      + 1 / 2
  )
  return toBuy
}

export const buyCrystalUpgrades = (i: number, auto = false) => {
  const u = i - 1

  let c = 0
  if (player.upgrades[73] > 0.5 && player.currentChallenge.reincarnation !== 0) {
    c += 10
  }

  const costReduction = getRuneEffects('prism', 'costDivisorLog10')

  const toBuy = calculateCrystalBuy(i)

  if (toBuy + c > player.crystalUpgrades[u]) {
    player.crystalUpgrades[u] = 100 / 100 * (toBuy + c)
    /* Automation no longer spends Crystals. Late game players experience weird 'zeroing' of Crystals
       When they can afford Crystal Upgrades, due to precision issues. It is easier to just
       Not spend crystals before this becomes a significant issue. */
    if (toBuy > 0 && !auto) {
      player.prestigeShards = player.prestigeShards.sub(
        Decimal.pow(
          10,
          G.crystalUpgradesCost[u] - costReduction
            + G.crystalUpgradeCostIncrement[u] * (1 / 2 * Math.pow(toBuy - 1 / 2, 2) - 1 / 8)
        )
      )
      if (!auto) {
        crystalupgradedescriptions(i)
      }
      // This can sometimes just happen... yeah pretty bad!
      player.prestigeShards = player.prestigeShards.max(0)
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
