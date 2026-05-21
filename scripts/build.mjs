import { mkdirSync } from 'node:fs'
import { dirname } from 'node:path'
import { build, context } from 'esbuild'
import { resolveBuildConfig } from './buildConfig.mjs'

const watch = process.argv.includes('--watch')
const config = resolveBuildConfig()
const isProd = config.mode === 'production'

const outfile = config.platform === 'steam'
  ? './dist/dist/out.js'
  : './build/dist/out.js'

mkdirSync(dirname(outfile), { recursive: true })

const options = {
  entryPoints: ['src/Synergism.ts'],
  bundle: true,
  minify: isProd,
  sourcemap: !isProd,
  target: 'es2020',
  outfile,
  define: {
    PROD: String(isProd),
    DEV: String(!isProd),
    PLATFORM: JSON.stringify(config.platform),
    API_BASE_URL: JSON.stringify(config.apiBaseUrl),
    WS_BASE_URL: JSON.stringify(config.wsBaseUrl),
    CANONICAL_HOST: JSON.stringify(config.canonicalHost)
  },
  logLevel: 'info'
}

console.log(`build mode=${config.mode} platform=${config.platform}`)
console.log(`  api=${config.apiBaseUrl}`)
console.log(`  ws=${config.wsBaseUrl}`)
console.log(`  canonical=${config.canonicalHost}`)
console.log(`  outfile=${outfile}`)

if (watch) {
  const ctx = await context(options)
  await ctx.watch()
  console.log('watching…')
} else {
  await build(options)
}
