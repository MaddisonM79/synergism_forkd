import {
  limitedAscensionsAchievementPointValue as logicLimitedAscensionsAP,
  limitedAscensionsEffect as logicLimitedAscensionsEffect,
  limitedAscensionsSingularityRequirement as logicLimitedAscensionsSR,
  limitedTimeAchievementPointValue as logicLimitedTimeAP,
  limitedTimeEffect as logicLimitedTimeEffect,
  limitedTimeSingularityRequirement as logicLimitedTimeSR,
  noAmbrosiaUpgradesAchievementPointValue as logicNoAmbrosiaAP,
  noAmbrosiaUpgradesEffect as logicNoAmbrosiaEffect,
  noAmbrosiaUpgradesSingularityRequirement as logicNoAmbrosiaSR,
  noOcteractsAchievementPointValue as logicNoOcteractsAP,
  noOcteractsEffect as logicNoOcteractsEffect,
  noOcteractsSingularityRequirement as logicNoOcteractsSR,
  noQuarkUpgradesAchievementPointValue as logicNoQuarkAP,
  noQuarkUpgradesEffect as logicNoQuarkEffect,
  noQuarkUpgradesSingularityRequirement as logicNoQuarkSR,
  noSingularityUpgradesAchievementPointValue as logicNoSingAP,
  noSingularityUpgradesEffect as logicNoSingEffect,
  noSingularityUpgradesSingularityRequirement as logicNoSingSR,
  oneChallengeCapAchievementPointValue as logicOneChalCapAP,
  oneChallengeCapEffect as logicOneChalCapEffect,
  oneChallengeCapSingularityRequirement as logicOneChalCapSR,
  sadisticPrequelAchievementPointValue as logicSadisticAP,
  sadisticPrequelEffect as logicSadisticEffect,
  sadisticPrequelSingularityRequirement as logicSadisticSR,
  type SingularityChallengeDataKeys as LogicSingularityChallengeDataKeys,
  type SingularityChallengeRewards as LogicSingularityChallengeRewards,
  taxmanLastStandAchievementPointValue as logicTaxmanAP,
  taxmanLastStandEffect as logicTaxmanEffect,
  taxmanLastStandSingularityRequirement as logicTaxmanSR
} from '@synergism/logic'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import {
  calculateExalt3AscensionLimit,
  calculateExalt4EffectiveSingularityMultiplier,
  calculateExalt6PenaltyPerSecond,
  calculateExalt6TimeLimit,
  calculateGoldenQuarks
} from './Calculate'
import { singularity } from './Reset'
import { runes } from './Runes'
import { format, player } from './Synergism'
import { Alert, Confirm } from './UpdateHTML'
import { toOrdinal } from './Utility'
import { Globals as G } from './Variables'

// Re-exported from @synergism/logic so existing call sites that import
// these types from this module keep compiling unchanged.
export type SingularityChallengeRewards = LogicSingularityChallengeRewards
export type SingularityChallengeDataKeys = LogicSingularityChallengeDataKeys

interface ISingularityChallengeData {
  baseReq: number
  maxCompletions: number
  unlockSingularity: number
  HTMLTag: SingularityChallengeDataKeys
  singularityRequirement: (baseReq: number, completions: number) => number
  achievementPointValue: (n: number) => number
  scalingrewardcount: number
  uniquerewardcount: number
  resetTime?: boolean
  completions?: number
  enabled?: boolean
  alternateDescription?: () => string
  highestSingularityCompleted?: number
}

interface ISingularityChallengeDataWithEffect<
  T extends SingularityChallengeDataKeys,
  K extends keyof SingularityChallengeRewards[T]
> extends ISingularityChallengeData {
  effect: (n: number, key: K) => SingularityChallengeRewards[T][K]
}

export class SingularityChallenge {
  public name
  public description
  public baseReq
  public completions
  public maxCompletions
  public unlockSingularity
  public HTMLTag
  public highestSingularityCompleted
  public enabled
  public resetTime
  public singularityRequirement
  public achievementPointValue
  public alternateDescription
  public scalingrewardcount
  public uniquerewardcount
  #key: string

