import {
  type RangeLevelAndCost,
  researchBaseCosts as logicResearchBaseCosts,
  researchLevelCostRanges as logicResearchLevelCostRanges,
  researchMaxLevels as logicResearchMaxLevels
} from '@synergism/logic'
import Decimal, { type DecimalSource } from 'break_infinity.js'
import i18next from 'i18next'
import { getAchievementReward } from './Achievements'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { getResetResearches } from './Reset'
import { runes } from './Runes'
import { calculateSingularityDebuff } from './singularity'
import { format, player } from './Synergism'
import { revealStuff, updateChallengeDisplay } from './UpdateHTML'
import { sortDecimalWithIndices, updateClassList } from './Utility'

interface IResearchData {
  baseCost: Decimal
  maxLevel: number
  buyToLevel: (budget: Decimal, baseCost: Decimal, currLevel: number, maxLevel: number) => number
  costForLevels: (baseCost: Decimal, currLevel: number, buyTo: number) => Decimal
  unlocked: () => boolean
}

type RangeCondition = {
  range: [number, number]
  condition: () => boolean
}

const researchUnlockRanges: RangeCondition[] = [
  { range: [0, 0], condition: () => true }, // Not sure if needed!
  { range: [1, 76], condition: () => player.unlocks.reincarnate },
  { range: [77, 77], condition: () => runes.thrift.isUnlocked() },
  { range: [78, 78], condition: () => player.unlocks.reincarnate },
  { range: [79, 79], condition: () => runes.prism.isUnlocked() },
  { range: [80, 80], condition: () => runes.duplication.isUnlocked() },
  { range: [81, 100], condition: () => player.unlocks.anthill },
  { range: [101, 118], condition: () => player.unlocks.talismans },
  { range: [119, 123], condition: () => player.unlocks.ascensions },
  { range: [124, 125], condition: () => Boolean(getAchievementReward('antSacrificeUnlock')) },
  { range: [126, 140], condition: () => player.ascensionCount > 0 },
  { range: [141, 155], condition: () => player.highestchallengecompletions[11] > 0 },
  { range: [156, 170], condition: () => player.highestchallengecompletions[12] > 0 },
  { range: [171, 185], condition: () => player.highestchallengecompletions[13] > 0 },
  { range: [186, 200], condition: () => player.highestchallengecompletions[14] > 0 }
]

const createResearchDataMap = (
  rangeLC: RangeLevelAndCost[],
  rangeU: RangeCondition[],
  costs: DecimalSource[],
  maxLevels: DecimalSource[]
): Record<number, IResearchData> => {
  const dataMap: Record<number, IResearchData> = {}

  const unlockLookup: Record<number, () => boolean> = {}
  for (const { range, condition } of rangeU) {
    const [start, end] = range
    for (let i = start; i <= end; i++) {
      unlockLookup[i] = condition
    }
  }

  const levelCostLookup: Record<number, { level: typeof rangeLC[0]['level']; cost: typeof rangeLC[0]['cost'] }> = {}
  for (const { range, level, cost } of rangeLC) {
    const [start, end] = range
    for (let i = start; i <= end; i++) {
      levelCostLookup[i] = { level, cost }
    }
  }

  for (let i = 0; i < costs.length && i < maxLevels.length; i++) {
    const levelCostFunctions = levelCostLookup[i]
    const unlockFunction = unlockLookup[i]

    if (levelCostFunctions && unlockFunction) {
      dataMap[i] = {
        baseCost: new Decimal(costs[i]),
        maxLevel: Number(maxLevels[i]),
        buyToLevel: levelCostFunctions.level,
        costForLevels: levelCostFunctions.cost,
        unlocked: unlockFunction
      }
    }
  }

  return dataMap
}

export const researchData = createResearchDataMap(
  logicResearchLevelCostRanges,
  researchUnlockRanges,
  logicResearchBaseCosts,
  logicResearchMaxLevels
)

export const isResearchUnlocked = (index: number): boolean => {
  const unlockFunction = researchData[index].unlocked
  return unlockFunction ? unlockFunction() : false
}

const getBuyableResearchLevel = (index: number): number => {
  const buyToLevelFunc = researchData[index].buyToLevel
  const baseCost = researchData[index].baseCost
  const currLevel = player.researches[index]
  const maxLevel = researchData[index].maxLevel
  const budget = player.obtainium

  const researchCostMulti = calculateSingularityDebuff('Researches')

  return buyToLevelFunc(budget, baseCost.times(researchCostMulti), currLevel, maxLevel)
}

