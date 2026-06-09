# Singularity & ambrosia

The meta-layer above ascension. **Singularity** resets everything for **golden quarks**, which buy
golden-quark and **octeract** upgrades plus **perks**. **Ambrosia** (and its sibling **red ambrosia**)
is a parallel idle currency spent on **blueberry** upgrades. Source: `singularity.ts`,
`SingularityChallenges.ts`, `Octeracts.ts`, `BlueberryUpgrades.ts`, `RedAmbrosiaUpgrades.ts`.

> **Whole layer is paused in Rust.** The pieces are ported but `singularity_count` never increments,
> so nothing here actually fires yet. Colors reflect that: machinery 🟩/🟨 but the layer is 🟧 inert.

## Singularity

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef stub fill:#ef6c00,color:#fff,stroke:#bf360c;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  singReset["Singularity reset · paused"]:::stub
  goldenQuarks["Golden quarks"]:::ported
  gqUpg["GQ upgrades"]:::ported
  octeracts["Octeracts"]:::ported
  octUpg["Octeract upgrades"]:::ported
  singChal["Singularity challenges · inert"]:::stub
  singPerks["Perks · milestones · inert"]:::stub

  asc["Ascension ↗ ascension-cubes"]:::ext
  quarks["Quarks ↗ meta-economy"]:::ext
  score["Ascension score ↗ ascension-cubes"]:::ext
  coins["Broad mults ↗ core-economy"]:::ext

  asc ==>|"reset all"| singReset
  quarks -->|"convert"| goldenQuarks
  singReset -->|"grant"| goldenQuarks
  goldenQuarks -->|"buy"| gqUpg
  gqUpg -->|"× score"| score
  gqUpg -->|"broad mults"| coins
  asc -->|"gain"| octeracts
  octeracts -->|"buy"| octUpg
  octUpg -->|"broad mults"| coins
  singReset -->|"enable"| singChal
  singChal -->|"reward"| singPerks
  singPerks -->|"broad mults"| coins
```

## Ambrosia / blueberry / red ambrosia

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  ambrosia["Ambrosia"]:::ported
  blueberries["Blueberries · loadouts"]:::ported
  blueberryUpg["Blueberry upgrades"]:::partial
  redAmbrosia["Red ambrosia"]:::ported
  redAmbUpg["Red ambrosia upgrades"]:::ported

  sing["Singularity ↗ (above)"]:::ext
  quarks["Quarks ↗ meta-economy"]:::ext
  cubes["Cubes ↗ ascension-cubes"]:::ext

  sing -->|"unlock"| ambrosia
  ambrosia -->|"convert"| blueberries
  ambrosia -->|"spend"| blueberryUpg
  blueberries -->|"allocate"| blueberryUpg
  blueberryUpg -->|"boost"| quarks
  blueberryUpg -->|"boost"| cubes
  redAmbrosia -->|"spend"| redAmbUpg
  redAmbUpg -->|"boost"| ambrosia
```

## Port status

| System | Status | Rust |
|---|---|---|
| Singularity reset / layer | 🟧 Stub (paused) | `state/singularity.rs`, `tick/mod.rs:5600+` |
| Golden quarks + GQ upgrades | 🟩 Ported (inert) | `state/golden_quarks.rs`, `mechanics/golden_quark_upgrades.rs` |
| Octeracts + upgrades | 🟩 Ported (inert) | `state/octeract_upgrades.rs`, `mechanics/octeracts.rs` |
| Singularity challenges / perks | 🟧 Stub | ported but never entered/triggered |
| Ambrosia | 🟩 Ported | `state/ambrosia.rs`, `mechanics/ambrosia.rs` |
| Blueberry upgrades | 🟨 Partial | `mechanics/blueberry_upgrades.rs` — `effective_levels` deferred to caller |
| Red ambrosia + upgrades | 🟩 Ported | `state/red_ambrosia.rs`, `mechanics/red_ambrosia_*.rs` |

## Porting notes

- This layer is **parked**, not broken: golden-quark / octeract / perk / challenge machinery is ported
  but every reward is frozen at identity because the count never moves. Reviving it needs a production
  path that increments `singularity_count`.
- GQ per-slot metadata is zeroed at default (`cost_per_level=0`, `max_level=0`) — a free-unlimited-level
  hazard if an unseeded buy ever runs (medium finding).
- A few singularity reward fns have zero production callers (`no_quark_upgrades_effect`,
  `sadistic_prequel`, `taxman_last_stand`).
