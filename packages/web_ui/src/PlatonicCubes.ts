import {
  calculateAscensionScorePlatonicBlessing as logicCalcAscensionScore,
  calculateCubeMultiplierPlatonicBlessing as logicCalcCubeMult,
  calculateGlobalSpeedPlatonicBlessing as logicCalcGlobalSpeed,
  calculateHypercubeBlessingMultiplierPlatonicBlessing as logicCalcHypercubeBlessingMult,
  calculateHypercubeMultiplierPlatonicBlessing as logicCalcHypercubeMult,
  calculatePlatonicMultiplierPlatonicBlessing as logicCalcPlatonicMult,
  calculateTaxPlatonicBlessing as logicCalcTax,
  calculateTesseractMultiplierPlatonicBlessing as logicCalcTesseractMult
} from '@synergism/logic'
import { player } from './Synergism'

// Thin shims over @synergism/logic's pure platonic-blessing calculators.
// Each delegates a single read of `player.platonicBlessings` to the logic
// function. Call sites in Statistics, UpdateVisuals, Hypercubes, etc. keep
// invoking these as zero-arg helpers.

export const calculateCubeMultiplierPlatonicBlessing = () => logicCalcCubeMult(player.platonicBlessings)
export const calculateTesseractMultiplierPlatonicBlessing = () => logicCalcTesseractMult(player.platonicBlessings)
export const calculateHypercubeMultiplierPlatonicBlessing = () => logicCalcHypercubeMult(player.platonicBlessings)
export const calculatePlatonicMultiplierPlatonicBlessing = () => logicCalcPlatonicMult(player.platonicBlessings)
export const calculateHypercubeBlessingMultiplierPlatonicBlessing = () =>
  logicCalcHypercubeBlessingMult(player.platonicBlessings)
export const calculateTaxPlatonicBlessing = () => logicCalcTax(player.platonicBlessings)
export const calculateAscensionScorePlatonicBlessing = () => logicCalcAscensionScore(player.platonicBlessings)
export const calculateGlobalSpeedPlatonicBlessing = () => logicCalcGlobalSpeed(player.platonicBlessings)
