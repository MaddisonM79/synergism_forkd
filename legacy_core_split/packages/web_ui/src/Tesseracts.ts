import {
  calculateAcceleratorHypercubeBlessing as logicCalcAcceleratorHyper,
  calculateAcceleratorTesseractBlessing as logicCalcAcceleratorTess,
  calculateAntELOHypercubeBlessing as logicCalcAntELOHyper,
  calculateAntELOTesseractBlessing as logicCalcAntELOTess,
  calculateAntSacrificeHypercubeBlessing as logicCalcAntSacrificeHyper,
  calculateAntSacrificeTesseractBlessing as logicCalcAntSacrificeTess,
  calculateAntSpeedHypercubeBlessing as logicCalcAntSpeedHyper,
  calculateAntSpeedTesseractBlessing as logicCalcAntSpeedTess,
  calculateGlobalSpeedHypercubeBlessing as logicCalcGlobalSpeedHyper,
  calculateGlobalSpeedTesseractBlessing as logicCalcGlobalSpeedTess,
  calculateHypercubeBlessingMultiplierPlatonicBlessing as logicCalcHypercubeBlessingMultPlatonic,
  calculateMultiplierHypercubeBlessing as logicCalcMultiplierHyper,
  calculateMultiplierTesseractBlessing as logicCalcMultiplierTess,
  calculateObtainiumHypercubeBlessing as logicCalcObtainiumHyper,
  calculateObtainiumTesseractBlessing as logicCalcObtainiumTess,
  calculateOfferingHypercubeBlessing as logicCalcOfferingHyper,
  calculateOfferingTesseractBlessing as logicCalcOfferingTess,
  calculateRuneEffectivenessHypercubeBlessing as logicCalcRuneEffectivenessHyper,
  calculateRuneEffectivenessTesseractBlessing as logicCalcRuneEffectivenessTess,
  calculateSalvageHypercubeBlessing as logicCalcSalvageHyper,
  calculateSalvageTesseractBlessing as logicCalcSalvageTess
} from '@synergism/logic'
import { player } from './Synergism'

// Thin shims over @synergism/logic's pure tesseract-blessing calculators.
// Each tesseract function takes the matching hypercube-blessing value as its
// amplifier source. The hypercube layer is itself pure but depends on the
// platonic amplifier, so the shim composes the full chain on each call:
//
//   tesseractBlessing
//     ← hypercubeBlessing
//         ← platonicHypercubeBonusAmplifier
//
// Two zero-arg helpers keep the chain DRY without changing public semantics.

const platonicAmp = () => logicCalcHypercubeBlessingMultPlatonic(player.platonicBlessings)
const hyperBlessing = (
  fn: (state: typeof player.hypercubeBlessings, amp: number) => number
) => fn(player.hypercubeBlessings, platonicAmp())

export const calculateAcceleratorTesseractBlessing = () =>
  logicCalcAcceleratorTess(player.tesseractBlessings, hyperBlessing(logicCalcAcceleratorHyper))
export const calculateMultiplierTesseractBlessing = () =>
  logicCalcMultiplierTess(player.tesseractBlessings, hyperBlessing(logicCalcMultiplierHyper))
export const calculateOfferingTesseractBlessing = () =>
  logicCalcOfferingTess(player.tesseractBlessings, hyperBlessing(logicCalcOfferingHyper))
export const calculateObtainiumTesseractBlessing = () =>
  logicCalcObtainiumTess(player.tesseractBlessings, hyperBlessing(logicCalcObtainiumHyper))
export const calculateAntSacrificeTesseractBlessing = () =>
  logicCalcAntSacrificeTess(player.tesseractBlessings, hyperBlessing(logicCalcAntSacrificeHyper))
export const calculateRuneEffectivenessTesseractBlessing = () =>
  logicCalcRuneEffectivenessTess(player.tesseractBlessings, hyperBlessing(logicCalcRuneEffectivenessHyper))
export const calculateGlobalSpeedTesseractBlessing = () =>
  logicCalcGlobalSpeedTess(player.tesseractBlessings, hyperBlessing(logicCalcGlobalSpeedHyper))

// Outliers — the hypercube blessing for each isn't an amplifier but a direct
// multiplier / cap value. The hypercube-side Salvage and AntELO functions are
// amplifier-free; AntSpeed uses the platonic amp like the rest.
export const calculateSalvageTesseractBlessing = () =>
  logicCalcSalvageTess(player.tesseractBlessings, logicCalcSalvageHyper(player.hypercubeBlessings))
export const calculateAntSpeedTesseractBlessing = () =>
  logicCalcAntSpeedTess(player.tesseractBlessings, hyperBlessing(logicCalcAntSpeedHyper))
export const calculateAntELOTesseractBlessing = () =>
  logicCalcAntELOTess(player.tesseractBlessings, logicCalcAntELOHyper(player.hypercubeBlessings))
