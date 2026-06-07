<!--
Generated 2026-06-06 by a 48-agent parity-audit workflow (3 inventory · 20 family compare · 20 adversarial verify · 4 cross-cutting skeptics · 1 synthesis).
212 findings: 2 critical · 31 high · 54 medium · 61 low · 64 info. All high-confidence.
C1, C2, H1 independently re-verified from source by the orchestrator after the workflow.
This file is untracked scratch — keep or delete freely.
-->

# Synergism Forkd — TS->Rust Logic Parity Audit

## Headline verdict

The port is a **high-quality but partial** translation: the leaf math is excellent and the per-dt economic spine is structurally faithful, but a single orchestration bug and several systematically-unwired acquisition/awarding subsystems mean **headless play diverges from the TypeScript reference the moment a save leaves turn-0 default state**. Honest overall parity for *reachable mid-game play* is roughly **55–65%** — the formula libraries are ~90%+ faithful (and adversarially re-verified against the monolith, not just the extracted package), but the live tick produces materially wrong player-facing numbers because (a) the global-speed multiplier is dropped from all resource generation, (b) achievements/cube-opening/rune-blessing/constant-upgrade acquisition never run so dozens of bonuses stay inert at level 0, and (c) the entire reincarnation→ascension challenge ladder is unreachable because completing challenge 10 never sets the ascension unlock. **The single biggest risk is the global-speed-multiplier omission** (`tick/mod.rs:565`): it is confirmed by four independent agents, airtight, in *normal* progression, and silently invisible because it is identity at default state so every per-function golden vector and the default sim pass clean.

A second structural truth frames everything below: the Rust crate mirrors the **extracted** pure-fn package (`legacy/core_split/packages/logic`), which is itself a deliberate **subset** of the monolith that excludes the entire "acquisition/awarding" tier (anything that lived next to the `Currency` class, the DOM, or `Math.random()`). Most large gaps are *inherited from that extraction boundary*, not Rust regressions — but they are still missing logic that a headless parity build must eventually own.

## Subsystem status table

| Subsystem | Status | ~Ported | Top risk (in-play) |
|---|---|---|---|
| Global aggregation spine (tack/dt) | faithful-with-gaps | 93% | **CRITICAL: global-speed mult dropped from all generation** |
| Accelerators / tax | faithful-with-gaps | 90% | upgrade-68 floor-order regression; boostAccelerator unported |
| Coin economy (producers/multipliers/crystal) | faithful-with-gaps | 99% | BUYMAX+1 off-by-one (low reach) |
| Prestige / reincarnation economy | faithful-with-gaps | 90% | **prestige_shards dual-slice desync (crystals never grow)** |
| Cube buildings / blessings | faithful-with-gaps | 95% | blessing effects wired but **cube-opening absent → levels frozen at 0** |
| Cube upgrades / platonic | faithful-with-gaps | 90% | cube-upgrade 4/5/6 regrant deferred to reset |
| Runes | faithful-with-gaps | 70% | **effective-level pipeline unported → raw level fed to all effects** |
| Talismans | faithful-with-gaps | 50% | rune-level bonus dropped; rarity never recomputed (effects→0) |
| Ants — production | faithful-with-gaps | 95% | **true-ant-level (free levels + extinction divisor) bypassed everywhere** |
| Ants — sacrifice/ELO/masteries | faithful-with-gaps | 78% | **sacrifice payout entirely unimplemented**; raw-level ELO |
| Challenges | faithful-with-gaps | 80% | **c10 never unlocks ascensions; c15 exponent never accrues** |
| Corruptions | faithful-with-gaps | 82% | ant true-level bypass (shared); hyperchallenge inflation neutralized |
| Achievements | partial | 33% | **no auto-grant + points frozen → crystal/mythos mults under-credit in normal play** |
| Shop | faithful-with-gaps | 90% | bonus-level composition unmodeled; noQuarkUpgrades zeroing absent |
| Golden quarks / octeracts | faithful-with-gaps | 95% | GQ per-slot metadata zeroed at default (free unlimited levels hazard) |
| Ambrosia / blueberry / red | faithful-with-gaps | 97% | effectiveLevels (free-level + Exalt zero) deferred to caller |
| Hepteracts | faithful-with-gaps | 60% | 4 effects fed RAW .bal → DR past LIMIT 1000 skipped |
| Singularity | faithful-with-gaps (paused) | 65% | whole layer runtime-INERT (count never moves); 3 reward fns dead |
| Tick orchestration / reset | faithful-with-gaps | 72% | resetOfferings unwired; updateAll autobuyers absent; offline-sim absent |
| State schema | faithful-with-gaps | 80% | `unlocks` only 8/21 keys; runeBlessings type/semantics divergence |
| Test / parity harness | faithful-with-gaps | 18% (golden) | orchestration + aggregators have NO TS-anchored vectors |

## Critical findings

