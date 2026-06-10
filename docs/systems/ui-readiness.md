# UI readiness — logic-tier migration audit

**Date:** 2026-06-10 · **Baseline:** `main` @ `0a9230eb` (incl. PRs #265, #269, #272, #273)

> **Status update (2026-06-10, UI vertical slice):** the work this audit recommended has started
> landing. The Dioxus 0.7 app shell (grouped nav + HUD + theme tokens), the number formatter, a
> playable Buildings section, saves UI, the 20 Hz loop driver, and **the offline catch-up gap
> below is CLOSED** (`crates/synergismforkd_ui_web/src/catch_up.rs` — chunked `time_warp` ticks
> off `saved_at_ms`, 24 h cap, progress dialog). `SaveHost` is now persistence-only (state lives
> in the UI signal) and `TickOutput` carries `DerivedTickStats` for HUD rates/previews. Desktop
> shell decision ratified: **Dioxus only, no Tauri** (CLAUDE.md amendment due at the desktop
> milestone).

A source-verified assessment of whether the Rust logic tier is complete enough to start building
the Dioxus UI tree. Every claim below was checked against `crates/` source, **not** against status
docs (two of three audit passes for this report were initially misled by stale docs — see
[Stale-docs cleanup](#stale-docs-cleanup)).

## Verdict

**The logic tier is ready for UI work.** One real gap remains (offline catch-up orchestration,
host-tier), plus two deliberate external deferrals (promo codes, backend-fed quark terms).
Everything else a UI needs — a tick entry point, a complete action surface, an event stream, and a
tested save host — exists.

## What's complete

### Mechanics layer — effectively 100 % coverage

All 71 TS mechanics files in `legacy/core_split/packages/logic/src/mechanics/` (plus `math/`,
`events/`) have Rust counterparts in `crates/synergismforkd_logic/src/` — ~819 TS exports map to
~855 Rust items (the surplus is extra helpers/constants). The TS `cubes/` subdir is split across
`cube_blessings.rs`, `cube_opening.rs`, `hepteract_effects.rs`, `hypercube_blessings.rs`,
`platonic_blessings.rs`, `tesseract_blessings.rs`. No `todo!()` / `unimplemented!()` / trivially
constant stub bodies were found.

### Tick orchestration — head + middle + tail all wired

The TS `tack` pipeline (`tack.ts` / `tackMiddle.ts` / `tackTail.ts`) is fully represented in
`crates/synergismforkd_logic/src/tick/mod.rs` (`phase_automation`, lines ~6748–7176):

| TS phase | TS cases | Rust call site |
|---|---|---|
| Head | 11 timer-advancement cases (reset counters, ascension/singularity, quark/GQ, octeract, auto-potion, ambrosia, red-ambrosia) | `timers::advance_*` @ ~6781–6900 |
| Middle | rune sacrifice | `automatic_tools::advance_rune_sacrifice` @ ~6967 |
| Middle | ant sacrifice (timers → ready-check → perform) | `automatic_tools::advance_ant_sacrifice_timers` / `check_ant_sacrifice_ready` / `ant_sacrifice::perform_ant_sacrifice` @ ~6981–7018 |
| Middle | addObtainium | `automatic_tools::add_obtainium` @ ~7031 |
| Middle | auto-research (manual + Roomba) | `auto_research::process_auto_research_tick` @ ~7047 |
| Tail | addOfferings | `automatic_tools::add_offerings` @ ~7068 |
| Tail | challenge sweep | `challenge_sweep::tick_challenge_sweep` @ ~7080 |
| Tail | auto-resets | `auto_reset::apply_auto_resets` + `reset::perform_reset` @ ~7100 |

The middle phase is covered by tests (e.g. `phase_automation_middle_credits_obtainium`). All 13
`updateAll` autobuyer families self-drive via `tick/auto_buy.rs`.

### Action surface — every player-facing interaction has a path

Entry point (pure; the host owns the clock and the state):

```rust
pub fn tack(state: &mut GameState, input: &TackInput) -> TickOutput
// TackInput { dt, time_warp, player_actions: SmallVec<[PlayerAction; 4]> }
// TickOutput { events: SmallVec<[CoreEvent; 16]> }  // ~103 variants
```

`PlayerAction` covers: `Buy(BuyRequest)` (24 routes — producers/max, upgrades, researches,
runes, talismans, ant producers/upgrades/masteries, crystal/constant/cube/platonic upgrades,
hepteract craft + expand, shop, GQ/octeract/ambrosia upgrades, accelerator/multiplier/boost,
particle + tesseract buildings), `Reset(ResetRequest)` (prestige → transcension → reincarnation →
ascension(+challenge) → singularity(+challenge)), `OpenCubes` (all 4 tiers, with max),
`ToggleAuto` (10 flags), `SetCorruptionLevel`, `EnterChallenge`, `ToggleSingularityChallenge`
(exalt enter/exit), `ConfigureSingularityElevator` + `TeleportToSingularity`, `SelectCampaign`.

`claim_export_rewards` (`tick/mod.rs`) is the export-quark/GQ claim path, tested.
`daily_reset` (`tick/mod.rs:497`) includes the overflux orbs→powder conversion.

### Save tier — complete and tested

`crates/synergismforkd_ui_web/src/save_host.rs` (`SaveHost`) provides boot/load (with
achievement-point recompute), 5 s autosave, force-persist (for `visibilitychange` /
`beforeunload`), export-with-reward-claim, clipboard import with corrupt-blob rejection (a corrupt
save is **not** clobbered), and hard reset. Storage is trait-abstracted: `localStorage` on wasm,
in-memory fake on native — the orchestration is unit-tested on native. Saves carry a
`saved_at_ms` host-clock stamp on the envelope (`crates/synergismforkd_save/src/lib.rs`).

All ~30 `GameState` slices serialize; all logic-affecting toggles (auto-reset modes/amounts,
challenge sweep slots, rune/ant sacrifice, potions, research mode) live in `AutomationState` /
`AntsState` and round-trip.

## The one real gap: offline catch-up

The *pieces* all exist, but nothing connects them:

- the save envelope stamps `saved_at_ms`, and `SaveHost::boot` returns it via
  `BootOutcome::Loaded { saved_at_ms }` — then drops it;
- `TackInput::time_warp` is the designed catch-up mode (skips head + middle automation, still runs
  generation + tail) and the testkit sim already exercises it (`SimConfig.time_warp`);
- the TS reference is `calculateOffline()` (`legacy/original/src/Calculate.ts`) — a capped
  ~200-step loop over the elapsed wall-clock.

**Missing:** the host-tier loop that, on boot, takes `now_ms − saved_at_ms`, chunks it, and runs
`time_warp` ticks before handing the state to the UI. Without it, an idle game silently loses all
closed-tab progress. This is `ui_web` / `SaveHost` work, **not** a logic-crate gap, and can be
built in parallel with the UI tree — but it must land before anything ships.

## Deliberate deferrals (not blockers)

| Item | Where it lives in TS | Why deferred |
|---|---|---|
| Promo / add codes (`promocodes()`, daily code) | `web_ui/src/ImportExport.ts` | External-API tier; parked with the backend plan (`BACKEND_API_PLAN.md`). The `addCodesUsed` quark-mult term stays neutral. |
| Backend-fed quark terms (shopPanthema, infiniteAscent, event/patreon bonuses) | server-driven | Neutral until the backend exists. |
| Config/debug flags; Statistics StatLine *assemblers* | `web_ui` | UI-tier by design — the UI calls mechanics functions directly for display values. |

## Recommended order into UI work

1. **Start the Dioxus UI tree** in `synergismforkd_ui` (currently a 10-line placeholder) against
   the `tack` / `PlayerAction` / `CoreEvent` / `SaveHost` surface — nothing in logic blocks it.
2. **In parallel: offline catch-up** in `SaveHost` (consume `saved_at_ms` + `time_warp` chunks).
3. **Later: promo codes / backend terms** once `lmam_api` exists.

## Stale-docs cleanup

Two artifacts were actively producing wrong audit conclusions and were corrected alongside this
report:

- `tick/mod.rs` module doc said phase 5 (automation) was "stubbed" — it has been fully wired for
  some time; the comment is now fixed.
- The repo-root `PARITY_AUDIT.md` (2026-06-06 snapshot claiming 55–65 % parity with open
  C1/C2/H1–H7 findings — all since fixed) has been deleted. The per-domain pages in this folder
  are the living status record.
