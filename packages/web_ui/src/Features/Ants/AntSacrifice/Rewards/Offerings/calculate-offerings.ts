import { calculateAntSacrificeOffering as logicCalculateAntSacrificeOffering } from '@synergism/logic'
import type Decimal from 'break_infinity.js'
import { calculateAntSacrificeMultiplier, calculateOfferings } from '../../../../../Calculate'
import { offeringObtainiumTimeModifiers } from '../../../../../Statistics'
import { player } from '../../../../../Synergism'

export const calculateAntSacrificeOffering = (stageMult: number): Decimal => {
  const timeMultiplier = offeringObtainiumTimeModifiers(player.antSacrificeTimer, true).reduce(
    (a, b) => a * b.stat(),
    1
  )
  return logicCalculateAntSacrificeOffering({
    antSacMult: calculateAntSacrificeMultiplier(),
    stageMult,
    timeMultiplier,
    offeringMult: calculateOfferings(false),
    currentOfferings: player.offerings,
    taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled,
    taxmanLastStandCompletions: player.singularityChallenges.taxmanLastStand.completions
  })
}
