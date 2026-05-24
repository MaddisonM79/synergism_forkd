import type Decimal from 'break_infinity.js'
import { calculateAntSacrificeTalismanItem as logicCalculateAntSacrificeTalismanItem } from '@synergism/logic'
import type { TalismanCraftItems } from '../../../../../Talismans'

export const calculateAntSacrificeTalismanItem = (
  item: TalismanCraftItems,
  elo: number,
  rewardMult: Decimal,
  stageMult: number
): Decimal => logicCalculateAntSacrificeTalismanItem({ item, elo, rewardMult, stageMult })
