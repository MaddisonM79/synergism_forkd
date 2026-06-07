<!--
Derived from PARITY_AUDIT.md (48-agent parity audit, 2026-06-06). Logic-only; no UI work.
This is the ordered remediation + remaining-migration roadmap. Issue IDs (C1, H2, M-*, L-*) cross-reference the audit.
Untracked scratch at the worktree root — keep or delete freely.
-->

# Synergism Forkd — Logic Migration & Parity Remediation Plan

**Scope:** logic conversion only (`crates/synergismforkd_logic`). No UI/HUD. Parity target = the TS monolith (`legacy/original/src`), with the extracted package (`legacy/core_split/packages/logic`) as the function-level reference. Source of findings: [PARITY_AUDIT.md](PARITY_AUDIT.md).

## Guiding principles (apply to every item)

1. **Parity-first.** A faithful number beats a fast one. Where a producing subsystem isn't ported yet, keep the **neutral-default** (identity input) — but record it so the compounded drift is visible.
2. **Build the oracle before scaling fixes.** The audit's central lesson: the composition/orchestration layer has *no* TS-anchored test coverage, which is exactly where the critical bugs hid. The **state-snapshot parity harness (P0.4)** is the backbone for everything after P0.
3. **Schema-add gate.** Per `CLAUDE.md`, adding a field to `logic/src/state/` requires explicit user permission (save-size rule). Standing-OK for `player.*` fields; `G.*` per-field. Every item that needs new state is flagged **[SCHEMA]** and consolidated in §"State-schema changes".
4. **Verify against source, not memory.** Project memory has drifted in both directions (see audit §Memory drift). Read the monolith + git log before porting.
5. **Gates after every change:** `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all --check` · `cargo build -p synergismforkd_ui_web --target wasm32-unknown-unknown`. Conventional-commit titles.

---

## Preferred order (master sequence)

