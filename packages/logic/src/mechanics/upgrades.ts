import { Decimal } from '../math/bignum'
import type { CoreEvent } from '../events/types'
import type { UpgradesState } from '../state/schema'

// Upgrade purchases keyed by resource tier (coin / prestige / transcend /
// reincarnation). Each tier dispatches to its own currency and, on a
// successful buy, flips a different set of `*no*upgrades` achievement-gate
// flags. The flag-flip happens unconditionally per call (not gated on
// affordability) — matches the original buyUpgrades behavior.

export type UpgradeTier = 'coin' | 'prestige' | 'transcend' | 'reincarnation'

export interface BuyUpgradeInput {
  tier: UpgradeTier
  /** Upgrade index in the bitmap (0..N). 1-based per `upgradeRequirements` convention. */
  pos: number
  /** log10 of the cost in the tier's currency — actual cost is 10^costExponent. */
  costExponent: number
  /**
   * Mirror of the original `!upgradeRequirements[pos]` guard. The OLD code
   * checked whether the requirement *function* existed (out-of-bounds guard)
   * rather than calling it. All current entries are () => true, so this is
   * effectively a bounds check; callers should pass
   * `upgradeRequirements[pos] !== undefined`.
   */
  requirementExists: boolean
}

const RESOURCE_BY_TIER: Record<UpgradeTier, 'coins' | 'prestigePoints' | 'transcendPoints' | 'reincarnationPoints'> = {
  coin: 'coins',
  prestige: 'prestigePoints',
  transcend: 'transcendPoints',
  reincarnation: 'reincarnationPoints'
}

export function buyUpgrades(
  state: UpgradesState,
  input: BuyUpgradeInput
): { state: UpgradesState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: UpgradesState = {
    coins: new Decimal(state.coins),
    prestigePoints: new Decimal(state.prestigePoints),
    transcendPoints: new Decimal(state.transcendPoints),
    reincarnationPoints: new Decimal(state.reincarnationPoints),
    upgrades: [...state.upgrades],
    prestigenocoinupgrades: state.prestigenocoinupgrades,
    transcendnocoinupgrades: state.transcendnocoinupgrades,
    transcendnocoinorprestigeupgrades: state.transcendnocoinorprestigeupgrades,
    reincarnatenocoinupgrades: state.reincarnatenocoinupgrades,
    reincarnatenocoinorprestigeupgrades: state.reincarnatenocoinorprestigeupgrades,
    reincarnatenocoinprestigeortranscendupgrades: state.reincarnatenocoinprestigeortranscendupgrades,
    reincarnatenocoinprestigetranscendorgeneratorupgrades: state.reincarnatenocoinprestigetranscendorgeneratorupgrades
  }

  // Out-of-bounds guard from the original buyUpgrades. Returns the cloned
  // state untouched (no flag flips, no purchase) when the requirement entry
  // doesn't exist.
  if (!input.requirementExists) return { state: next, events }

  const currencyKey = RESOURCE_BY_TIER[input.tier]
  const cost = Decimal.pow(10, input.costExponent)

  // Purchase attempt. Mirrors the OLD guard exactly: affordable AND not
  // already owned. On success, deduct cost, set bitmap entry, emit event.
  if (next[currencyKey].gte(cost) && next.upgrades[input.pos] === 0) {
    next[currencyKey] = next[currencyKey].sub(cost)
    next.upgrades[input.pos] = 1
    events.push({
      kind: 'upgrade-purchased',
      tier: input.tier,
      pos: input.pos,
      spent: cost
    })
  }

  // Flag-flip matrix — independent of buy success. Match the original:
  //   coin    → 7 flags
  //   prestige → 4 flags
  //   transcend → 2 flags
  //   reincarnation → no flips
  if (input.tier === 'transcend') {
    next.reincarnatenocoinprestigeortranscendupgrades = false
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  if (input.tier === 'prestige') {
    next.transcendnocoinorprestigeupgrades = false
    next.reincarnatenocoinorprestigeupgrades = false
    next.reincarnatenocoinprestigeortranscendupgrades = false
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  if (input.tier === 'coin') {
    next.prestigenocoinupgrades = false
    next.transcendnocoinupgrades = false
    next.transcendnocoinorprestigeupgrades = false
    next.reincarnatenocoinupgrades = false
    next.reincarnatenocoinorprestigeupgrades = false
    next.reincarnatenocoinprestigeortranscendupgrades = false
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }

  return { state: next, events }
}
