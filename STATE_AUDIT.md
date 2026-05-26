# State-slice Audit

Read-only sweep of the ~72 ported mechanic modules. For each subsystem
this lists:

- **Reads** — state fields the ported formulas already consume as
  inputs (currently passed in as `f64` / `Decimal` scalars).
- **Mutates** — fields any state-mutating port would need to write.
  Most subsystems have no state-mutating port yet — these are
  formula-only at present.
- **Status** — `EXISTS` (slice landed), `PARTIAL` (some fields live,
  rest missing), `MISSING` (nothing landed).

The legacy reference is `legacy_core_split/packages/web_ui/src/types/Synergism.ts`
and the per-feature `Features/*/structs/structs.ts` files.

---

## Already-landed slices (baseline)

These nine slices exist in `crates/synergismforkd_logic/src/state/`:

| Slice                          | Backs                                     |
| ------------------------------ | ----------------------------------------- |
| `AcceleratorState`             | `accelerator_boosts`, `accelerators`      |
| `MultiplierState`              | `multipliers`                             |
| `ProducerFamilyState`          | `producers` (Coin / Diamond / Mythos / Particle families) |
| `ParticleBuildingsState`       | `particle_buildings`                      |
| `CrystalUpgradesState`         | `crystal_upgrades`                        |
| `UpgradesState`                | `upgrades`                                |
| `BlessingValues` (×3 layers)   | `cube_blessings`, `tesseract_blessings`, `hypercube_blessings` |
| `PlatonicBlessings`            | `platonic_blessings`                      |
| `TesseractBuildingsState`      | `tesseract_buildings`                     |

---

## Missing slices, grouped by subsystem

### 1. Ants — **MISSING**

Backs: `ant_producers`, `ant_sacrifice_reward_calc`, `ant_sacrifice_rewards`,
`ant_upgrade_levels`, `ant_upgrades`, `ant_masteries`, `ant_reborn_elo`.

**Reads** (formula inputs):
- `player.ants.producers[0..=8].purchased` (f64 ×9)
- `player.ants.producers[0..=8].generated` (Decimal ×9)
- `player.ants.masteries[0..=8].mastery` (u8 ×9, 0..=12)
- `player.ants.masteries[0..=8].highestMastery` (u8 ×9)
- `player.ants.upgrades[0..=15]` (f64 ×16)
- `player.ants.crumbs` (Decimal)
- `player.ants.immortalELO` (f64)
- `player.ants.rebornELO` (f64)
- `player.ants.antSacrificeCount` (f64)

**Mutates** (when buy/sacrifice mechanics land):
- All of the above + `crumbsThisSacrifice`, `crumbsEverMade`, `currentSacrificeId`
- `quarksGainedFromAnts` (f64)
- `highestRebornELODaily` / `highestRebornELOEver` (`Vec<{elo: f64, sacrificeId: u32}>` ×2, leaderboards)
- `toggles` (10-field struct: autobuyProducers/Masteries/Upgrades, maxBuyProducers/Upgrades, autoSacrificeEnabled/Threshold/Mode, alwaysSacrificeMaxRebornELO, onlySacrificeMaxRebornELO)

**Proposed lean MVP slice** (defer leaderboards + toggles until they land in mechanics):
~30 leaf scalars / Decimals.

---

### 2. Cube tier balances — **MISSING**

Backs: `cube_blessings`, `tesseract_blessings`, `hypercube_blessings`,
`platonic_blessings`, `cube_upgrades`, `platonic_upgrade_costs`, `calculate::calc_corruption_stuff` outputs.

**Reads/Mutates**:
- `player.wowCubes` (Decimal)
- `player.wowTesseracts` (Decimal — already plain `f64` in `TesseractBuildingsState`, may need widening)
- `player.wowHypercubes` (Decimal)
- `player.wowPlatonicCubes` (Decimal)
- `player.wowHepteracts` (Decimal)
- `player.wowAbyssals` (f64)
- `player.wowOcteracts` (Decimal)
- `player.totalWowOcteracts` (Decimal)
- Per-tier "this ascension" counters

Could land as one `CubeBalancesState` slice (~12 fields).

---

### 3. Cube + platonic upgrades — **MISSING**

Backs: `cube_upgrades`, `platonic_upgrade_costs`, plus dozens of
`cubeUpgrade[X]` and `platonicUpgrade[X]` reads scattered across
`update_all_multiplier`, `update_all_tick`, `calculate`,
`golden_quark_upgrades`, etc.

