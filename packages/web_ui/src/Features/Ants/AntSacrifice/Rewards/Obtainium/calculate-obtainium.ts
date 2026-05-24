import { calculateAntSacrificeObtainium as logicCalculateAntSacrificeObtainium } from '@synergism/logic'
import type Decimal from 'break_infinity.js'
import { calculateAntSacrificeMultiplier, calculateObtainium } from '../../../../../Calculate'
import { offeringObtainiumTimeModifiers } from '../../../../../Statistics'
import { player } from '../../../../../Synergism'

export const calculateAntSacrificeObtainium = (stageMult: number, useTime = true): Decimal => {
  const timeMultiplier = offeringObtainiumTimeModifiers(player.antSacrificeTimer, useTime).reduce(
    (a, b) => a * b.stat(),
    1
  )
  return logicCalculateAntSacrificeObtainium({
    antSacMult: calculateAntSacrificeMultiplier(),
    stageMult,
    timeMultiplier,
    obtainiumMult: calculateObtainium(false),
    currentObtainium: player.obtainium,
    taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled,
    taxmanLastStandCompletions: player.singularityChallenges.taxmanLastStand.completions
  })
}