  public constructor (data: ISingularityChallengeData, key: string) {
    const name = i18next.t(`singularityChallenge.data.${key}.name`)
    const description = i18next.t(
      `singularityChallenge.data.${key}.description`
    )
    this.name = name
    this.description = description
    this.baseReq = data.baseReq
    this.completions = data.completions ?? 0
    this.maxCompletions = data.maxCompletions
    this.unlockSingularity = data.unlockSingularity
    this.HTMLTag = data.HTMLTag
    this.highestSingularityCompleted = data.highestSingularityCompleted ?? 0
    this.enabled = data.enabled ?? false
    this.resetTime = data.resetTime ?? false
    this.singularityRequirement = data.singularityRequirement
    this.achievementPointValue = data.achievementPointValue
    this.alternateDescription = data.alternateDescription ?? undefined
    this.scalingrewardcount = data.scalingrewardcount
    this.uniquerewardcount = data.uniquerewardcount

    this.updateIconHTML()
    this.updateChallengeCompletions()
    this.#key = key
  }

  public computeSingularityRquirement () {
    return this.singularityRequirement(this.baseReq, this.completions)
  }

  public updateChallengeCompletions () {
    let updateVal = 0
    while (
      this.singularityRequirement(this.baseReq, updateVal)
        <= this.highestSingularityCompleted
    ) {
      updateVal += 1
    }

    this.completions = Math.min(this.maxCompletions, updateVal)
  }

  public challengeEntryHandler () {
    if (!this.enabled) {
      return this.enableChallenge()
    } else {
      return this.exitChallenge(runes.antiquities.level > 0)
    }
  }

  public async enableChallenge () {
    if (player.highestSingularityCount < this.unlockSingularity) {
      return Alert(
        i18next.t('singularityChallenge.enterChallenge.lowSingularity')
      )
    }
    const confirmation = await Confirm(
      i18next.t('singularityChallenge.enterChallenge.confirmation', {
        name: this.name
      })
    )

    if (!confirmation) {
      return Alert(i18next.t('singularityChallenge.enterChallenge.decline'))
    }

    if (!player.insideSingularityChallenge) {
      const setSingularity = this.computeSingularityRquirement()
      const holdSingTimer = player.singularityCounter
      const holdQuarkExport = player.quarkstimer
      const holdGoldenQuarkExport = player.goldenQuarksTimer
      const goldenQuarkGain = calculateGoldenQuarks()
      const currentGQ = player.goldenQuarks
      this.enabled = true
      G.currentSingChallenge = this.HTMLTag
      player.insideSingularityChallenge = true
      singularity(setSingularity)

      if (!this.resetTime) {
        player.singularityCounter = holdSingTimer
      } else {
        player.singularityCounter = 0
      }
      player.goldenQuarks = currentGQ + goldenQuarkGain
      player.quarkstimer = holdQuarkExport
      player.goldenQuarksTimer = holdGoldenQuarkExport

      this.updateChallengeHTML()
      return Alert(
        i18next.t('singularityChallenge.enterChallenge.acceptSuccess', {
          name: this.name,
          tier: this.completions + 1,
          singReq: this.computeSingularityRquirement()
        })
      )
    } else {
      return Alert(
        i18next.t('singularityChallenge.exitChallenge.acceptFailure')
      )
    }
  }

