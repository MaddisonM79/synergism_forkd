// Parity tests for the challenge-sweep state machine.
// Old bodies transcribed verbatim from packages/web_ui/src/Challenges.ts
// (`tickChallengeSweep` + `sweepTransitionFunc`, ~lines 407-575
// pre-migration), minus the side effects (resetCheck,
// toggleAutoChallengeModeText, toggleChallenges) which stay in web_ui as
// CoreEvent handlers.

import { describe, expect, it } from 'vitest'
import {
  type SweepStates,
  tickChallengeSweep as newTickChallengeSweep,
  type TickChallengeSweepInput,
  type TickChallengeSweepResult
} from '../../src/tick/challengeSweep'

// Verbatim transcription. Mirrors the legacy code shape:
//   - the four early-return paths (enable, disable, idle no-op)
//   - timer accumulation
//   - sweepTransition + reference-equality change detection
const oldTickChallengeSweep = (input: TickChallengeSweepInput): TickChallengeSweepResult => {
  const wasEnabled = input.state.kind !== 'idle'
  const isEnabled = input.shouldRunSweep

  if (!wasEnabled && isEnabled) {
    const from: SweepStates = { kind: 'idle' }
    const to: SweepStates = { kind: 'initial_wait' }
    return {
      state: to,
      timeSinceLastStateChange: 0,
      events: [{ kind: 'challenge-sweep-transitioned', from, to }]
    }
  }

  if (wasEnabled && !isEnabled) {
    const from = input.state
    const to: SweepStates = { kind: 'idle' }
    return {
      state: to,
      timeSinceLastStateChange: 0,
      events: [{ kind: 'challenge-sweep-transitioned', from, to }]
    }
  }

  if (!isEnabled) {
    return {
      state: input.state,
      timeSinceLastStateChange: input.timeSinceLastStateChange,
      events: []
    }
  }

  const elapsed = input.timeSinceLastStateChange + input.dt
  const newState = sweepTransition(input.state, elapsed, input)

  if (newState !== input.state) {
    return {
      state: newState,
      timeSinceLastStateChange: 0,
      events: [{ kind: 'challenge-sweep-transitioned', from: input.state, to: newState }]
    }
  }

  return {
    state: input.state,
    timeSinceLastStateChange: elapsed,
    events: []
  }
}

const sweepTransition = (state: SweepStates, elapsed: number, input: TickChallengeSweepInput): SweepStates => {
  switch (state.kind) {
    case 'idle':
      return state
    case 'initial_wait':
      if (elapsed >= input.timerStart) {
        if (input.nextRegularChallengeFromInitial === -1) {
          return { kind: 'finished' }
        }
        return {
          kind: 'active',
          index: input.nextRegularChallengeFromInitial,
          explored: new Set([input.nextRegularChallengeFromInitial])
        }
      }
      return state
    case 'active':
      if (elapsed >= input.timerExit) {
        if (input.nextRegularChallengeFromActive === -1) {
          if (input.challenge15AutoExponentCheck) {
            return { kind: 'c15_wait' }
          }
          return { kind: 'initial_wait' }
        }
        return { kind: 'enter_wait', toIndex: input.nextRegularChallengeFromActive, explored: state.explored }
      }
      return state
    case 'enter_wait':
      if (elapsed >= input.timerEnter) {
        return {
          kind: 'active',
          index: state.toIndex,
          explored: new Set([...state.explored, state.toIndex])
        }
      }
      return state
    case 'c15_wait':
      if (elapsed >= 5) {
        return { kind: 'initial_wait' }
      }
      return state
    case 'finished':
      if (input.isFinishedStillValid) {
        return state
      }
      return { kind: 'initial_wait' }
    default:
      throw new Error(`Unhandled SweepState kind: ${(state as { kind: string }).kind}`)
  }
}

const defaultInput = (overrides: Partial<TickChallengeSweepInput> = {}): TickChallengeSweepInput => ({
  dt: 0.025,
  state: { kind: 'idle' },
  timeSinceLastStateChange: 0,
  shouldRunSweep: false,
  timerStart: 5,
  timerExit: 30,
  timerEnter: 2,
  initialIndex: 1,
  nextRegularChallengeFromInitial: -1,
  nextRegularChallengeFromActive: -1,
  challenge15AutoExponentCheck: false,
  isFinishedStillValid: false,
  ...overrides
})

