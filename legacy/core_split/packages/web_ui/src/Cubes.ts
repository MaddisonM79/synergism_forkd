import type Decimal from 'break_infinity.js'
import {
  calculateAcceleratorCubeBlessing as logicCalcAcceleratorCube,
  calculateAntELOCubeBlessing as logicCalcAntELOCube,
  calculateAntSacrificeCubeBlessing as logicCalcAntSacrificeCube,
  calculateAntSpeedCubeBlessing as logicCalcAntSpeedCube,
  calculateGlobalSpeedCubeBlessing as logicCalcGlobalSpeedCube,
  calculateMultiplierCubeBlessing as logicCalcMultiplierCube,
  calculateObtainiumCubeBlessing as logicCalcObtainiumCube,
  calculateOfferingCubeBlessing as logicCalcOfferingCube,
  calculateRuneEffectivenessCubeBlessing as logicCalcRuneEffectivenessCube,
  calculateSalvageCubeBlessing as logicCalcSalvageCube,
  getCubeCost as logicGetCubeCost,
  type GetCubeCostResult,
  getCubeMax as logicGetCubeMax,
  getCubeUpgradeBaseCost
} from '@synergism/logic'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { researchData, updateResearchBG } from './Research'
import { calculateSingularityDebuff, getGQUpgradeEffect } from './singularity'
import { format, player } from './Synergism'
import {
  calculateAcceleratorTesseractBlessing,
  calculateAntELOTesseractBlessing,
  calculateAntSacrificeTesseractBlessing,
  calculateAntSpeedTesseractBlessing,
  calculateGlobalSpeedTesseractBlessing,
  calculateMultiplierTesseractBlessing,
  calculateObtainiumTesseractBlessing,
  calculateOfferingTesseractBlessing,
  calculateRuneEffectivenessTesseractBlessing,
  calculateSalvageTesseractBlessing
} from './Tesseracts'
import { revealStuff } from './UpdateHTML'
import { upgradeupdate } from './Upgrades'

export type IMultiBuy = GetCubeCostResult

// dprint-ignore
const cubeAutomationIndices = [
  4, 5, 6, 7, 8, 9, 10, // row 1
  20,                   // row 2
  26, 27,               // row 3
  48, 49                // row 5
]

// dprint-ignore
const researchAutomationIndices = [
  41, 42, 43, 44, 45, 46, 47, 48, 49, 50, // row 2
  61, 71, 72, 73, 74, 75, // row 3
  124,                    // row 5
  130, 135, 145, 150,     // row 6
  175,                    // row 7
  190                     // row 8
]

const getCubeMax = (i: number) =>
  logicGetCubeMax({
    cubeUpgradeIndex: i,
    cubeUpgrade57: player.cubeUpgrades[57]
  })

const getCubeCost = (i: number, buyMax: boolean): IMultiBuy =>
  logicGetCubeCost({
    cubeUpgradeIndex: i,
    buyMax,
    currentLevel: player.cubeUpgrades[i]!,
    maxLevel: getCubeMax(i),
    wowCubes: Number(player.wowCubes),
    singularityDebuff: i <= 50 ? calculateSingularityDebuff('Cube Upgrades') : 1
  })