  public async exitChallenge (success: boolean) {
    if (!success) {
      const extra = runes.antiquities.level === 0
        ? i18next.t('singularityChallenge.exitChallenge.incompleteWarning')
        : ''
      const confirmation = await Confirm(
        i18next.t('singularityChallenge.exitChallenge.confirmation', {
          name: this.name,
          tier: this.completions + 1,
          warning: extra
        })
      )
      if (!confirmation) {
        return Alert(i18next.t('singularityChallenge.exitChallenge.decline'))
      }
    }

    this.enabled = false
    G.currentSingChallenge = undefined
    player.insideSingularityChallenge = false
    const highestSingularityHold = player.highestSingularityCount
    const holdSingTimer = player.singularityCounter
    const holdQuarkExport = player.quarkstimer
    const holdGoldenQuarkExport = player.goldenQuarksTimer
    this.updateIconHTML()
    if (success) {
      this.highestSingularityCompleted = player.singularityCount
      this.updateChallengeCompletions()
      singularity(highestSingularityHold)
      player.singularityCounter = holdSingTimer
      return Alert(
        i18next.t('singularityChallenge.exitChallenge.acceptSuccess', {
          tier: toOrdinal(this.completions),
          name: this.name
        })
      )
    } else {
      singularity(highestSingularityHold)
      player.singularityCounter = holdSingTimer
      player.quarkstimer = holdQuarkExport
      player.goldenQuarksTimer = holdGoldenQuarkExport
      return Alert(
        i18next.t('singularityChallenge.exitChallenge.acceptFailure')
      )
    }
  }

  /**
   * Given a Singularity Challenge, give a concise information regarding its data.
   * @returns A string that details the name, description, metadata.
   */
  toString (): string {
    const color = this.completions === this.maxCompletions
      ? 'var(--orchid-text-color)'
      : 'white'
    const enabled = this.enabled
      ? `<span style="color: var(--red-text-color)">${
        i18next.t(
          'general.enabled'
        )
      }</span>`
      : ''
    return `<span style="color: gold">${this.name}</span> ${enabled}
      ${
      i18next.t(
        'singularityChallenge.toString.tiersCompleted'
      )
    }: <span style="color: ${color}">${this.completions}/${this.maxCompletions}</span>
      <span style="color: pink">${
      i18next.t(
        'singularityChallenge.toString.canEnter',
        {
          unlockSing: this.unlockSingularity,
          highestSing: player.highestSingularityCount
        }
      )
    }</span>
    <span style="color: gold">${
      i18next.t(
        'singularityChallenge.toString.currentTierSingularity'
      )
    } <span style="color: var(--orchid-text-color)">${
      this.singularityRequirement(
        this.baseReq,
        this.completions
      )
    }</span></span>
    <span style="color: lightblue">${
      this.alternateDescription !== undefined ? this.alternateDescription() : this.description
    }</span>`
  }
  // Numerates through total reward count for Scaling & Unique string for EXALTS.
  scaleString (): string {
    let text = ''
    for (let i = 1; i <= this.scalingrewardcount; i++) {
      const list = i18next.t(`singularityChallenge.data.${this.HTMLTag}.ScalingReward${i}`)
      text += i > 1 ? `\n${list}` : list
    }
    return text
  }

  // Ditto. Also worth mentioning this implementation means the list size can be arbitrary!
  uniqueString (): string {
    let text = ''
    for (let i = 1; i <= this.uniquerewardcount; i++) {
      const list = i18next.t(`singularityChallenge.data.${this.HTMLTag}.UniqueReward${i}`)
      text += i > 1 ? `\n${list}` : list
    }
    return text
  }

  public updateChallengeHTML (): void {
    DOMCacheGetOrSet('singularityChallengesInfo').innerHTML = this.toString()
    DOMCacheGetOrSet('singularityChallengesScalingRewards').innerHTML = this.scaleString()
    DOMCacheGetOrSet('singularityChallengesUniqueRewards').innerHTML = this.uniqueString()
  }

  public updateIconHTML (): void {
    const color = this.enabled ? 'orchid' : ''
    DOMCacheGetOrSet(this.HTMLTag).style.backgroundColor = color
  }

  public get rewardAP () {
    return this.achievementPointValue(this.completions)
  }

  public get maxAP () {
    return this.achievementPointValue(this.maxCompletions)
  }

  valueOf (): ISingularityChallengeData {
    return {
      baseReq: this.baseReq,
      HTMLTag: this.HTMLTag,
      maxCompletions: this.maxCompletions,
      achievementPointValue: this.achievementPointValue,
      scalingrewardcount: this.scalingrewardcount,
      singularityRequirement: this.singularityRequirement,
      uniquerewardcount: this.uniquerewardcount,
      unlockSingularity: this.unlockSingularity,
      completions: this.completions,
      enabled: this.enabled,
      highestSingularityCompleted: this.highestSingularityCompleted,
      resetTime: this.resetTime
    }
  }