const getCostForResearchLevels = (index: number, buyTo: number): Decimal => {
  const costForLevelsFunc = researchData[index].costForLevels
  const baseCost = researchData[index].baseCost
  const currLevel = player.researches[index]

  const researchCostMulti = calculateSingularityDebuff('Researches')

  return costForLevelsFunc(baseCost.times(researchCostMulti), currLevel, buyTo)
}

export const researchOrderByCost: number[] = sortDecimalWithIndices(logicResearchBaseCosts)

// For mode 'manual'
export const updateResearchAuto = (index: number) => {
  DOMCacheGetOrSet(`res${player.autoResearch || 1}`).classList.remove('researchRoomba')
  DOMCacheGetOrSet(`res${index}`).classList.add('researchRoomba')
  player.autoResearch = index

  // Research is maxed
  if (isResearchMaxed(index)) {
    updateClassList(`res${player.autoResearch}`, ['researchMaxed'], ['researchPurchased'])
  } else if (player.researches[index] >= 1) {
    // Research purchased above level 0 but not maxed
    updateClassList(`res${player.autoResearch}`, ['researchPurchased'], ['researchMaxed'])
  } else {
    // Research has not been purchased yet
    updateClassList(`res${player.autoResearch}`, [], ['researchPurchased', 'researchMaxed'])
  }
}

export const updateResearchRoomba = () => {
  if (isResearchMaxed(player.autoResearch) || !isResearchUnlocked(player.autoResearch)) {
    DOMCacheGetOrSet(`res${player.autoResearch || 1}`).classList.remove('researchRoomba')
    player.roombaResearchIndex = Math.min(researchOrderByCost.length - 1, player.roombaResearchIndex + 1)
    player.autoResearch = researchOrderByCost[player.roombaResearchIndex]
  }

  // Loops us back to the start
  if (player.roombaResearchIndex === 200 && !isResearchUnlocked(200)) {
    player.roombaResearchIndex = 0
    player.autoResearch = researchOrderByCost[player.roombaResearchIndex]
  }
  DOMCacheGetOrSet(`res${player.autoResearch || 1}`).classList.add('researchRoomba')
}

/**
 * Should the user have access to roomba autoResearch
 * @returns boolean
 */
export const roombaResearchEnabled = (): boolean => {
  return (player.cubeUpgrades[9] === 1 || player.highestSingularityCount > 10)
}
/**
 * Attempts to buy the research of the index selected. This is hopefully an improvement over buyResearch. Fuck
 * @param index
 * @param auto
 * @returns
 */
export const buyResearch = (index: number, auto: boolean, hover: boolean) => {
  if (isResearchMaxed(index) || !isResearchUnlocked(index)) {
    return
  }

  // Get our costs, and determine if anything is purchasable.
  const buyAmount = (player.researchBuyMaxToggle || auto || hover) ? Number.POSITIVE_INFINITY : 1
  const maxLevel = researchData[index].maxLevel

  let levelToBuy = getBuyableResearchLevel(index)
  levelToBuy = Math.min(maxLevel, levelToBuy, player.researches[index] + buyAmount)

  const researchCost = getCostForResearchLevels(index, levelToBuy)

  // If the cost is 0, then we are only able to buy up to currentLevel, which is true
  // when the cost to the next level is too prohibitive (getCost is cumulative)
  const canBuy = researchCost.gt(0)

  if (canBuy) {
    player.researches[index] = levelToBuy
    player.obtainium = player.obtainium.sub(researchCost)
    // Quick check after upgrading for max. This is to update any automation regardless of auto state
    if (isResearchMaxed(index)) {
      DOMCacheGetOrSet(`res${player.autoResearch || 1}`).classList.remove('researchRoomba')
    }

    researchDescriptions(index, auto)

    if (index >= 47 && index <= 50) {
      player.unlocks.rrow1 ||= player.researches[47] > 0
      player.unlocks.rrow2 ||= player.researches[48] > 0
      player.unlocks.rrow3 ||= player.researches[49] > 0
      player.unlocks.rrow4 ||= player.researches[50] > 0
      revealStuff()
    }
    if ((index >= 66 && index <= 70) || index === 105) {
      updateChallengeDisplay()
    }
  }

  return
}

