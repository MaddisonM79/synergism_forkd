# @synergism/logic

Headless game logic for Synergism. Long-term target for a Rust port (browser via WASM, desktop via Tauri, server-side simulators).

## Boundary rules

**Nothing under `src/` may:**

- Import anything outside `packages/logic/`.
- Reference `document`, `window`, `localStorage`, `sessionStorage`, `navigator`, `location`.
- Reference `DOMCacheGetOrSet` or any DOM cache utility.
- Call `i18next.t()` or import `i18next` at all.
- Call `Alert`, `Confirm`, `Prompt`, `Notification` (the custom modal helpers in `packages/web_ui/`).
- Import from `@synergism/web_ui` or `@synergism/desktop_ui`. Direction is **UI → logic** only.
- Read or write UI state: `currentTab`, `saveString`, modal-visibility flags, theme selection, etc.

Enforcement:

- `tsconfig.json` uses `"lib": ["ES2022"]` — no DOM types. Browser globals become type errors.
- `.oxlintrc.json` at the repo root has an `overrides` block scoped to `packages/logic/**/*.ts` that adds `no-restricted-globals` and `no-restricted-imports` for the above.

## Function shape

Every public function should follow:

```ts
(state: GameState, input: TInput) => { state: GameState, events: CoreEvent[] }
```

State is data-in / data-out. Side effects (DOM, audio, alerts, persistence) are the UI tier's job — logic communicates intent via the `events` array.

## Layout

```
src/
├── index.ts        # public API
├── state/          # GameState type, defaults, migrations, serialization
├── math/           # bignum wrapper, pure formatters
├── mechanics/      # game-rule modules (one mechanic per file)
│   └── cubes/      # cube-family submodules
├── events/         # CoreEvent union
└── tick/           # pure tick body (no setInterval)
```

## Status

Empty scaffolding as of this commit. Code migrates in from `packages/web_ui/src/` one mechanic at a time. See `/Users/maddisonmarkham/.claude/plans/lexical-jingling-sloth.md` (the scaffolding plan) and follow-up plans for the order.