### C1. Global-speed multiplier dropped from ALL resource generation
- **Confirmed by 4 agents** (global-aggregation-spine, tick-orchestration-reset, skeptic-phase-order, accelerators-tax-adjacent) — highest possible cross-confirmation; rated **critical**.
- **What is wrong**: The monolith computes `timeMult = calculateGlobalSpeedMult()` and calls `resourceGain(dt * timeMult)` (`legacy/original/src/Synergism.ts:4604-4605`); the extracted `resourceGain.ts:27-28` documents its `dt` param as **"already scaled by globalSpeedMult by the caller"**. The Rust `tack` computes the multiplier (`compute_global_speed_mult_pre`, `tick/mod.rs:474`) and applies it to the prestige/transcend/reincarnation **timers** and the ant-sac timer — but passes **raw `input.dt`** to `phase_generation` (`tick/mod.rs:565`). `resource_gain.rs:294` then does only `dt_scaled = dt/0.025` with zero speed reference.
- **rust_location**: `crates/synergismforkd_logic/src/tick/mod.rs:565` (call) + `crates/synergismforkd_logic/src/mechanics/resource_gain.rs:294` (consumer)
- **ts_reference**: `legacy/original/src/Synergism.ts:4604-4605`; contract doc `legacy/core_split/packages/logic/src/mechanics/resourceGain.ts:27-28`
- **In-play consequence**: Every coin / diamond / mythos / particle / ascend-shard production and the passive prestige/transcend/reincarnation point drips are under-counted by exactly `calculateGlobalSpeedMult()`. That multiplier exceeds 1 in essentially all real play (`1 + researches[121]/50` alone reaches ~3× at the level-100 cap of an ordinary mid-game reincarnation research; plus speed rune, cube/tess/hyper/platonic global-speed blessings, cube upgrades 18/34/52, Chronos talisman, dilation corruption, singularity perks). The ant-generation leg correctly uses raw dt (matches monolith), so **only the resourceGain leg is wrong** — which is the entire core economy. Also slows c1-5 auto-completion timing.
- **Why it hid**: identity at default (mult = 1.0), so per-function golden vectors (which pass pre-scaled dt) and the default sim never exercise it.
- **Remediation**: pass `input.dt * compute_global_speed_mult_pre(state)` into `phase_generation`. **Effort: ~1 line + a multi-tick regression test** asserting coin growth scales with a raised research-121. Low effort, high payoff — do this first.

### C2. Completing challenge 10 never unlocks ascensions — the entire c11–c15 ladder is dead in normal play
- **Confirmed by 2 agents** (challenges, skeptic-done-claims); rated **critical**.
- **What is wrong**: The monolith reincarnation-challenge block sets `unlocks.anthill` (highest c8>0), `unlocks.talismans`+`unlocks.blessings` (c9>0), `unlocks.ascensions` (c10>0) (`Synergism.ts:3691-3700`). The Rust highest-rise match arm (`tick/mod.rs:4641-4647`) handles **only q_idx 11/12/13/14** with `_ => {}` for everything else — there is **no branch for reincarnation indices 8/9/10**. `reset_counters.ascension_unlocked` is consumed as the c11 entry gate (`tick/mod.rs:4375`) but is assigned `true` **only inside `#[cfg(test)]`** (lines 6562/6606/6628), never in production. And there are no `anthill`/`talismans_unlocked`/`blessings_unlocked` fields on `ResetCountersState` at all.
- **rust_location**: `crates/synergismforkd_logic/src/tick/mod.rs:4641-4647`
- **ts_reference**: `legacy/original/src/Synergism.ts:3691-3700`
- **In-play consequence**: A player can complete c6–c10 but can **never enter any ascension challenge** (c11 gates on the never-set flag; c12–15 gate on completing c11 first). The c10→ascension chain — the spine of the ascension endgame — is unreachable through normal progression. Note `highestChallengeRewards` (the quark award) is a *separate* call and was ported, so its presence does **not** cover this unlock.
- **Remediation**: wire `ascension_unlocked = true` on the c10 highest rise (the field already exists); add `anthill`/`talismans_unlocked`/`blessings_unlocked` to `ResetCountersState`. **Effort: small for the c10→ascension wire (1 line, no schema change); +schema-add permission for the three reincarnation unlock bools (anthill/talismans/blessings gate whether those mechanics are active at all).**

## High findings

### H1. `prestige_shards` dual-slice desync — diamonds/crystals never accumulate, crystal coin multiplier permanently undervalued
- **Single-agent (prestige-reincarnation-economy), adversarially traced across all three reference layers; self-admitted in code.**
- **What is wrong**: `resource_gain.rs:380` SEEDS prestige shards from `state.reset_counters.prestige_shards` (which is only ever read here and zeroed on reset — never written during generation, so permanently 0), but `tick/mod.rs:4842` WRITES the result back to a **different** slice `state.crystal_upgrades.prestige_shards`. The two slices are never synced. The monolith uses **one** canonical `player.prestigeShards` for all four roles (seed, writeback, crystal-multiplier read, buy-spend) — `Synergism.ts:3179`.
- **rust_location**: `crates/synergismforkd_logic/src/mechanics/resource_gain.rs:380` (seed) vs `crates/synergismforkd_logic/src/tick/mod.rs:4842` (writeback); slices `state/reset_counters.rs:39`, `state/crystal_upgrades.rs:21`
- **ts_reference**: `legacy/original/src/Synergism.ts:3179-3182`
- **In-play consequence**: Three effects in **normal early/mid play**: (1) diamond/prestige shards (= crystals) never grow across ticks (every tick computes `0 + one_tick_production` and overwrites); (2) the crystal **coin** multiplier (`tick/mod.rs:824`) reads a tiny one-tick value, under-crediting coin production throughout early/mid game; (3) crystal-upgrade purchases that decrement the slice are erased the same tick (buy runs at line 564, generation overwrites at 565). `transcend_shards`/`reincarnation_shards` do **not** share this bug.
- **Remediation**: seed `resource_gain.rs:380` from `state.crystal_upgrades.prestige_shards` so read-slice = write-slice = multiplier-slice = buy-slice. **Effort: 1 line + a 2-tick growth regression test (the existing single-tick test passes coincidentally and masks it).**

