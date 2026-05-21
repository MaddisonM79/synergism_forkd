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
4. (Optional) override env defaults by copying [`.env`](.env) to `.env.local` and editing. The committed `.env` points at the legacy `synergism.cc` backend; `.env.local` overrides per-developer values (gitignored).
5. Switch to a new branch:
   ```sh
   git checkout -b my-branch-name
   ```
6. Start the dev server:
   ```sh
   node --run dev
   ```
   Runs Vite on port 3000 with HMR — module changes hot-replace, no full reload, ~200ms feedback. To smoke-test against the prod build (with real `_headers` applied at the edge), run `node --run preview:cf` instead — that builds, stages, and serves via `wrangler pages dev`.
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
| `VITE_API_BASE_URL` | `https://synergism.cc` | Backend HTTP origin (login, payments, messages, translations) |
| `VITE_WS_BASE_URL` | `wss://synergism.cc` | Consumables WebSocket origin |
| `VITE_CANONICAL_HOST` | `synergism.cc` | Canonical web hostname (gates browser-only purchase UI; tightens MSW's unhandled-request guard) |

Vite injects these into the JS bundle via `import.meta.env` and substitutes them into [`index.html`](index.html) using its native `%VITE_*%` syntax. [`scripts/stage.mjs`](scripts/stage.mjs) renders [`_headers.template`](_headers.template) → `build/_headers` with the same values, since Vite doesn't model Cloudflare's `_headers` file natively.

The committed [`.env`](.env) holds the defaults. Cascading override order: `.env.local` > `.env.{mode}.local` > `.env.{mode}` > `.env`. Production reads from CF Pages dashboard environment variables (which take precedence over everything in the repo).

## Deploy (Cloudflare Pages)

```sh
npm run cloudflare:build
```

This runs `vite build` (writes hashed bundle + transformed `index.html` to `build/`, copies Pictures/translations via `vite-plugin-static-copy`) and then `cloudflare:stage` (renders `_headers` into `build/`). The result at `build/` is exactly what gets uploaded to CF Pages.

### Cloudflare Pages dashboard settings

- **Framework preset:** None
- **Build command:** `npm run cloudflare:build`
- **Build output directory:** `build`
- **Node version:** see `engines.node` in [package.json](package.json) (currently `>=22.21.0`)
- **Environment variables:** `VITE_API_BASE_URL`, `VITE_WS_BASE_URL`, `VITE_CANONICAL_HOST` (set per environment: Production gets the prod values, Preview gets staging values)

### Security headers

[`_headers.template`](_headers.template) is the source of truth; `cloudflare:stage` renders it into `build/_headers` with `{{API_BASE_URL}}` / `{{WS_BASE_URL}}` / `{{CANONICAL_HOST}}` substituted from the environment. The `<meta http-equiv>` CSP in `index.html` is kept in sync via Vite's `%VITE_*%` substitution and serves as a fallback for local `file://` loads.

### Smoke-testing against the prod build

```sh
node --run preview:cf
```

Builds + stages + serves `build/` via `wrangler pages dev` on port 3000. Use this to verify the CSP / HSTS / cache rules behave as expected before pushing.
