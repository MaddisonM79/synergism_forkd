# Synergism Forkd — Project Context for Claude

## Project overview

- **Name**: Synergism Forkd (Rust port of the TypeScript game Synergism)
- **Tech stack**: Rust (workspace of 8 crates), Dioxus for UI, Tauri for the desktop shell
- **Status**: bare-bones scaffold. Most crates contain a single placeholder function; real porting is the long-term work
- **Repository**: Cargo workspace at repo root
- **Legacy TS** lives in `legacy/original/` (pre-split) and `legacy/core_split/` (current `packages/`) — frozen reference, **not maintained**

## Repo layout

```
/  (repo root)
├── Cargo.toml                          # workspace manifest
├── rust-toolchain.toml                 # stable + clippy + rustfmt + wasm32 target
├── .cargo/config.toml
├── crates/
│   ├── synergismforkd_bignum/          # break_eternity wrapper (Decimal); stub
│   ├── synergismforkd_common/          # shared types, IDs, error enums
│   ├── synergismforkd_logic/           # headless game logic (state/math/mechanics/events/tick)
│   ├── synergismforkd_save/            # save format + (de)serialization + migrations
│   ├── synergismforkd_ui/              # platform-agnostic Dioxus components
│   ├── synergismforkd_ui_web/          # WASM browser entry point
│   ├── synergismforkd_ui_desktop/      # Tauri shell (Win/Mac/Linux)
│   └── synergismforkd_testkit/         # fixtures, mock state, sim runner, parity helpers + synergismforkd-sim CLI
├── assets/
│   ├── translations/en.json
│   ├── pictures/
│   └── sounds/
├── .github/workflows/
│   ├── rust-ci.yml                     # build + test + clippy on Win/Mac/Linux
│   ├── rust-bench.yml                  # nightly criterion
│   └── desktop-release.yml             # tagged Tauri releases
└── legacy/
    ├── original/                       # frozen pre-split TS, reference only
    └── core_split/                     # current packages/ snapshot (TS logic + web_ui)
```

## Agent role & workflow

### Primary tasks
- Port mechanics from `legacy/core_split/packages/logic/src/mechanics/` into `crates/synergismforkd_logic/src/mechanics/`
- Flesh out the Dioxus UI tree in `synergismforkd_ui` (consumed by web + desktop)
- Wire Tauri integration in `synergismforkd_ui_desktop` once the UI tree is non-empty
- Maintain parity with the TS implementation through the porting period

### Required actions
1. **Always ask permission** before adding fields to the game-state schema in `synergismforkd_logic/src/state/` (the Rust equivalent of "before touching `player`" — it affects save-file size).
2. **Check back with user** after writing significant code.
3. **Ask questions** when task requirements are unclear.

## Crate boundary rules

The TS-era boundary (`packages/logic` could not touch DOM / UI / i18n) generalizes to the whole Rust workspace.

