// Talisman rarity / level math, migrated from packages/web_ui/src/Talismans.ts.
// The talisman data table (i18n closures, isUnlocked predicates, baseMult/
// maxLevel constants) stays in web_ui — this module owns the pure formulas
// that map (level, maxLevel, rarity) → display rarity, the levels-until-next-
// rarity counter, and the per-level affordability check.

import type { Decimal } from '../math/bignum'
import type { TalismanCraftCosts, TalismanCraftItems } from './talismanCosts'

// ─── Rarity value table ────────────────────────────────────────────────────

/**
 * Rune-bonus multipliers applied per rarity tier. Rarity 0 means "talisman
 * locked" and contributes nothing; rarities 1–10 are the displayed rarity
 * tiers (Common → Mythic), with stacking bonuses above rarity 7 awarded by
 * crossing 2x, 4x, 8x the talisman's `maxLevel`.
 */
export const rarityValues: Readonly<Record<number, number>> = Object.freeze({
  0: 0,
  1: 1,
  2: 1.2,
  3: 1.5,
  4: 1.8,
  5: 2.1,
  6: 2.5,
  7: 3,
  8: 3.25,
  9: 3.5,
  10: 4
})

// ─── Rarity from level ─────────────────────────────────────────────────────

export interface ComputeTalismanRarityInput {
  /** `talismans[t].isUnlocked()`. When false, rarity is forced to 0. */
  isUnlocked: boolean
  /** `talismans[t].level`. */
  level: number
  /**
   * `talismans[t].maxLevel`. NOT the cap including levelCapIncrease — the
   * raw `maxLevel` constant on the talisman data table. The rarity tier
   * formula uses ratios of this value (level / maxLevel ≥ 1, 2, 4, 8).
   */
  maxLevel: number
}

/**
 * Display rarity for a talisman, 0–10. Locked talismans get 0. Unlocked
 * talismans get `1 + min(6, floor(6 * level / maxLevel)) + extraRarity`,
 * where `extraRarity` adds +1 for each of the 2×, 4×, and 8× `maxLevel`
 * thresholds the talisman has crossed.
 */
export function computeTalismanRarity (input: ComputeTalismanRarityInput): number {
  if (!input.isUnlocked) {
    return 0
  }
  const levelRatio = input.level / input.maxLevel
  let extraRarity = 0
  if (levelRatio >= 1) {
    if (levelRatio >= 2) extraRarity += 1
    if (levelRatio >= 4) extraRarity += 1
    if (levelRatio >= 8) extraRarity += 1
  }
  return 1 + Math.min(6, Math.floor(6 * levelRatio)) + extraRarity
}

// ─── Levels until next rarity tier ─────────────────────────────────────────

export interface LevelsUntilTalismanRarityIncreaseInput {
  /** `talismans[t].level`. */
  level: number
  /** `talismans[t].maxLevel`. */
  maxLevel: number
  /** `talismans[t].rarity` — current rarity tier. */
  currentRarity: number
  /** `getTalismanLevelCap(t)` — `maxLevel + levelCapIncrease()`. */
  levelCap: number
}

/**
 * Levels remaining until the next rarity tier triggers. Once `level` reaches
 * `maxLevel` the rarity stops ratcheting via the level-ratio thresholds (the
 * 2×/4×/8× extras still fire, but this helper ignores them — UI just buys
 * up to the cap once you're past the maxLevel mark).
 */
export function levelsUntilTalismanRarityIncrease (
  input: LevelsUntilTalismanRarityIncreaseInput
): number {
  if (input.level >= input.maxLevel) {
    return input.levelCap - input.level
  }
  const levelReq = Math.ceil(input.maxLevel * input.currentRarity / 6)
  return levelReq - input.level
}

// ─── Affordability check ───────────────────────────────────────────────────

export interface AffordableTalismanLevelInput {
  /** Per-item cost for the next level — output of the talisman's cost progression. */
  costs: TalismanCraftCosts
  /**
   * Per-item budget available. Same keys as `costs`. For real purchases this
   * is the player's owned fragments; during save-loading it's the saved
   * `fragmentsInvested` snapshot.
   */
  budget: Record<TalismanCraftItems, Decimal>
  /**
   * Floating-point cushion applied to the budget. The web_ui caller uses
   * 1.0001 when re-deriving level from invested fragments after a save load
   * (compensating for Decimal round-trip imprecision); 1 for live purchases.
   */
  bufferMult: number
}

/**
 * Returns true iff every item in `costs` is ≤ `budget[item] * bufferMult`.
 * Walks the cost map directly so unused keys (e.g. tier-locked fragments at
 * zero cost) don't affect the result — they're trivially satisfied.
 */
export function affordableTalismanLevel (input: AffordableTalismanLevelInput): boolean {
  for (const item in input.costs) {
    const key = item as TalismanCraftItems
    if (input.costs[key].gt(input.budget[key].times(input.bufferMult))) {
      return false
    }
  }
  return true
}

// ─── Sum of rarities ───────────────────────────────────────────────────────

/**
 * Sum of all talisman rarities. Used by the achievement-points formula for
 * the rarity-based progressive achievement. Trivial reduce, lifted to logic
 * so the web_ui side doesn't have to iterate the talismans map every time.
 */
export function sumOfTalismanRarities (rarities: readonly number[]): number {
  let sum = 0
  for (const r of rarities) {
    sum += r
  }
  return sum
}
