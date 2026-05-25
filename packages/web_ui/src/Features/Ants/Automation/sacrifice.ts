import i18next from 'i18next'
import { format, player } from '../../../Synergism'
import { antSacrificeRewards } from '../AntSacrifice/Rewards/calculate-rewards'
import { AutoSacrificeModes } from '../toggles/structs/sacrifice'
import type { AutoSacrificeModeData } from './structs/structs'

export const autoSacrificeData: Record<AutoSacrificeModes, AutoSacrificeModeData> = {
  [AutoSacrificeModes.InGameTime]: {
    modeName: () => i18next.t('ants.autoSacrifice.inGameTimer.name'),
    infoText: () =>
      i18next.t('ants.autoSacrifice.inGameTimer.info', {
        curr: format(player.antSacrificeTimer, 2, true),
        req: format(player.ants.toggles.autoSacrificeThreshold, 0, true)
      }),
    modeHTMLcolor: 'var(--lightseagreen-text-color)'
  },
  [AutoSacrificeModes.RealTime]: {
    modeName: () => i18next.t('ants.autoSacrifice.realLifeTimer.name'),
    infoText: () =>
      i18next.t('ants.autoSacrifice.realLifeTimer.info', {
        curr: format(player.antSacrificeTimerReal, 2, true),
        req: format(player.ants.toggles.autoSacrificeThreshold, 0, true)
      }),
    modeHTMLcolor: 'lightgray'
  },
  [AutoSacrificeModes.ImmortalELOGain]: {
    modeName: () => i18next.t('ants.autoSacrifice.immortalELOGain.name'),
    infoText: () =>
      i18next.t('ants.autoSacrifice.immortalELOGain.info', {
        curr: format(antSacrificeRewards().immortalELO, 0, true),
        req: format(player.ants.toggles.autoSacrificeThreshold, 0, true)
      }),
    modeHTMLcolor: 'crimson'
  },
  [AutoSacrificeModes.MaxRebornELO]: {
    modeName: () => i18next.t('ants.autoSacrifice.maxRebornELO.name'),
    infoText: () => {
      const rebornELOToGain = player.ants.immortalELO - player.ants.rebornELO
      return i18next.t('ants.autoSacrifice.maxRebornELO.info', {
        curr: format(rebornELOToGain, 0, true)
      })
    },
    modeHTMLcolor: '#00DDFF'
  }
}
