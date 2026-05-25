// Parity tests for processAutoResearchTick.
// Old body transcribed verbatim from packages/web_ui/src/Synergism.ts tack
// (the two if-blocks for manual mode + Roomba research automation).

import { describe, expect, it } from 'vitest'
import type { CoreEvent } from '../../src/events/types'
import { CalcECC } from '../../src/mechanics/challenges'
import {
  processAutoResearchTick as newProcessAutoResearchTick,
  type ProcessAutoResearchTickInput,
  type ProcessAutoResearchTickResult
} from '../../src/tick/autoResearch'

const oldProcessAutoResearchTick = (
  input: ProcessAutoResearchTickInput
): ProcessAutoResearchTickResult => {
  // Verbatim legacy: two independent ifs with mutually-exclusive
  // autoResearchMode values; at most one event ever emitted per tick.
  const events: CoreEvent[] = []
  if (input.autoResearchToggle && input.autoResearch > 0 && input.autoResearchMode === 'manual') {
    events.push({ kind: 'auto-research-manual-requested' })
  }
  if (
    input.autoResearchToggle
    && input.autoResearch > 0
    && input.roombaUnlocked
    && input.autoResearchMode === 'cheapest'
  ) {
    const maxCount = 1 + Math.floor(CalcECC('ascension', input.challengecompletions14))
    events.push({ kind: 'auto-research-roomba-requested', maxCount })
  }
  return { events }
}

const defaultInput = (
  overrides: Partial<ProcessAutoResearchTickInput> = {}
): ProcessAutoResearchTickInput => ({
  autoResearchToggle: true,
  autoResearch: 5,
  autoResearchMode: 'manual',
  roombaUnlocked: false,
  challengecompletions14: 0,
  ...overrides
})

describe('parity: processAutoResearchTick', () => {
  const cases: Array<{ name: string, input: ProcessAutoResearchTickInput }> = [
    {
      name: 'manual mode — all gates pass (emits manual event)',
      input: defaultInput()
    },
    {
      name: 'master toggle off — no event',
      input: defaultInput({ autoResearchToggle: false })
    },
    {
      name: 'autoResearch === 0 — no event',
      input: defaultInput({ autoResearch: 0 })
    },
    {
      name: 'autoResearch negative (defensive) — no event',
      input: defaultInput({ autoResearch: -1 })
    },
    {
      name: 'manual mode, but autoResearch === 0 — no event',
      input: defaultInput({ autoResearch: 0, autoResearchMode: 'manual' })
    },
    {
      name: 'Roomba mode with roombaUnlocked false — no event',
      input: defaultInput({ autoResearchMode: 'cheapest', roombaUnlocked: false })
    },
    {
      name: 'Roomba mode with roombaUnlocked true — emits Roomba event with maxCount = 1',
      input: defaultInput({
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 0
      })
    },
    {
      name: 'Roomba mode with challengecompletions14 = 25 — maxCount scales via CalcECC',
      input: defaultInput({
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 25
      })
    },
    {
      name: 'Roomba mode with challengecompletions14 = 60 — maxCount fully scaled',
      input: defaultInput({
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 60
      })
    },
    {
      name: 'Roomba mode with challengecompletions14 = 750 — maxCount large',
      input: defaultInput({
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 750
      })
    },
    {
      name: 'manual mode + roombaUnlocked true (Roomba gate ignored, manual fires)',
      input: defaultInput({
        autoResearchMode: 'manual',
        roombaUnlocked: true,
        challengecompletions14: 100
      })
    },
    {
      name: 'manual mode + toggle off (no event even with mode set)',
      input: defaultInput({ autoResearchToggle: false, autoResearchMode: 'manual' })
    },
    {
      name: 'Roomba mode + toggle off (no event even with unlock)',
      input: defaultInput({
        autoResearchToggle: false,
        autoResearchMode: 'cheapest',
        roombaUnlocked: true
      })
    },
    {
      name: 'high autoResearch index, manual mode',
      input: defaultInput({ autoResearch: 200, autoResearchMode: 'manual' })
    },
    {
      name: 'mid-singularity setup — Roomba with c14 completions',
      input: defaultInput({
        autoResearch: 200,
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 12
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newProcessAutoResearchTick(c.input)
      const oldR = oldProcessAutoResearchTick(c.input)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
