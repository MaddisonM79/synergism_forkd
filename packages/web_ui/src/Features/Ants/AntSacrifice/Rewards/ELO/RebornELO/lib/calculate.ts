import {
  calculateAvailableRebornELO as logicCalculateAvailableRebornELO,
  calculateStageRebornSpeedMult as logicCalculateStageRebornSpeedMult,
  calculateTotalProductionForRebornELO as logicCalculateTotalProductionForRebornELO
} from '@synergism/logic'
import { rebornELOCreationSpeedMultStats } from '../../../../../../../Statistics'
import { player } from '../../../../../../../Synergism'
import { thresholdModifiers } from '../Stages/lib/threshold'

export const calculateAvailableRebornELO = () =>
  logicCalculateAvailableRebornELO({
    immortalELO: player.ants.immortalELO,
    rebornELO: player.ants.rebornELO
  })

export const rebornELOCreationSpeedMult = () => {
  return rebornELOCreationSpeedMultStats.reduce((a, b) => a * b.stat(), 1)
}

export const calculateSecondsToMaxRebornELO = () => {
  const stageMod = thresholdModifiers().rebornSpeedMult
  const baseProductionPerSecond = rebornELOCreationSpeedMult() / stageMod

  const stageRebornSpeedMult = logicCalculateStageRebornSpeedMult(player.singularityCount)
  const discountRequiredProduction = logicCalculateTotalProductionForRebornELO({
    rebornELO: player.ants.rebornELO,
    stageRebornSpeedMult
  })
  const totalRequiredProduction = logicCalculateTotalProductionForRebornELO({
    rebornELO: player.ants.immortalELO,
    stageRebornSpeedMult
  })

  return (totalRequiredProduction - discountRequiredProduction) / baseProductionPerSecond
}