### H2. Ant true-level (`calculateTrueAntLevel`) bypassed at all production sites — free-level doubling + extinction divisor both dropped
- **Confirmed by 3 agents** (ants-production, corruptions, ants-sacrifice for the ELO sub-path); higher confidence; **the single most pervasive multiplier defect**.
- **What is wrong**: The monolith reads *every* ant-upgrade effect via `getAntUpgradeEffect → effect(calculateTrueAntLevel(X))`, where `trueLevel = (level + min(level, freeLevels)) / corruptionEffects('extinction')` (÷1 for the 4 exempt upgrades). The Rust feeds **raw `state.ants.upgrades[i]`** at ~13 wired sites. `calculate_true_ant_level` *is* ported and correctly applied at exactly **one** site (the Mortuus research[192] cube sub-term, `tick/mod.rs:2188`), proving the asymmetry is an omission, not a missing function.
- **rust_location**: `crates/synergismforkd_logic/src/tick/mod.rs:679,722,788,966,1193,1244,1368,1521,1651,2145,2233,2821,3108,3485,3487` (raw level); correct ref at `:2188`
- **ts_reference**: `legacy/original/src/Features/Ants/AntUpgrades/lib/total-levels.ts:6-18` + `upgrade-effects.ts:5-8`
- **In-play consequence**: Two opposing effects, both large: free levels (researches 97/98/102/132/200, c8/c9/c11 ECC, constantUpgrade6 — all **ant-sacrifice-era, not deep-late-game**) can nearly **double** the effective level; extinction corruption (≥1 on any ascension; baked to 11 in c15 loadouts) **divides by 1.25..15**. Blast radius: coin mult, multiplier mult, accelerator-boost mult, building power, tax reduction, obtainium mult, ant-speed, mortuus global-speed, mortuus2 ascension-speed, ascension-score base, wow-cubes, ant-ELO, ant-sacrifice ELO. The Coins term is especially severe — its level is an **exponent** on crumbs.
- **Correction to one agent**: this is **not** blocked on the extinction divisor — `extinction_divisor_at_level()` over `EXTINCTION_DIVISOR` *is* fully ported and the level is `corruptions.used.levels[EXTINCTION_INDEX]`; the working Mortuus consumer proves both terms are computable today. Only the two free-level *inputs* (`freeAntUpgrades` achievement reward, `challenge15` bonusAntLevel) remain neutral-defaulted.
- **Remediation**: route every listed site through `calculate_true_ant_level(i)` (supplying free-level inputs as neutral 0/1 until those subsystems land). **Effort: medium (~13 call-site edits + a free-level regression fixture).**

### H3. Rune effective-level pipeline unported — raw purchased level fed into all rune effects
- **Confirmed by 3 agents** (runes, talismans, ants for the consuming-side picture); self-admitted only for the speed-spirit line.
- **What is wrong**: The monolith evaluates every rune effect at `getRuneEffectiveLevel = !unlocked ? 0 : reincarnationChal9 ? 1 : (level + freeLevels()) * effectiveLevelMult()` (`Runes.ts:807-816`). The Rust passes **raw `rune_levels[i]`** at every site (prism/duplication/speed/thrift/SI/finiteDescent/antiquities). No `getRuneEffectiveLevel` equivalent exists. The supply side is also dead: `rune_level_bonuses.rs` aggregators and the `rune_free_levels` state field have **zero callers** (verified).
- **rust_location**: `crates/synergismforkd_logic/src/tick/mod.rs:778,858,932,953-959,1017,1038-1042,1236-1242,1353-1356,1626,2239,2739,2817,3128,4791`; dead supply at `mechanics/rune_level_bonuses.rs`, `state/runes.rs:54`
- **ts_reference**: `legacy/original/src/Runes.ts:807-816,286-292,121-197`
- **In-play consequence**: Verified to bite in **early reincarnation** play — `firstFiveEffectiveRuneLevelMult`'s first stat line is `1 + researches[4]/10 * (1 + CalcECC)` and researches[4] is an early reincarnation research, so `effectiveLevelMult > 1` in normal mid-reincarnation play, under-stating accelerator-power, free-accelerators, global-speed, multiplier-boosts, free-multipliers, tax-reduction, crystal-production, and SI/thrift offering/obtainium. `freeLevels()` also becomes nonzero early. The `isUnlocked→0` and chal9→1 clamps are entirely absent.
- **Note**: the **blessing** chain (H4) is a sibling of this — its dropped multiplier includes the rune's *own* level, so blessing effects stay pinned near 1.0 far longer than the monolith.
- **Remediation**: implement `get_rune_effective_level()` (compose existing `rune_level_bonuses` + a new `effectiveLevelMult` from the Statistics StatLine) and route all sites through it. **Effort: medium-high (new composer + ~15 call sites + wire the dead `rune_free_levels`).**

### H4. Rune blessing & spirit effective-power chains unported (blessing effects stay ~1.0)
- **Single-agent (runes), traced line-by-line.**
- **What is wrong**: Monolith `getRuneBlessingPower = blessing.level * (rune.level + freeLevels()) * otherBlessingMultipliers()` (`RuneBlessings.ts:197-204`). The Rust passes the **raw blessing level** (`tick/mod.rs:961,1364,3068`). Because the dropped multiplier folds in the rune's own (hundreds-to-thousands) level, effective power is many OOM larger than the raw level. The effect *formulas* are faithful; only the power argument is wrong. Spirits: only `duplication_rune_spirit_effects` is consumed (raw level, `tick/mod.rs:2236`); speed-spirit is hardcoded 1.0; prism/thrift/SI spirit fns have **zero** production callers (all singularity-tier, identity at default).
- **rust_location**: `crates/synergismforkd_logic/src/tick/mod.rs:961-962,1364,3068-3069` (blessings)
- **ts_reference**: `legacy/original/src/RuneBlessings.ts:42-56,197-204`
- **In-play consequence**: Blessings unlock at reincarnation challenge 9 (mid/late reincarnation-ascension, pre-singularity). Once unlocked, the Rust holds the `(1 + power/1e6)` blessing effects near 1.0 far longer than the monolith — under-crediting the cube-multiplier path (blessings ARE live there) and several production multipliers. **Compounds with C2** (blessings are unreachable today because c9 unlock is also missing — see C2).
- **Remediation**: ties into H3 (`otherBlessingMultipliers` + the rune-level fold). **Effort: medium, shares machinery with H3.**

