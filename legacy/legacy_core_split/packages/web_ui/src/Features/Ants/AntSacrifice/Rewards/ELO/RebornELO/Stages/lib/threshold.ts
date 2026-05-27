import {
  calculateLeftoverRebornELO as logicCalculateLeftoverRebornELO,
  calculateRebornELOThresholds as logicCalculateRebornELOThresholds,
  calculateStageRebornSpeedMult as logicCalculateStageRebornSpeedMult,
  perRebornELOStageModifiers,
  quarkMultiplierPerRebornELOThreshold,
  rebornELOStageModifiers as logicRebornELOStageModifiers,
  rebornELOThresholdTranches,
  calculateToNextRebornELOThreshold as logicCalculateToNextRebornELOThreshold
} from '@synergism/logic'
import { player } from '../../../../../../../../Synergism'

// Re-export the logic-side constants so existing call sites that import
// these names from this module keep compiling unchanged.
export const thresholdTranches = rebornELOThresholdTranches
export const quarkMultiplierPerThreshold = quarkMultiplierPerRebornELOThreshold

export const calculateStageRebornSpeedMult = () =>
  logicCalculateStageRebornSpeedMult(player.singularityCount)

export const calculateRebornELOThresholds = (elo?: number) =>
  logicCalculateRebornELOThresholds(elo ?? player.ants.rebornELO)

export const calculateToNextELOThreshold = (rebornELO: number, stage?: number) =>
  logicCalculateToNextRebornELOThreshold(rebornELO, stage)

export const calculateLeftoverELO = (rebornELO: number, stage?: number) =>
  logicCalculateLeftoverRebornELO(rebornELO, stage)

export const thresholdModifiers = () =>
  logicRebornELOStageModifiers({
    rebornELO: player.ants.rebornELO,
    singCount: player.singularityCount
  })

// Kept for any caller that still references the per-stage base modifiers.
export { perRebornELOStageModifiers as perThresholdModifiers }
