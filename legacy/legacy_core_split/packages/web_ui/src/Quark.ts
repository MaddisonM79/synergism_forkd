import { quarkHandler as logicQuarkHandler } from '@synergism/logic'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { calculateCubeQuarkMultiplier, calculateQuarkMultiplier } from './Calculate'
import { apiBaseUrl } from './Config'
import { getOcteractUpgradeEffect } from './Octeracts'
import { format, player } from './Synergism'

export const quarkHandler = () =>
  logicQuarkHandler({
    research195: player.researches[195],
    researchesSum: player.researches[99]
      + player.researches[100]
      + player.researches[125]
      + player.researches[180]
      + player.researches[195],
    exportQuarkMult: getOcteractUpgradeEffect('octeractExportQuarks', 'exportQuarkMult'),
    quarksTimer: player.quarkstimer,
    cubeMult: calculateCubeQuarkMultiplier()
  })

let bonus = 0
let personalQuarkBonus = 0
let globalQuarkBonus = 0

const recalculateBonus = () => {
  bonus = 100 * (1 + globalQuarkBonus / 100) * (1 + personalQuarkBonus / 100) - 100
}

const updateQuarkUI = (personalBonus: number, globalBonus: number) => {
  const currentBonus = DOMCacheGetOrSet('currentBonus')
  if (personalBonus > 0) {
    currentBonus.innerHTML = i18next.t('settings.quarkBonusExtended', {
      globalBonus,
      personalBonusMult: format(1 + personalBonus / 100, 3, true),
      totalBonus: format(bonus, 2, true)
    })
  } else {
    currentBonus.innerHTML = i18next.t('settings.quarkBonusSimple', { globalBonus })
  }
}

export const getQuarkBonus = () => bonus

export const getGlobalBonus = () => globalQuarkBonus
export const getPersonalBonus = () => personalQuarkBonus

export const setPersonalQuarkBonus = (personalBonus: number) => {
  personalQuarkBonus = personalBonus
  recalculateBonus()

  updateQuarkUI(personalBonus, globalQuarkBonus)
}

const setGlobalQuarkBonus = (globalBonus: number) => {
  globalQuarkBonus = globalBonus
  recalculateBonus()

  updateQuarkUI(personalQuarkBonus, globalBonus)
}

export class QuarkHandler {
  /** Quark amount */
  private QUARKS = 0

  constructor (quarks: number) {
    this.QUARKS = quarks
  }

  /*** Calculates the number of quarks to give with the current bonus. */
  applyBonus (amount: number) {
    return amount * calculateQuarkMultiplier()
  }

  /** Subtracts quarks, as the name suggests. */
  add (amount: number, useBonus: boolean, addToQuarksThisSingularity: boolean) {
    this.QUARKS += useBonus ? this.applyBonus(amount) : amount
    if (addToQuarksThisSingularity) {
      player.quarksThisSingularity += useBonus ? this.applyBonus(amount) : amount
    }
    return this
  }

  /** Add quarks, as suggested by the function's name. */
  sub (amount: number) {
    this.QUARKS -= amount
    if (this.QUARKS < 0) {
      this.QUARKS = 0
    }

    return this
  }

  public toString (val: number): string {
    return format(Math.floor(this.applyBonus(val)), 0, true)
  }

  /**
   * Resets the amount of quarks saved but keeps the bonus amount.
   */
  public reset () {
    this.QUARKS = 0
  }

  valueOf () {
    return this.QUARKS
  }

  [Symbol.toPrimitive] = (t: string) => t === 'number' ? this.QUARKS : null
}

export const refreshQuarkBonus = async () => {
  const response = await fetch(`${apiBaseUrl}/api/v1/quark-bonus`)
  // eslint-disable-next-line no-shadow
  const { bonus } = await response.json() as { bonus: number }

  setGlobalQuarkBonus(bonus)
}
