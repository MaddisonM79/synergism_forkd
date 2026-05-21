declare const PROD: boolean | undefined
declare const DEV: boolean | undefined
declare const PLATFORM: 'steam' | undefined
declare const API_BASE_URL: string | undefined
declare const WS_BASE_URL: string | undefined
declare const CANONICAL_HOST: string | undefined

export const version = '4.2.4 May 10, 2026: Steam!!!'

// Backend endpoints, injected at build time by scripts/build.mjs. Defaults
// preserve legacy behavior so an unconfigured build still works against
// synergism.cc; forks set API_BASE_URL / WS_BASE_URL / CANONICAL_HOST in
// .env or the CF Pages dashboard.
export const apiBaseUrl = typeof API_BASE_URL === 'undefined' ? 'https://synergism.cc' : API_BASE_URL
export const wsBaseUrl = typeof WS_BASE_URL === 'undefined' ? 'wss://synergism.cc' : WS_BASE_URL
export const canonicalHost = typeof CANONICAL_HOST === 'undefined' ? 'synergism.cc' : CANONICAL_HOST

export const isCanonicalHost = location.hostname === canonicalHost

/**
 * If true, the version is marked as a testing version.
 */
export const testing = false
export const lastUpdated = new Date('##LAST_UPDATED##')

export const prod = typeof PROD === 'undefined' ? false : PROD
export const dev = typeof DEV === 'undefined' ? false : DEV

export const platform = typeof PLATFORM === 'undefined' ? 'browser' : PLATFORM

export const ticksPerSecond = 200
