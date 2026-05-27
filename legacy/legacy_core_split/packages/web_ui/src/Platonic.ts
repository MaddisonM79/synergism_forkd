import {
  checkPlatonicUpgradeAffordability as logicCheckPlatonicUpgradeAffordability,
  type PlatonicUpgradeAffordability,
  platonicUpgradeBaseCosts,
  platonicUpgradePriceMultiplier as logicPlatonicUpgradePriceMultiplier
} from '@synergism/logic'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { hepteracts } from './Hepteracts'
import { calculateSingularityDebuff } from './singularity'
import { format, player } from './Synergism'
import { Alert, revealStuff } from './UpdateHTML'

// Pre-extracts player's per-resource balances + abyssal hept balance and
// hands them to logic's pure affordability check.
const checkPlatonicUpgrade = (index: number, auto = false): PlatonicUpgradeAffordability => {
  const baseCost = platonicUpgradeBaseCosts[index]
  const priceMultiplier = logicPlatonicUpgradePriceMultiplier({
    priceMult: baseCost.priceMult,
    currentLevel: player.platonicUpgrades[index],
    maxLevel: baseCost.maxLevel,
    singularityDebuff: calculateSingularityDebuff('Platonic Costs')
  })
  return logicCheckPlatonicUpgradeAffordability({
    index,
    currentLevel: player.platonicUpgrades[index],
    priceMultiplier,
    autoMode: auto,
    currentResources: {
      obtainium: player.obtainium as unknown as number,
      offerings: player.offerings as unknown as number,
      cubes: player.wowCubes as unknown as number,
      tesseracts: player.wowTesseracts as unknown as number,
      hypercubes: player.wowHypercubes as unknown as number,
      platonics: player.wowPlatonicCubes as unknown as number
    },
    abyssalBalance: hepteracts.abyss.BAL
  })
}

const computePriceMultiplier = (index: number): number => {
  const baseCost = platonicUpgradeBaseCosts[index]
  return logicPlatonicUpgradePriceMultiplier({
    priceMult: baseCost.priceMult,
    currentLevel: player.platonicUpgrades[index],
    maxLevel: baseCost.maxLevel,
    singularityDebuff: calculateSingularityDebuff('Platonic Costs')
  })
}

export const createPlatonicDescription = (index: number) => {
  let translationKey = 'wowCubes.platonicUpgrades.descriptionBox.upgradeLevel'
  if (player.platonicUpgrades[index] === platonicUpgradeBaseCosts[index].maxLevel) {
    translationKey = 'wowCubes.platonicUpgrades.descriptionBox.upgradeLevelMaxed'
  }
  const resourceCheck = checkPlatonicUpgrade(index)
  const priceMultiplier = computePriceMultiplier(index)

  DOMCacheGetOrSet('platonicUpgradeDescription').innerHTML = i18next.t(
    `wowCubes.platonicUpgrades.descriptions.${index}`
  )
  DOMCacheGetOrSet('platonicUpgradeLevel').textContent = i18next.t(translationKey, {
    a: format(player.platonicUpgrades[index]),
    b: format(platonicUpgradeBaseCosts[index].maxLevel)
  })
  DOMCacheGetOrSet('platonicOfferingCost').textContent = i18next.t(
    'wowCubes.platonicUpgrades.descriptionBox.offeringCost',
    {
      a: format(player.offerings),
      b: format(platonicUpgradeBaseCosts[index].offerings * priceMultiplier)
    }
  )
  DOMCacheGetOrSet('platonicObtainiumCost').textContent = i18next.t(
    'wowCubes.platonicUpgrades.descriptionBox.obtainiumCost',
    {
      a: format(player.obtainium),
      b: format(platonicUpgradeBaseCosts[index].obtainium * priceMultiplier)
    }
  )
  DOMCacheGetOrSet('platonicCubeCost').textContent = i18next.t('wowCubes.platonicUpgrades.descriptionBox.cubeCost', {
    a: format(player.wowCubes.valueOf()),
    b: format(platonicUpgradeBaseCosts[index].cubes * priceMultiplier)
  })
  DOMCacheGetOrSet('platonicTesseractCost').textContent = i18next.t(
    'wowCubes.platonicUpgrades.descriptionBox.tesseractCost',
    {
      a: format(player.wowTesseracts.valueOf()),
      b: format(platonicUpgradeBaseCosts[index].tesseracts * priceMultiplier)
    }
  )
  DOMCacheGetOrSet('platonicHypercubeCost').textContent = i18next.t(
    'wowCubes.platonicUpgrades.descriptionBox.hypercubeCost',
    {
      a: format(player.wowHypercubes.valueOf()),
      b: format(platonicUpgradeBaseCosts[index].hypercubes * priceMultiplier)
    }
  )
  DOMCacheGetOrSet('platonicPlatonicCost').textContent = i18next.t(
    'wowCubes.platonicUpgrades.descriptionBox.platonicCost',
    {
      a: format(player.wowPlatonicCubes.valueOf()),
      b: format(platonicUpgradeBaseCosts[index].platonics * priceMultiplier)
    }
  )
  DOMCacheGetOrSet('platonicHepteractCost').textContent = i18next.t(
    'wowCubes.platonicUpgrades.descriptionBox.hepteractCost',
    {
      a: format(hepteracts.abyss.BAL, 0, true),
      b: format(Math.floor(platonicUpgradeBaseCosts[index].abyssals * priceMultiplier), 0, true)
    }
  )

  DOMCacheGetOrSet('platonicOfferingCost').style.color = resourceCheck.offerings ? 'lime' : 'var(--crimson-text-color)'
  DOMCacheGetOrSet('platonicObtainiumCost').style.color = resourceCheck.obtainium ? 'lime' : 'var(--crimson-text-color)'
  DOMCacheGetOrSet('platonicCubeCost').style.color = resourceCheck.cubes ? 'lime' : 'var(--crimson-text-color)'
  DOMCacheGetOrSet('platonicTesseractCost').style.color = resourceCheck.tesseracts
    ? 'lime'
    : 'var(--crimson-text-color)'
  DOMCacheGetOrSet('platonicHypercubeCost').style.color = resourceCheck.hypercubes
    ? 'lime'
    : 'var(--crimson-text-color)'
  DOMCacheGetOrSet('platonicPlatonicCost').style.color = resourceCheck.platonics ? 'lime' : 'var(--crimson-text-color)'
  DOMCacheGetOrSet('platonicHepteractCost').style.color = resourceCheck.abyssals ? 'lime' : 'var(--crimson-text-color)'

  if (player.platonicUpgrades[index] < platonicUpgradeBaseCosts[index].maxLevel) {
    DOMCacheGetOrSet('platonicUpgradeLevel').style.color = 'cyan'

    if (resourceCheck.canBuy) {
      DOMCacheGetOrSet('platonicCanBuy').style.color = 'gold'
      DOMCacheGetOrSet('platonicCanBuy').textContent = i18next.t(
        'wowCubes.platonicUpgrades.descriptionBox.platonicCanBuy'
      )
    } else {
      DOMCacheGetOrSet('platonicCanBuy').style.color = 'var(--crimson-text-color)'
      DOMCacheGetOrSet('platonicCanBuy').textContent = i18next.t(
        'wowCubes.platonicUpgrades.descriptionBox.platonicCannotBuy'
      )
    }
  }

  if (player.platonicUpgrades[index] === platonicUpgradeBaseCosts[index].maxLevel) {
    DOMCacheGetOrSet('platonicUpgradeLevel').style.color = 'gold'
    DOMCacheGetOrSet('platonicCanBuy').style.color = 'var(--orchid-text-color)'
    DOMCacheGetOrSet('platonicCanBuy').textContent = i18next.t(
      'wowCubes.platonicUpgrades.descriptionBox.platonicCanBuyMaxed'
    )
  }
}

