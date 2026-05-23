import {
  getLevelMilestone as logicGetLevelMilestone,
  getLevelReward as logicGetLevelReward,
  type LevelMilestoneKey,
  levelMilestones as logicLevelMilestones,
  type LevelRewardKey,
  levelRewards as logicLevelRewards,
  salvageChallengeBuffEffect as logicSalvageChallengeBuffEffect
} from '@synergism/logic'
import i18next from 'i18next'
import { achievementLevel } from './Achievements'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { resetTimeThreshold } from './Calculate'
import { format, formatAsPercentIncrease, player } from './Synergism'

type SynergismLevelReward = LevelRewardKey

interface SynergismLevelRewardData {
  name: () => string
  description: () => string
  effect: (lv: number) => number
  effectDescription: () => string
  minLevel: number
  defaultValue: number
  nameColor: string
}

// Helper to assemble each reward entry: pure formula bits come from
// @synergism/logic's levelRewards table, web_ui owns the i18n/colour-only
// surface. The `effectDescription` closure reads `achievementLevel` indirectly
// via the local getLevelReward shim below, so it always reflects the live
// player state — matching the legacy behaviour.
const mkRewardEntry = (
  key: LevelRewardKey,
  nameColor: string,
  effectDescription: () => string
): SynergismLevelRewardData => {
  const logic = logicLevelRewards[key]
  return {
    name: () => i18next.t(`achievements.levelRewards.${key}.name`),
    description: () => i18next.t(`achievements.levelRewards.${key}.description`),
    effect: logic.effect,
    effectDescription,
    minLevel: logic.minLevel,
    defaultValue: logic.defaultValue,
    nameColor
  }
}

// Effect-description string for a percent-multiplier reward (1.23x → +23%).
const percentIncreaseEffectDesc = (key: LevelRewardKey) => () =>
  i18next.t(`achievements.levelRewards.${key}.effect`, {
    mult: formatAsPercentIncrease(getLevelReward(key), 2)
  })

export const synergismLevelRewards: Record<SynergismLevelReward, SynergismLevelRewardData> = {
  salvage: mkRewardEntry('salvage', 'green', () =>
    i18next.t('achievements.levelRewards.salvage.effect', {
      salvage: format(getLevelReward('salvage'), 0, true)
    })),
  quarks: mkRewardEntry('quarks', 'cyan', percentIncreaseEffectDesc('quarks')),
  offerings: mkRewardEntry('offerings', 'orange', percentIncreaseEffectDesc('offerings')),
  obtainium: mkRewardEntry('obtainium', 'pink', percentIncreaseEffectDesc('obtainium')),
  ants: mkRewardEntry('ants', 'burlywood', () =>
    i18next.t('achievements.levelRewards.ants.effect', {
      elo: format(getLevelReward('ants'), 0, true)
    })),
  wowCubes: mkRewardEntry('wowCubes', 'lightgrey', percentIncreaseEffectDesc('wowCubes')),
  wowTesseracts: mkRewardEntry('wowTesseracts', 'orchid', percentIncreaseEffectDesc('wowTesseracts')),
  wowHyperCubes: mkRewardEntry('wowHyperCubes', 'crimson', percentIncreaseEffectDesc('wowHyperCubes')),
  wowPlatonicCubes: mkRewardEntry(
    'wowPlatonicCubes',
    'lightgoldenrodyellow',
    percentIncreaseEffectDesc('wowPlatonicCubes')
  ),
  wowHepteractCubes: mkRewardEntry('wowHepteractCubes', 'mediumpurple', percentIncreaseEffectDesc('wowHepteractCubes')),
  wowOcteracts: mkRewardEntry('wowOcteracts', 'turquoise', percentIncreaseEffectDesc('wowOcteracts')),
  ambrosiaLuck: mkRewardEntry('ambrosiaLuck', 'lime', () =>
    i18next.t('achievements.levelRewards.ambrosiaLuck.effect', {
      luck: format(getLevelReward('ambrosiaLuck'), 0, true)
    })),
  redAmbrosiaLuck: mkRewardEntry(
    'redAmbrosiaLuck',
    'red',
    () =>
      i18next.t('achievements.levelRewards.redAmbrosiaLuck.effect', {
        luck: format(getLevelReward('redAmbrosiaLuck'), 0, true)
      })
  )
}

const synergismLevelReward = Object.keys(synergismLevelRewards) as SynergismLevelReward[]

// Thin shim over @synergism/logic's getLevelReward — reads the live
// `achievementLevel` and forwards.
export const getLevelReward = (reward: SynergismLevelReward): number => logicGetLevelReward(reward, achievementLevel)

