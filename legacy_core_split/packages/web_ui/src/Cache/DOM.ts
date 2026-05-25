const MAX_CACHE_SIZE = 1e4
const PRUNE_TO = 8e3

const DOMCache = new Map<string, HTMLElement>()

export const DOMCacheGetOrSet = (id: string) => {
  const cachedEl = DOMCache.get(id)
  if (cachedEl) {
    DOMCache.delete(id)
    DOMCache.set(id, cachedEl)
    return cachedEl
  }

  const el = document.getElementById(id)

  if (!el) {
    throw new TypeError(`Element with id "${id}" was not found on page?`)
  }

  DOMCache.set(id, el)

  if (DOMCache.size > MAX_CACHE_SIZE) {
    console.error(`Possible memory leak detected ${DOMCache.size} dom elements cached, pruning oldest`)
    const iter = DOMCache.keys()
    while (DOMCache.size > PRUNE_TO) {
      const next = iter.next()
      if (next.done) break
      DOMCache.delete(next.value)
    }
  }

  return el
}

export const DOMCacheHas = (id: string) => DOMCache.has(id)

// Non-throwing variant of DOMCacheGetOrSet — returns null when the element
// doesn't exist (instead of throwing). Touch-on-get LRU behavior matches
// DOMCacheGetOrSet. Misses are NOT cached, so a subsequent create() + lookup
// works correctly (see Statistics.ts create-if-missing pattern).
export const DOMCacheGet = (id: string): HTMLElement | null => {
  const cachedEl = DOMCache.get(id)
  if (cachedEl) {
    DOMCache.delete(id)
    DOMCache.set(id, cachedEl)
    return cachedEl
  }

  const el = document.getElementById(id)
  if (!el) return null

  DOMCache.set(id, el)
  return el
}

// Invalidates a cached id. Call this BEFORE detaching / replacing the
// underlying DOM node so subsequent DOMCacheGetOrSet calls re-fetch instead
// of returning a stale reference (see Campaign.ts createCampaignIconHTMLS).
export const DOMCacheDelete = (id: string): void => {
  DOMCache.delete(id)
}
