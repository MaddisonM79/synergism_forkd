# Singularity & ambrosia

The meta-layer above ascension. **Singularity** resets everything for **golden quarks**, which buy
golden-quark and **octeract** upgrades plus **perks**. **Ambrosia** (and its sibling **red ambrosia**)
is a parallel idle currency spent on **blueberry** upgrades. Source: `singularity.ts`,
`SingularityChallenges.ts`, `Octeracts.ts`, `BlueberryUpgrades.ts`, `RedAmbrosiaUpgrades.ts`.

> **The layer is now LIVE — with nothing deferred.** `perform_singularity_reset` (`tick/reset.rs`)
> increments `singularity_count`, grants golden quarks (`calculateGoldenQuarks`), and rebuilds the
> player from a blank save preserving meta-progression — triggered by `ResetRequest::Singularity`
> (gated on the antiquities rune). The 80-entry GQ-upgrade metadata is seeded, so costs are real.
> The **exalt enter/exit loop** (`PlayerAction::ToggleSingularityChallenge`), the **elevator triad**
> (locked / slow-climb / teleport, `ConfigureSingularityElevator` + `TeleportToSingularity`), and the
> limitedTime **preserveQuarks** branch are all ported.

## Singularity

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef stub fill:#ef6c00,color:#fff,stroke:#bf360c;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  singReset["Singularity reset · live"]:::ported
  goldenQuarks["Golden quarks"]:::ported
  gqUpg["GQ upgrades"]:::ported
  octeracts["Octeracts"]:::ported
  octUpg["Octeract upgrades"]:::ported
  singChal["Singularity challenges"]:::ported
  singPerks["Perks · milestones · table"]:::ported

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
  blueberryUpg["Blueberry upgrades"]:::ported
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
| Singularity reset / layer | 🟩 Ported (live) | `tick/reset.rs::perform_singularity_reset`, `calculate_golden_quarks` |
| Golden quarks + GQ upgrades | 🟩 Ported | `state/golden_quarks.rs` (80-entry metadata seeded), `mechanics/golden_quark_upgrades.rs` |
| Octeracts + upgrades | 🟩 Ported | `state/octeract_upgrades.rs`, `mechanics/octeracts.rs` |
| Singularity challenges (Exalts) | 🟩 Ported | enter/exit loop (`reset.rs::toggle_singularity_challenge`), 9-row meta + requirement ladder (`mechanics/singularity_challenges.rs`), completions drive the effect readers + the exalt progressive |
| Singularity perks / milestones | 🟩 Mostly | milestone formulas (`singularity_milestones.rs`) + the **53-perk table** (`singularity_perks.rs`: roster + `levels[]` ladders + `getLastUpgradeInfo` helpers). Effect-wiring documented per-perk: the multiplicative perks are WIRED; the rest are DEFERRED on an unported *consumer* (ant-speed stat product) or are export-flow / UI / external |
| Ambrosia | 🟩 Ported | `state/ambrosia.rs`, `mechanics/ambrosia.rs` |
| Blueberry upgrades | 🟩 Ported | `mechanics/blueberry_upgrades.rs`; effective levels populated (`populate_ambrosia_free_levels`), quark upgrades wired |
| Red ambrosia + upgrades | 🟩 Ported | `state/red_ambrosia.rs`, `mechanics/red_ambrosia_*.rs` |

## Porting notes

- ✅ **Revived.** `perform_singularity_reset` (`Reset.ts:1063-1285`) is the blankSave reconstruction:
  reset to `GameState::default()` (== `reset_save`), restore the meta-progression survivors
  (achievements, GQ balance + grant + all 80 upgrades + `total_quarks_ever`, octeract / ambrosia-upgrade
  / red-ambrosia trees, shop, singularity challenge state + counts, automation prefs, RNG/level,
  never-tier rune/talismans, ant high-water marks, `cubeUpgrades[80]`, and the prestige/transcend/
  reincarnation counts once highest ≥ 8). Count auto-climbs `max(highest, count + lookahead)`.
