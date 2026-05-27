import { loadEnv } from './loadEnv.mjs'

// Resolve build-time config from environment variables. Vite reads these
// directly via import.meta.env; this helper is used by scripts/stage.mjs
// when rendering _headers.template (Vite doesn't model Cloudflare's _headers
// file, so we template it ourselves).
//
// Defaults preserve legacy behavior so an unconfigured build still produces
// a working bundle pointed at the legacy host. Forks set VITE_API_BASE_URL
// / VITE_WS_BASE_URL / VITE_CANONICAL_HOST in .env.{mode} (locally) or in
// the CF Pages dashboard.
export function resolveBuildConfig (mode = process.env.NODE_ENV ?? 'production') {
  loadEnv(mode)

  const apiBaseUrl = process.env.VITE_API_BASE_URL ?? 'https://synergism.cc'
  const wsBaseUrl = process.env.VITE_WS_BASE_URL ?? 'wss://synergism.cc'
  const canonicalHost = process.env.VITE_CANONICAL_HOST ?? new URL(apiBaseUrl).hostname

  return { apiBaseUrl, wsBaseUrl, canonicalHost, mode }
}
