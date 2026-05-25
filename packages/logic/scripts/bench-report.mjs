#!/usr/bin/env node
// Reads vitest's bench JSON output and prints a per-benchmark report
// with every available statistic (hz / mean / median / min / max /
// p75 / p99 / p995 / p999 / sd / rme / sample count). Also writes
// the same report as Markdown to BENCH_REPORT.md so it can be
// committed or attached to a PR.
//
// Invoked by `npm run bench:report` in packages/logic — that script:
//   1. Runs `vitest bench --run --outputJson=bench-results.json`.
//   2. Pipes the JSON through this formatter.
//
// Usage standalone:
//   node scripts/bench-report.mjs [path/to/results.json]
// Defaults to ./bench-results.json next to the package root.

import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PACKAGE_ROOT = resolve(__dirname, '..')

const jsonPath = process.argv[2]
  ? resolve(process.cwd(), process.argv[2])
  : resolve(PACKAGE_ROOT, 'bench-results.json')

let raw
try {
  raw = readFileSync(jsonPath, 'utf-8')
} catch (err) {
  console.error(`bench-report: could not read ${jsonPath}`)
  console.error(err.message)
  process.exit(1)
}

const data = JSON.parse(raw)

// ─── Formatting helpers ────────────────────────────────────────────────

// Format a duration in milliseconds with adaptive precision: very fast
// ops show ns, fast ops show μs, slow ops show ms.
const formatTime = (ms) => {
  if (!Number.isFinite(ms)) return 'n/a'
  if (ms < 1e-3) return `${(ms * 1_000_000).toFixed(1)} ns`
  if (ms < 1) return `${(ms * 1000).toFixed(2)} μs`
  if (ms < 100) return `${ms.toFixed(3)} ms`
  return `${ms.toFixed(2)} ms`
}

const formatHz = (hz) => {
  if (!Number.isFinite(hz)) return 'n/a'
  if (hz >= 1e9) return `${(hz / 1e9).toFixed(2)}G/s`
  if (hz >= 1e6) return `${(hz / 1e6).toFixed(2)}M/s`
  if (hz >= 1e3) return `${(hz / 1e3).toFixed(2)}K/s`
  return `${hz.toFixed(2)}/s`
}

const formatRme = (rme) => `±${rme.toFixed(2)}%`

// Pad a string to fixed width (right-aligned for numbers, left for names).
const padR = (s, w) => String(s).padStart(w)
const padL = (s, w) => String(s).padEnd(w)

const shortFile = (filepath) => filepath.split('/').slice(-3).join('/')

// ─── Per-benchmark row + section ───────────────────────────────────────

const COLUMNS = [
  { key: 'name', label: 'benchmark', width: 60, align: 'L' },
  { key: 'hz', label: 'ops/sec', width: 12, fmt: formatHz },
  { key: 'mean', label: 'mean', width: 12, fmt: formatTime },
  { key: 'median', label: 'median', width: 12, fmt: formatTime },
  { key: 'min', label: 'min', width: 12, fmt: formatTime },
  { key: 'max', label: 'max', width: 12, fmt: formatTime },
  { key: 'p75', label: 'p75', width: 12, fmt: formatTime },
  { key: 'p99', label: 'p99', width: 12, fmt: formatTime },
  { key: 'p995', label: 'p99.5', width: 12, fmt: formatTime },
  { key: 'p999', label: 'p99.9', width: 12, fmt: formatTime },
  { key: 'sd', label: 'std-dev', width: 12, fmt: formatTime },
  { key: 'rme', label: 'rme', width: 9, fmt: formatRme },
  { key: 'sampleCount', label: 'samples', width: 11 }
]

const renderConsoleHeader = () => {
  const cells = COLUMNS.map((c) =>
    c.align === 'L' ? padL(c.label, c.width) : padR(c.label, c.width)
  )
  const line = cells.join(' │ ')
  const sep = COLUMNS.map((c) => '─'.repeat(c.width)).join('─┼─')
  return `${line}\n${sep}`
}

const renderConsoleRow = (b) => {
  const cells = COLUMNS.map((c) => {
    const v = b[c.key]
    const s = c.fmt ? c.fmt(v) : String(v ?? '')
    return c.align === 'L' ? padL(s, c.width) : padR(s, c.width)
  })
  return cells.join(' │ ')
}

const renderMarkdownTable = (benchmarks) => {
  const headers = COLUMNS.map((c) => c.label)
  const sep = COLUMNS.map((c) => (c.align === 'L' ? ':---' : '---:'))
  const rows = benchmarks.map((b) =>
    COLUMNS.map((c) => {
      const v = b[c.key]
      return c.fmt ? c.fmt(v) : String(v ?? '')
    })
  )
  const fmt = (cells) => `| ${cells.join(' | ')} |`
  return [fmt(headers), fmt(sep), ...rows.map(fmt)].join('\n')
}

// ─── File-level aggregation ────────────────────────────────────────────

const aggregateGroup = (group) => {
  const bs = group.benchmarks
  if (bs.length === 0) return null
  const totalSamples = bs.reduce((s, b) => s + (b.sampleCount ?? 0), 0)
  const meanRme = bs.reduce((s, b) => s + (b.rme ?? 0), 0) / bs.length
  return { count: bs.length, totalSamples, meanRme }
}

const aggregateFile = (file) => {
  const allBenches = file.groups.flatMap((g) => g.benchmarks)
  return {
    groupCount: file.groups.length,
    benchCount: allBenches.length,
    totalSamples: allBenches.reduce((s, b) => s + (b.sampleCount ?? 0), 0),
    totalTime: allBenches.reduce((s, b) => s + (b.totalTime ?? 0), 0)
  }
}