export const updatePlatonicUpgradeBG = (i: number) => {
  const a = DOMCacheGetOrSet(`platUpg${i}`)

  const maxLevel = platonicUpgradeBaseCosts[i].maxLevel
  a.classList.remove('green-background', 'purple-background')

  if (player.platonicUpgrades[i] > 0 && player.platonicUpgrades[i] < maxLevel) {
    a.classList.add('purple-background')
  } else if (player.platonicUpgrades[i] === maxLevel) {
    a.classList.add('green-background')
  }
}

export const buyPlatonicUpgrades = (index: number, auto = false) => {
  if (index <= 0) return

  // eslint-disable-next-line no-constant-condition
  while (true) {
    const resourceCheck = checkPlatonicUpgrade(index, auto)
    const priceMultiplier = computePriceMultiplier(index)

    if (resourceCheck.canBuy) {
      player.platonicUpgrades[index] += 1
      // Auto Platonic Upgrades no longer claim the cost of Offerings and Obtainiums
      if (!auto) {
        player.obtainium = player.obtainium.sub(Math.floor(platonicUpgradeBaseCosts[index].obtainium * priceMultiplier))
        player.offerings = player.offerings.sub(Math.floor(platonicUpgradeBaseCosts[index].offerings * priceMultiplier))
      }
      player.wowCubes.sub(Math.floor(platonicUpgradeBaseCosts[index].cubes * priceMultiplier))
      player.wowTesseracts.sub(Math.floor(platonicUpgradeBaseCosts[index].tesseracts * priceMultiplier))
      player.wowHypercubes.sub(Math.floor(platonicUpgradeBaseCosts[index].hypercubes * priceMultiplier))
      player.wowPlatonicCubes.sub(Math.floor(platonicUpgradeBaseCosts[index].platonics * priceMultiplier))
      hepteracts.abyss.BAL -= Math.floor(platonicUpgradeBaseCosts[index].abyssals * priceMultiplier)

      if (index === 20 && !auto && player.singularityCount === 0) {
        void Alert(
          i18next.t('wowCubes.platonicUpgrades.20Bought')
        )
      }
    } else {
      break
    }

    if (
      player.platonicUpgrades[index] === platonicUpgradeBaseCosts[index].maxLevel || player.singularityCount === 0
      || !player.maxPlatToggle
    ) {
      break
    }
  }
  createPlatonicDescription(index)
  updatePlatonicUpgradeBG(index)
  revealStuff()
}

export const autoBuyPlatonicUpgrades = () => {
  if (
    player.autoPlatonicUpgradesToggle
    && player.highestSingularityCount >= 50
  ) {
    for (let i = 1; i < player.platonicUpgrades.length; i++) {
      if (player.platonicUpgrades[i] < platonicUpgradeBaseCosts[i].maxLevel) {
        const resourceCheck = checkPlatonicUpgrade(i, true)
        if (resourceCheck.canBuy) {
          buyPlatonicUpgrades(i, true)
        }
      }
    }
  }
}
