# Ascension & cubes

Ascension converts a run into an **ascension score**, which is the gate for the four cube tiers. Cubes
are opened for random **blessings** and spent on **cube/platonic upgrades** that boost the whole game.
Above a Challenge-15 threshold, score also yields **hepteracts**, which auto-craft into **overflux**
that loops back to boost cubes and quarks. Source: `Calculate.ts:1135-1294`
(`calculateAscensionScore`, `CalcCorruptionStuff`), `Cubes.ts`/`Platonic.ts`, `Hepteracts.ts`.

## Ascension score & the cube tiers

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  ascReset["Ascension reset"]:::ported
  ascShards["Ascend shards"]:::ported
  ascBldg["Ascend buildings ×5"]:::ported
  ascScore["Ascension score"]:::partial
  wowCubes["Wow cubes"]:::ported
  tesseracts["Tesseracts"]:::ported
  hypercubes["Hypercubes"]:::ported
  platonicCubes["Platonic cubes"]:::ported
  cubeOpen["Cube opening · RNG"]:::ported
  cubeBless["Cube blessings"]:::ported
  platBless["Platonic blessings"]:::ported
  cubeUpg["Cube upgrades"]:::ported
  platUpg["Platonic upgrades"]:::ported

  corr["Corruptions ↗ challenges"]:::ext
  ch["Challenges 11–15 ↗ challenges"]:::ext
  camp["Campaign ↗ challenges"]:::ext
  fd["finiteDescent rune ↗ runes"]:::ext
  gq["GQ upgrades ↗ singularity"]:::ext
  prod["Production ↗ core-economy"]:::ext

  ascReset -->|"grant"| ascShards
  ascShards -->|"buy"| ascBldg
  ascReset -->|"compute"| ascScore
  ascBldg -->|"+ base"| ascScore
  ch -->|"ECC base"| ascScore
  corr -->|"× multiplier"| ascScore
  camp -->|"× / bonus"| ascScore
  fd -->|"× score"| ascScore
  gq -->|"× score"| ascScore

  ascScore -->|"threshold"| wowCubes
  ascScore -->|"threshold"| tesseracts
  ascScore -->|"threshold"| hypercubes
  ascScore -->|"threshold"| platonicCubes
  wowCubes -->|"spend"| cubeOpen
  tesseracts -->|"spend"| cubeOpen
  hypercubes -->|"spend"| cubeOpen
  platonicCubes -->|"spend"| cubeOpen
  cubeOpen -->|"roll"| cubeBless
  cubeOpen -->|"roll"| platBless
  wowCubes -->|"spend"| cubeUpg
  platonicCubes -->|"spend"| platUpg
  cubeUpg -->|"boost"| prod
  cubeBless -->|"boost"| prod
  platUpg -->|"ascension speed"| ascReset
  platUpg -->|"+ score"| ascScore
  platBless -->|"+ score"| ascScore
```

## Hepteracts & overflux

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef bug fill:#f9a825,color:#000,stroke:#d50000,stroke-width:3px;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  ch15["Challenge 15 ↗ challenges"]:::ported
  hepteracts["Hepteracts ·6"]:::partial
  hepAutoCraft["Hepteract auto-craft"]:::ported
  overfluxPowder["Overflux powder"]:::ported
  overfluxOrbs["Overflux orbs"]:::ported
  wowCubes["Wow cubes (this page)"]:::ported
  quarks["Quarks ↗ meta-economy"]:::ext
  accel["Accel / mult ↗ core-economy"]:::ext

  ch15 -->|"score unlocks"| hepteracts
  hepteracts -->|"auto-craft"| hepAutoCraft
  hepAutoCraft -->|"convert"| overfluxPowder
  hepAutoCraft -->|"convert"| overfluxOrbs
  overfluxPowder -->|"× cube"| wowCubes
  overfluxPowder -->|"× quark"| quarks
  overfluxOrbs -->|"× cube"| wowCubes
  hepteracts -->|"accel/mult"| accel
```

## How it connects

- **In:** corruptions, campaign, challenges 11–15, the finiteDescent rune, ant upgrades, and GQ
  upgrades all feed **ascension score**. C15 (on [challenges-corruptions](challenges-corruptions.md))
  unlocks hepteracts.
- **Out:** cube/platonic upgrades and blessings boost core production; overflux loops back into cubes
  and quarks.

## Port status

| System | Status | Rust |
|---|---|---|
| Ascension reset + award | 🟩 Ported | `tick/reset.rs:460-650` |
| Ascension score | 🟨 Partial | under-credited by **H2** (ant true-level) and a c11 collapse |
| Cube tiers + opening (RNG) | 🟩 Ported | `mechanics/cube_opening.rs` (was audit **H6**) |
| Cube/platonic blessings + upgrades | 🟩 Ported | `cube_blessings.rs`, `platonic_blessings.rs`, `cube_upgrades.rs`, `platonic_upgrade_costs.rs` |
| Hepteracts | 🟨 Partial | `mechanics/hepteract_values.rs`, `hepteract_effects.rs` |
| Overflux orbs / powder | 🟩 Ported | `mechanics/overflux_bonuses.rs` |

## Porting notes / open bugs

- **Cube opening** (audit **H6**) and **ascension reset/award** are done — this whole tree is largely
  faithful and is one of the more complete areas.
- **Hepteracts:** raw `.bal` is fed to 4 effects (chronos/hyperrealism/accelerator/multiplier),
  skipping the diminishing-returns softening past 1000 (medium finding). Challenge + acceleratorBoost
  hepteracts correctly call `hepteract_effective()`.
- **Score** is the main carrier of the **H2** ant-true-level shortfall — see [ants.md](ants.md).
- Cube upgrades 4/5/6 regrant on ascension reset rather than immediately (medium: intra-ascension loss).