// Rank benchmarks within a group by `hz` (highest = fastest). Vitest
// already provides `rank` but recomputing makes the report self-contained.
const annotateRanks = (benchmarks) => {
  const sorted = [...benchmarks].sort((a, b) => (b.hz ?? 0) - (a.hz ?? 0))
  const rankMap = new Map(sorted.map((b, i) => [b.id, i + 1]))
  const fastestHz = sorted[0]?.hz ?? 0
  return benchmarks.map((b) => ({
    ...b,
    derivedRank: rankMap.get(b.id),
    speedRatio: fastestHz > 0 ? (b.hz ?? 0) / fastestHz : null
  }))
}

// ─── Rendering ─────────────────────────────────────────────────────────

const consoleLines = []
const mdLines = []

const log = (line = '') => {
  consoleLines.push(line)
  mdLines.push(line)
}
const logConsoleOnly = (line) => consoleLines.push(line)
const logMdOnly = (line) => mdLines.push(line)

const ISO_NOW = new Date().toISOString()
const NODE_VERSION = process.version

let osLabel = 'unknown'
try {
  osLabel = execSync('uname -smr', { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim()
} catch {
  /* leave default */
}

logMdOnly('# Benchmark Report')
logMdOnly('')
logMdOnly(`- Generated: \`${ISO_NOW}\``)
logMdOnly(`- Host: \`${osLabel}\``)
logMdOnly(`- Node: \`${NODE_VERSION}\``)
logMdOnly(`- Source: \`${shortFile(jsonPath)}\``)
logMdOnly('')

logConsoleOnly('')
logConsoleOnly(`Benchmark report  ${ISO_NOW}  ${osLabel}  node ${NODE_VERSION}`)
logConsoleOnly(`Source: ${jsonPath}`)
logConsoleOnly('')

for (const file of data.files) {
  const stats = aggregateFile(file)
  const fileLabel = shortFile(file.filepath)
  log('')
  logConsoleOnly('═'.repeat(180))
  log(`## ${fileLabel}`)
  log('')
  log(
    `${stats.groupCount} group(s) · ${stats.benchCount} benchmark(s) · `
      + `${stats.totalSamples.toLocaleString()} total samples · `
      + `${(stats.totalTime / 1000).toFixed(2)}s total bench time`
  )
  log('')

  for (const group of file.groups) {
    const groupLabel = group.fullName.split(' > ').slice(-1)[0]
    const annotated = annotateRanks(group.benchmarks)
    const agg = aggregateGroup(group)
    log(`### ${groupLabel}`)
    log('')
    log(
      `${agg.count} benchmark(s) · `
        + `${agg.totalSamples.toLocaleString()} samples · `
        + `mean rme ${formatRme(agg.meanRme)}`
    )
    log('')

    // Console table
    logConsoleOnly(renderConsoleHeader())
    for (const b of annotated) {
      logConsoleOnly(renderConsoleRow(b))
    }
    logConsoleOnly('')

    // Markdown table
    logMdOnly(renderMarkdownTable(annotated))
    logMdOnly('')

    // Per-group speed ranking
    log('Speed ranking within group (fastest → slowest):')
    log('')
    const ranked = [...annotated].sort((a, b) => a.derivedRank - b.derivedRank)
    for (const b of ranked) {
      const ratio = b.speedRatio !== null
        ? (b.speedRatio === 1 ? 'baseline' : `${(1 / b.speedRatio).toFixed(2)}× slower than fastest`)
        : 'n/a'
      log(`  ${b.derivedRank}. ${b.name} — ${formatHz(b.hz)} (${ratio})`)
    }
    log('')
  }
}

// ─── Cross-file summary ────────────────────────────────────────────────

const allBenches = data.files.flatMap((f) =>
  f.groups.flatMap((g) =>
    g.benchmarks.map((b) => ({ ...b, file: shortFile(f.filepath), group: g.fullName.split(' > ').slice(-1)[0] }))
  )
)
const totalSamples = allBenches.reduce((s, b) => s + (b.sampleCount ?? 0), 0)
const totalTime = allBenches.reduce((s, b) => s + (b.totalTime ?? 0), 0)
const noisiest = [...allBenches].sort((a, b) => (b.rme ?? 0) - (a.rme ?? 0)).slice(0, 5)
const slowest = [...allBenches].sort((a, b) => (a.hz ?? 0) - (b.hz ?? 0)).slice(0, 5)

log('')
log('## Summary')
log('')
log(`- ${data.files.length} file(s)`)
log(`- ${allBenches.length} benchmark(s) across all files`)
log(`- ${totalSamples.toLocaleString()} samples collected`)
log(`- ${(totalTime / 1000).toFixed(2)}s total bench time`)
log('')

log('### Top 5 noisiest benchmarks (highest rme)')
log('')
for (const b of noisiest) {
  log(`- \`${b.name}\` (${b.group}) — ${formatRme(b.rme ?? 0)} · ${formatHz(b.hz)} · ${formatTime(b.mean)} mean`)
}
log('')

log('### Top 5 slowest benchmarks (lowest hz)')
log('')
for (const b of slowest) {
  log(`- \`${b.name}\` (${b.group}) — ${formatHz(b.hz)} · ${formatTime(b.mean)} mean`)
}
log('')

// ─── Write outputs ─────────────────────────────────────────────────────

console.log(consoleLines.join('\n'))

const mdPath = resolve(PACKAGE_ROOT, 'BENCH_REPORT.md')
writeFileSync(mdPath, mdLines.join('\n'))
console.log(`\nMarkdown report written to ${mdPath}`)