const getLevelRewardDescription = (reward: SynergismLevelReward) => {
  const name = synergismLevelRewards[reward].name()
  const description = synergismLevelRewards[reward].description()
  const effectDesc = synergismLevelRewards[reward].effectDescription()
  const minimumLevel = synergismLevelRewards[reward].minLevel > 0
    ? i18next.t('achievements.levelRewards.minLevel', {
      level: synergismLevelRewards[reward].minLevel
    })
    : i18next.t('achievements.levelRewards.noLevelReq')

  const nameColor = synergismLevelRewards[reward].nameColor

  DOMCacheGetOrSet('synergismLevelMultiLine').innerHTML = `
        <span style="color:${nameColor}">${name}</span><br>
        ${minimumLevel}<br>
        ${description}<br>
        ${effectDesc}
    `
}

export const generateLevelRewardHTMLs = () => {
  const alreadyGenerated = document.getElementsByClassName('synergismLevelRewardType').length > 0
  if (alreadyGenerated) {
    return
  }
  const rewardTable = DOMCacheGetOrSet('synergismLevelRewardsTable')
  for (const reward of synergismLevelReward) {
    const capitalizedName = reward.charAt(0).toUpperCase() + reward.slice(1)

    const div = document.createElement('div')
    div.classList.add('synergismLevelRewardType')

    const img = document.createElement('img')
    img.id = `synergismLevelReward${capitalizedName}`
    img.src = `Pictures/Achievements/Rewards/${capitalizedName}.png`
    img.alt = synergismLevelRewards[reward].name()
    img.style.cursor = 'pointer'

    const boundGetLevelRewardDescription = getLevelRewardDescription.bind(null, reward)

    img.addEventListener('click', boundGetLevelRewardDescription)
    img.addEventListener('mouseover', boundGetLevelRewardDescription)
    img.addEventListener('focus', boundGetLevelRewardDescription)
    div.appendChild(img)
    rewardTable.appendChild(div)
  }
}

// Extends the logic-side `LevelMilestoneKey` with the one milestone whose
// effect needs live player state (`salvageChallengeBuff`). All other entries
// in the web_ui table delegate `effect` to logic; the salvage one calls into
// `logicSalvageChallengeBuffEffect` with input sourced from `player`.
type SynergismLevelMilestones = LevelMilestoneKey | 'salvageChallengeBuff'

interface SynergismLevelMilestoneData {
  name: () => string
  description: () => string
  effect: () => number
  defaultValue: number // If level is not reached.
  effectDescription: () => string
  levelReq: number
  displayOrder: number
}

// Builds a milestone-table entry from the logic-side data. Web_ui supplies the
// i18n closures (name/description/effectDescription) and the displayOrder; the
// effect/levelReq/defaultValue come from the logic levelMilestones table. The
// `effect` thunk wraps the pure formula with the live `achievementLevel`.
const mkMilestoneEntry = (
  key: LevelMilestoneKey,
  displayOrder: number,
  effectDescription: () => string
): SynergismLevelMilestoneData => {
  const logic = logicLevelMilestones[key]
  return {
    name: () => i18next.t(`achievements.levelMilestones.${key}.name`),
    description: () => i18next.t(`achievements.levelMilestones.${key}.description`),
    effect: () => logic.effect(achievementLevel),
    defaultValue: logic.defaultValue,
    effectDescription,
    levelReq: logic.levelReq,
    displayOrder
  }
}

// Effect-description string for "is this milestone unlocked yet?" rewards.
// Used by the 11 unlock-flag milestones (autoPrestige, tier1-5 crystal, etc.)
// whose effect returns 1 when active.
const unlockedFlagDesc = (key: LevelMilestoneKey) => () => {
  const unlocked = getLevelMilestone(key) === 1
  return i18next.t(`achievements.levelMilestones.${key}.effect`, {
    unlocked: unlocked ? i18next.t('achievements.rewardTypes.unlocked') : i18next.t('achievements.rewardTypes.locked')
  })
}

// Sources the live player-challenge state for the one impure milestone.
const salvageChallengeBuffEffect = () =>
  logicSalvageChallengeBuffEffect({
    inAnyChallenge: player.currentChallenge.transcension !== 0
      || player.currentChallenge.reincarnation !== 0
      || player.currentChallenge.ascension !== 0,
    inAscension15: player.currentChallenge.ascension === 15,
    insideSingularityChallenge: player.insideSingularityChallenge
  })