  key () {
    return this.#key
  }
}

export const singularityChallengeData: {
  [K in SingularityChallengeDataKeys]: ISingularityChallengeDataWithEffect<K, keyof SingularityChallengeRewards[K]>
} = {
  noSingularityUpgrades: {
    baseReq: 1,
    maxCompletions: 15,
    unlockSingularity: 25,
    HTMLTag: 'noSingularityUpgrades',
    singularityRequirement: logicNoSingSR,
    achievementPointValue: logicNoSingAP,
    scalingrewardcount: 2,
    uniquerewardcount: 5,
    effect: logicNoSingEffect
  },
  oneChallengeCap: {
    baseReq: 10,
    maxCompletions: 15,
    unlockSingularity: 40,
    HTMLTag: 'oneChallengeCap',
    singularityRequirement: logicOneChalCapSR,
    achievementPointValue: logicOneChalCapAP,
    scalingrewardcount: 3,
    uniquerewardcount: 4,
    effect: logicOneChalCapEffect
  },
  noOcteracts: {
    baseReq: 75,
    maxCompletions: 15,
    unlockSingularity: 100,
    achievementPointValue: logicNoOcteractsAP,
    HTMLTag: 'noOcteracts',
    singularityRequirement: logicNoOcteractsSR,
    scalingrewardcount: 2,
    uniquerewardcount: 3,
    effect: logicNoOcteractsEffect,
    alternateDescription: () => {
      const completions = player.singularityChallenges.noOcteracts.completions
      let stringText = i18next.t('singularityChallenge.data.noOcteracts.description')
      if (completions > 0) {
        const effectiveSingMult = calculateExalt4EffectiveSingularityMultiplier(completions, true)
        const effectMod1 = i18next.t('singularityChallenge.data.noOcteracts.effectMod1', {
          sing: format(effectiveSingMult, 0, true)
        })
        stringText += `<br>${effectMod1}`
      }
      return stringText
    }
  },
  limitedAscensions: {
    baseReq: 7,
    maxCompletions: 10,
    unlockSingularity: 50,
    achievementPointValue: logicLimitedAscensionsAP,
    HTMLTag: 'limitedAscensions',
    singularityRequirement: logicLimitedAscensionsSR,
    scalingrewardcount: 2,
    uniquerewardcount: 3,
    effect: logicLimitedAscensionsEffect,
    alternateDescription: () => {
      const ascensionLimit = calculateExalt3AscensionLimit(player.singularityChallenges.limitedAscensions.completions)
      const baseDesc = i18next.t('singularityChallenge.data.limitedAscensions.description')
      const effectMod1 = i18next.t('singularityChallenge.data.limitedAscensions.effectMod1', {
        ascensions: format(ascensionLimit, 0, true)
      })
      const effectMod2 = i18next.t('singularityChallenge.data.limitedAscensions.effectMod2', {
        ascensions: format(ascensionLimit, 0, true)
      })
      const effectMod3 = i18next.t('singularityChallenge.data.limitedAscensions.effectMod3')
      return `${baseDesc}<br>${effectMod1}<br>${effectMod2}<br>${effectMod3}`
    }
  },
  noAmbrosiaUpgrades: {
    baseReq: 150,
    maxCompletions: 15,
    unlockSingularity: 166,
    achievementPointValue: logicNoAmbrosiaAP,
    HTMLTag: 'noAmbrosiaUpgrades',
    singularityRequirement: logicNoAmbrosiaSR,
    scalingrewardcount: 5,
    uniquerewardcount: 8,
    effect: logicNoAmbrosiaEffect
  },
  noQuarkUpgrades: {
    baseReq: 20,
    maxCompletions: 10,
    unlockSingularity: 66,
    achievementPointValue: logicNoQuarkAP,
    HTMLTag: 'noQuarkUpgrades',
    singularityRequirement: logicNoQuarkSR,
    scalingrewardcount: 6,
    uniquerewardcount: 3,
    effect: logicNoQuarkEffect,
    alternateDescription: () => {
      const introText = i18next.t('singularityChallenge.data.noQuarkUpgrades.description')
      const chalText = i18next.t('singularityChallenge.data.noQuarkUpgrades.challengeDesc')
      return `${introText}<br>${chalText}`
    }
  },
  limitedTime: {
    baseReq: 203,
    maxCompletions: 15,
    unlockSingularity: 216,
    achievementPointValue: logicLimitedTimeAP,
    HTMLTag: 'limitedTime',
    singularityRequirement: logicLimitedTimeSR,
    scalingrewardcount: 5,
    uniquerewardcount: 3,
    effect: logicLimitedTimeEffect,
    alternateDescription: () => {
      const completions = player.singularityChallenges.limitedTime.completions
      const baseDesc = i18next.t('singularityChallenge.data.limitedTime.description')
      const timeLimit = calculateExalt6TimeLimit(completions)
      const perSecondPenalty = calculateExalt6PenaltyPerSecond(completions)

      const timeMod1 = i18next.t('singularityChallenge.data.limitedTime.timeMod1', {
        time: format(timeLimit, 0, true)
      })
      const timeMod2 = i18next.t('singularityChallenge.data.limitedTime.timeMod2', {
        perSecondDivisor: format(perSecondPenalty, 3, true)
      })
      const timeMod3 = i18next.t('singularityChallenge.data.limitedTime.timeMod3')

      return `${baseDesc}<br>${timeMod1}<br>${timeMod2}<br>${timeMod3}`
    }
  },
  sadisticPrequel: {
    baseReq: 120,
    maxCompletions: 15,
    unlockSingularity: 256,
    achievementPointValue: logicSadisticAP,
    HTMLTag: 'sadisticPrequel',
    singularityRequirement: logicSadisticSR,
    scalingrewardcount: 3,
    uniquerewardcount: 4,
    effect: logicSadisticEffect
  },
  taxmanLastStand: {
    baseReq: 240,
    maxCompletions: 10,
    unlockSingularity: 281,
    achievementPointValue: logicTaxmanAP,
    HTMLTag: 'taxmanLastStand',
    singularityRequirement: logicTaxmanSR,
    scalingrewardcount: 5,
    uniquerewardcount: 3,
    effect: logicTaxmanEffect,
    alternateDescription: () => {
      const completions = player.singularityChallenges.taxmanLastStand.completions
      const baseDesc = i18next.t('singularityChallenge.data.taxmanLastStand.description')
      const salvText = i18next.t('singularityChallenge.data.taxmanLastStand.salvageMod')
      const taxText = i18next.t('singularityChallenge.data.taxmanLastStand.taxMod')
      const offText = i18next.t('singularityChallenge.data.taxmanLastStand.offeringMod')
      const obtText = i18next.t('singularityChallenge.data.taxmanLastStand.obtainiumMod')
      let stringText = `${baseDesc}<br>${salvText}<br>${taxText}<br>${offText}<br>${obtText}`

      if (completions >= 2) {
        const capMod = i18next.t('singularityChallenge.data.taxmanLastStand.capMod')
        stringText += `<br>${capMod}`
      }
      if (completions >= 5) {
        const tributeMod = i18next.t('singularityChallenge.data.taxmanLastStand.tributeMod')
        stringText += `<br>${tributeMod}`
      }
      if (completions >= 8) {
        const omegaMod = i18next.t('singularityChallenge.data.taxmanLastStand.omegaMod')
        stringText += `<br>${omegaMod}`
      }
      return stringText
    }
  }
}

export const getSingularityChallengeEffect = <
  T extends SingularityChallengeDataKeys,
  K extends keyof SingularityChallengeRewards[T]
>(challenge: T, key: K): SingularityChallengeRewards[T][K] => {
  const completions = player.singularityChallenges[challenge].completions
  return singularityChallengeData[challenge].effect(completions, key) as SingularityChallengeRewards[T][K]
}

export const maxAPFromChallenges = Object.values(singularityChallengeData).reduce(
  (acc, challenge) => acc + challenge.achievementPointValue(challenge.maxCompletions),
  0
)