export const isResearchMaxed = (index: number) => player.researches[index] >= researchData[index].maxLevel

export const researchDescriptions = (index: number, auto = false) => {
  const buyAmount = (player.researchBuyMaxToggle || auto) ? Number.POSITIVE_INFINITY : 1

  const y = i18next.t(`researches.descriptions.${index}`)
  const p = `res${index}`

  let levelToBuy = getBuyableResearchLevel(index)
  levelToBuy = Math.min(researchData[index].maxLevel, levelToBuy, player.researches[index] + buyAmount)

  let obtainiumCost = new Decimal(0)

  // If levelToBuy is = current level, either we've already maxxed the upgrade
  // OR we cannot afford any levels. Check which one.
  if (levelToBuy === player.researches[index]) {
    // If max level, we don't actually need to change anything
    // If not max level, we need to show the cost of the next level
    if (!isResearchMaxed(index)) {
      levelToBuy += 1
      obtainiumCost = getCostForResearchLevels(index, levelToBuy)
    }
  } else {
    obtainiumCost = getCostForResearchLevels(index, levelToBuy)
  }

  let z = i18next.t('researches.cost', {
    x: format(obtainiumCost, 0, false),
    y: format(levelToBuy - player.researches[index], 0, true)
  })

  if (isResearchMaxed(index)) {
    DOMCacheGetOrSet('researchcost').style.color = 'Gold'
    DOMCacheGetOrSet('researchinfo3').style.color = 'plum'
    updateClassList(p, ['researchMaxed'], ['researchAvailable', 'researchPurchased', 'researchPurchasedAvailable'])
    z += i18next.t('researches.maxed')
  } else {
    DOMCacheGetOrSet('researchcost').style.color = 'limegreen'
    DOMCacheGetOrSet('researchinfo3').style.color = 'white'
    if (player.researches[index] > 0) {
      updateClassList(p, ['researchPurchased', 'researchPurchasedAvailable'], [
        'researchAvailable',
        'researchMaxed'
      ])
    } else {
      updateClassList(p, ['researchAvailable'], ['researchPurchased', 'researchMaxed'])
    }
  }

  if (player.obtainium.lt(obtainiumCost) && !isResearchMaxed(index)) {
    DOMCacheGetOrSet('researchcost').style.color = 'var(--crimson-text-color)'
    updateClassList(p, [], ['researchMaxed', 'researchAvailable', 'researchPurchasedAvailable'])
  }

  DOMCacheGetOrSet('researchinfo2').innerHTML = y
  DOMCacheGetOrSet('researchcost').textContent = z
  DOMCacheGetOrSet('researchinfo3').textContent = i18next.t('researches.level', {
    x: player.researches[index],
    y: researchData[index].maxLevel
  })
  const resetInfo = DOMCacheGetOrSet('researchinfo4')

  if (getResetResearches().includes(index)) {
    resetInfo.textContent = i18next.t('researches.resets')
    resetInfo.classList.remove('crimsonText')
  } else {
    resetInfo.textContent = i18next.t('researches.doesNotReset')
    resetInfo.classList.add('crimsonText')
  }
}

// This should only happen in rare cases, when an update changes max levels
// We still need to handle this on each load, since very old savefiles likely have
// several overcaps
export const refundOvercapResearches = () => {
  for (let i = 1; i <= player.researches.length - 1; i++) {
    if (player.researches[i] > researchData[i].maxLevel) {
      const overcapLevel = player.researches[i]
      player.researches[i] = researchData[i].maxLevel

      // This works because this function computes the cost to get from current level
      // (which is maxLevel at this point) to the overcapLevel, and it's a cumulative function.
      const obtainiumSpentAboveCap = getCostForResearchLevels(i, overcapLevel)
      player.obtainium = player.obtainium.add(obtainiumSpentAboveCap)
    }
  }
}

export const updateResearchBG = (index: number) => {
  const id = `res${index}`
  if (player.researches[index] > 0 && !isResearchMaxed(index)) {
    updateClassList(id, ['researchPurchased'], ['researchMaxed'])
  } else if (player.researches[index] > 0 && isResearchMaxed(index)) {
    updateClassList(id, ['researchMaxed'], ['researchPurchased'])
  } else {
    updateClassList(id, [], ['researchPurchased', 'researchMaxed'])
  }
}