| # | Phase | What | Why here | Net effort |
|---|---|---|---|---|
| **P0** | Stop the bleeding + safety net | ✅ 3 one-line fixes (C1/H1/C2, [PR #257](https://github.com/MaddisonM79/synergism_forkd/pull/257)) · remaining: snapshot parity harness + c8/c9 unlocks | Highest impact-per-line; the harness guards all later work | M-H |
| **P1** | Wire the already-ported-but-dead logic | ✅ 6 fixes done (H2, upgrade-68, hepteract DR, c10-hoist, count-mult, comments — PR #257) · remaining: offerings (medium), GQ metadata seed | Cheap, high-value, no new subsystems | M |
| **P2** | Effective-level pipelines | Runes / blessings / spirits / talisman rarity / shop bonus-levels / ambrosia effective | Shared machinery; fixes a broad swath of multiplier under-crediting | M-H |
| **P3** | Acquisition / awarding tier | Achievements awarding, cube `open()`, ant-sacrifice executor | The big extraction-boundary gaps; restores whole economies | L |
| **P4** | Automation + offline | updateAll autobuyer layer **[SCHEMA]**, offline catch-up driver | Idle-loop correctness; depends on P0 harness to be safe | M-H |
| **P5** | Remaining buys + systems | boostAccelerator (+reset overhaul), const-upgrade/rune-blessing/red-ambrosia buys, campaign, rune-roster fix, residual arithmetic | Completes the surface; several are late-gated | M |
| **P6** | Singularity activation | `singularity()` reset + count increment (decision gate: currently paused) | Unblocks an entire ported-but-inert layer; explicit go/no-go | M-L |

Rationale for the spine: **P0 fixes are verified-from-source one-liners** that correct the entire core economy and unlock the ascension endgame — land them immediately, each with a targeted regression test. The **harness** then locks them in and becomes the diff-vs-TS oracle so P1+ can move fast without silent drift. P1 is pure wiring (formulas already ported). P2 shares one composition mechanism. P3 is the heavy net-new porting. P4–P6 complete automation, the remaining buy/feature surface, and the paused singularity layer.

---

## P0 — Correctness quick-wins + the safety net

> ✅ **P0.1 (C1), P0.2 (H1), P0.3 (C2) — DONE in [PR #257](https://github.com/MaddisonM79/synergism_forkd/pull/257)**, each verified from source with a regression test; all four gates green (1177 logic tests). Remaining in P0: the c8/c9 unlock fields (P0.3b) and the parity harness (P0.4).

### P0.3b — [C2/L-schema] Add the reincarnation feature unlocks  **[SCHEMA — granted]**  *(DEFERRED — no consumers yet)*
- **Change:** add `anthill` (c8), `talismans_unlocked` + `blessings_unlocked` (c9) and wire them in the same match arm (`Synergism.ts:3691-3700`).
- **Finding (2026-06-06):** schema permission is granted, BUT **nothing in the logic crate reads these flags yet** — the anthill / talisman-activation / blessing-activation consumers aren't ported. Adding+setting the fields now is pure dead scaffolding (zero behavior change). **Do this AS PART OF the feature that consumes it**: anthill pairs with ant-sacrifice (P3.3/H7), blessings with the blessing power chain (P2.2/H4), talismans with P2.3. Deferred until then to avoid dead state.

### P0.4 — Build the parity harness for the composition layer  *(infrastructure — first installment DONE 2026-06-07)*
- **Why:** 18 leaf fns are TS-anchored; the 91 TS `*.parity.test.ts` oracles run against TS, not Rust; the full `tack` test asserts only an event count. C1/H1/upgrade-68 were all invisible to the suite. This is the single highest-leverage infra investment.
- ✅ **First installment:** `parity/generate_aggregator_vectors.mjs` → `fixtures/parity_aggregators.json` → `calculate_aggregators_match_typescript` in `parity.rs`. Anchors the **pure-input `calculate.ts` COMBINE aggregators** — `calculateGlobalSpeedMult` (the C1 mult, both DR branches), `calculateAscensionSpeedMult`, `calculateActualAntSpeedMult` (all 5 ascension-challenge exponent branches + C15 platonic mitigation), `calculateAmbrosiaLuck`, `calculateAmbrosiaGenerationSpeed`, `getReductionValue`, `calculateOfferings` (Exalt-8 cap), `calculateObtainium` (c14 zero-out + illiteracy-DR) — **8 functions, 36 non-default vectors**. These take flat input structs on both sides, so the same input drives both with no GameState mapping. The extracted package runs headless via `npm install --ignore-scripts` + `npx tsx` (verified).
- **Key structural finding:** the extracted `tackBody` takes **pre-evaluated input bundles**, not a `GameState`, and `computeGlobalMultipliers`/`updateAllMultiplier` take ~60-field flat inputs — so the Rust *state-coupled* assemblers (`compute_global_multipliers`, `compute_offerings`/`compute_obtainium` StatLine assembly, full `tack`) can't be anchored by feeding the same input. They need either a **GameState→TS-input fixture mapping** or a **headless monolith oracle** (the monolith owns the real `tack(dt)` over `player`/`G` but pulls DOM/i18n).
- **Remaining P0.4:** (a) `computeGlobalMultipliers`/`updateAllMultiplier` via a seeded-GameState→input fixture; (b) the offering/obtainium **assemblers** (the StatLine state-reads — only the monolith is an independent oracle); (c) the full-`tack` resource snapshot (the C1/H1-class wiring oracle). RNG carve-out applies (don't bit-anchor ambrosia/cube-open rolls). **Effort: medium-high remaining.**

---

## P1 — Wire the already-ported-but-dead logic

> The formulas exist and are tested; they're just not called, called with the wrong argument, or called in the wrong order. Cheapest parity wins after P0. Guard each with a P0.4 fixture.

> ✅ **Done in [PR #257](https://github.com/MaddisonM79/synergism_forkd/pull/257):** P1.1 (H2, `d6e17564`) · P1.5 (upgrade-68 floor order, `1a148392`) · P1.4 (hepteract DR, `478730a5`) · P1.3 (c10 award pre-destroy-lists, `7d8f2673`) · P1.6 (count-mult ECC scaling, `c7b16739`) · P1.7-comments (false award/achievement comments, `684e4353`). H2's free-level *inputs* still arrive with achievements (P3.1) / c15 exponent (P5.4).

### P1.2 — Award offerings on every reset tier  ✅ DONE (2026-06-06)
- **Shipped:** ported the `calculateOfferings()` input-builder into `tick/mod.rs` — `compute_base_offerings` (Σ allBaseOfferingStats), `compute_offering_mult` (Π allOfferingStats, 51 lines), `compute_offering_time_multiplier`; refactored the obtainium time-mult into a shared `offering_obtainium_time_multiplier(state, time, time_mult_check)`; wired the award at the top of `apply_base_reset` (the unconditional Reset.ts:422 `resetOfferings()`, so every tier credits; ascension nets to zero via the existing reset.rs:617 wipe). Also ported the missing `challenge_15_rewards::offering`. 10 new tests; all gates green.
- **AUDIT CORRECTION (verified from source):** the award target is **`automation.offerings`** (= TS `player.offerings`), **NOT `runes.rune_shards`**. `runes.rune_shards` is **dead state** (read/written nowhere; the legacy `runeshards` was migrated→`offerings` in PlayerUpdateVarSchema.ts). The "automation.offerings vs runes.rune_shards dual-slice desync (sibling of H1)" question is resolved: **not a desync** — nothing reads the second slice. `rune_shards` is removable dead scaffolding (schema perm + migration; deferred).
- **Why it mattered:** the passive trickle is gated on `highest_challenge_completions[3]>0`, so a pre-challenge-3 player previously had **zero** offering income; now every reset awards `calculateOfferings()`.
- **Follow-ups left as neutral-defaults** (faithful at all reachable states): `AchievementBonus` line needs an `offering_bonus` reader (prestigeCount≥1000 achievement) — wire with P3.1/H5; `ParticleUpgrade3x5` needs `maxObtainium` tracking; `ThriftSpirit` needs the spirit-power chain (P2.2b); `Jack`/shopPanthema needs ShopPanthemaBonusLevels (P2.4).

### P1.7-remainder — GQ per-slot metadata seeding  *(low — data)*
- **Comments:** ✅ DONE (`684e4353`) — false award/achievement comments corrected; the "corruption-effects unported" one was already gone; the BUYMAX "correct" comments are deferred to **P5.7** (corrected when the off-by-one is fixed, since flipping the comment without the code change would assert an unverified bug).
- **GQ metadata [data]:** `state/golden_quarks.rs` has `cost_per_level=0`/`max_level=0`, which `buy_gq_upgrade` treats as the unlimited sentinel → free unlimited GQ levels if a buy is driven off un-seeded state. Seed the static cost/cap table (UI-tier data, not schema). **Effort: small.**

---

## P2 — Effective-level pipelines (shared machinery)

> One conceptual mechanism — "evaluate effects at *effective* level, not raw purchased level" — underlies several subsystems. Build the composer once, route many sites.

### P2.1a — [H3] Rune effective-level mult  ✅ DONE (`546288fe`, TS-anchored)
- `first_five_effective_rune_level_mult` (13-factor Statistics StatLine: 8 research + ConstantUpgrade9 + Research4x9 live; Challenge15/MidasTribute/AchievementBonus neutral 1.0) + `first_five_effective_rune_level` (chal9→1 clamp, else level·mult) wired across the 8 first-five rune sites (speed/dup/prism/thrift/SI). Verified vs verbatim Statistics.ts (1/6/2/43/22.95). Identity at default.

### P2.1b — [H3 remainder] unlock gates + SI quark-mult  *(deferred — identity at default)*
- **freeLevels:** ✅ DONE (`a71aa1a8`) — `rune_free_levels` folds the shared firstFiveFreeLevels (FreeRunes ant true-level + `7·min(constantUpgrades[7],1000)`) + the speed/duplication coin/upgrade bonus aggregators into the effective level, wiring the previously-dead `rune_level_bonuses` aggregators. talisman_bonus + prism/thrift/SI per-rune bonuses neutral 0 (unported).
- **isUnlocked gates:** speed=true; SI=`researches[82]>0` (ported, safe); dup/prism/thrift=`getAchievementReward('*RuneUnlock')` (achievement-gated — **must wait for H5 / achievement awarding**, else defaulting to locked wrongly zeroes the runes). Deferred to land with H5.
- **SI extra mult:** `SIEffectiveRuneLevelMult` = essentially the quark-gain multiplier (plastic talisman, platonic, shopPanthema, campaign, infiniteAscent, quark hepteract — mostly unported). Port alongside the quark-multiplier subsystem.
- **Also:** antiquities/finiteDescent (effectiveLevelMult=1) still read raw level — only lose the chal9/unlock clamps (minor; wrap when convenient).

### P2.2a — [H4] Rune blessing power  ✅ DONE (`3a00be74`, TS-anchored)
- `other_blessing_multipliers` (researches 134/194/160 + midas talisman blessingBonus live; epicFragments + challenge15 neutral 1.0) + `rune_blessing_power` = `blessing.level · rune.level · mult`, wired across the speed/duplication/SI blessing sites. Verified vs verbatim RuneBlessings.ts (1/1.69/2/2/6.76). Identity at default. freeLevels deferred (P2.1b).

### P2.2b — Rune spirit power  *(deferred — singularity-gated/identity)*
- The spirit power chain mirrors blessings, but the prism/thrift/SI spirit fns are singularity-gated (identity at default, sing 0); only `duplication_rune_spirit_effects` is consumed (raw level). Port alongside singularity activation (P6) or when a non-identity spirit path becomes reachable.

### P2.3 — Talisman rarity + effective level  *(BLOCKED on UI-tier data — deferred until UI)*
- **Finding (2026-06-06):** `compute_talisman_rarity(isUnlocked, level, max_level)` IS ported, but its `max_level` input is the **raw maxLevel constant kept in the UI tier** (like shop cost tables — caller-provided, per the project's boundary). There is no maxLevel table in logic and no caller to recompute rarity in a logic-only build, which is exactly why the buy fn leaves it "to UI". Recomputing it logic-side would require baking the 7 maxLevel constants into logic, crossing the UI-tier boundary. **Deferred until the UI tier exists** (it will drive the rarity recompute on buy/load). The rarity-indexed effects (midas/chronos/polymath/…) stay at rarity 0 until then.
- **Audit correction:** the "ascension challenges 11/12/14 multi-complete per tick" finding is a **false positive** — Synergism.ts:3586-3594 caps `maxInc=1` only while *inside* ascension challenge 13 (which disables instantChallenge); the Rust already matches. Verified + recorded (`582f0f05`). The real challengeExtension inconsistency in the same area WAS fixed (`582f0f05`).

### P2.4 — Shop bonus-level composition  *(medium)*
- **Build:** `getShopLevel`/`getBonusLevels` (tick currently passes raw stored level). Add the bonus sources (topHat rune, ambrosia/redAmbrosia free-upgrade nodes, InfinityUpgrades) and the `noQuarkUpgrades`/`!isUnlocked` level-zeroing.
- **Why:** shop rewards too low when any bonus active; too high (challenge too easy) during the noQuarkUpgrades singularity challenge; `shopPanthema` can't be non-identity without this. **Effort: medium.**

### P2.5 — Ambrosia/blueberry effective levels  *(medium — low play-reach today)*
- **Build:** effective-level composition (free-level + Exalt-zero) for ambrosia/blueberry (`blueberry_upgrades.rs:412-686`). Red-ambrosia raw-level is already correct.
- **Why:** bites in the two Exalt challenges + once any red-ambrosia free-level upgrade is bought; compounds through quarks2/cubes2/luck2/quarks3/cubes3 milestone params. **Effort: medium.**

---

## P3 — Acquisition / awarding tier (the big extraction-boundary gaps)

> These lived next to the `Currency` class / DOM / `Math.random()` in the monolith and were excluded from the extracted package. Net-new porting; restores whole economies.

### P3.1 — [H5] Achievement awarding + points  *(large — slices 1-2-3a DONE 2026-06-06)*
- **Why:** `achievement_points` is the **exponent** of the crystal mult `(1+0.01·crystalUpgrade[0])^points` (`global_multipliers.rs:344`) and the mythos mult `1.01^points·(points/5+1)` (`:383`, both read `state.achievements.achievement_points` at `:197`) — under-credited in **mid-game**. Plus dozens of achievement-gated bonuses stay inert.
- ✅ **Slice 1 (this session):** new `mechanics/achievement_awards.rs` — `award_achievement` (incremental `achievement_points += pointValue`, mirroring `awardAchievement`), `building_achievement_check` (the five `*OwnedCoin` groups, wired into the coin-producer `dispatch_buy` arms), `reset_achievement_check` (the `{prestige,transcend,reincarnation}PointGain` groups, wired into the three top-level `perform_*_reset` **before** the reset body so the offering/obtainium awards see the updated points). Indices/thresholds/pointValues extracted programmatically from the legacy array (brace-matched, not hand-counted). 11 new tests incl. a keystone end-to-end (`achievement_points_drive_the_mythos_exponent`). The bitmap is now **written**, not just read.
- ✅ **Slice 2 (this session):** `challenge_achievement_check` — the 14 challenge groups (`challengeN` rows = `challengecompletions[N] >= threshold`, indices 78-147 + 197-224 + 305-381), wired into `complete_active_challenge` right after the completion count is written. Indices/thresholds/pointValues extracted programmatically. 4 new tests incl. an end-to-end completion (`challenge_completion_awards_challenge_achievements`).
- ✅ **Slice 3a (this session):** progressive achievements — `update_progressive_slot` (Math.max cache + point-delta fold, a port of `updateProgressiveCache`/`updateProgressiveAP`) + the per-tick `update_progressive_achievements` driver, run first in `phase_global_state` so the exponents see the fresh total. All 12 slots wired; the point formulas were already ported in `achievement_points.rs`. Reachable now: runeLevel, freeRuneLevel, ambrosia/redAmbrosia counts, antMasteries, rebornELO (0 until ant-sacrifice), singularityCount (0, paused), talismanRarities (0, P2.3). Neutral-defaulted to 0 (input not in logic): `exalts` (rewardAP, paused) and the 3 maxed-upgrade families (`maxLevel` is UI-tier). 3 new tests incl. e2e `progressive_achievements_accrue_through_tack`.
- **Slice 3b (later):** the ungrouped one-off tail (the no-accelerator/no-mult reset achievements indices 57-74 + the per-challenge `chalNNoGen`/`diamondSearch`/`extraChallenging`/`sadisticAch`) + the quark-on-award reward + the full-table recompute (`compute_achievement_points`) + the maxed-upgrade families once `maxLevel` reaches logic.
- **Sequencing:** group-checks + progressive captured the bulk of reachable points; the ungrouped tail follows.

### P3.2 — [H6] Cube `open()` across all four layers  *(medium-large)*
- **Build:** `WowCubes.open` (`CubeExperimental.ts:177`), `WowTesseracts.open` (256), `WowHypercubes.open` (297), `WowPlatonicCubes.open` (338) — spend balance → add blessing levels via the weighted bulk distribution + per-cube RNG pdf. Reserve new `RngPurpose::Cubes/Tesseracts/Hypercubes/PlatonicCubes`.
- **Why:** all blessing *effects* are ported but levels are frozen at 0 — the entire blessing economy evaluates at identity. **Parity note:** the rolled remainder is unseeded `Math.random()` in TS → parity-acceptable in expectation (bulk opens hit deterministic expected values); don't bit-anchor it. **Effort: medium per layer; medium with a shared abstraction.**

### P3.3 — [H7] Ant-sacrifice executor + reward StatLine  *(medium)*
- **Build:** consume `CoreEvent::AntSacrificeTriggered` (`automatic_tools.rs:213`, currently emitted + dropped). Port `sacrificeAnts()`: credit immortalELO/offerings/obtainium (unless asc-14), all 7 talisman tiers (if c9>0), award `sacMult`, **reset ants to crumbs**. Assemble `calculateAntSacrificeMultiplier` (the `antSacrificeRewardStats` StatLine — no Rust assembler yet).
- **Also:** unblocks the `cubeUpgrades[47]`-gated ant-sacrifice obtainium source (`tick/mod.rs:2945-2949`, currently 0). **Effort: medium.**

---

## P4 — Automation + offline

### P4.1 — updateAll autobuyer layer  *(medium-high)*  **[SCHEMA]**
- **Build:** the per-tick autobuy the monolith runs on its 50 ms `fastUpdates` loop (excluded from the extracted `tackBody`): producer/accelerator/multiplier/crystal/cube autobuyers (gated by `toggles[1..26]` + unlock upgrades), `autobuyAnts`, `autoUpgrades`, immortal-ELO auto-gain, constant-upgrade autobuy. Slot into `phase_automation`.
- **Why:** autobuyers-on players currently get no automatic producer/ant growth → the idle loop stalls.
- **Gate:** needs new automation-state toggle fields → **schema permission**. **Effort: medium-high.**

### P4.2 — Offline catch-up driver  *(medium)*
- **Build:** `calculateOffline` equivalent (the 200-step offline progression: timer + resource accrual). Today `time_warp` only gates head/middle, no offline driver.
- **Note:** partly reconstructable by the UI looping `tack`, but the monolith has genuine offline-specific logic (and `calculateOffline:773` confirms the C1 `dt*timeMult` contract). Decide whether this lives in logic or is a caller responsibility. **Effort: medium.**

---

## P5 — Remaining buys + systems

### P5.1 — boostAccelerator buy  *(medium — blocked)*
- **Build:** `BuyRequest::AcceleratorBoost` + the buy loop (cost formula already ported; `acceleratorBoostBought` frozen at 0). **Blocked on the reset-system refactor** — the pre-upgrade-46 path calls `reset('prestige')` inline. Sequence after the reset orchestration is clean. **Effort: medium.**

### P5.2 — Remaining buy families  *(medium)*
- **Build:** constant-upgrade buy flow, rune-blessing buy flow, red-ambrosia buy flow (beyond the 13 done). Single-level buy + respec tracking (`*Invested` **[SCHEMA]**) deferred across GQ/octeract/ambrosia/talisman/cube/platonic. **Effort: medium.**

### P5.3 — Challenge requirement neutral-defaults  *(medium)*
- **Fix:** the internally-inconsistent `challengeExtension` (zeroed in completion loop `:4543` but read by the sweep `:3588`); the c10 requirement reduction (researches 140/155/170/185, zeroed `:4482`); the hyperchallenge corruption inflation (neutralized to 1.0 `:4483`, making challenges too easy under a hyperchallenge loadout). Also fix the ascension 11/12/14 multi-complete-per-tick loop (`:4628`; monolith caps ascension comp at +1/tick). **Effort: medium.**

### P5.4 — c15 exponent accrual  *(medium)*
- **Build:** the c15 exponent currently never accrues (frozen 0 → all c15 rewards identity, hepteracts unreachable via c15). Wire the accrual + the `challenge15_exponent` state field that already exists. **Effort: medium.**

### P5.5 — Rune roster correction  *(low — late-gated)*  **[SCHEMA]**
- **Fix:** the 7-rune roster dropped InfiniteAscent and added finiteDescent (`state/runes.rs:19-34`); InfiniteAscent's `cubeMult = 1+n/100` is forced to 1.0 (`tick/mod.rs:2030`). Add InfiniteAscent + state slots for horseShoe/topHat. Singularity-shop-gated. **Effort: low + schema.**

### P5.6 — Campaign token system  *(medium — verify scope)*
- **Build:** the campaign-token *bonuses that feed logic* (14 dormant consumers). The campaign UI is out of scope, but confirm which campaign multipliers are logic-tier and wire them. **Effort: medium.**

### P5.7 — BUYMAX off-by-one  *(low — unreached)*
- **Fix:** `BUYMAX+1.0` recursion in producers (`:253`), multipliers (`:77`), accelerators (`:77`), particle buildings (`:80`) — should be plain `BUYMAX`. Particle buildings is a factor-of-2 overcost; the others leak into owned-count. Only bites above 1e15 owned. Correct now while touching those files (and it would fail the extracted `acceleratorBoosts.parity.test.ts` BUYMAX vector). **Effort: 4 × 1 line.**

### P5.8 — cube-upgrade 4/5/6 regrant timing  *(low-medium)*
- **Fix:** regrant `upgrades[94..100]` at buy-time, not deferred to ascension reset (`cube_upgrades.rs:264-314`) — `[100]` drips transcendPoints in normal play. **Effort: small.**

---

## P6 — Singularity activation (decision gate)

> Currently **paused = runtime-inert**: no production path raises `singularityCount` or any challenge `.completions`; the `singularity()` reset is unported. The GQ-upgrade/octeract math **is** ported and live; penalties/exalt/challenges/milestones are ported but never fire; 3 challenge reward fns (`no_quark_upgrades_effect`, `sadistic_prequel_effect`, `taxman_last_stand_effect`) + exalt-4's mult are dead code.

- **Decision:** keep paused, or activate. **No work until you say go.**
- **If activate:** port `singularity()` reset + count increment, wire the exalt/challenge entry+completion, then un-neutral the singularity-gated cube/score/GQ lines (audit notes these are `=1` at sing 0). Re-validate with the P0.4 harness across sing>0 fixtures. **Effort: medium once unpaused (much is already ported).**

---

## Remaining systems to migrate (inventory)

Whole subsystems not yet ported (or only partially), independent of the bug fixes above:

| System | State | Plan item | Notes |
|---|---|---|---|
| Achievement awarding + points | read-only bitmap; points frozen | P3.1 | exponent of crystal/mythos mults |
| Cube/Tesseract/Hypercube/Platonic `open()` | effects ported; levels frozen 0 | P3.2 | needs RngPurpose variants |
| Ant-sacrifice payout | event emitted + dropped | P3.3 | + reward StatLine assembler |
| updateAll autobuyer layer | absent | P4.1 | **[SCHEMA]** toggle fields |
| Offline catch-up (`calculateOffline`) | absent | P4.2 | logic-vs-caller decision |
| Rune effective-level + free-levels | raw level fed; supply dead | P2.1 | |
| Rune blessing/spirit power | raw level fed | P2.2 | |
| Talisman rarity + midas/mortuus/plastic | rarity frozen 0; consumers unwired | P2.3 | ~50% ported |
| Shop bonus-levels + noQuarkUpgrades zeroing | raw level; shopPanthema stubbed | P2.4 | |
| boostAccelerator buy | cost ported; loop absent | P5.1 | blocked on reset refactor |
| Constant-upgrade / rune-blessing / red-ambrosia buys | absent | P5.2 | |
| c15 exponent accrual | frozen 0 | P5.4 | gates hepteracts-via-c15 |
| Campaign token bonuses | 14 dormant consumers | P5.6 | logic-tier subset only |
| InfiniteAscent rune (+ horseShoe/topHat) | dropped from roster | P5.5 | **[SCHEMA]** |
| Singularity activation (`singularity()` + count) | inert | P6 | decision gate |
| PseudoCoins / premium layer | absent | — | premium-currency; likely out of scope, confirm |
| Live-event system (`eventBuffs`) | verify stub state | P5.6-adjacent | confirm whether logic-tier |
| Respec tracking (`*Invested`) | deferred | P5.2 | **[SCHEMA]** |

---

## State-schema changes (require permission — `CLAUDE.md` save-size rule)

Batch these for approval as each phase is reached. Standing-OK covers `player.*` fields; surface anyway so save size is a conscious choice.

- **P0.3b:** expand `unlocks` 8/21 → full set (esp. `anthill`, `talismans`, `blessings`, plus `coinone..coinfour`, `rrow1-4`). Gates whether mechanics are active.
- **P4.1:** automation autobuyer toggle fields (`toggles[1..26]` equivalents, `autobuyAnts`, `autoUpgrades`, constant-upgrade autobuy flags).
- **P5.2:** `*Invested` respec-tracking fields across buy families.
- **P5.5:** rune slots for InfiniteAscent / horseShoe / topHat.
- **Type change (flag for review):** `runeBlessings`/`runeSpirits` are `[f64;7]` *level* arrays but should be Decimal *currency-amount* maps (5 runes) — semantics + precision divergence (audit L-state).
- **Lower priority:** buy-amount selectors, blueberry/corruption loadout presets, daily cube/quark counters, singularity-elevator/`singularityMatter`/ultimate-pixels, `offlinetick`/fastest-clear, cube-auto-open toggles.

---

## Testing strategy (per phase)

- **Backbone:** the P0.4 state-snapshot parity harness. Every P1+ item ships with at least one non-default multi-tick fixture diffing Rust vs the TS oracle.
- **Anti-pattern to avoid:** porter-authored expected values duplicated into both impl and assertion (the same-author hazard behind the hidden bugs). Prefer TS-generated expectations.
- **RNG carve-out:** do **not** bit-anchor ambrosia/red-ambrosia luck or cube-open rolls (Xoshiro ≠ MersenneTwister by design); test their *expected* behavior / probability gates instead.
- **Regression floor:** keep the 1,174 inline tests green; add the missing aggregate tests for `compute_global_multipliers`, `update_all_multiplier`, `compute_obtainium`, cube multiplier (each currently 4–10 single-branch assertions, no full-product TS check).

---

## Effort summary

| Phase | Items | Rough effort |
|---|---|---|
| P0 | ✅ C1, H1, C2 done (PR #257) · remaining: c8/c9 schema + parity harness | medium-high remaining |
| P1 | ✅ 6 fixes done (PR #257) · offerings (medium) + GQ seed remain | medium remaining |
| P2 | runes/blessings/talisman/shop/ambrosia effective levels | medium-high |
| P3 | achievements, cube-open ×4, ant-sacrifice | large |
| P4 | autobuyers [schema], offline | medium-high |
| P5 | boostAccelerator, buys, challenge defaults, c15, rune roster, campaign, residual | medium |
| P6 | singularity activation (gated) | medium-low (decision first) |

**Done ([PR #257](https://github.com/MaddisonM79/synergism_forkd/pull/257), 13 commits):** C1 · H1 · C2 · H2 · upgrade-68 · hepteract DR · c10-award hoist · count-mult ECC · comment fixes · **P2.1a rune effective-level mult (H3)** · **P2.2a rune blessing power (H4)** · **P2.1b rune freeLevels** · **challengeExtension completion cap** — StatLine ports TS-anchored. The rune effective-level pipeline is complete. **Audit corrections this session:** P2.3 talisman rarity is BLOCKED on UI-tier maxLevel data (deferred until UI, not a clean logic fix); the "ascension multi-complete" finding is a verified false positive. **Next (all medium+):** P1.2 offerings input-builder · **P0.4** parity harness · **P3** acquisition tier (achievements/cube-open/ant-sacrifice). P2.3/P2.2b spirit/SI-quark-mult deferred until UI / singularity / H5.
