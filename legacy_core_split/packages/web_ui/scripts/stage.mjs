import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolveBuildConfig } from './buildConfig.mjs'

// Render _headers.template into build/_headers, substituting env-driven
// placeholders. Vite handles every other static asset (index.html, the JS
// bundle, Synergism.css, favicon.ico via index.html <link>, and
// Pictures/translations via vite-plugin-static-copy), so this script is
// intentionally tiny.
//
// Run AFTER `vite build`. Vite's emptyOutDir clears build/ at the start of
// its own build, so doing it before would just be wasted work.

const config = resolveBuildConfig()

mkdirSync('build', { recursive: true })

const placeholders = {
  '{{API_BASE_URL}}': config.apiBaseUrl,
  '{{WS_BASE_URL}}': config.wsBaseUrl,
  '{{CANONICAL_HOST}}': config.canonicalHost
}

let headers = readFileSync('_headers.template', 'utf8')
for (const [key, value] of Object.entries(placeholders)) {
  headers = headers.replaceAll(key, value)
}
writeFileSync('build/_headers', headers)

console.log('rendered build/_headers')
console.log(`  api=${config.apiBaseUrl}`)
console.log(`  ws=${config.wsBaseUrl}`)
console.log(`  canonical=${config.canonicalHost}`)