### `synergismforkd_bignum`
- Thin re-export of [`break-eternity-rs`](https://crates.io/crates/break-eternity-rs). `Decimal` is `Copy`, supports the standard arithmetic operators, and exposes the full BE.js helper set (`log10`, `ln`, `pow`, `tetrate`, `iteratedexp`, `iteratedlog`, `slog`, `sqrt`, `cbrt`, `gamma`, `factorial`, `lambertw`, …).
- **Trust model**: `break-eternity-rs` is **project-controlled** (owner: `MaddisonM79`). The 0.3.0 series is stable; the two prior versions (0.1.0, 0.2.0) were yanked to fix inherited bugs and are not expected to recur.
- Other crates depend on `synergismforkd_bignum` only, never on the upstream crate directly — that keeps the underlying impl swappable later.
- Exposes a `serde` cargo feature (on by default); consumers that don't serialize state can opt out with `default-features = false`.

### `synergismforkd_logic`
- **No** `wasm-bindgen`, `web-sys`, `js-sys`, network, filesystem, time-of-day, or async runtime.
- **No** Dioxus, no Tauri, no axum.
- Public functions follow `(state, input) -> (state, events)` shape. Side effects live in the UI / API tiers; logic communicates intent via the returned events.

### `synergismforkd_ui`
- Dioxus components only. **No platform-specific code** (`#[cfg(target_arch = "wasm32")]`, Tauri imports, filesystem). Those live in `ui_web` or `ui_desktop`.

### `synergismforkd_ui_web`
- WASM entry point. May use `wasm-bindgen`, `web-sys`, `dioxus-web`. Should not contain game logic.

### `synergismforkd_ui_desktop`
- Tauri shell. May use `tauri`, Tauri commands, OS APIs. Loads the `ui_web` bundle in its webview.
- Game logic runs in-process in WASM alongside the UI to avoid per-tick IPC. Tauri commands are reserved for native-only operations (file pickers, Steam SDK, Discord RPC).

### `synergismforkd_testkit`
- Test-only utilities. Re-exports fixtures, mock builders, the sim runner, parity helpers. Other crates depend on `testkit` from `[dev-dependencies]` only.

Direction is **UI → logic → bignum, common**. Never the reverse.

## Save system

The Rust save format is **fresh** — no compatibility with the TS savefile.

**Before adding a field to `synergismforkd_logic::state`:**
1. Get explicit permission from the user.
2. Add the field to the right slice in `crates/synergismforkd_logic/src/state/`.
3. If the field needs to persist, add it to the save schema in `crates/synergismforkd_save/src/` and write a migration if the schema version bumps.

## Development patterns

### String internationalization
- All user-facing text goes in `assets/translations/en.json`.
- i18n is a UI-tier concern only — never look up translation keys from `synergismforkd_logic`, `synergismforkd_save`, or `synergismforkd_bignum`.
- The UI crates load translations at build time (or via the asset pipeline once it exists).

### Bignum
- Always import `Decimal` from `synergismforkd_bignum`, never directly from `break_eternity` (or whatever underlies it).
- `Decimal` is `Copy` — pass by value, no `&` or `.clone()` needed at call sites. Operators (`+`, `-`, `*`, `/`) consume copies of their operands.

### Module structure (logic)
Mirror the TS `packages/logic/src/` layout exactly:
- `state/` — sliced state types, one slice per mechanic family
- `math/` — RNG, sigmoid, summation, increment helpers (not bignum)
- `mechanics/` — one file per mechanic; subdirs for families (`cubes/`, etc.)
- `events/` — `CoreEvent` enum, one variant per outcome
- `tick/` — orchestrator (`tack`, `tack_middle`, `tack_tail`)

Each mechanic function takes a state **slice**, not the full game state. Composition happens at the boundary (UI or testkit).

## Code conventions

### Critical
- **Run `cargo fmt` and `cargo clippy --workspace --all-targets`** before submitting. CI rejects warnings.
- **No `unsafe`** unless explicitly justified in a comment and approved by the user.
- **Boundary discipline**: see the per-crate rules above. If a crate needs something it shouldn't import, that's usually a sign the API in `logic` is wrong — fix the API, don't break the boundary.

### General
- Match existing module / file structure within each crate.
- Prefer `pub(crate)` over `pub` for internal helpers.
- Avoid premature abstraction — port the TS function verbatim first, then refactor if needed once tests cover it.
- Hoist module-scope constants out of hot loops:

```rust
// wrong
fn my_function() -> Vec<i32> {
    vec![1, 2, 3, 4, 5]
}

// right
const ARR: &[i32] = &[1, 2, 3, 4, 5];
fn my_function() -> &'static [i32] {
    ARR
}
```

## Desktop / Steam (deferred)

Steam SDK integration and Discord Rich Presence are planned for the Tauri shell (`synergismforkd_ui_desktop`) but not wired in the scaffold. Until that PR lands, treat the workspace as browser-first. The Tauri shell currently ships a placeholder `main`.

## Legacy folders

`legacy/original/` and `legacy/core_split/` are reference material. **Do not** modify their contents; **do not** add CI for them. Reading them while porting mechanics is encouraged. Each folder's own `package.json` still works locally if you want to run the TS for comparison (`cd legacy/core_split && npm test`).

## Testing

- `cargo test --workspace` runs unit + integration tests.
- `cargo run -p synergismforkd_testkit --bin synergismforkd-sim` drives the headless sim.
- `cargo bench` runs criterion (once benches are added under `crates/<crate>/benches/`).
- Bench shapes mirror the TS perf harness in `legacy/core_split/packages/logic/test/perf/` for cross-side comparability.

## Quick reference

| What | Command |
|---|---|
| Build everything | `cargo build --workspace` |
| Run tests | `cargo test --workspace` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Format | `cargo fmt --all` |
| Headless sim | `cargo run -p synergismforkd_testkit --bin synergismforkd-sim` |
| WASM build | `cargo build -p synergismforkd_ui_web --target wasm32-unknown-unknown` |
