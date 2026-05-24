import { logicGenerateAntsAndCrumbs } from '@synergism/logic'
import { calculateActualAntSpeedMult } from '../../../../Calculate'
import { player } from '../../../../Synergism'
import { activateELO } from '../../AntSacrifice/Rewards/ELO/RebornELO/lib/create-reborn'
import { AntProducers, LAST_ANT_PRODUCER } from '../../structs/structs'

export const canGenerateAntCrumbs = (): boolean => {
  // Cube 5x8 is "Wow! I want to ALWAYS generate Galactic Crumbs."
  return player.challengecompletions[8] > 0 || player.cubeUpgrades[48] > 0
}

export const generateAntsAndCrumbs = (dt: number): void => {
  const antSpeedMult = calculateActualAntSpeedMult()

  // Pack the per-tier state (0..LAST_ANT_PRODUCER) into the logic input.
  const producersInput = []
  for (let i = 0; i <= LAST_ANT_PRODUCER; i++) {
    const ant = i as AntProducers
    producersInput.push({
      generated: player.ants.producers[ant].generated,
      purchased: player.ants.producers[ant].purchased,
      masteryLevel: player.ants.masteries[ant].mastery
    })
  }

  const result = logicGenerateAntsAndCrumbs({
    dt,
    antSpeedMult,
    producers: producersInput,
    crumbs: player.ants.crumbs,
    crumbsThisSacrifice: player.ants.crumbsThisSacrifice,
    crumbsEverMade: player.ants.crumbsEverMade
  })

  // Write back generated values per tier.
  for (let i = 0; i <= LAST_ANT_PRODUCER; i++) {
    player.ants.producers[i as AntProducers].generated = result.producersGenerated[i]
  }
  player.ants.crumbs = result.crumbs
  player.ants.crumbsThisSacrifice = result.crumbsThisSacrifice
  player.ants.crumbsEverMade = result.crumbsEverMade

  // ELO activation has wall-clock + DOM + quark-adding side effects;
  // stays in web_ui for now. Runs in the same order as legacy.
  activateELO(dt)
}
