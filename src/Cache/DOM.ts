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