const synergismLevelMilestones: Record<SynergismLevelMilestones, SynergismLevelMilestoneData> = {
  offeringTimerScaling: mkMilestoneEntry('offeringTimerScaling', 1, () => {
    const mult = getLevelMilestone('offeringTimerScaling') === 1
      ? Math.max(1, player.prestigecounter / resetTimeThreshold())
      : 1
    return i18next.t('achievements.levelMilestones.offeringTimerScaling.effect', {
      mult: formatAsPercentIncrease(mult, 2)
    })
  }),
  autoPrestige: {
    ...mkMilestoneEntry('autoPrestige', 2, () => {
      const autoPrestige = getLevelMilestone('autoPrestige') === 1
      return i18next.t('achievements.levelMilestones.autoPrestige.effect', {
        autoPrestige: autoPrestige
          ? i18next.t('achievements.rewardTypes.unlocked')
          : i18next.t('achievements.rewardTypes.locked')
      })
    })
  },
  speedRune: mkMilestoneEntry('speedRune', 3, () =>
    i18next.t('achievements.levelMilestones.speedRune.effect', {
      speedRune: format(getLevelMilestone('speedRune'), 2, true)
    })),
  duplicationRune: mkMilestoneEntry(
    'duplicationRune',
    4,
    () =>
      i18next.t('achievements.levelMilestones.duplicationRune.effect', {
        duplicationRune: format(getLevelMilestone('duplicationRune'), 2, true)
      })
  ),
  prismRune: mkMilestoneEntry('prismRune', 5, () =>
    i18next.t('achievements.levelMilestones.prismRune.effect', {
      prismRune: format(getLevelMilestone('prismRune'), 2, true)
    })),
  thriftRune: mkMilestoneEntry('thriftRune', 6, () =>
    i18next.t('achievements.levelMilestones.thriftRune.effect', {
      thriftRune: format(getLevelMilestone('thriftRune'), 2, true)
    })),
  SIRune: mkMilestoneEntry('SIRune', 7, () =>
    i18next.t('achievements.levelMilestones.SIRune.effect', {
      siRune: format(getLevelMilestone('SIRune'), 2, true)
    })),
  // The five tier crystal autobuyers share the same `unlocked/locked` strings.
  // They use the `autobuy` interpolation key (not `unlocked`), so they can't
  // use the shared `unlockedFlagDesc` helper directly.
  tier1CrystalAutobuy: mkMilestoneEntry('tier1CrystalAutobuy', 8, () => {
    const autobuy = getLevelMilestone('tier1CrystalAutobuy') === 1
    return i18next.t('achievements.levelMilestones.tier1CrystalAutobuy.effect', {
      autobuy: autobuy ? i18next.t('achievements.rewardTypes.unlocked') : i18next.t('achievements.rewardTypes.locked')
    })
  }),
  tier2CrystalAutobuy: mkMilestoneEntry('tier2CrystalAutobuy', 9, () => {
    const autobuy = getLevelMilestone('tier2CrystalAutobuy') === 1
    return i18next.t('achievements.levelMilestones.tier2CrystalAutobuy.effect', {
      autobuy: autobuy ? i18next.t('achievements.rewardTypes.unlocked') : i18next.t('achievements.rewardTypes.locked')
    })
  }),
  tier3CrystalAutobuy: mkMilestoneEntry('tier3CrystalAutobuy', 10, () => {
    const autobuy = getLevelMilestone('tier3CrystalAutobuy') === 1
    return i18next.t('achievements.levelMilestones.tier3CrystalAutobuy.effect', {
      autobuy: autobuy ? i18next.t('achievements.rewardTypes.unlocked') : i18next.t('achievements.rewardTypes.locked')
    })
  }),
  tier4CrystalAutobuy: mkMilestoneEntry('tier4CrystalAutobuy', 11, () => {
    const autobuy = getLevelMilestone('tier4CrystalAutobuy') === 1
    return i18next.t('achievements.levelMilestones.tier4CrystalAutobuy.effect', {
      autobuy: autobuy ? i18next.t('achievements.rewardTypes.unlocked') : i18next.t('achievements.rewardTypes.locked')
    })
  }),
  tier5CrystalAutobuy: mkMilestoneEntry('tier5CrystalAutobuy', 12, () => {
    const autobuy = getLevelMilestone('tier5CrystalAutobuy') === 1
    return i18next.t('achievements.levelMilestones.tier5CrystalAutobuy.effect', {
      autobuy: autobuy ? i18next.t('achievements.rewardTypes.unlocked') : i18next.t('achievements.rewardTypes.locked')
    })
  }),
  achievementTalismanUnlock: mkMilestoneEntry(
    'achievementTalismanUnlock',
    13,
    unlockedFlagDesc('achievementTalismanUnlock')
  ),
  runeAutobuyImprover: mkMilestoneEntry(
    'runeAutobuyImprover',
    13.5,
    () =>
      i18next.t('achievements.levelMilestones.runeAutobuyImprover.effect', {
        mult: formatAsPercentIncrease(getLevelMilestone('runeAutobuyImprover'), 0)
      })
  ),
  achievementTalismanEnhancement: mkMilestoneEntry(
    'achievementTalismanEnhancement',
    14,
    () =>
      i18next.t('achievements.levelMilestones.achievementTalismanEnhancement.effect', {
        level: format(getLevelMilestone('achievementTalismanEnhancement'), 0, true)
      })
  ),
  // salvageChallengeBuff is the one milestone whose effect needs player state.
  // The structure mirrors mkMilestoneEntry but with the salvage-specific effect
  // thunk inline.
  salvageChallengeBuff: {
    name: () => i18next.t('achievements.levelMilestones.salvageChallengeBuff.name'),
    description: () => i18next.t('achievements.levelMilestones.salvageChallengeBuff.description'),
    effect: salvageChallengeBuffEffect,
    defaultValue: 0,
    effectDescription: () =>
      i18next.t('achievements.levelMilestones.salvageChallengeBuff.effect', {
        salvage: format(getLevelMilestone('salvageChallengeBuff'), 0, true)
      }),
    levelReq: 180,
    displayOrder: 15
  },
  antSpeed2Autobuyer: mkMilestoneEntry('antSpeed2Autobuyer', 16, unlockedFlagDesc('antSpeed2Autobuyer')),
  wowCubesAutobuyer: mkMilestoneEntry('wowCubesAutobuyer', 17, unlockedFlagDesc('wowCubesAutobuyer')),
  ascensionScoreAutobuyer: mkMilestoneEntry(
    'ascensionScoreAutobuyer',
    18,
    unlockedFlagDesc('ascensionScoreAutobuyer')
  ),
  mortuus2Autobuyer: mkMilestoneEntry('mortuus2Autobuyer', 19, unlockedFlagDesc('mortuus2Autobuyer'))
}

