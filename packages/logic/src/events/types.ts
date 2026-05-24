import type { Decimal } from '../math/bignum'

// Discriminated union of events the logic core emits for the UI tier to
// react to. UI subscribers translate these into user-facing effects
// (notifications, sounds, animations, achievement awards, etc.).
//
// The union grows as mechanics migrate. Each event should carry enough data
// for the UI to render its effect without re-reading GameState — that keeps
// the boundary one-way.
export type CoreEvent =
  | {
      kind: 'accelerators-purchased'
      before: number
      after: number
      spent: Decimal
    }
  | {
      kind: 'multipliers-purchased'
      before: number
      after: number
      spent: Decimal
    }
  | {
      kind: 'particle-buildings-purchased'
      /** Which of the five particle buildings was purchased (1 = first, 5 = fifth). */
      index: 1 | 2 | 3 | 4 | 5
      before: number
      after: number
      spent: Decimal
    }
  | {
      kind: 'crystal-upgrade-purchased'
      /** 1-based crystal upgrade index. */
      i: number
      before: number
      after: number
      /** prestigeShards spent (zero when triggered by an autobuyer — autobuyers get the levels free). */
      spent: Decimal
    }
  | {
      kind: 'upgrade-purchased'
      /** Resource tier the upgrade was bought against. */
      tier: 'coin' | 'prestige' | 'transcend' | 'reincarnation'
      /** Upgrade index within the bitmap (1-based; pos=0 is the historical sentinel). */
      pos: number
      spent: Decimal
    }
  | {
      kind: 'producers-purchased'
      /** Producer family that was purchased — see ProducerType in mechanics/producers.ts. */
      type: 'Coin' | 'Diamonds' | 'Mythos' | 'Particles'
      /** Position within the family (1 = first, 5 = fifth). */
      index: 1 | 2 | 3 | 4 | 5
      before: number
      after: number
      spent: Decimal
    }
  | {
      kind: 'tesseract-buildings-purchased'
      /** Which of the five ascension-tier (tesseract) buildings (1..5). */
      index: 1 | 2 | 3 | 4 | 5
      before: number
      after: number
      /** wowTesseracts spent (plain number — WowTesseracts wrapper stays in web_ui). */
      spent: number
    }
  // ─── Tick events ──────────────────────────────────────────────────────────
  // Emitted by the migrated tick body. UI subscribers translate these into the
  // side effects the legacy in-place tick performed directly (revealStuff,
  // achievement notifications, visual research/ant/octeract refreshes, etc.).
  | {
      kind: 'resources-gained'
      /** Per-tick coin gain (after taxdivisor + maxexponent clamp). Zero if produceTotal < 0.001. */
      coins: Decimal
      /** Per-tick prestige-point gain from upgrade 93 (zero otherwise). */
      prestigePoints: Decimal
      /** Per-tick transcend-point gain from upgrade 100 (zero otherwise). */
      transcendPoints: Decimal
      /** Per-tick reincarnation-point gain from cubeUpgrade 28 (zero otherwise). */
      reincarnationPoints: Decimal
      /** Per-tick prestigeShard gain from diamond production (zero in t-chal 3 / r-chal 10). */
      prestigeShards: Decimal
      /** Per-tick transcendShard gain from mythos production (zero in t-chal 3 / r-chal 10). */
      transcendShards: Decimal
      /** Per-tick reincarnationShard gain from particle production. */
      reincarnationShards: Decimal
      /** Per-tick ascendShard gain from the first ascension building. */
      ascendShards: Decimal
    }
  | {
      kind: 'auto-reset-triggered'
      /** Which reset tier auto-fired this tick. */
      tier: 'prestige' | 'transcension' | 'reincarnation' | 'ascension'
      /** Whether the threshold check was point-amount based or wall-clock based. */
      mode: 'amount' | 'time'
    }
  | {
      kind: 'achievement-group-awarded'
      /** Group key passed to awardAchievementGroup() — e.g. 'constant', 'antCrumbs'. */
      group: string
    }
  | {
      kind: 'auto-tool-fired'
      /** Which automaticTools branch fired. */
      tool: 'runeSacrifice' | 'antSacrifice' | 'addObtainium' | 'addOfferings'
    }
  | {
      kind: 'challenge-sweep-transitioned'
      /** SweepState kind transitioned out of. */
      from: string
      /** SweepState kind transitioned into. */
      to: string
    }
  | {
      kind: 'reveal-needed'
      /** Which legacy revealStuff() trigger fired — names mirror the four checks in resourceGain. */
      trigger: 'coinone' | 'cointwo' | 'cointhree' | 'coinfour'
    }
  | {
      kind: 'challenge-auto-completed'
      /** challengecompletions index that was just incremented (1..5). */
      challengeIndex: 1 | 2 | 3 | 4 | 5
      /** New completion count after increment. */
      newCompletions: number
    }
