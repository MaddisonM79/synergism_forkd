import { existsSync, readFileSync } from 'node:fs'

// Cascading .env loader: .env.local > .env.{mode}.local > .env.{mode} > .env.
// Higher-precedence files win; existing process.env always wins (so CF Pages
// dashboard env vars and shell exports are untouchable).
//
// Quoted values have surrounding quotes stripped. # starts a comment.
// Empty lines and comment-only lines are ignored.
export function loadEnv (mode = process.env.NODE_ENV ?? 'production') {
  const files = [
    '.env.local',
    `.env.${mode}.local`,
    `.env.${mode}`,
    '.env'
  ]

  for (const file of files) {
    if (!existsSync(file)) continue
    const content = readFileSync(file, 'utf8')
    for (const raw of content.split('\n')) {
      const line = raw.trim()
      if (!line || line.startsWith('#')) continue
      const eq = line.indexOf('=')
      if (eq < 0) continue
      const key = line.slice(0, eq).trim()
      let value = line.slice(eq + 1).trim()
      if (
        (value.startsWith('"') && value.endsWith('"'))
        || (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1)
      }
      if (!(key in process.env)) {
        process.env[key] = value
      }
    }
  }
}
