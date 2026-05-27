import {
  calculateAcceleratorHypercubeBlessing as logicCalcAccelerator,
  calculateAntELOHypercubeBlessing as logicCalcAntELO,
  calculateAntSacrificeHypercubeBlessing as logicCalcAntSacrifice,
  calculateAntSpeedHypercubeBlessing as logicCalcAntSpeed,
  calculateGlobalSpeedHypercubeBlessing as logicCalcGlobalSpeed,
  calculateHypercubeBlessingMultiplierPlatonicBlessing as logicCalcHypercubeBlessingMultPlatonic,
  calculateMultiplierHypercubeBlessing as logicCalcMultiplier,
  calculateObtainiumHypercubeBlessing as logicCalcObtainium,
  calculateOfferingHypercubeBlessing as logicCalcOffering,
  calculateRuneEffectivenessHypercubeBlessing as logicCalcRuneEffectiveness,
  calculateSalvageHypercubeBlessing as logicCalcSalvage
} from '@synergism/logic'
import { player } from './Synergism'

// Thin shims over @synergism/logic's pure hypercube-blessing calculators.
// Eight of the ten take a precomputed platonic-amplifier number — the shim
// recomputes it from player.platonicBlessings on every call (same semantics
// as the OLD implementation, which called the platonic getter in-line per
// hypercube-blessing function).

const amp = () => logicCalcHypercubeBlessingMultPlatonic(player.platonicBlessings)

export const calculateAcceleratorHypercubeBlessing = () => logicCalcAccelerator(player.hypercubeBlessings, amp())
export const calculateMultiplierHypercubeBlessing = () => logicCalcMultiplier(player.hypercubeBlessings, amp())
export const calculateOfferingHypercubeBlessing = () => logicCalcOffering(player.hypercubeBlessings, amp())
export const calculateObtainiumHypercubeBlessing = () => logicCalcObtainium(player.hypercubeBlessings, amp())
export const calculateAntSpeedHypercubeBlessing = () => logicCalcAntSpeed(player.hypercubeBlessings, amp())
export const calculateAntSacrificeHypercubeBlessing = () => logicCalcAntSacrifice(player.hypercubeBlessings, amp())
export const calculateRuneEffectivenessHypercubeBlessing = () =>
  logicCalcRuneEffectiveness(player.hypercubeBlessings, amp())
export const calculateGlobalSpeedHypercubeBlessing = () => logicCalcGlobalSpeed(player.hypercubeBlessings, amp())
export const calculateSalvageHypercubeBlessing = () => logicCalcSalvage(player.hypercubeBlessings)
export const calculateAntELOHypercubeBlessing = () => logicCalcAntELO(player.hypercubeBlessings)