describe('parity: tickChallengeSweep', () => {
  const cases: Array<{ name: string, input: TickChallengeSweepInput }> = [
    // ─── shouldRunSweep gates ──────────────────────────────────────────
    {
      name: 'idle + !shouldRunSweep — stays idle, no event',
      input: defaultInput()
    },
    {
      name: 'idle + shouldRunSweep — boots to initial_wait (event)',
      input: defaultInput({ shouldRunSweep: true })
    },
    {
      name: 'initial_wait + !shouldRunSweep — tears down to idle (event)',
      input: defaultInput({
        state: { kind: 'initial_wait' },
        timeSinceLastStateChange: 3,
        shouldRunSweep: false
      })
    },
    {
      name: 'active + !shouldRunSweep — fires resetCheck via event with from.index',
      input: defaultInput({
        state: { kind: 'active', index: 3, explored: new Set([1, 2, 3]) },
        timeSinceLastStateChange: 10,
        shouldRunSweep: false
      })
    },
    {
      name: 'active(reincarnation idx 8) + !shouldRunSweep — from.index preserved',
      input: defaultInput({
        state: { kind: 'active', index: 8, explored: new Set([6, 7, 8]) },
        timeSinceLastStateChange: 10,
        shouldRunSweep: false
      })
    },

    // ─── initial_wait transitions ──────────────────────────────────────
    {
      name: 'initial_wait + elapsed < timerStart — stays (timer accumulates)',
      input: defaultInput({
        state: { kind: 'initial_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 4.9,
        dt: 0.025,
        timerStart: 5
      })
    },
    {
      name: 'initial_wait + elapsed >= timerStart + nextChallenge !== -1 → active',
      input: defaultInput({
        state: { kind: 'initial_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 4.99,
        dt: 0.025,
        timerStart: 5,
        nextRegularChallengeFromInitial: 1
      })
    },
    {
      name: 'initial_wait + elapsed >= timerStart + nextChallenge === -1 → finished',
      input: defaultInput({
        state: { kind: 'initial_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 4.99,
        dt: 0.025,
        timerStart: 5,
        nextRegularChallengeFromInitial: -1
      })
    },

    // ─── active transitions ────────────────────────────────────────────
    {
      name: 'active + elapsed < timerExit — stays',
      input: defaultInput({
        state: { kind: 'active', index: 1, explored: new Set([1]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 25,
        dt: 0.025,
        timerExit: 30
      })
    },
    {
      name: 'active + elapsed >= timerExit + nextChallenge=2 → enter_wait(toIndex=2)',
      input: defaultInput({
        state: { kind: 'active', index: 1, explored: new Set([1]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 29.99,
        dt: 0.025,
        timerExit: 30,
        nextRegularChallengeFromActive: 2
      })
    },
    {
      name: 'active + elapsed >= timerExit + nextChallenge=-1 + chal15Check → c15_wait',
      input: defaultInput({
        state: { kind: 'active', index: 10, explored: new Set([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 29.99,
        dt: 0.025,
        timerExit: 30,
        nextRegularChallengeFromActive: -1,
        challenge15AutoExponentCheck: true
      })
    },
    {
      name: 'active + elapsed >= timerExit + nextChallenge=-1 + !chal15Check → initial_wait',
      input: defaultInput({
        state: { kind: 'active', index: 10, explored: new Set([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 29.99,
        dt: 0.025,
        timerExit: 30,
        nextRegularChallengeFromActive: -1,
        challenge15AutoExponentCheck: false
      })
    },

    // ─── enter_wait transitions ────────────────────────────────────────
    {
      name: 'enter_wait + elapsed < timerEnter — stays',
      input: defaultInput({
        state: { kind: 'enter_wait', toIndex: 2, explored: new Set([1]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 1.9,
        dt: 0.025,
        timerEnter: 2
      })
    },
    {
      name: 'enter_wait + elapsed >= timerEnter → active (explored ∪ toIndex)',
      input: defaultInput({
        state: { kind: 'enter_wait', toIndex: 2, explored: new Set([1]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 1.99,
        dt: 0.025,
        timerEnter: 2
      })
    },
    {
      name: 'enter_wait — explored set carries forward and adds toIndex',
      input: defaultInput({
        state: { kind: 'enter_wait', toIndex: 7, explored: new Set([1, 2, 3, 4, 5, 6]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 1.99,
        dt: 0.025,
        timerEnter: 2
      })
    },

    // ─── c15_wait transitions ──────────────────────────────────────────
    {
      name: 'c15_wait + elapsed < 5 — stays',
      input: defaultInput({
        state: { kind: 'c15_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 4.9,
        dt: 0.025
      })
    },
    {
      name: 'c15_wait + elapsed >= 5 → initial_wait',
      input: defaultInput({
        state: { kind: 'c15_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 4.99,
        dt: 0.025
      })
    },

    // ─── finished transitions ──────────────────────────────────────────
    {
      name: 'finished + isFinishedStillValid — stays',
      input: defaultInput({
        state: { kind: 'finished' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 100,
        dt: 0.025,
        isFinishedStillValid: true
      })
    },
    {
      name: 'finished + !isFinishedStillValid → initial_wait',
      input: defaultInput({
        state: { kind: 'finished' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 100,
        dt: 0.025,
        isFinishedStillValid: false
      })
    },

    // ─── Edge cases ────────────────────────────────────────────────────
    {
      name: 'dt = 0 — no timer change, no transition',
      input: defaultInput({
        state: { kind: 'initial_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 3,
        dt: 0
      })
    },
    {
      name: 'large dt vaults past threshold in one tick',
      input: defaultInput({
        state: { kind: 'initial_wait' },
        shouldRunSweep: true,
        timeSinceLastStateChange: 0,
        dt: 100,
        timerStart: 5,
        nextRegularChallengeFromInitial: 1
      })
    },
    {
      name: 'enter_wait toIndex=6 (reincarnation crosses transcension boundary)',
      input: defaultInput({
        state: { kind: 'enter_wait', toIndex: 6, explored: new Set([1, 2, 3, 4, 5]) },
        shouldRunSweep: true,
        timeSinceLastStateChange: 5,
        dt: 0,
        timerEnter: 2
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newTickChallengeSweep(c.input)
      const oldR = oldTickChallengeSweep(c.input)
      // State equality — kinds match
      expect(newR.state.kind).toBe(oldR.state.kind)
      // Discriminated fields by kind
      if (newR.state.kind === 'active' && oldR.state.kind === 'active') {
        expect(newR.state.index).toBe(oldR.state.index)
        expect([...newR.state.explored].sort((a, b) => a - b)).toEqual([...oldR.state.explored].sort((a, b) => a - b))
      }
      if (newR.state.kind === 'enter_wait' && oldR.state.kind === 'enter_wait') {
        expect(newR.state.toIndex).toBe(oldR.state.toIndex)
        expect([...newR.state.explored].sort((a, b) => a - b)).toEqual([...oldR.state.explored].sort((a, b) => a - b))
      }
      expect(newR.timeSinceLastStateChange).toBe(oldR.timeSinceLastStateChange)
      expect(newR.events.length).toBe(oldR.events.length)
      // Per-event field equality (kinds, from/to kinds, indexes where applicable)
      for (let i = 0; i < newR.events.length; i++) {
        const ne = newR.events[i]
        const oe = oldR.events[i]
        expect(ne.kind).toBe(oe.kind)
        if (ne.kind === 'challenge-sweep-transitioned' && oe.kind === 'challenge-sweep-transitioned') {
          expect(ne.from.kind).toBe(oe.from.kind)
          expect(ne.to.kind).toBe(oe.to.kind)
          if (ne.from.kind === 'active' && oe.from.kind === 'active') {
            expect(ne.from.index).toBe(oe.from.index)
          }
          if (ne.to.kind === 'active' && oe.to.kind === 'active') {
            expect(ne.to.index).toBe(oe.to.index)
          }
        }
      }
    })
  }
})