export const cubeUpgradeDesc = (i: number, buyMax = player.cubeUpgradesBuyMaxToggle) => {
  const metaData = getCubeCost(i, buyMax)
  const a = DOMCacheGetOrSet('cubeUpgradeName')
  const b = DOMCacheGetOrSet('cubeUpgradeDescription')
  const c = DOMCacheGetOrSet('cubeUpgradeCost')
  const d = DOMCacheGetOrSet('cubeUpgradeLevel')
  const maxLevel = getCubeMax(i)

  a.textContent = i18next.t(`cubes.upgradeNames.${i}`)
  b.innerHTML = i18next.t(`cubes.upgradeDescriptions.${i}`)
  c.textContent = i18next.t('cubes.cubeMetadata.cost', {
    value1: format(metaData.cost, 0, true),
    value2: format(metaData.levelCanBuy - player.cubeUpgrades[i]!, 0, true)
  })
  c.style.color = 'var(--green-text-color)'
  d.textContent = i18next.t('cubes.cubeMetadata.level', {
    value1: format(player.cubeUpgrades[i], 0, true),
    value2: format(maxLevel, 0, true)
  })
  d.style.color = 'white'

  // This conditional is true only in the case where you can buy zero levels.
  if (Number(player.wowCubes) < metaData.cost) {
    c.style.color = 'var(--crimson-text-color)'
  }
  if (player.cubeUpgrades[i] === maxLevel) {
    c.style.color = 'gold'
    c.textContent = i18next.t('cubes.cubeMetadata.maxLevel')
    d.style.color = 'plum'
  }
}

export const updateCubeUpgradeBG = (i: number) => {
  const a = DOMCacheGetOrSet(`cubeUpg${i}`)
  const maxCubeLevel = getCubeMax(i)
  const cubeUpgrade = player.cubeUpgrades[i]!
  if (cubeUpgrade > maxCubeLevel) {
    player.wowCubes.add((cubeUpgrade - maxCubeLevel) * getCubeUpgradeBaseCost(i))
    player.cubeUpgrades[i] = maxCubeLevel
  }

  a.classList.remove('green-background', 'purple-background')

  if (cubeUpgrade > 0 && cubeUpgrade < maxCubeLevel) {
    a.classList.add('purple-background')
  }
  if (player.cubeUpgrades[i] === maxCubeLevel) {
    a.classList.add('green-background')
  }
}

export const awardAutosCookieUpgrade = () => {
  for (const i of cubeAutomationIndices) {
    const maxLevel = getCubeMax(i)
    player.cubeUpgrades[i] = maxLevel
    updateCubeUpgradeBG(i)
  }

  for (const i of researchAutomationIndices) {
    player.researches[i] = researchData[i].maxLevel
    updateResearchBG(i)
  }
}

export const buyCubeUpgrades = (i: number, buyMax = player.cubeUpgradesBuyMaxToggle, auto = false) => {
  // Actually lock for HTML exploit
  if (
    (i > 50 && i <= 55 && !getGQUpgradeEffect('cookies', 'unlocked'))
    || (i > 55 && i <= 60 && !getGQUpgradeEffect('cookies2', 'unlocked'))
    || (i > 60 && i <= 65 && !getGQUpgradeEffect('cookies3', 'unlocked'))
    || (i > 65 && i <= 70 && !getGQUpgradeEffect('cookies4', 'unlocked'))
    || (i > 70 && !getGQUpgradeEffect('cookies5', 'unlocked'))
  ) {
    return
  }

  const metaData = getCubeCost(i, buyMax)
  const maxLevel = getCubeMax(i)
  if (Number(player.wowCubes) >= metaData.cost && player.cubeUpgrades[i]! < maxLevel) {
    player.wowCubes.sub(100 / 100 * metaData.cost)
    player.cubeUpgrades[i] = metaData.levelCanBuy
  } else {
    return
  }

  if (i === 4 && player.cubeUpgrades[4] > 0) {
    for (let j = 94; j <= 98; j++) {
      player.upgrades[j] = 1
      upgradeupdate(j, true)
    }
  }
  if (i === 5 && player.cubeUpgrades[5] > 0) {
    player.upgrades[99] = 1
    upgradeupdate(99, true)
  }
  if (i === 6 && player.cubeUpgrades[6] > 0) {
    player.upgrades[100] = 1
    upgradeupdate(100, true)
  }

  if (i === 51 && player.cubeUpgrades[51] > 0) {
    awardAutosCookieUpgrade()
  }

  if (i === 57 && player.cubeUpgrades[57] > 0) {
    for (let j = 1; j < player.cubeUpgrades.length; j++) {
      updateCubeUpgradeBG(j)
    }
  }

  if (!auto) {
    cubeUpgradeDesc(i)
    revealStuff()
  }
  updateCubeUpgradeBG(i)
}

