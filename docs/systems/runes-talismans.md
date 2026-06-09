# Runes, talismans & offerings

**Offerings** are the fuel: spent to level the **runes**, their **blessings**, and their **spirits**.
**Talismans** (crafted from rarity **fragments**) add bonus rune levels. The runes then boost almost
everything — accelerators, multipliers, global speed, offerings/obtainium, quarks, and ascension
score. Source: `Runes.ts` (`getRuneEffects`, roster at `Runes.ts:24-30`), `Talismans.ts`,
`RuneBlessings.ts`, `RuneSpirits.ts`.

## Diagram

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef bug fill:#f9a825,color:#000,stroke:#d50000,stroke-width:3px;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  offerGen["Offering generators"]:::ported
  offerings["Offerings"]:::ported
  runes["Runes ×7 +finiteDescent"]:::ported
  runeBless["Rune blessings"]:::ported
  runeSpirits["Rune spirits"]:::partial
  talismans["Talismans ×7"]:::partial
  fragments["Rarity fragments"]:::partial

  resets["Resets award ↗ reset-cascade"]:::ext
  antSac["Ant sacrifice ↗ ants"]:::ext
  prod["Accel/mult/speed ↗ core-economy"]:::ext
  obt["Obtainium ↗ core-economy"]:::ext
  quarks["Quarks ↗ meta-economy"]:::ext
  score["Ascension score ↗ ascension-cubes"]:::ext

  offerGen -->|"generate"| offerings
  resets -->|"award"| offerings
  offerings -->|"level EXP"| runes
  offerings -->|"level"| runeBless
  offerings -->|"level"| runeSpirits
  fragments -->|"craft"| talismans
  antSac -->|"craft items"| fragments
  talismans -->|"+ levels"| runes
  runeBless -->|"amplify"| runes
  runeSpirits -->|"amplify"| runes

  runes -->|"accel/mult/speed"| prod
  runes -->|"× offerings"| offerings
  runes -->|"× obtainium"| obt
  runes -->|"× quarks"| quarks
  runes -->|"finiteDescent"| score
```

## The roster

Seven indexed runes — **speed, duplication, prism, thrift, superiorIntellect, infiniteAscent,
antiquities** — plus a special **finiteDescent** (ascension-score). Effects (per `Runes.ts`): speed →
accelerator power + global speed; duplication → multiplier boosts + tax reduction; prism → production
+ cost divisor; thrift → cost delay + salvage; superiorIntellect → offerings + obtainium;
infiniteAscent / antiquities → late-game OOM bonuses; finiteDescent → ascension score.

## Port status

| System | Status | Rust |
|---|---|---|
| Offerings | 🟩 Ported | `mechanics/resource_gain.rs` (awarded on every reset tier) |
| Runes | 🟩 Ported | `state/runes.rs`, `mechanics/rune_*.rs` — effective-level pipeline now wired (was H3) |
| Rune blessings | 🟩 Ported | `mechanics/rune_blessing_effects.rs` — fed `rune_blessing_power(…)` (was H4) |
| Rune spirits | 🟨 Partial | `mechanics/rune_spirit_effects.rs` (several inert at default) |
| Talismans + fragments | 🟨 Partial | `state/talismans.rs`, `mechanics/talisman_*.rs` |

## Porting notes / open bugs

- **H3 — effective-level pipeline: fixed (PR #265).** Rune effects now read
  `first_five_effective_rune_level = (raw + free) × effectiveness_mult` (`tick/mod.rs:824`), and
  `infiniteAscent` is present in the roster. (This was the map's most prominent bug in the first draft,
  cut before #265.)
- **H4 — blessing power: fixed (PR #265).** Blessing effects are now fed `rune_blessing_power(state, …)`
  rather than the raw level, so they scale instead of pinning near 1.0×.
- **Talismans (still partial):** rarity is never recomputed → stays 0, which zeroes all rarity-indexed
  effects. Rune assignment still maps the legacy-deprecated schema.
- **Thrift blessing** is blocked on the accelerator-boost buy (see [core-economy.md](core-economy.md)).
