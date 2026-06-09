# Infrastructure

The cross-cutting tech that every gameplay system rides on: the **game loop** (tick), the **calculate
engine**, the central **state schema**, the **events** enum, **save**/import-export, the **UI** render
layer, the **automation** overseer, and the **RNG**. In Rust these map to the crate boundary
**UI → logic → bignum/common** (never the reverse). Source: `Synergism.ts`, `Calculate.ts`,
`UpdateHTML.ts`/`UpdateVisuals.ts`, `Tabs.ts`, `ImportExport.ts`, the `core_split/.../tick/` modules.

## Diagram

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef stub fill:#ef6c00,color:#fff,stroke:#bf360c;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  gameLoop["Game loop · tack/middle/tail"]:::partial
  calcEng["Calculate engine"]:::ported
  stateSchema["Player state schema"]:::partial
  events["Events enum"]:::ported
  save["Save · Import/Export"]:::stub
  uiRender["UI render · Tabs/HTML/Visuals"]:::stub
  autoOverseer["Automation overseer"]:::ported
  rng["RNG · seed"]:::ported

  prod["Production ↗ core-economy"]:::ext
  resets["Resets ↗ reset-cascade"]:::ext
  cubeOpen["Cube opening ↗ ascension-cubes"]:::ext

  gameLoop -->|"drives"| calcEng
  gameLoop -->|"each tick"| prod
  calcEng -->|"reads/writes"| stateSchema
  events -.->|"emitted by"| gameLoop
  stateSchema -.->|"persist"| save
  save -.->|"load"| stateSchema
  uiRender -.->|"reads"| stateSchema
  rng -->|"seeded rolls"| cubeOpen
  gameLoop -->|"automation"| autoOverseer
  autoOverseer -->|"auto-reset / roomba / sweep"| resets
```

## Crate boundaries (Rust)

| Concern | Crate | Rule |
|---|---|---|
| Game loop, calc, state, events | `synergismforkd_logic` | no UI / wasm / fs / time-of-day / async |
| Big numbers | `synergismforkd_bignum` | thin `break-eternity-rs` wrapper; `Decimal` is `Copy` |
| Shared IDs / errors | `synergismforkd_common` | leaf dependency |
| Components | `synergismforkd_ui` | Dioxus only, platform-agnostic |
| Browser / desktop | `synergismforkd_ui_web` / `_desktop` | wasm / Tauri shells |
| Fixtures, sim runner | `synergismforkd_testkit` | dev-dependency only |

## Port status

| System | Status | Rust |
|---|---|---|
| Tick / game loop | 🟨 Partial | `tick/mod.rs` (phases wired; `updateAll` autobuyers absent) |
| Calculate engine | 🟩 Ported | `mechanics/calculate.rs`, `math/*` (leaf math faithful; golden-vector coverage thin) |
| State schema | 🟨 Partial | `state/` (~80%; `unlocks` only 8/21 keys; some rune-blessing type divergence) |
| Events enum | 🟩 Ported | `events/mod.rs` |
| Save / Import-Export | 🟧 Stub | `crates/synergismforkd_save/` (serde scaffold; no migration) |
| UI render | 🟧 Stub | `synergismforkd_ui*` (scaffold) |
| Automation overseer | 🟩 Ported | `tick/auto_reset.rs`, `auto_research.rs`, `challenge_sweep.rs`, `automatic_tools.rs` |
| RNG | 🟩 Ported | deterministic Xoshiro, per-purpose seeding (used by cube opening) |

## Porting notes

- The **logic core is healthy**; the main infrastructure gaps are **save-load** (blocks the achievement
  full-table recompute and the Rust save format) and the **UI** tree (still scaffold).
- State schema is the gating dependency for several features: adding fields requires explicit sign-off
  (it affects save-file size) per the project rules.
- `updateAll` autobuyers are the notable tick gap — without them an idle loop won't auto-purchase
  producers/ants/cubes the way the monolith's 50 ms loop does.