### H5. Achievement auto-granting absent + points frozen — crystal & mythos multipliers under-credit in NORMAL play
- **Confirmed by 2 agents** (achievements, subsystem-census); self-admitted (treated as UI-tier).
- **What is wrong**: The monolith grants achievements during play via `awardAchievement → unlockCondition() → player.achievements[i]=1`, driven by `resetAchievementCheck`/`challengeAchievementCheck`/`buildingAchievementCheck`. **None** of this exists in the logic crate — the `[u8;509]` bitmap is read at ~12 sites and **written at zero** (verified). Separately, `achievement_points` is **never recomputed** during a tick (`compute_achievement_points` has zero callers; the only write is a unit-test fixture), and the progressive-cache `Math.max` accumulation is also unported.
- **rust_location**: `crates/synergismforkd_logic/src/state/achievements.rs` (read-only bitmap + points); `mechanics/achievement_points.rs:170` (no caller); consumed at `mechanics/global_multipliers.rs:343-344,382-384`
- **ts_reference**: `legacy/original/src/Achievements.ts:3499-3514,3518,3572-3584`
- **In-play consequence**: The bitmap stays all-zero forever, so every achievement-gated bonus (early building/challenge/reset achievements granting +accelerators, +multipliers, +crystal mult) is permanently inert — you earn dozens in the first minutes; here none fire. **Critically, `achievement_points` is the EXPONENT of two multipliers reachable in ordinary prestige/reincarnation play**: the crystal multiplier `(1 + 0.01·crystalUpgrade[0])^points` (`global_multipliers.rs:343`) and the mythos multiplier `1.01^points·(points/5+1)` (`global_multipliers.rs:382`, upgrade 47). Frozen points under-credit real coin-economy multipliers in **mid-game, not only late**.
- **Remediation**: this is a genuine design fork (awarding was deliberately delegated to a UI tier that doesn't exist headless). For a headless parity build, port the check functions + `updateAchievementPoints` + progressive cache. **Effort: large** (the awarding/detection machinery is substantial). Even a partial port of the reset/building/challenge group-checks would unfreeze the most common bonuses.

### H6. Cube-opening absent — blessing levels frozen at 0 across all four layers
- **Confirmed by 2 agents** (cube-buildings-blessings, subsystem-census).
- **What is wrong**: All ~40 per-stat blessing-**effect** calcs are ported and consumed in production, but `Cube.open()` — the action that spends cubes/tesseracts/hypercubes/platonic-cubes to **add** blessing levels via the weighted bulk distribution + per-cube RNG pdf — is entirely missing (verified: no blessing-level write anywhere in the crate). Inherited from the extraction boundary (`open()` lived on the `Currency` class).
- **rust_location**: `crates/synergismforkd_logic/src/mechanics/cube_blessings.rs` etc. (effects only); blessing-level state never mutated outside reset
- **ts_reference**: `legacy/original/src/CubeExperimental.ts` — `WowCubes.open` (177), `WowTesseracts.open` (256), `WowHypercubes.open` (297), `WowPlatonicCubes.open` (338)
- **In-play consequence**: Cube balances accumulate (gained on ascension) but can **never be converted into blessings**, so the entire blessing economy evaluates at level 0 (identity). Opening cubes for blessings is a primary mid/late-game activity. The rolled remainder uses `Math.random()` (unseeded), so a future port needs a new `RngPurpose` but is automatically parity-acceptable in expectation (most cubes open in bulk at deterministic expected values).
- **Remediation**: port `Cube.open()` for all 4 layers + reserve `RngPurpose::Cubes/Tesseracts/...`. **Effort: medium-large per layer; medium with shared abstraction.**

### H7. Ant-sacrifice payout entirely unimplemented (event emitted, dropped)
- **Confirmed by 2 agents** (ants-sacrifice, cube-buildings for the multiplier sub-input); self-admitted for the multiplier input.
- **What is wrong**: `tick/automatic_tools.rs:213` emits `CoreEvent::AntSacrificeTriggered` but **nothing consumes it** (verified: enum decl + a `let _ =` discard + emit + a test assertion, no executor). The monolith's `sacrificeAnts()` credits immortalELO, offerings, obtainium (unless asc-14), all 7 talisman tiers (if c9>0), awards `sacMult`, and **resets the ants to crumbs**. None is wired. The faithful leaf calculators are callerless dead code. A second blocker: `calculateAntSacrificeMultiplier` (the `antSacrificeRewardStats` StatLine) has no Rust assembler, so even with an executor the reward magnitudes can't be produced.
- **rust_location**: `crates/synergismforkd_logic/src/tick/automatic_tools.rs:213` + `tick/mod.rs:5133-5152` (collected, no execution)
- **ts_reference**: `legacy/original/src/Features/Ants/AntSacrifice/sacrifice.ts:29`
- **In-play consequence**: On auto-sacrifice the player gains nothing and the board is never reset. The `cubeUpgrades[47]`-gated ant-sacrifice obtainium source is also neutral-defaulted to 0 (`tick/mod.rs:2945-2949`).
- **Remediation**: port `sacrificeAnts()` executor + assemble `antSacrificeRewardStats`. **Effort: medium.**

## Medium / Low findings

**Medium — wiring/arithmetic defects reachable in normal-to-late play:**

- **upgrade-68 floor/divide order inverted** (`mechanics/update_all_multiplier.rs:142`). Rust does `floor(log10(taxdivisor))/1000`; monolith does `floor(log10(taxdivisor)/1000)` (`Synergism.ts:2541`). The fractional over-credit enters the additive accumulator `a` **before** the multiplicative rune/research/cube/ant chain and only floors at the end, so a +0.5 becomes +5 after a 10× chain, shifting the floored `freeMultiplier` (a coin/diamond exponent). Fires when `upgrade[68]>0` AND `taxdivisor ≥ 1e1000` (deep ascension). Confirmed by 2 agents; Rust-only (extracted TS is faithful). **Fix: 1 line.**

- **boostAccelerator buy loop unported** (`mechanics/accelerator_boosts.rs`; no `BuyRequest::AcceleratorBoost`). Cost formula ported, buy loop absent → `acceleratorBoostBought` frozen at 0, removing the entire accelerator-boost prestige-purchase progression (a normal mid-game coin multiplier). Self-admitted; blocked on the reset-system overhaul (the pre-upgrade-46 path calls `reset('prestige')` inline). Confirmed by 2 agents.

- **resetOfferings() unwired** (`tick/reset.rs:118-119`). The monolith awards `calculateOfferings()` on **every** prestige/transcension/reincarnation as the first statement of `reset()`; the Rust awards none (ascension only zeroes). The passive trickle (`tick/mod.rs:5190`) is gated on `highest_challenge_completions[3]>0`, so a **pre-challenge-3 player has zero offering income from any source**. `calculate_offerings` is ported — pure wiring gap. Confirmed by 2 agents (tick-orchestration, skeptic-reset).

- **updateAll building/ant autobuyer layer absent** (no per-tick autobuy in `phase_automation`). The monolith runs `updateAll` (producer/accelerator/multiplier/crystal/cube autobuyers gated by `toggles[1..26]`+unlock upgrades, `autobuyAnts`, `autoUpgrades`, immortal-ELO auto-gain, constant-upgrade autobuy) on a **separate 50ms `fastUpdates` loop**, not `tack` — so the extracted `tackBody` excluded it and the Rust only consolidated the auto-ascension decision. A player with autobuyers on gets no automatic producer/ant growth, stalling the idle loop. **Requires new automation-state toggle fields → schema-add permission.**

- **Ascension reset c10 cube-reward computed AFTER its inputs are wiped** (`tick/reset.rs:576-626` runs after `resetResearches`/`resetAnts`/`resetRunes`). The monolith captures `CalcCorruptionStuff`/`calculateAscensionCount` **before** the wipes (`Reset.ts:419-420`). Rust reads already-zeroed researches (137/152/167/182/192/197, 97/98/102/132), ant upgrades (14/13/11), and finiteDescent rune → wowCubes + dependent tesseract/hyper/platonic/hepteract awards and ascensionCount **under-credited** when those are owned. Identity at default. Flagged by skeptic-reset as **ordering/high**; note this partially contradicts the tick-orchestration agent's "the award is fully implemented" — both are right (it IS implemented, but at the wrong point in the sequence). **Fix: hoist the capture before the sub-resets.**

- **Shop bonus-level composition unmodeled** (`getShopLevel`/`getBonusLevels`; tick passes raw stored level). Every shop reward is too **low** once any bonus source is active (topHat rune, ambrosia/redAmbrosia free-upgrade nodes, InfinityUpgrades, active noQuarkUpgrades). Conversely, the `noQuarkUpgrades`-enabled and `!isUnlocked` level-zeroing is also absent → during that singularity challenge non-Utility shop bonuses stay applied (rewards too **high**, challenge too easy). `shopPanthema` is hard-stubbed to identity in 5+ pipelines and can never be non-identity without the bonus-level system.

- **Hepteract effects fed RAW `.bal`** for chronos/hyperrealism/accelerator/multiplier (`tick/mod.rs:934,1018,1527,2319`), skipping the `hepteractEffective` DR softening past LIMIT=1000. acceleratorBoost (`:658`) and challenge (`challenges.rs:474`) DO apply it, proving the omission. Reachable in **post-C15 pre-singularity** play (hepteract unlock + expand are c15-gated). Example: multiplier BAL=10000 → Rust 4.0× vs monolith ~1.475×. **Fix: wrap each `.bal` in `hepteract_effective(...)` as `:658` already does** (thread chronos's `DR_INCREASE = platonicUpgrades[19]/750`).

- **Challenge requirement neutral-defaults**: `challengeExtension` shop cap zeroed in the completion loop (`tick/mod.rs:4543`) but **read by the sweep** (`:3588`) — internally inconsistent, reincarnation completions plateau below the sweep's expectation. c10 requirement reduction (researches 140/155/170/185) zeroed (`:4482`) → c10 goal stays too high. Hyperchallenge corruption inflation neutralized to 1.0 (`:4483`) → transcend/reincarnation challenges **easier** than monolith under a hyperchallenge loadout. All self-admitted; all reachable.

- **Ascension challenges 11/12/14 can multi-complete per tick** (shared `while counter < max_inc` loop applied to all tiers, `tick/mod.rs:4628`); monolith increments ascension comp/highest by at most +1/tick. Bites with `instantChallenge` shop upgrades. Currently unreachable (blocked by C2).

- **Count-multiplier rewards absent** (prestige/transcension/reincarnationCountMultiplier + CalcECC scaling). Rust increments counts by flat +1 (`reset.rs:149/231/322`); monolith adds `floor(count·multiplier)`. These counts gate ported rewards (obtainium #468, offering #439, salvage #453), so undercounting slightly under-credits them.

- **GQ per-slot metadata zeroed at default** (`state/golden_quarks.rs:212-257`): `cost_per_level=0` and `max_level=0` make `buy_gq_upgrade` treat slots as the unlimited sentinel → **free unlimited GQ levels** if a buy is driven off un-seeded state. Octeract side is **not** affected (caller-supplied cost/cap — correction to a prior framing). UI-tier data table not yet seeded.

- **Test coverage**: tick orchestration and the big aggregators (global mult, update_all_multiplier, obtainium, cube mult) have **no golden vectors** — only single-branch hand-written assertions authored by the porter (same-author hazard). See the dedicated section below.

**Low — bounded, late-gated, or near-inert (grouped):**

- **BUYMAX+1.0 off-by-one** in producers (`producers.rs:253`), multipliers (`multipliers.rs:77`), accelerators (`accelerators.rs:77`), particle buildings (`particle_buildings.rs:80`). All recurse with `BUYMAX+1.0` where TS/monolith use plain `BUYMAX`; leaks a factor into returned cost (and thus owned count) **only above 1e15 owned** — a tetration-tier wall unreached in play. **Correction to two compares: this is NOT "semantically equivalent" — particle_buildings is a factor-of-2 overcost and the producer/multiplier cases leak into state — the self-describing "correct" comments are false.** Untested on both sides. Also: the cube-economy off-by-ones would fail the extracted `acceleratorBoosts.parity.test.ts` BUYMAX golden vector at >1e15.

- **cube-upgrade 4/5/6 upgrade-slot regrant deferred to ascension reset** instead of buy-time (`cube_upgrades.rs:264-314`). The regranted `upgrades[94..100]` are automation upgrades (e.g. `[100]` drips transcendPoints) that affect generation in normal play; a player who buys 4/5/6 mid-ascension loses the bonus until next ascension. Bounded to current ascension. Re-rated low→**medium** by one agent.

- **constUpgrade1Buff/constUpgrade2Buff hardcoded 0 with a factually-wrong comment** (`tick/mod.rs:867-870`). Achievement #270 (ascendShards ≥ 1e50000) grants both = 0.01; the comment "no achievement grants it" is false. Drops +0.01 from the const-upgrade-1 base and +10 from the const-upgrade-2 percent cap once earned. Confirmed by 3 agents. Extreme late-game, but the comment must be corrected.

- **Singularity layer runtime-INERT**: no production path raises `singularityCount` or any challenge `.completions` (every mutation is `#[cfg(test)]`; no `singularity()` reset). So the *whole* layer — penalties (dead code), the genuinely-wired challenge/exalt terms, milestone bonuses — resolves to identity at runtime. 3 challenge reward-effect fns (`no_quark_upgrades_effect`, `sadistic_prequel_effect`, `taxman_last_stand_effect`) have **zero** production callers. Exalt 4's multiplier is consumed only by dead code. Consistent with "singularity paused" policy; default play faithful, feature non-functional by design.

- **InfiniteAscent rune entirely dropped** (wrong 7-rune roster: Rust dropped infiniteAscent, added finiteDescent — `state/runes.rs:19-34`). InfiniteAscent's `cubeMult = 1+n/100` is in the **global** cube multiplier; forced to 1.0 (`tick/mod.rs:2030`). Singularity-shop-gated, so bites only post-singularity. Also no state slot for horseShoe/topHat.

- **State-schema gaps** (subsystem inventory): `unlocks` only 8/21 keys (missing coinone..coinfour early-game gates, rrow1-4, anthill/blessings/talismans feature unlocks — overlaps C2); `runeBlessings`/`runeSpirits` stored as `[f64;7]` *level* arrays but are Decimal *currency-amount* maps (5 runes) in the monolith (type+semantics divergence, lossy above f64 precision); hepteract craft drops `HEPTERACT_CONVERSION`/`OTHER_CONVERSIONS`/`TIMES_CAP_EXTENDED`; daily-cube open/quark counters absent; singularity-elevator/`singularityMatter`/ultimate-pixels absent; per-resource buy-amount selectors, blueberry/corruption loadouts, `offlinetick`/fastest-clear, cube-auto-open toggles all absent. Most correspond to whole unported actions, not data existing mechanics need.

- **Offline catch-up (`calculateOffline`) absent** (`tick/mod.rs` time_warp only gates the head/middle, no 200-step offline driver). Genuine offline-progression logic (timer + resource accrual) with no Rust equivalent; partly an orchestration concern the UI could reconstruct by looping `tack`. Also re-confirms C1 (`calculateOffline:773` likewise does `resourceGain(timeTick * timeMult)`).

- **Effective-levels deferred to caller** (ambrosia/blueberry: `blueberry_upgrades.rs:412-686`; red-ambrosia raw-level is **correct**). Bites in the two Exalt challenges and once any red-ambrosia free-level upgrade is bought; compounds through quarks2/cubes2/luck2/quarks3/cubes3 milestone params. Single-level buy + respec tracking (`*Invested`) also deferred across GQ/octeract/ambrosia/talisman/cube/platonic buys.

- **Talisman rarity never recomputed** (`talisman_levels.rs:198-254` skips it, "left to UI") → `talisman_rarity` stays 0, zeroing every rarity-indexed talisman effect; `rune_assignments` models a deprecated pre-rework schema; midas/mortuus/plastic effect consumers unwired even though those talismans exist in state.

- **resetCurrency one-tick lag + phase-order glue** (skeptic-phase-order): `updateAllTick`/`updateAllMultiplier` order swapped → fresh-vs-stale `totalMultiplier` in transcension-challenge-1 acceleratorEffect (medium); reset-currency point-gains computed from pre-coin-gain coins (one-tick lag, self-correcting); singularity head timer reordered 8→5 (behaviorally inert — **correction to the tick agent's "1:1 legacy order" claim, it is not**).

## Cross-cutting risks

**RNG containment — solid (skeptic-rng, high confidence).** Every RNG draw flows through `RngState::draw(purpose) → math::rng::next_f64` (Xoshiro256PlusPlus, per-purpose). Only **two live consumers**: ambrosia luck (`tick/timers.rs:511`) and red-ambrosia luck (`:643`), both used purely as probability gates (`value < frac(luck/100)`), so expected value is **PRNG-agnostic** (P(value<frac)=frac under both MT and Xoshiro). No entropy leaks: `from_entropy` has zero callers; logic uses `from_seed(0)`; no `thread_rng`/`SystemTime`/`Date::now` in logic. The Rust is *more* deterministic than the monolith (which seeded only 3 streams + `Math.random()` elsewhere). Two notes: (1) the TS `ambrosiaTimers.parity.test.ts` asserts MT-bit-exact values — **correctly NOT ported** (structurally unportable to Xoshiro; don't let anyone reintroduce it as hardcoded expectations); (2) cube-opening's unseeded `Math.random()` reward distribution (H6) needs new `RngPurpose` variants when it lands but is parity-acceptable in expectation.

**Tick phase ordering — faithful spine, two real lags.** The critical issue is C1 (the dt-scaling bug, which is a *missing multiplier* not a misorder). Beyond it: the `updateAllTick`/`updateAllMultiplier` swap (fresh `totalMultiplier` leaks one tick early into transcension-challenge-1 acceleratorEffect — medium) and the reset-currency one-tick coin lag (self-correcting). The g_cache taxdivisor one-tick-lag was adversarially **confirmed faithful** (it correctly reproduces the monolith's mutable-global stale-read/fresh-write ordering, including into the coin-cap). The singularity-timer reorder is inert today but is a latent footgun if a singularity-counter-dependent head timer is ever added.

**Neutral-default debt as a whole.** The port's dominant pattern is "port the formula, feed it an identity input where the producing subsystem isn't ported yet." This is *faithful at default state* and well-documented (the 61/19 gap markers are overwhelmingly inert at turn-0), but it is the mechanism behind nearly every high finding: H2 (free-level/divisor inputs), H3/H4 (effective-level/blessing-power), H5 (frozen points), the shop bonus-levels, the campaign-token system (14 dormant consumers), PseudoCoins (premium-layer multipliers → identity across cube/quark/obtainium/offering StatLines). The risk is that the **sum** of these silently-identity terms makes mid/late-game numbers drift far from the reference even though each is individually "correct at level 0." There is no aggregate test that would catch the compounded drift.

**Missing state fields.** The `unlocks` 8/21 gap is the most central (it gates whether mechanics are *active at all* and overlaps C2). The `runeBlessings`/`runeSpirits` type divergence (level-array vs Decimal-currency-map) is a latent lossy-precision + missing-ledger issue. Several gaps (buy-amount selectors, autobuyer toggles, loadout presets) block whole player-action/automation families and will need **schema-add permission** per project rules.

**Test / parity-coverage blind spots.** See the dedicated section — the short version: the leaf math is genuinely TS-anchored, but **the entire composition/aggregation/orchestration layer is anchored only to porter-authored expectations**, which is precisely the layer where C1, H1, and the upgrade-68 bug live (none of which any TS-anchored test catches).

## Memory / status drift

The project notes are stale in **both** directions — git history has moved past memory in places and the code is behind optimistic claims in others:

- **Memory is STALE-pessimistic on corruptions.** The note "corruption-effects open / unported" is wrong: corruption effects **are** ported and **are** applied to production (recession→building power, viscosity→accelerators, dilation→ascension speed, illiteracy→obtainium DR, deflation→ant production; score/difficulty mults wired). Two compares independently confirmed this and flagged the stale in-code comments at `tick/mod.rs:2096-2098` that *also* wrongly say "corruption-effects system is unported." **Update memory and delete those comments.**

- **Memory is STALE-optimistic on "buy surface / AutomationPre / reset tiers / ascension all COMPLETE."** Substantially true but with real holes the note hides: `boostAccelerator` is the one missing buy family; `resetOfferings` is unwired; the ascension c10 reward is computed at the wrong sequence point; and "RESET+AWARD through ascension complete" obscures that the **c10→ascension unlock** (C2) was never wired, making the ascension *challenge* ladder unreachable.

- **"Challenge c1-15 done" is materially overstated** (skeptic-done-claims): c1-14 + highestChallengeRewards are faithful, but **c15 exponent never accrues** (frozen at 0 → all c15 rewards identity, hepteracts unreachable via c15) and the reincarnation→ascension unlocks are absent (C2).

- **"singularity (paused)" is accurate** and the code matches the intent — but worth recording that "paused" here means *runtime-inert* (count never moves), so even the genuinely-ported singularity formulas (penalties, exalt, challenges, milestones) never fire. The memory should note that the GQ-upgrade/octeract math IS ported and live, while perks + `singularity()` reset are the actual holes.

- **Stale in-code docstrings** beyond corruptions: `tick/reset.rs:481-483` lists the c10 wow-cube award as "Deferred" when the body fully implements it (cosmetic, but misleading); `accelerators.rs:73-77` / `particle_buildings.rs:77-79` assert the BUYMAX+1.0 off-by-one is "correct"/"equivalent" when it is neither.

## Test & parity-harness coverage assessment

Two distinct test systems, with a hard line between them:

- **Golden-vector harness (TS-anchored, trustworthy but tiny):** `crates/synergismforkd_testkit/src/parity.rs` pins exactly **18 leaf functions** to actual TS output via `parity_vectors.json` (16 fns) + `parity_decimal.json` (2 fns) — the scalar math (summations/sigmoid/quadratic), 9 singularity-milestone fns, and 2 Decimal cost fns. These are generated by running the **frozen TS** over fixed grids and replayed with a calibrated tolerance (1e-9·|ts|+1e-12 for f64; log10-space 1e-9 for Decimal). Adversarially re-verified as genuinely anchored, edge-cases included. **This foundation is solid.**

- **Inline `#[test]` (broad but porter-authored):** 1174 hand-written tests give wide per-mechanic branch coverage and table-driven cases use TS-matching magic constants. **But the expected values were written by the porter**, so a transcription slip duplicated into both impl and assertion passes silently — exactly the failure mode the TS old-vs-new oracle is designed to catch.

**The critical blind spot:** the **91 TS `*.parity.test.ts` files** (the strong independent oracle — globalMultipliers, updateAllTick, resourceGain, tackBody/Middle/Tail, tickHarness, coinProduction, corruptions, challenges, autoReset, …) run against **TS, not Rust**. Only ~4 families are mirrored by the 18-fn Rust harness. Concretely:
- The full `tack` test (`tick/mod.rs:5424`) asserts only `events.len()==1` and the event variant — **zero resource numbers.**
- `compute_global_multipliers`, `update_all_multiplier`, and especially `compute_obtainium` (~40 sub-terms) have only 4–10 single-branch assertions; **no test sets many flags and checks the full product against a TS number.**
- The headless `run_sim` tallies event counts, never compares final-state resource quantities — identical event counts mask divergent numbers.
- The save crate has 4 tests (greenfield format, TS-parity N/A — acceptable now).

This is why **C1, H1, and the upgrade-68 bug are invisible to the test suite**: they live in the composition/orchestration layer, which has no TS-anchored coverage, and they are identity at default so the smoke tests pass. **The highest-value test investment is a state-snapshot parity harness**: serialize a `GameState` in, run `tack` (or an aggregator), snapshot the post-state out, and diff against the TS oracle over a multi-tick non-default grid. The `.mjs` generator is currently restricted to "deterministic, RNG-free, dependency-free leaf fns," so this needs a different fixture-capture approach.

## Prioritized next steps (logic-only)

1. **Fix C1: scale generation dt by global-speed mult** (`tick/mod.rs:565` → `input.dt * compute_global_speed_mult_pre(state)`). *Rationale:* single highest-impact correctness bug, affects the entire core economy in all normal play, ~1 line. *Effort: trivial code + 1 regression test (raise research-121, assert coin growth scales).* **Do first.**

2. **Fix H1: prestige_shards slice desync** (seed `resource_gain.rs:380` from `crystal_upgrades.prestige_shards`). *Rationale:* crystals/diamonds never accumulate and the crystal coin multiplier is permanently undervalued in early/mid play; 1 line. *Effort: trivial + a 2-tick growth test (the existing single-tick test masks it).*

3. **Wire C2: c10 → `ascension_unlocked`** (add the missing branch at `tick/mod.rs:4641-4647`). *Rationale:* unlocks the entire ascension-challenge endgame, which is otherwise dead; the field already exists (no schema change for this part). *Effort: small.* Then request schema-add for `anthill`/`talismans_unlocked`/`blessings_unlocked` (c8/c9 unlocks).

4. **Route all ant-effect sites through `calculate_true_ant_level` (H2).** *Rationale:* the most pervasive multiplier defect (13 sites, exponent-level impact on coins), and the divisor is already ported — only neutral free-level inputs needed. *Effort: medium (~13 edits + fixture).*

5. **Build a state-snapshot parity harness** (serialize GameState → `tack`/aggregator → snapshot → diff vs TS over a non-default multi-tick grid). *Rationale:* without this, fixes #1/#2/#4 and every future composition change are unguarded; it is the test infrastructure the whole audit shows is missing. *Effort: medium-high (new fixture-capture path distinct from the leaf `.mjs` generator).* Prioritize before the larger subsystem ports so regressions are caught.

6. **Implement the rune effective-level pipeline (H3 + H4 + the dead `rune_level_bonuses`/`rune_free_levels`).** *Rationale:* understates a broad swath of production multipliers from early reincarnation; the supply-side aggregators are already written and just need a `effectiveLevelMult` composer + wiring. *Effort: medium-high (~15 sites).*

7. **Wire `resetOfferings()` on every reset tier** (`tick/reset.rs`, using the ported `calculate_offerings`). *Rationale:* pre-challenge-3 players currently have zero offering income; pure wiring gap. *Effort: small.* (Bundle the easy arithmetic fixes here: upgrade-68 floor order, the hepteract RAW-`.bal` DR wrap, the false "correct" off-by-one comments.)

8. **Hoist the ascension c10 cube-reward capture before the sub-resets** (`tick/reset.rs`). *Rationale:* under-credits wowCubes + dependent awards + ascensionCount for any player owning the destroy-listed researches/ant-upgrades/finiteDescent. *Effort: small (reorder).*

9. **Port `Cube.open()` for all 4 blessing layers + reserve `RngPurpose::Cubes/...` (H6).** *Rationale:* the blessing economy is non-functional headless; primary mid/late activity. *Effort: medium per layer.*

10. **Port the ant-sacrifice executor + `antSacrificeRewardStats` (H7).** *Rationale:* sacrifice currently pays nothing and never resets the board. *Effort: medium.*

11. **Port achievement awarding + `updateAchievementPoints` + progressive cache (H5).** *Rationale:* unfreezes the crystal/mythos point-exponent multipliers (mid-game) and dozens of inert bonuses; genuine design fork (was delegated to a nonexistent UI tier). *Effort: large — sequence after the cheap high-impact fixes; a partial port of the reset/building/challenge group-checks captures most of the value.*

Items 12+ (lower priority, mostly late- or singularity-gated): shop bonus-level composition + noQuarkUpgrades zeroing; constant-upgrade/rune-blessing/red-ambrosia buy flows; updateAll autobuyer layer (needs schema permission); GQ per-slot metadata seeding; campaign-token system; offline-catchup driver; the InfiniteAscent rune roster correction.