- ✅ **GQ metadata seeded** (`GQ_UPGRADE_SEEDS`, 80 rows) — `cost_per_level` / `max_level` / cost-form
  now real, closing the free-unlimited-level hazard.
- ✅ **Exalt enter/exit loop ported** (`PlayerAction::ToggleSingularityChallenge`): enter gates on
  `unlockSingularity` + not-inside-an-Exalt, holds the singularity counter and quark/GQ export timers
  across a jump to `singularityRequirement(baseReq, completions)`; exit succeeds iff antiquities was
  re-purchased inside — completions re-derive from the requirement ladder *before* the return jump
  (legacy ordering), then back to the held highest. Export timers restore only on failure (verbatim).
  Completions now grow in play, lighting the exalt effect readers and the progressive exalt slot.
- ✅ **Elevator triad ported** (3 new `SingularityState` fields; slowClimb defaults *true* per
  blankSave): locked → the target floor (the was-at-highest +1 bump applies regardless, verbatim),
  slow-climb → `count + 1`, else the lookahead auto-climb. `ConfigureSingularityElevator` clamps the
  target like the legacy input listener; `TeleportToSingularity` ascends via a full singularity
  (counter held when antiquities is unpurchased) and **descends by just setting the count — no reset**
  (verbatim quirk).
- ✅ **`preserveQuarks` ported:** `worlds` resets with the rebuild unless the limitedTime reward is
  active, in which case the balance carries across (Reset.ts:1190).
- ✅ **Perk table modeled.** `mechanics/singularity_perks.rs` ports `singularityPerks`
  (`singularity.ts:2303-3156`) as data — the `SingularityPerkId` enum + the 53-row roster with each
  perk's exact `levels[]` ladder (extracted mechanically) + the `getLastUpgradeInfo` helpers
  (`level_at` / `is_active` / `next_threshold`) + `active_perk_count`. Closes the "full perk list is
  not modeled" gap. The module header tags each perk's effect-wiring: the big multiplicative perks
  (`goldenCoins` / `skrauQ` / `goldenRevolution2` / `primalPower` / `derpSmiths` / `immaculateAlchemy`
  / salvage / token / ELO / blueberry) are already **WIRED** (mostly via the meta-economy sweep). The
  four **ant** perks resolved as: `antGodsCornucopia` **WIRED** (the `SingularityPerk` term of the
  fully-ported `compute_ant_speed_mult`); `forTheLoveOfTheAntGod` **WIRED** (reset-time regrants, see
  below); `irishAnt` / `irishAnt2` are **NONE** — they exist only in the `singularityPerks` table with
  no mechanical consumer anywhere in TS, so the faithful port leaves them no-ops. The
  `goldenRevolution` GQ-per-second family belongs to the export-reward flow; the rest are UI/external.
- ✅ **`forTheLoveOfTheAntGod` ported** (`apply_for_the_love_ant_regrants`): the
  `highestSingularityCount >= 10/15/20` producer / upgrade / crumb regrants that run at the tail of
  every ant reset (`Features/Ants/.../player/reset.ts`, outside the tier gate) — Workers 20→40 +
  Breeders + MetaBreeders, AntSpeed/Mortuus `max`-merged, crumbs 1e50 at sing 20. Applied on all
  three ant-reset paths (sacrifice / ascension / singularity); the singularity path uses the
  pre-increment `old_highest` (`resetAnts(singularity)` runs at Reset.ts:1103, before the count bump).
- ✅ **Export-reward claim ported** (`claim_export_rewards`, `ImportExport.ts:254-273`): the
  `goldenQuarksTimer` → golden-quarks (gated on `goldenQuarks3`, × the `highestSing ≥ 100` export
  bonus) and `quarkstimer` → `worlds` + `quarksThisSingularity` (× the full quark multiplier)
  conversions, each keeping its window remainder. The host calls it on a real export; previously the
  timers accrued with nowhere to land. The external subscriber term is a parked RMT seam → neutral.
