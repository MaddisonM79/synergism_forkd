import { Decimal } from '../math/bignum'
import type { CoreEvent } from '../events/types'
import type { CrystalUpgradesState } from '../state/schema'

// Crystal upgrades — a separate upgrade ladder bought with prestigeShards.
// Unlike the discrete buyUpgrades bitmap, each crystal upgrade has an integer
// level that grows whenever the player has shards to spare. The buy formula
// computes the maximum affordable level analytically (no loop), then sets the
// level if it exceeds the current one and deducts the cost (manual path only —
// autobuyer is granted free levels to dodge a late-game precision issue).

export interface BuyCrystalUpgradesInput {
  /** 1-based crystal upgrade index. */
  i: number
  /** True when the autobuyer is driving — skips the prestigeShards deduction. */
  auto: boolean
  /** Prism rune cost-divisor (log10) — getRuneEffects('prism', 'costDivisorLog10'). */
  prismCostDivisorLog10: number
  /** Base cost (log10) for this upgrade — G.crystalUpgradesCost[i-1]. */
  crystalUpgradesCost: number
  /** Cost growth (log10) per level — G.crystalUpgradeCostIncrement[i-1]. */
  crystalUpgradeCostIncrement: number
  /** player.upgrades[73] — gates the +10 bonus when also in a reincarnation challenge. */
  upgrade73: number
  /** player.currentChallenge.reincarnation !== 0 — gates the +10 bonus. */
  inAnyReincarnationChallenge: boolean
}

// Internal — closed-form solve for "max level affordable with current shards".
// The cumulative cost to reach level n is roughly
//   crystalUpgradesCost + crystalUpgradeCostIncrement * (n*(n-1)/2)
// in log10. Invert to find n given log10(prestigeShards + 1).
function calculateCrystalBuy(
  prestigeShards: Decimal,
  prismCostDivisorLog10: number,
  crystalUpgradesCost: number,
  crystalUpgradeCostIncrement: number
): number {
  const exponent = Decimal.log(prestigeShards.add(1), 10)
  return Math.floor(
    Math.pow(
      Math.max(
        0,
        2 * (exponent + prismCostDivisorLog10 - crystalUpgradesCost) / crystalUpgradeCostIncrement + 1 / 4
      ),
      1 / 2
    )
      + 1 / 2
  )
}

export function buyCrystalUpgrades(
  state: CrystalUpgradesState,
  input: BuyCrystalUpgradesInput
): { state: CrystalUpgradesState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: CrystalUpgradesState = {
    prestigeShards: new Decimal(state.prestigeShards),
    crystalUpgrades: [...state.crystalUpgrades]
  }
  const u = input.i - 1

  // Bonus levels: +10 when player owns upgrade 73 AND is currently inside any
  // reincarnation challenge. The bonus levels do NOT contribute to the cost.
  let c = 0
  if (input.upgrade73 > 0.5 && input.inAnyReincarnationChallenge) {
    c += 10
  }

  const toBuy = calculateCrystalBuy(
    next.prestigeShards,
    input.prismCostDivisorLog10,
    input.crystalUpgradesCost,
    input.crystalUpgradeCostIncrement
  )

  const before = next.crystalUpgrades[u]
  const target = toBuy + c

  if (target > before) {
    // Preserved verbatim: `100 / 100 * (toBuy + c)` is a legacy no-op
    // multiplier; matches the original byte-for-byte.
    next.crystalUpgrades[u] = 100 / 100 * target
    const startingShards = new Decimal(next.prestigeShards)
    if (toBuy > 0 && !input.auto) {
      next.prestigeShards = next.prestigeShards.sub(
        Decimal.pow(
          10,
          input.crystalUpgradesCost - input.prismCostDivisorLog10
            + input.crystalUpgradeCostIncrement * (1 / 2 * Math.pow(toBuy - 1 / 2, 2) - 1 / 8)
        )
      )
      next.prestigeShards = next.prestigeShards.max(0)
    }
    events.push({
      kind: 'crystal-upgrade-purchased',
      i: input.i,
      before,
      after: next.crystalUpgrades[u],
      spent: startingShards.sub(next.prestigeShards)
    })
  }

  return { state: next, events }
}
