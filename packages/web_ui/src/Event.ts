import {
  BuffType as LogicBuffType,
  calculateEventSourceBuff as logicCalculateEventSourceBuff,
  consumableEventBuff as logicConsumableEventBuff,
  getEventBuff as logicGetEventBuff
} from '@synergism/logic'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { apiBaseUrl } from './Config'
import { allDurableConsumables, type PseudoCoinConsumableNames } from './Login'
import { getGQUpgradeEffect } from './singularity'
import { getTimePinnedToLoadDate, player } from './Synergism'
import { revealStuff } from './UpdateHTML'
import { timeRemainingHours } from './Utility'
import { Globals as G } from './Variables'

// Re-exported so existing call sites that import `BuffType` from this module
// keep compiling unchanged.
export const BuffType = LogicBuffType
export type BuffType = LogicBuffType

interface GameEvent {
  name: string[]
  url: string[]
  start: number
  end: number
  quark: number
  goldenQuark: number
  cubes: number
  powderConversion: number
  ascensionSpeed: number
  globalSpeed: number
  ascensionScore: number
  antSacrifice: number
  offering: number
  obtainium: number
  octeract: number
  blueberryTime: number
  ambrosiaLuck: number
  oneMind: number
  color: string[]
}

let nowEvent: GameEvent | null = null
export const getEvent = () => nowEvent

export const eventCheck = async () => {
  if (!player.dayCheck) {
    return
  }

  const response = await fetch(`${apiBaseUrl}/events/get`)

  if (!response.ok) {
    throw new Error(`Failed to fetch events: HTTP ${response.status} ${response.statusText}`.trim())
  }

  const apiEvents = await response.json() as GameEvent

  nowEvent = null

  const now = new Date(getTimePinnedToLoadDate()).getTime()
  if (now >= apiEvents.start && now <= apiEvents.end && apiEvents.name.length) {
    nowEvent = apiEvents
  }

  const eventNowEndDate = new Date(nowEvent?.end ?? 0)
  DOMCacheGetOrSet('globalEventTimer').textContent = timeRemainingHours(eventNowEndDate)

  const updateIsEventCheck = G.isEvent

  updateGlobalsIsEvent()

  if (G.isEvent !== updateIsEventCheck) {
    revealStuff()
  }
}

export const eventBuffType: (keyof typeof BuffType)[] = [
  'Quark',
  'GoldenQuark',
  'Cubes',
  'PowderConversion',
  'AscensionSpeed',
  'GlobalSpeed',
  'AscensionScore',
  'AntSacrifice',
  'Offering',
  'Obtainium',
  'Octeract',
  'BlueberryTime',
  'AmbrosiaLuck',
  'OneMind'
]

export const calculateEventSourceBuff = (buff: BuffType): number =>
  logicCalculateEventSourceBuff(
    buff,
    getEvent(),
    getGQUpgradeEffect('oneMind', 'unlocked'),
    allDurableConsumables.HAPPY_HOUR_BELL.amount
  )

export const getEventBuff = (buff: BuffType): number =>
  logicGetEventBuff(buff, getEvent(), getGQUpgradeEffect('oneMind', 'unlocked'))

export const consumableEventBuff = (buff: BuffType): number =>
  logicConsumableEventBuff(buff, allDurableConsumables.HAPPY_HOUR_BELL.amount)

const isConsumableActive = (name?: PseudoCoinConsumableNames) => {
  if (typeof name === 'string') {
    return allDurableConsumables[name].amount > 0
  }

  return allDurableConsumables.HAPPY_HOUR_BELL.amount !== 0
}

export const updateGlobalsIsEvent = () => {
  return G.isEvent = getEvent() !== null || isConsumableActive()
}
