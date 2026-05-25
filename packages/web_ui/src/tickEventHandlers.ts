// Central CoreEvent dispatcher for the per-tick body. Replaces the six
// scattered inline dispatchers that were growing across Synergism.ts,
// Helper.ts, and Challenges.ts as more subsystems migrated to logic.
//
// Pattern: every shim that calls into @synergism/logic and receives an
// events[] array dispatches it through `dispatchTickEvent` here. The
// event kinds form a discriminated union (see
// packages/logic/src/events/types.ts); each case maps to the
// corresponding web_ui side effect — async resetCheck, achievement
// awards, visual refreshes, modal triggers, etc.
//
// New event kinds should be added here in one place when the logic
// emits them.

import { type AchievementGroups, awardAchievementGroup, challengeAchievementCheck, resetAchievementCheck } from './Achievements'
import { dispatchSweepTransition } from './Challenges'
import type { CoreEvent } from '@synergism/logic'
import { sacrificeAnts } from './Features/Ants/AntSacrifice/sacrifice'
import { reset } from './Reset'
import { useConsumable } from './Shop'
import { Tabs } from './Tabs'
import { visualUpdateAmbrosia, visualUpdateOcteracts, visualUpdateResearch } from './UpdateVisuals'
import { revealStuff, updateChallengeLevel } from './UpdateHTML'
import { Globals as G } from './Variables'

/**
 * Translate one CoreEvent into its web_ui side effect. No return value —
 * the event was already produced by logic, this just plays its UI
 * counterpart. Switch is exhaustive over the union so adding a new
 * variant to CoreEvent surfaces as a TS error here.
 */
export function dispatchTickEvent (event: CoreEvent): void {
  switch (event.kind) {
    // ─── Purchase events ──────────────────────────────────────────────
    // Emitted by purchase actions (acceleratorBoosts, multipliers, etc.).
    // Not part of the tick body's event flow today — kept here so the
    // central dispatcher is exhaustive over CoreEvent.
    case 'accelerators-purchased':
    case 'multipliers-purchased':
    case 'particle-buildings-purchased':
    case 'crystal-upgrade-purchased':
    case 'upgrade-purchased':
    case 'producers-purchased':
    case 'tesseract-buildings-purchased':
      return

    // ─── resourceGain events ─────────────────────────────────────────
    case 'resources-gained':
      // Reserved for future use; resourceGain doesn't emit this yet.
      return
    case 'achievement-group-awarded':
      awardAchievementGroup(event.group as AchievementGroups)
      return
    case 'challenge-auto-completed':
      challengeAchievementCheck(event.challengeIndex)
      updateChallengeLevel(event.challengeIndex)
      return
    case 'reveal-needed':
      // Reserved for future resourceGain reveals (coinone-coinfour gates).
      revealStuff()
      return

    // ─── automaticTools events ───────────────────────────────────────
    case 'auto-tool-fired':
      if (event.tool === 'addObtainium' && G.currentTab === Tabs.Research) {
        visualUpdateResearch()
      }
      return

    // ─── ambrosia events ─────────────────────────────────────────────
    case 'ambrosia-gained':
    case 'red-ambrosia-gained':
      visualUpdateAmbrosia()
      return

    // ─── octeract events ─────────────────────────────────────────────
    case 'octeract-tick-fired':
      visualUpdateOcteracts()
      return

    // ─── autoPotion events ───────────────────────────────────────────
    // Logic emits one event per side (offering / obtainium) when its
    // dispense timer crossed threshold this tick. Translate to the
    // useConsumable call the legacy autoPotion case made inline.
    case 'auto-potion-fired':
      useConsumable(
        event.type === 'offering' ? 'offeringPotion' : 'obtainiumPotion',
        true,
        event.amount,
        event.fastMode
      )
      return

    // ─── autoReset events ────────────────────────────────────────────
    case 'auto-reset-triggered':
      if (event.tier === 'prestige') {
        // Bug-for-bug parity: legacy prestige *time* mode awards the
        // 'transcension' achievement check; amount mode awards 'prestige'.
        // Documented in packages/logic/src/tick/autoReset.ts header.
        if (event.mode === 'time') {
          resetAchievementCheck('transcension')
        } else {
          resetAchievementCheck('prestige')
        }
        reset('prestige', true)
      } else if (event.tier === 'transcension') {
        resetAchievementCheck('transcension')
        reset('transcension', true)
      } else if (event.tier === 'reincarnation') {
        resetAchievementCheck('reincarnation')
        reset('reincarnation', true)
      }
      // 'ascension' tier is forward-looking and not emitted today.
      return

    // ─── challengeSweep events ───────────────────────────────────────
    case 'challenge-sweep-transitioned':
      dispatchSweepTransition(event.from, event.to)
      return

    // ─── antSacrifice events ─────────────────────────────────────────
    // Logic's checkAntSacrificeReady emits this when canAutoSacrifice's
    // conditions are met for the active mode. sacrificeAnts() reads the
    // latest player state itself (no payload needed) and fans out into
    // reward credit, ant reset, talisman + achievement awards, and
    // history record — all un-migrated subsystems that stay in web_ui.
    case 'ant-sacrifice-triggered':
      sacrificeAnts()
      return
  }
}
