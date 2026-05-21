# Contributing

## Prerequisites

Before running any of the commands below, make sure you have installed:

- NodeJS >= 22.21.0 — https://nodejs.org/en/
- git — https://git-scm.com/downloads

## Recommended

- VSCode — https://code.visualstudio.com/Download

## Workflow

1. Fork this repository.
2. Clone your fork:
   ```sh
   git clone https://github.com/<USERNAME>/unknown_game
   cd unknown_game
   ```
3. Install dependencies:
   ```sh
   npm install
   ```
4. (Optional) configure local env vars by copying [`.env.example`](.env.example) to `.env.development`. Defaults point at the legacy `synergism.cc` backend so this is only needed if you want to target a different API host.
5. Switch to a new branch:
   ```sh
   git checkout -b my-branch-name
   ```
6. Start the dev server:
   ```sh
   node --run dev
   ```
   This stages `build/`, watches `src/` with esbuild, and serves `build/` via `wrangler pages dev` on port 3000 — so the local server applies the same `_headers` (CSP, etc.) that production gets.
7. Make your changes and verify them in the browser.
8. Type-check:
   ```sh
   node --run check:tsc
   ```
9. Lint:
   ```sh
   node --run lint
   node --run csslint
   ```
10. Stage and commit:
    ```sh
    git add <files>
    git commit -m "short description"
    ```
11. Push and open a pull request against this repository.

## Pull request titles

PR titles follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format and are checked by the `pr-title-lint` workflow:

```
type(optional-scope): short lowercase subject
```

Allowed types: `feat`, `fix`, `chore`, `ci`, `docs`, `perf`, `refactor`, `revert`, `security`, `style`, `test`, `ts`, `ux`.

Examples:

- `fix: prevent rune 7 race during singularity`
- `ux: shorten notification slide-in to 200ms`
- `ci: install deep-object-diff for translation check`
- `feat(ants): add Reborn-ELO tranche 11`

Since merges are squashed, the PR title becomes the commit message on `main`. Individual commits on your feature branch can use any message you like.

## Save format changes

Changes that add or modify fields on the `player` object affect every player's savefile size and migration path. **Before making such a change, please open an issue first** to discuss the schema impact.

See also [CLAUDE.md](CLAUDE.md) for the agent-facing contribution rules (i18n, DOM caching, Steam/Electron gating, etc.).

## Build-time configuration

Three environment variables shape the build:

| Variable | Default | Purpose |
| --- | --- | --- |
| `API_BASE_URL` | `https://synergism.cc` | Backend HTTP origin (login, payments, messages, translations) |
| `WS_BASE_URL` | `wss://synergism.cc` | Consumables WebSocket origin |
| `CANONICAL_HOST` | hostname of `API_BASE_URL` | Canonical web hostname (gates browser-only purchase UI; tightens MSW's unhandled-request guard) |

These are injected into the bundle by [`scripts/build.mjs`](scripts/build.mjs) via esbuild `--define`, and substituted into [`index.html`](index.html) and [`_headers.template`](_headers.template) by [`scripts/stage.mjs`](scripts/stage.mjs). Local dev reads them from `.env.development` (cascading: `.env.local` > `.env.{mode}.local` > `.env.{mode}` > `.env`). Production reads them from CF Pages dashboard environment variables.

## Deploy (Cloudflare Pages)

```sh
npm run cloudflare:build
```

This runs `cloudflare:stage` (assembles `build/` with templated `index.html` + `_headers`) and then `build:esbuild` (writes `build/dist/out.js`). The result at `build/` is exactly what gets uploaded to CF Pages:

- `index.html`, `Synergism.css`, `favicon.ico`
- `Pictures/`, `translations/`
- `dist/out.js`
- `_headers` (CSP + standard security headers — applied at the CF edge)

### Cloudflare Pages dashboard settings

- **Framework preset:** None
- **Build command:** `npm run cloudflare:build`
- **Build output directory:** `build`
- **Node version:** see `engines.node` in [package.json](package.json) (currently `>=22.21.0`)
- **Environment variables:** `API_BASE_URL`, `WS_BASE_URL`, `CANONICAL_HOST` (set per environment: Production gets the prod values, Preview gets staging values)

### Security headers

[`_headers.template`](_headers.template) is the source of truth; `cloudflare:stage` renders it into `build/_headers` with `{{API_BASE_URL}}` / `{{WS_BASE_URL}}` / `{{CANONICAL_HOST}}` substituted from the environment. The `<meta http-equiv>` CSP in `index.html` is kept in sync via the same template substitution and serves as a fallback for local `file://` loads.
