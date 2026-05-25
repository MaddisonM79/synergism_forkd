import {
  baseQuarksFromRebornELOStages as logicBaseQuarksFromRebornELOStages,
  calculateRebornELOThresholds as logicCalculateRebornELOThresholds,
  quarksFromELOMult as logicQuarksFromELOMult
} from '@synergism/logic'
import { player } from '../../../../../../../../Synergism'
import { calculateLeaderboardValue } from './calculate-leaderboard'

export const quarksFromELOMult = (): number =>
  logicQuarksFromELOMult(calculateLeaderboardValue(player.ants.highestRebornELOEver))

export const availableQuarksFromELO = (): number => {
  const totalELOValue = calculateLeaderboardValue(player.ants.highestRebornELODaily)
  const numStages = logicCalculateRebornELOThresholds(totalELOValue)
  const { baseQuarks, stageMult } = logicBaseQuarksFromRebornELOStages(numStages)

  let antQuarkMult = quarksFromELOMult()
  antQuarkMult *= stageMult
  antQuarkMult *= (player.autoWarpCheck) ? 1 + player.dailyPowderResetUses : 1
  return Math.max(0, player.worlds.applyBonus(baseQuarks) * antQuarkMult - player.ants.quarksGainedFromAnts)
}