- `player.cubeUpgrades[1..N]` (~80 entries, f64 levels)
- `player.platonicUpgrades[1..N]` (~25 entries, f64 levels)

Could be `Vec<f64>` in a `CubeUpgradeLevelsState` slice (~2 vecs total).

---

### 4. Runes — **MISSING**

Backs: `rune_levels`, `rune_exp_multiplier`, `rune_upgrade_progression`,
`rune_effects`, `rune_blessing_effects`, `rune_spirit_effects`,
`rune_level_bonuses`.

**Reads/Mutates**:
- `player.runelevels[0..6]` (f64 ×7: speed, duplication, prism, thrift, superiorIntellect, antiquities, finiteDescent — sometimes finiteDescent/horseShoe variants extend this)
- `player.runeexp[0..6]` (f64 ×7)
- `player.runeShards` (Decimal)
- `player.runeBlessingLevels[0..N]` (f64 ×6, blessing levels)
- `player.runeSpiritLevels[0..N]` (f64 ×6, spirit levels)
- Free-rune bonus levels accumulated from talismans / ant-upgrade-8

One `RunesState` slice (~28 leaf fields).

---

### 5. Talismans — **MISSING**

Backs: `talisman_costs`, `talisman_levels`, `talisman_effects`.

**Reads/Mutates**:
- `player.talismanLevels[0..6]` (f64 ×7)
- `player.talismanRarity[0..6]` (f64 ×7)
- `player.talismanOne..Seven` (per-talisman fragment-allocation state — each is `[boolean, 0|1|2|3|4|5]` or similar)
- Fragment / shard balances:
  - `player.talismanShards`, `player.commonFragments`, `player.uncommonFragments`,
    `player.rareFragments`, `player.epicFragments`, `player.legendaryFragments`,
    `player.mythicalFragments` (all f64)

One `TalismansState` slice (~30 fields).

---

### 6. Hepteracts — **MISSING**

Backs: `hepteract_values`, `hepteract_effects`.

**Reads/Mutates**:
- `player.hepteractCrafts.{chronos,hyperrealism,quark,challenge,abyss,accelerator,acceleratorBoost,multiplier}.{BAL, CAP, BASE_CAP, AUTO, UNLOCKED}` — 8 craft types × 5 fields each
- `player.overfluxOrbs` (f64)
- `player.overfluxPowder` (f64)

One `HepteractsState` slice (~42 fields).

---

### 7. Challenges — **MISSING**

Backs: `challenges` (CalcECC), and effectively every formula that
reads `player.challengecompletions[i]` (which is most of them).

**Reads/Mutates**:
- `player.challengecompletions[1..=15]` (f64 ×15)
- `player.highestchallengecompletions[1..=15]` (f64 ×15)
- `player.currentChallenge.transcension` (u32)
- `player.currentChallenge.reincarnation` (u32)
- `player.currentChallenge.ascension` (u32)

One `ChallengesState` slice (~33 fields, mostly arrays).

---

### 8. Researches — **MISSING**

Backs: `researches`, plus `update_all_multiplier` / `update_all_tick`
read ~20 research slots each by index.

**Reads/Mutates**:
- `player.researches[1..200+]` (f64 vec)
- `player.researchPoints` (f64 or Decimal)
- `player.obtainium` (Decimal)

One `ResearchesState` slice. The data array is ~200 entries.

---

### 9. Achievements — **MISSING**

Backs: `achievement_levels`, `achievement_points`.

**Reads/Mutates**:
- `player.achievements[1..N]` (Vec<u8> or BitSet — N ≈ 280)
- `player.progressiveAchievements` (8 progressive-achievement-state structs)
- `player.achievementPoints` (f64)

One `AchievementsState` slice (~10 fields, one big vec).

---

### 10. Ambrosia — **MISSING**

Backs: `ambrosia`, `blueberry_upgrades`.

