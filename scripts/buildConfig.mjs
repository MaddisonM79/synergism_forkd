import { loadEnv } from './loadEnv.mjs'

// Resolve build-time config from environment variables, with synergism.cc
// fallbacks so an unconfigured build still produces a working bundle pointed
// at the legacy host. Any fork should set API_BASE_URL / WS_BASE_URL /
// CANONICAL_HOST in .env.{mode} (locally) or in the CF Pages dashboard.
export function resolveBuildConfig (mode = process.env.NODE_ENV ?? 'production') {
  loadEnv(mode)

  const apiBaseUrl = process.env.API_BASE_URL ?? 'https://synergism.cc'
  const wsBaseUrl = process.env.WS_BASE_URL ?? 'wss://synergism.cc'
  const canonicalHost = process.env.CANONICAL_HOST ?? new URL(apiBaseUrl).hostname
  const platform = process.env.PLATFORM ?? 'browser'

  return { apiBaseUrl, wsBaseUrl, canonicalHost, platform, mode }
}
