import type { Decimal } from '../math/bignum'
import type { SweepStates } from '../tick/challengeSweep'

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
      /** Full SweepState transitioned out of — handler routes
       * resetCheck('transcensionChallenge' | 'reincarnationChallenge')
       * by `from.index` when from.kind === 'active'. */
      from: SweepStates
      /** Full SweepState transitioned into — handler picks
       * toggleAutoChallengeModeText by `to.kind`, and additionally calls
       * toggleChallenges(to.index, true) when to.kind === 'active'. */
      to: SweepStates
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
  | {
      kind: 'ambrosia-gained'
      /** Total ambrosia gained this tick (sum across all loop iterations). */
      amount: number
    }
  | {
      kind: 'red-ambrosia-gained'
      /** Total red ambrosia gained this tick. */
      amount: number
    }
  | {
      kind: 'octeract-tick-fired'
      /** Number of integer 1-second giveaway buckets that crossed this tick.
       * Always ≥ 1 when emitted (the event only fires when at least one
       * giveaway-second elapsed; otherwise no refresh is needed). */
      amountOfGiveaways: number
    }
  | {
      kind: 'auto-potion-fired'
      /** Which side of the auto-potion dispense fired. The UI dispatcher
       * maps to `useConsumable('offeringPotion' | 'obtainiumPotion', ...)`. */
      type: 'offering' | 'obtainium'
      /** Number of potions to dispense this tick (timer crossed threshold
       * `amount` times). Always ≥ 1 when emitted. */
      amount: number
      /** Whether fast mode was active for this dispense — corresponds to
       * the `spend` arg of `useConsumable`. When `false`, `useConsumable`
       * skips the shopUpgrades count decrement. */
      fastMode: boolean
    }
  | {
      kind: 'ant-sacrifice-triggered'
      // Emitted by checkAntSacrificeReady when canAutoSacrifice's conditions
      // are met this tick. UI dispatcher invokes sacrificeAnts() which fans
      // out into resetAnts, talisman/achievement awards, and history record
      // (all un-migrated). No payload — the event is a pure intent signal;
      // sacrificeAnts() re-reads the latest player state itself.
    }
  | {
      kind: 'rune-sacrifice-triggered'
      // Emitted by advanceRuneSacrifice when the sacrifice timer crosses
      // autoSacrificeInterval and offerings > 0. UI dispatcher runs the
      // blessing/spirit/talisman/per-rune-or-all purchase fan-out and
      // recalculates the autoSacrificeInterval cache. No payload — the
      // dispatcher re-reads the latest player state itself (singularity
      // count gates, cube upgrade gates, autoSacrifice rune index, etc.).
    }
  | {
      kind: 'auto-research-manual-requested'
      // Emitted by processAutoResearchTick when autoResearchToggle is on,
      // autoResearch > 0, and mode === 'manual'. UI dispatcher calls
      // buyResearch(autoResearch, true, false) + updateResearchAuto. No
      // payload — the dispatcher reads player.autoResearch itself.
    }
  | {
      kind: 'auto-research-roomba-requested'
      // Emitted by processAutoResearchTick when the Roomba gates pass
      // (autoResearchToggle, autoResearch > 0, roombaUnlocked, mode ===
      // 'cheapest'). UI dispatcher runs the bounded while-loop in
      // runRoombaResearchSweep(maxCount).
      /** Max iterations for the Roomba sweep this tick — computed as
       * `1 + Math.floor(CalcECC('ascension', challengecompletions[14]))`. */
      maxCount: number
    }
  | {
      kind: 'obtainium-multiplier-recompute-requested'
      // Emitted by tackMiddle's obtainium branch when `research61 !== 1`,
      // mirroring the legacy `else { calculateObtainium() }` arm of the
      // research61 gate in tack. The legacy call discarded its return —
      // it's a vestigial "warm the calc" side-effect path. UI dispatcher
      // invokes calculateObtainium() (in Calculate.ts) to preserve the
      // existing behavior.
    }
