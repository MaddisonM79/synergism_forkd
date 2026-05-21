import { copyFileSync, cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { resolveBuildConfig } from './buildConfig.mjs'

// scripts/stage.mjs sets up build/ with all static assets and renders the
// templated index.html / _headers. Run BEFORE scripts/build.mjs — this script
// removes build/ wholesale and would clobber a fresh bundle if run after.

const config = resolveBuildConfig()

rmSync('build', { recursive: true, force: true })
mkdirSync('build/dist', { recursive: true })

cpSync('Pictures', 'build/Pictures', { recursive: true })
cpSync('translations', 'build/translations', { recursive: true })
copyFileSync('Synergism.css', 'build/Synergism.css')
copyFileSync('favicon.ico', 'build/favicon.ico')

const placeholders = {
  '{{API_BASE_URL}}': config.apiBaseUrl,
  '{{WS_BASE_URL}}': config.wsBaseUrl,
  '{{CANONICAL_HOST}}': config.canonicalHost
}

function render (templatePath, outPath) {
  let body = readFileSync(templatePath, 'utf8')
  for (const [key, value] of Object.entries(placeholders)) {
    body = body.replaceAll(key, value)
  }
  writeFileSync(outPath, body)
}

render('_headers.template', 'build/_headers')
render('index.html', 'build/index.html')

console.log('staged build/ for deploy')
console.log(`  api=${config.apiBaseUrl}`)
console.log(`  ws=${config.wsBaseUrl}`)
console.log(`  canonical=${config.canonicalHost}`)