const synergismLevelMilestone = Object.keys(synergismLevelMilestones) as SynergismLevelMilestones[]

// Thin shim over @synergism/logic's getLevelMilestone, with a special case
// for salvageChallengeBuff which lives in web_ui because its effect needs
// live player-challenge state.
export const getLevelMilestone = (milestone: SynergismLevelMilestones): number => {
  if (milestone === 'salvageChallengeBuff') {
    return achievementLevel >= 180 ? salvageChallengeBuffEffect() : 0
  }
  return logicGetLevelMilestone(milestone, achievementLevel)
}

const getLevelMilestoneDescription = (milestone: SynergismLevelMilestones) => {
  const name = synergismLevelMilestones[milestone].name()
  const description = synergismLevelMilestones[milestone].description()
  const effectDesc = synergismLevelMilestones[milestone].effectDescription()
  const minimumLevel = i18next.t('achievements.levelRewards.minLevel', {
    level: synergismLevelMilestones[milestone].levelReq
  })

  DOMCacheGetOrSet('synergismLevelMultiLine').innerHTML = `
        <span style="color:lightblue">${name}</span><br>
        ${minimumLevel}<br>
        ${description}<br>
        ${effectDesc}
    `
}

export const generateLevelMilestoneHTMLS = () => {
  const alreadyGenerated = document.getElementsByClassName('synergismLevelMilestoneType').length > 0
  if (alreadyGenerated) {
    return
  }
  const rewardTable = DOMCacheGetOrSet('synergismLevelMilestonesTable')
  for (const milestone of synergismLevelMilestone) {
    const capitalizedName = milestone.charAt(0).toUpperCase() + milestone.slice(1)

    const div = document.createElement('div')
    div.classList.add('synergismLevelMilestoneType')

    const img = document.createElement('img')
    img.id = `synergismLevelMilestone${capitalizedName}`
    img.src = `Pictures/Achievements/Milestones/${capitalizedName}.png`
    img.alt = synergismLevelMilestones[milestone].name()
    img.style.cursor = 'pointer'

    const boundGetLevelMilestoneDescription = getLevelMilestoneDescription.bind(null, milestone)

    img.addEventListener('click', boundGetLevelMilestoneDescription)
    img.addEventListener('mouseover', boundGetLevelMilestoneDescription)
    img.addEventListener('focus', boundGetLevelMilestoneDescription)
    div.appendChild(img)
    rewardTable.appendChild(div)
  }

  displayLevelStuff()
}

export const displayLevelStuff = () => {
  for (const key of synergismLevelReward) {
    const capitalizedName = key.charAt(0).toUpperCase() + key.slice(1)
    const id = `synergismLevelReward${capitalizedName}`
    const element = DOMCacheGetOrSet(id)
    if (achievementLevel >= synergismLevelRewards[key].minLevel) {
      element.style.display = 'inline-block'
    } else {
      element.style.display = 'none'
    }
  }

  for (const key of synergismLevelMilestone) {
    const capitalizedName = key.charAt(0).toUpperCase() + key.slice(1)
    const id = `synergismLevelMilestone${capitalizedName}`
    const element = DOMCacheGetOrSet(id)
    if (achievementLevel >= synergismLevelMilestones[key].levelReq) {
      element.style.display = 'inline-block'
    } else {
      element.style.display = 'none'
    }
  }
}
