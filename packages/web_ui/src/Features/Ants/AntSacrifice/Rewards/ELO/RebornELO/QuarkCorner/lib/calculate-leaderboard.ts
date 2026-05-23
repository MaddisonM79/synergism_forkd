import { calculateLeaderboardValue as logicCalculateLeaderboardValue } from '@synergism/logic'

export const calculateLeaderboardValue = (leaderboard: Array<{ elo: number; sacrificeId: number }>): number =>
  logicCalculateLeaderboardValue(leaderboard)
