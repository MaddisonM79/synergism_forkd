/// <reference types="vite/client" />

// Vite already declares the ambient ImportMeta.env with PROD/DEV/MODE.
// Augment it with our custom VITE_* keys via declaration merging.
declare global {
  interface ImportMetaEnv {
    readonly VITE_API_BASE_URL?: string
    readonly VITE_WS_BASE_URL?: string
    readonly VITE_CANONICAL_HOST?: string
  }
}

export const version = '4.2.4 May 10, 2026: Steam!!!'

// Backend endpoints, injected at build time by Vite. Defaults preserve
// legacy behavior so an unconfigured build still works against synergism.cc;
// forks set VITE_API_BASE_URL / VITE_WS_BASE_URL / VITE_CANONICAL_HOST in
// .env or the CF Pages dashboard.
export const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? 'https://synergism.cc'
export const wsBaseUrl = import.meta.env.VITE_WS_BASE_URL ?? 'wss://synergism.cc'
export const canonicalHost = import.meta.env.VITE_CANONICAL_HOST ?? new URL(apiBaseUrl).hostname

export const isCanonicalHost = location.hostname === canonicalHost

/**
 * If true, the version is marked as a testing version.
 */
export const testing = false
export const lastUpdated = new Date('##LAST_UPDATED##')

export const prod = import.meta.env.PROD
export const dev = import.meta.env.DEV

export const platform = 'browser' as const

export const ticksPerSecond = 200
