# Synergism Project Context for Claude

## Project Overview
- **Name**: Synergism (idle game)
- **Tech Stack**: TypeScript, HTML, CSS
- **URL**: https://synergism.cc
- **Repository**: npm workspaces monorepo. Frontend code lives in `packages/web_ui`; portable game logic lives in `packages/logic`; `packages/desktop_ui` is a placeholder for the planned Tauri integration.
- **Backend**: Connected via `packages/web_ui/src/Login.ts` with mocking in `packages/web_ui/src/mock/`

## Agent Role & Workflow
### Primary Tasks
- Implement frontend features
- Fix bugs and issues
- Architect new feature systems

### Required Actions
1. **Always ask permission** before adding variables to `player` object (affects savefile size)
2. **Check back with user** after writing significant code
3. **Ask questions** when task requirements are unclear

## File Structure Rules
```
packages/
  logic/                   # Headless, DOM-free game logic (future Rust port target)
    src/
      state/               # GameState type, defaults, migrations, serialize
      math/                # bignum wrapper, pure formatters
      mechanics/           # one mechanic per file; cubes/ for the cube family
      events/              # CoreEvent union
      tick/                # pure tick body
  web_ui/                  # Browser frontend (current Synergism.cc)
    src/                   # Existing game UI code
    index.html
    Synergism.css
    Pictures/
    translations/en.json   # Required for all new text strings (UI tier)
    scripts/               # build/staging helpers (cwd-relative)
  desktop_ui/              # Placeholder for the planned Tauri runtime
```

## Package boundary: `packages/logic`

`packages/logic` is the long-term target for a Rust port (browser via WASM, Tauri desktop, server-side simulators). Keep it portable.

**Nothing under `packages/logic/src/` may:**
- Import anything outside `packages/logic/`.
- Reference `document`, `window`, `localStorage`, `sessionStorage`, `navigator`, `location`.
- Reference `DOMCacheGetOrSet` or any DOM cache utility.
- Call `i18next.t()` or import `i18next` at all.
- Call `Alert`, `Confirm`, `Prompt`, `Notification` (the modal helpers in `packages/web_ui/src/`).
- Import from `@synergism/web_ui` or `@synergism/desktop_ui`. Direction is **UI → logic** only.
- Read or write UI state: `currentTab`, `saveString`, modal-visibility flags, theme selection, etc.

Public logic functions follow the shape `(state, input) => { state, events }`. Side effects live in the UI tier; logic communicates intent via the `events` array.

Enforcement: `packages/logic/tsconfig.json` excludes the DOM lib, and `.oxlintrc.json` has an `overrides` block scoped to `packages/logic/**/*.ts` that adds `no-restricted-globals` / `no-restricted-imports`.

## Development Patterns

### String Internationalization
- i18next: Add all user-facing text to `packages/web_ui/translations/en.json`
- **Styling**: `<<color|{{text}}>>` for colored text
- i18n is a UI-tier concern only — never call `i18next.t()` from `packages/logic`.

### Save System Variables
**CRITICAL**: Before adding to `player` object:
1. Get explicit permission from user
2. Add to `packages/web_ui/src/types/Synergism.ts`
3. Add to `packages/web_ui/src/saves/PlayerSchema.ts`
4. Variable location: `player` in `packages/web_ui/src/Synergism.ts`

Once mechanics start migrating to `packages/logic`, the game-state portion of these types will move to `packages/logic/src/state/schema.ts`; UI-state fields stay in `web_ui`.

## Code Conventions

### Critical Performance & Style Requirements
- **DOM Access (web_ui only)**: ALWAYS use `DOMCacheGetOrSet('elementId')` instead of `document.getElementById`
  - Import: `import { DOMCacheGetOrSet } from './Cache/DOM'`
  - Reason: Performance optimization through caching
  - `packages/logic` must not touch the DOM at all — see the boundary section above.

### General Patterns
- Follow existing TypeScript patterns in codebase
- Use established import/export structures
- Match existing naming conventions
- Maintain consistency with current architecture

### Desktop / Steam (deferred)

The Electron-based Steam build was removed. The plan is to reintroduce a desktop runtime via Tauri + Rust later in the roadmap, with Steam SDK integration and Discord Rich Presence reimplemented on that side. Until then, treat this as a browser-only codebase — there is no `platform === 'steam'` gate, no `PLATFORM` build-time macro, no `src/steam/` contract layer. Don't reintroduce a Steam/Electron abstraction. New desktop-only work should wait for the Tauri integration.

### Recommended Patterns
- Objects and arrays that are constant should be hoisted to the module scope when possible.

Example (wrong):
```ts
function myFunction () {
  const arr = [1, 2, 3, 4, 5]
  return arr
}
```

Example (correct):
```ts
const arr = [1, 2, 3, 4, 5]

function myFunction () {
  return arr
}
```