export const autoBuyCubeUpgrades = () => {
  if (
    player.autoCubeUpgradesToggle
    && player.highestSingularityCount >= 50
  ) {
    const cheapest = []

    for (let i = 1; i < player.cubeUpgrades.length; i++) {
      const maxLevel = getCubeMax(i)
      if (player.cubeUpgrades[i]! < maxLevel) {
        const metaData = getCubeCost(i, true)
        cheapest.push([i, metaData.cost, metaData.levelCanBuy])
      }
    }

    if (cheapest.length > 0) {
      let update = false

      cheapest.sort((a, b) => {
        return a[1] - b[1]
      })

      for (const value of cheapest) {
        const maxLevel = getCubeMax(value[0])
        const metaData = getCubeCost(value[0], true)
        if (
          Number(player.wowCubes) >= metaData.cost && player.cubeUpgrades[value[0]]! < maxLevel
          && (player.cubeUpgradesBuyMaxToggle || maxLevel === metaData.levelCanBuy)
        ) {
          buyCubeUpgrades(value[0], true, true)
          update = true
        }
      }

      if (update) {
        revealStuff()
      }
    }
  }
}

// Thin shims over @synergism/logic's pure cube-blessing calculators. Each
// reads the matching tesseract-blessing value (which itself composes through
// the hypercube and platonic-blessing layers in logic) and the per-function
// player.cubeUpgrades[N] level.
export const calculateAcceleratorCubeBlessing = () =>
  logicCalcAcceleratorCube(player.cubeBlessings, calculateAcceleratorTesseractBlessing(), player.cubeUpgrades[45])
export const calculateMultiplierCubeBlessing = () =>
  logicCalcMultiplierCube(player.cubeBlessings, calculateMultiplierTesseractBlessing(), player.cubeUpgrades[35])
export const calculateOfferingCubeBlessing = () =>
  logicCalcOfferingCube(player.cubeBlessings, calculateOfferingTesseractBlessing(), player.cubeUpgrades[24])
export const calculateSalvageCubeBlessing = () =>
  logicCalcSalvageCube(player.cubeBlessings, calculateSalvageTesseractBlessing(), player.cubeUpgrades[14])
export const calculateObtainiumCubeBlessing = () =>
  logicCalcObtainiumCube(player.cubeBlessings, calculateObtainiumTesseractBlessing(), player.cubeUpgrades[40])
// AntSpeed's tesseract result is a Decimal — pass it through unchanged so
// late-game values past Number precision survive the multiplication.
export const calculateAntSpeedCubeBlessing = () =>
  logicCalcAntSpeedCube(
    player.cubeBlessings,
    calculateAntSpeedTesseractBlessing(),
    player.cubeUpgrades[22]
  )
export const calculateAntSacrificeCubeBlessing = (): Decimal =>
  logicCalcAntSacrificeCube(
    player.cubeBlessings,
    calculateAntSacrificeTesseractBlessing(),
    player.cubeUpgrades[15]
  )
export const calculateAntELOCubeBlessing = () =>
  logicCalcAntELOCube(player.cubeBlessings, calculateAntELOTesseractBlessing(), player.cubeUpgrades[25])
export const calculateRuneEffectivenessCubeBlessing = () =>
  logicCalcRuneEffectivenessCube(
    player.cubeBlessings,
    calculateRuneEffectivenessTesseractBlessing(),
    player.cubeUpgrades[44]
  )
export const calculateGlobalSpeedCubeBlessing = () =>
  logicCalcGlobalSpeedCube(player.cubeBlessings, calculateGlobalSpeedTesseractBlessing(), player.cubeUpgrades[34])