**Reads/Mutates**:
- `player.ambrosia` (f64) — current balance
- `player.lifetimeAmbrosia` (f64)
- `player.blueberryTime` (f64) — generation accumulator
- `player.ambrosiaRNG` (f64)
- `player.spentBlueberries` (f64)
- `player.ambrosiaUpgrades[<name>]` (35 named upgrades, each `{level, freeLevel}` style — could collapse to a vec or hashmap)
- `player.singularityChallenges.noAmbrosiaUpgrades.enabled` (already covered under #13)

One `AmbrosiaState` slice (~6 leaf fields + a 35-entry upgrade-level map).

---

### 11. Red ambrosia — **MISSING**

Backs: `red_ambrosia_bonuses`, `red_ambrosia_upgrades`.

**Reads/Mutates**:
- `player.redAmbrosia` (f64) — balance
- `player.lifetimeRedAmbrosia` (f64)
- `player.redAmbrosiaTime` (f64)
- `player.redAmbrosiaRNG` (f64)
- `player.spentRedAmbrosia` / `spentRedBlueberries` (f64)
- `player.redAmbrosiaUpgrades[<name>]` (27 named upgrades, level field)

One `RedAmbrosiaState` slice (~5 leaf + 27-entry map).

---

### 12. Shop — **MISSING**

Backs: `shop_costs`, `shop_upgrades`, plus `update_all_*` reads about
10 shop slots each.

**Reads/Mutates**:
- `player.shopUpgrades[<name>]` (83 named upgrades, each is a single `f64` level)
- `player.shopPotionsConsumed` (f64)
- `player.shopBuyMaxToggle` (bool / enum)
- Per-upgrade infinite-shop-vouchers state for the late-game

One `ShopState` slice (~85 fields, mostly a wide vec).

---

### 13. Singularity — **MISSING**

Backs: `singularity_helpers`, `singularity_milestones`, `singularity_penalties`,
`singularity_challenges`, and `update_all_*` reads the count.

**Reads/Mutates**:
- `player.singularityCount` (f64)
- `player.highestSingularityCount` (f64)
- `player.singularityCounter` (f64, in-singularity timer)
- `player.singularityChallenges.<9 names>.{enabled, completions, highestSingularityCompleted}` (27 fields)

One `SingularityState` slice (~32 fields).

---

### 14. Golden quarks — **MISSING**

Backs: `gq_upgrade_cost`, `gq_upgrade_levels`, `golden_quark_upgrades` (~80 effects).

**Reads/Mutates**:
- `player.goldenQuarks` (Decimal)
- `player.goldenQuarksTimer` / per-singularity counter (f64)
- `player.singularityUpgrades[<name>]` (~80 entries, each `{level, freeLevel, maxLevel, canExceedCap, qualityOfLife, specialCostForm, ...}`)
- `player.runeBlessings` / `player.runeSpirits` shop-pricing fields (cross-couple with #4)

One `GoldenQuarksState` slice (~3 leaf + an 80-entry upgrade map).

---

### 15. Octeracts — **MISSING**

Backs: `octeract_bonuses`, `octeract_upgrade_levels`, `octeracts` (~42 effects).

**Reads/Mutates**:
- `player.octeractTimer` (f64)
- `player.octeractUpgrades[<name>]` (42 entries, `{level, freeLevel}` shape)

One `OcteractsState` slice (~42-entry upgrade map). Balances live in #2.

---

### 16. Corruptions — **MISSING**

Backs: `corruptions`, plus most ascension-related formulas read
`player.corruptions.used.<14 names>`.

**Reads/Mutates**:
- `player.corruptions.used.{viscosity, dilation, illiteracy, ...}` (14 named corruption levels)
- `player.corruptions.next.<14 names>` (next-ascension preview)
- `player.corruptions.used.totalCorruptionAscensionMultiplier` (f64 — derived but cached)
- `player.corruptions.used.loadout` (Vec<u32>, 14-long)

One `CorruptionsState` slice (~30 fields).

---

### 17. Ascension counters + reset stats — **PARTIAL**

Some reset-counter math lives in `reset_currency` and `reset_time_and_auto_obtainium`,
but no state slice has been added for the actual counters.

**Reads/Mutates**:
- `player.ascensionCount` (f64)
- `player.ascensionCounter` / `ascensionCounterReal` / `ascensionCounterRealReal` (f64 ×3, real-time clocks)
- `player.prestigeCount` (f64)
- `player.transcendCount` (f64)
- `player.reincarnationCount` (f64)
- `player.prestigeShards`, `player.transcendShards`, `player.reincarnationShards`, `player.diamondShards` (Decimal ×4 — some already in `CrystalUpgradesState`)
- `player.prestigeUnlocked` / `transcendUnlocked` / `reincarnationUnlocked` / `achievementsUnlocked` etc. (bools)

One `ResetCountersState` slice (~15 fields).

---

### 18. Coins + coin counters — **PARTIAL**

`UpgradesState` and `ProducerFamilyState` carry `coins`. The full coin-counter
suite isn't covered.

**Reads/Mutates**:
- `player.coins` (already)
- `player.coinsThisPrestige` (Decimal)
- `player.coinsThisTranscension` (Decimal)
- `player.coinsThisReincarnation` (Decimal)
- `player.coinsTotal` (Decimal)
- `player.firstOwnedCoin..fifthOwnedCoin` (already in `ProducerFamilyState` for the Coin family)

Mostly already there; needs an "extended-coin-counter" companion to `UpgradesState`.

---

### 19. Quarks — **MISSING**

Backs: `quarks`, plus quark-mult math scattered across ambrosia / sing / octeract.

**Reads/Mutates**:
- `player.worlds` (Quarks wrapper — in Rust likely a plain `Decimal` plus the bonus-mult helper)
- `player.quarksThisSingularity` (f64)
- `player.allTimeQuarks` (f64)

One `QuarksState` slice (~3 fields).

---

### 20. Campaigns + constants — **MISSING**

Backs: `campaign_token_rewards`, plus the `cubeUpgrade21/31/41`-style
constant lookups in `calculate`.

**Reads/Mutates**:
- `player.campaigns.<name>.completions` (~10 campaigns)
- `player.campaigns.tokensSpent` (f64)
- `player.constantUpgrades[1..N]` (~10 entries, f64)
- `player.ascendShards` (Decimal)

One `CampaignsState` + one `ConstantsState` (or merged) slice (~25 fields).

---

### 21. Event buffs — **PARTIAL (formulas exist, state missing)**

Backs: `event_buffs`, `potion_bonuses`, `overflux_bonuses`.

**Reads/Mutates**:
- `player.usedCoupons[<event-id>]` (Vec<u8> per event)
- `player.singularityChallenges.<n>.enabled` (already in #13)
- `player.dayCheck` (f64 timestamp — daily-reset gate)

One `EventBuffsState` slice (~6 fields).

---

### 22. Level / level rewards — **MISSING**

Backs: `level_milestones`, `level_rewards`.

**Reads/Mutates**:
- `player.level` (f64)
- `player.levelTier` (f64)
- `player.levelXP` (Decimal)

One `LevelState` slice (~3 fields).

---

## Summary

| # | Subsystem | Fields | Mutating mechanic exists? |
| - | --------- | ------ | ------------------------- |
| 1 | Ants | ~30 | No |
| 2 | Cube balances | ~12 | No |
| 3 | Cube + platonic upgrades | 2 vecs (~105 levels) | No |
| 4 | Runes | ~28 | No |
| 5 | Talismans | ~30 | No |
| 6 | Hepteracts | ~42 | No |
| 7 | Challenges | ~33 | No |
| 8 | Researches | ~3 + 200-vec | No |
| 9 | Achievements | ~10 + 280-vec | No |
| 10 | Ambrosia | ~6 + 35-map | No |
| 11 | Red ambrosia | ~5 + 27-map | No |
| 12 | Shop | ~85 | No |
| 13 | Singularity | ~32 | No |
| 14 | Golden quarks | ~3 + 80-map | No |
| 15 | Octeracts | 42-map | No |
| 16 | Corruptions | ~30 | No |
| 17 | Reset counters | ~15 | No (partial in upgrades/etc) |
| 18 | Coin counters | ~4 | Partial (already in UpgradesState) |
| 19 | Quarks | ~3 | No |
| 20 | Campaigns + constants | ~25 | No |
| 21 | Event buffs | ~6 | No |
| 22 | Level | ~3 | No |

**Total**: 22 missing/partial slices, roughly 600+ leaf fields once
all vecs/maps are expanded.

---

## Priority recommendations

If goal is **MVP playable game**:
1. **Coin counters + Coin family + Quarks** (#18, #19) — feeds the tick loop.
2. **Researches + Challenges + Corruptions** (#8, #7, #16) — drives most multipliers.
3. **Reset counters + Ascension** (#17) — basic progression scaffolding.
4. **Cube balances + Cube/platonic upgrades** (#2, #3) — ascension rewards.
5. **Runes + Talismans** (#4, #5) — mid-game progression.
6. **Achievements** (#9) — gates many upgrades.
7. **Shop** (#12) — paid-progression.
8. **Hepteracts** (#6).
9. **Ants** (#1) + reborn-ELO.
10. **Ambrosia + Red ambrosia** (#10, #11).
11. **Singularity + GQ + Octeracts** (#13, #14, #15) — end-game.
12. **Event/Level/Campaign/Constants** (#20, #21, #22) — late polish.

If goal is **tick orchestrator working end-to-end**:
- The tick reads `player.X.Y` everywhere. Need *all* slices before
  the tick can compose without taking 200+ scalar inputs. Could land
  them in a single sprint or land them as the tick orchestrator
  grows to need each one.
