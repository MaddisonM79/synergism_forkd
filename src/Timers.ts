import * as workerTimers from 'worker-timers'

interface ActiveTimer {
  id: number
  type: 'interval' | 'timeout'
}

const activeTimers: ActiveTimer[] = []
const namedIntervals = new Map<string, number>()

export const setInterval: typeof workerTimers['setInterval'] = (fn, delay) => {
  const timer = workerTimers.setInterval(fn, delay)
  activeTimers.push({ id: timer, type: 'interval' })
  return timer
}

export const clearInterval: typeof workerTimers['clearInterval'] = (timerId) => {
  for (const timer of activeTimers) {
    if (timer.type === 'interval' && timer.id === timerId) {
      workerTimers.clearInterval(timerId)
      activeTimers.splice(activeTimers.indexOf(timer), 1)
      // Also drop the name binding if this id is tracked there
      for (const [name, id] of namedIntervals) {
        if (id === timerId) {
          namedIntervals.delete(name)
          break
        }
      }
      return
    }
  }
}

// Idempotent named interval registration: if a timer with the given name is
// already live, clear it first. Catches the duplicate-interval bug class that
// 54f393e5 fixed by hand: re-entry into the bootstrap path no longer doubles
// the saveSynergy / fastUpdates / tick clocks.
export const setNamedInterval = (name: string, fn: () => void, delay: number): number => {
  const existing = namedIntervals.get(name)
  if (existing !== undefined) {
    clearInterval(existing)
  }
  const timer = setInterval(fn, delay)
  namedIntervals.set(name, timer)
  return timer
}

export const clearNamedInterval = (name: string): void => {
  const id = namedIntervals.get(name)
  if (id !== undefined) {
    clearInterval(id)
  }
}

export const setTimeout: typeof workerTimers['setTimeout'] = (fn, delay) => {
  const timer = workerTimers.setTimeout(() => {
    fn()
    clearTimeout(timer)
  }, delay)
  activeTimers.push({ id: timer, type: 'timeout' })
  return timer
}

export const clearTimeout: typeof workerTimers['clearTimeout'] = (timerId) => {
  for (const timer of activeTimers) {
    if (timer.type === 'timeout' && timer.id === timerId) {
      workerTimers.clearTimeout(timerId)
      activeTimers.splice(activeTimers.indexOf(timer), 1)
      return
    }
  }
}

export const clearTimers = (): void => {
  // create shallow copy to avoid mutation bug
  const timersCopy = [...activeTimers]
  for (const { id, type } of timersCopy) {
    if (type === 'interval') {
      clearInterval(id)
    } else {
      clearTimeout(id)
    }
  }
  namedIntervals.clear()
}
