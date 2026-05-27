import { computeFreeAntUpgradeLevels as logicComputeFreeAntUpgradeLevels } from '@synergism/logic'
import { getAchievementReward } from '../../../../Achievements'
import { CalcECC } from '../../../../Challenges'
import { player } from '../../../../Synergism'
import { Globals } from '../../../../Variables'

export const computeFreeAntUpgradeLevels = (): number =>
  logicComputeFreeAntUpgradeLevels({
    c9ReincarnationECC: CalcECC('reincarnation', player.challengecompletions[9]),
    constantUpgrade6: player.constantUpgrades[6],
    c11AscensionECC: CalcECC('ascension', player.challengecompletions[11]),
    research97: player.researches[97],
    research98: player.researches[98],
    research102: player.researches[102],
    research132: player.researches[132],
    research200: player.researches[200],
    freeAntUpgradesAchievementReward: +getAchievementReward('freeAntUpgrades'),
    challenge15BonusAntLevelValue: Globals.challenge15Rewards.bonusAntLevel.value,
    c11Active: player.currentChallenge.ascension === 11,
    c8Completions: player.challengecompletions[8],
    c9Completions: player.challengecompletions[9]
  })
