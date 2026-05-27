# Rust Best-Practices Audit — 2026-05-26

**Scope**: Full Synergism Forkd workspace (8 crates) plus configuration and supply chain.
**Method**: Three parallel read-only audits — Anvil (system design), Inquisitor (code idiom & correctness), Argus (supply chain) — plus a tooling pass (`cargo clippy --pedantic --nursery`, `cargo audit`, `cargo deny check`, `cargo doc`, `cargo tree`).
**Code under audit**: `synergismforkd_logic` (~33k LoC, 87 files), `synergismforkd_save` (~206 LoC), `synergismforkd_bignum` (~60 LoC). Stub crates (`common`, `ui`, `ui_web`, `ui_desktop`, `testkit`) audited at the configuration and forward-looking-design level only.
**No source code was modified.** This document is the only output.

---

## Executive Summary

**Verdict**: The foundation is **sound**. None of the locked architecture decisions need reversal — the (state, input) → (state, events) shape, no-Result-in-tick, sync-only tick, postcard saves, currency newtypes, per-purpose Xoshiro256++ RNG, and 8-crate workspace shape all hold up. The codebase compiles, tests pass, contains no `unsafe`, no soundness violations, no security advisories. This is well above the median for a 6-month-old hobby Rust port.

The real issues are **discipline**, not **design**. There are 39 findings: 3 High, 22 Medium, 13 Low, plus 11 forward-looking notes. Most are mechanical fixes. The configuration layer is where the biggest leverage is — tightening workspace lints, adding a `deny.toml`, and adding a `cargo-audit + cargo-deny` CI job changes the project's defaults from "permissive Rust" to "production Rust" in a single PR.

### Top 5 immediate actions (in priority order)

1. **[RBP-001] Lock the public-API boundary** — 1,169 `pub` items in `logic`, zero `pub(crate)`. Every undecorated `pub fn` is a de-facto external contract. Establish the policy now ("anything not re-exported from lib.rs is `pub(crate)`") before the next 100k LoC solidifies more implicit contracts.
2. **[RBP-002] Move `OsRng` out of `logic`** — `RngState::from_entropy()` is a syscall inside the no-syscall crate, and `RngState::default()` (via `GameState::default()`) panics if it fails. Either remove `from_entropy` from `logic` or make `Default` deterministic (`from_seed(0)`).
3. **[RBP-003] Adopt the tighter workspace lint policy** — Current `clippy::all = warn` leaves 779 unique pedantic warnings uncaught, including 775 `float_cmp` cases (several in production hot loops) and 366 `mul_add` candidates that *must* be suppressed because of TS parity. Drop-in `[workspace.lints]`, `clippy.toml`, and `rustfmt.toml` are in the appendix.
4. **[RBP-004] Wire `cargo-audit` + `cargo-deny` into CI** — Today nothing checks RustSec or license compatibility on PRs. Add a `supply-chain` job to `rust-ci.yml` and commit a `deny.toml`. Drop-in YAML and config in the appendix.
5. **[RBP-005] Acknowledge that `break-eternity-rs` is project-controlled** — `crates.io` shows `MaddisonM79` as sole owner. CLAUDE.md described it as third-party; this is now corrected. The two prior yanks (0.1.0, 0.2.0) were one-off fixes for inherited bugs from the previous developer and no future yanks are planned; `^0.3` is the right pin going forward.

Estimated effort for items 1–5: **one focused day** if grouped. Items 1, 3, and 4 each ship as a single PR. Item 2 has a quick path (document panic; mark `expect_used` deny + allow) and a full path (refactor entropy seeding into platform tiers); the user can pick.

---

## Findings

### HIGH

#### RBP-001 — `logic` crate has 1,169 `pub` items and zero `pub(crate)`
- **Dimension**: Design
- **Location**: workspace-wide, [crates/synergismforkd_logic/src/](crates/synergismforkd_logic/src/) (115 files)
- **Current state**: Across `logic`, every top-level item is `pub`. The crate root re-exports a curated "loop-edge" surface from [lib.rs](crates/synergismforkd_logic/src/lib.rs), but per-mechanic helpers, intermediate `*Result` field types, slice fields, and constants are all reachable via `synergismforkd_logic::mechanics::*::…`.
- **Why it matters**: CLAUDE.md explicitly prefers `pub(crate)`. Once `ui`, `save`, `api`, and `testkit` start importing from `logic`, every undecorated `pub` becomes an implicit external contract. Refactors that should be local (renaming an intermediate field, splitting a per-mechanic helper) silently become breaking changes.
- **Recommendation**: Two-tier fix. (a) Establish a policy in CLAUDE.md: anything not re-exported from [crates/synergismforkd_logic/src/lib.rs](crates/synergismforkd_logic/src/lib.rs) is `pub(crate)`. (b) Add `tests/api_surface.rs` that asserts the size of the public-symbol set so each new `pub` is a deliberate review decision. Defer the mass sweep until after the next mechanic PR merges; lock the policy *now* so new code doesn't compound the problem. Pair with the workspace lint `unreachable_pub = "warn"` (in the appendix lint block) which surfaces over-visibility automatically.
- **Effort**: M (policy + lint = S; sweep = M)

#### RBP-002 — `RngState::from_entropy()` is a syscall in `logic` and `Default` panics if it fails
- **Dimension**: Design + Code
- **Location**: [crates/synergismforkd_logic/src/state/rng.rs:65](crates/synergismforkd_logic/src/state/rng.rs:65); WASM workaround at [crates/synergismforkd_ui_web/Cargo.toml:24](crates/synergismforkd_ui_web/Cargo.toml:24)
- **Current state**:
  ```rust
  pub fn from_entropy() -> Self {
      let mut master = Xoshiro256PlusPlus::from_rng(OsRng).expect("OsRng should not fail");
      ...
  }
  impl Default for RngState {
      fn default() -> Self { Self::from_entropy() }
  }
  ```
  `GameState` derives `Default` and contains `RngState`, so `GameState::default()` (used in every test) does a syscall. `OsRng` on Unix calls `getrandom(2)`, on Windows calls `BCryptGenRandom`. CLAUDE.md says no syscall in `logic`. The WASM consequence is already half-noticed: `ui_web/Cargo.toml` carries a `getrandom = { features = ["js"] }` workaround.
- **Why it matters**: (a) API Guidelines **C-FAILURE** — fallible functions should return `Result`. (b) `Default::default` is conventionally non-panicking per **C-COMMON-TRAITS**. (c) Tests that call `GameState::default()` think they're getting a reproducible baseline — they're not. (d) The crate-level `#![cfg_attr(not(test), deny(clippy::unwrap_used))]` doesn't cover `.expect()`; this is the *only* `.expect()` in non-test `logic` code.
- **Recommendation**: Two options.
  - **Full fix (M)**: Remove `from_entropy` from `logic`. Each platform tier owns its seed source: `ui_web` reads from `crypto.getRandomValues` via wasm-bindgen, `ui_desktop` reads from `OsRng`, `testkit` uses a deterministic seed. `GameState::default()` requires an explicit seed, or use a builder `GameState::new_with_seed(0)`.
  - **Quick fix (S)**: Make `Default` deterministic (`from_seed(0)`); keep `from_entropy` as an explicit opt-in but add `# Panics` rustdoc and add `clippy::expect_used = "deny"` to the workspace lints with a localized `#[allow]` on that one line.
- **Effort**: S (documentation + lint) or M (full move)

#### RBP-003 — 775 `clippy::float_cmp` warnings, including production hot loops
- **Dimension**: Code
- **Location**: 775 occurrences across `logic`. Production examples:
  - [crates/synergismforkd_logic/src/mechanics/octeracts.rs:320](crates/synergismforkd_logic/src/mechanics/octeracts.rs:320) — `if level == 6.0`
  - [crates/synergismforkd_logic/src/mechanics/summations.rs:128](crates/synergismforkd_logic/src/math/summations.rs:128) — `if determinant == 0.0`
  - [crates/synergismforkd_logic/src/mechanics/summations.rs:88](crates/synergismforkd_logic/src/math/summations.rs:88) — `if buy_to_level == base_level`
- **Why it matters**: `clippy::float_cmp` (pedantic). Direct `==`/`!=` on `f64` is well-known to be unreliable (`0.1 + 0.2 != 0.3`). For game economy code, a single equality miss can drop a purchase loop one iteration short. The 775 total includes ~720 in tests (acceptable with epsilon helpers); the production cases are the concern — they mirror the TS sentinel-as-cap pattern but should carry localized `#[allow(clippy::float_cmp)]` with a justifying comment so they're auditable.
- **Recommendation**: (a) Add `clippy::float_cmp = "warn"` to workspace lints (the appendix block does this). (b) Audit the production cases: convert "is this float exactly the integer N?" checks to `(x - 6.0).abs() < f64::EPSILON` or compare as integers up front. (c) Document the intentional sentinel cases with a localized `#[allow(clippy::float_cmp)] // sentinel: mirrors TS …` comment. (d) For tests, introduce a `assert_float_eq!(a, b, epsilon = 1e-9)` helper to silence the lint and surface real precision drift.
- **Effort**: L (audit + cleanup across ~50 production sites)

### MEDIUM

#### RBP-006 — 366 `clippy::suboptimal_flops` candidates must remain suppressed during the parity phase
- **Dimension**: Code (parity-driven)
- **Location**: 366 occurrences across `mechanics/`. Examples in [crates/synergismforkd_logic/src/mechanics/producers.rs:165,175,185](crates/synergismforkd_logic/src/mechanics/producers.rs:165).
- **Current state**: Code uses `a * b + c` extensively. Clippy nursery suggests `a.mul_add(b, c)` (fused multiply-add, one rounding vs two).
- **Why it matters**: TS's V8 does not lower to FMA. Replacing `a * b + c` with `a.mul_add(b, c)` changes the floating-point output bit-for-bit, which breaks parity testing. Per [[feedback_port_structure_pragmatism]] and CLAUDE.md's "port verbatim first" rule, `mul_add` is the wrong rewrite during porting.
- **Recommendation**: Add `clippy::suboptimal_flops = "allow"` to workspace lints with a comment explaining the parity constraint. Revisit when the parity harness retires.
- **Effort**: S

#### RBP-007 — Three `while hi - lo > 0.5` loops have no progress guarantee
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/mechanics/accelerators.rs:158](crates/synergismforkd_logic/src/mechanics/accelerators.rs:158), [crates/synergismforkd_logic/src/mechanics/multipliers.rs:143](crates/synergismforkd_logic/src/mechanics/multipliers.rs:143), [crates/synergismforkd_logic/src/mechanics/particle_buildings.rs:136](crates/synergismforkd_logic/src/mechanics/particle_buildings.rs:136)
- **Current state**: Binary search where `hi` and `lo` are `f64`; condition is `while hi - lo > 0.5`. No iteration backstop.
- **Why it matters**: For very large operands (Decimal magnitudes in tetration territory), `hi - lo` can lose precision such that the difference never falls below 0.5 even after a step — the tick loop hangs.
- **Recommendation**: Add an iteration cap: `for _ in 0..MAX_ITERS { … if hi - lo <= 0.5 { break } }` with `MAX_ITERS = 64` (more than enough for log₂(f64::MAX)) and a `debug_assert!` on convergence.
- **Effort**: S

#### RBP-008 — `unreachable!()` on a const-data invariant
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/mechanics/ant_reborn_elo.rs:162](crates/synergismforkd_logic/src/mechanics/ant_reborn_elo.rs:162)
- **Current state**: `unreachable!("calculate_to_next_reborn_elo_threshold ran off the tranche list")` at the end of a loop over `REBORN_ELO_THRESHOLD_TRANCHES`. The `# Panics` rustdoc notes the terminal tranche has `stages = INFINITY`.
- **Why it matters**: The invariant lives in `const` data, not in the type. If the tranche list is ever edited to drop the infinity terminator, `unreachable!()` becomes reachable.
- **Recommendation**: Add a compile-time assertion that the terminator is intact:
  ```rust
  const _: () = {
      let last = REBORN_ELO_THRESHOLD_TRANCHES[REBORN_ELO_THRESHOLD_TRANCHES.len() - 1].stages;
      assert!(last.is_infinite() && last.is_sign_positive());
  };
  ```
  Either keep the `unreachable!()` (now provably unreachable) or replace the loop with an iterator chain that proves termination.
- **Effort**: S

#### RBP-009 — Workspace lint policy is the bare minimum
- **Dimension**: Config
- **Location**: [Cargo.toml:26-30](Cargo.toml:26)
- **Current state**: Only `rust::unused_crate_dependencies = warn` and `clippy::all = warn` (priority -1). Crate-level `deny(clippy::unwrap_used)` lives only in `logic/src/lib.rs:1`. No `unsafe_code = "forbid"` (despite CLAUDE.md mandating no unsafe). No `pedantic`, `nursery`, or restriction lints. No rustdoc lints. No `missing_docs`.
- **Why it matters**: `clippy::all` is just `correctness + suspicious + style + complexity + perf`. The 779 unique pedantic warnings include the float_cmp issues (RBP-003), missing `# Errors` docs (RBP-013), and the cast-truncation hazards (RBP-019). With no rustdoc lints, the 10 `cargo doc` warnings (RBP-014) accumulate silently because CI doesn't run `cargo doc`.
- **Recommendation**: Adopt the drop-in `[workspace.lints]` block in Appendix A and the `clippy.toml` in Appendix B. Both are tuned for the current state of the project — they enable pedantic/nursery as `warn`, suppress the parity-incompatible lints with comments, and formalize the no-panic convention via `disallowed-methods`.
- **Effort**: S (adopt config; burning down the resulting warnings is M and can happen incrementally)

#### RBP-010 — No `cargo-audit` or `cargo-deny` in CI
- **Dimension**: Supply chain + Config
- **Location**: [.github/workflows/rust-ci.yml](.github/workflows/rust-ci.yml) (no supply-chain job)
- **Current state**: CI runs `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`. Dependabot opens weekly PRs, but PRs are gated only by build/test/clippy. A vulnerable dep merged via green CI lands without ever being checked against RustSec.
- **Why it matters**: NIST SP 800-218 (SSDF) PW.4.1 requires vulnerability scanning as a release gate. OWASP A06:2021 ("Vulnerable and Outdated Components") is the most common modern attack vector. Today `cargo audit` is **clean** (0/1098 advisories) — the gap is the lack of *continuous* enforcement.
- **Recommendation**: Add the `supply-chain` job from Appendix D to `rust-ci.yml` (parallel to `build-test`). Commit the `deny.toml` from Appendix C. Pair with RBP-018 (add `publish = false` to all 8 workspace crates) so the `bans` subcheck stops false-positiving on path-dep wildcards.
- **Effort**: S

#### RBP-011 — GitHub Actions pinned by floating tag, not SHA
- **Dimension**: Supply chain
- **Location**: [.github/workflows/rust-ci.yml](.github/workflows/rust-ci.yml), [.github/workflows/rust-bench.yml](.github/workflows/rust-bench.yml), [.github/workflows/desktop-release.yml](.github/workflows/desktop-release.yml) — `actions/checkout@v6`, `actions/cache@v5`
- **Current state**: Three of six workflows use mutable tag references. The Claude workflows (`claude.yml`, `claude-auto-review.yml`, `pr-title-lint.yml`) already SHA-pin — proof the project understands the pattern, applied inconsistently.
- **Why it matters**: `tj-actions/changed-files` (CVE-2025-30066) exfiltrated CI secrets across thousands of repos in March 2025 by re-pointing a `@v45` tag. OpenSSF Scorecard "Pinned-Dependencies" check requires SHA pinning. MITRE ATT&CK T1195.002.
- **Recommendation**: Pin every `uses:` line to a full SHA with the tag as a trailing comment, the way `claude.yml:28` already does:
  ```yaml
  uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
  ```
  Dependabot will keep them updated.
- **Effort**: S

#### RBP-012 — `break-eternity-rs` trust model: project-controlled, stable on 0.3.0
- **Dimension**: Supply chain
- **Location**: [Cargo.toml](Cargo.toml) workspace deps; [CLAUDE.md](CLAUDE.md) bignum section
- **Current state**: `break-eternity-rs` is project-controlled (owner: `MaddisonM79`). CLAUDE.md originally implied third-party maintenance. The two prior yanks (0.1.0, 0.2.0) were one-off fixes for inherited bugs from the previous developer and no further yanks are planned; the 0.3.0 series is stable.
- **Why it matters**: Knowing the trust model is what it is (rather than guessing from yank history) means the pin policy can be the normal one — `^0.3` rather than `=0.3.0`. The original audit overweighted the yank risk; the user-confirmed history resolves it.
- **Status**: **Applied.** `Cargo.toml` workspace dep is `break-eternity-rs = { version = "0.3", default-features = false }` with the project-controlled comment inline. CLAUDE.md now explicitly notes the project-controlled trust model.
- **Effort**: S (done)

#### RBP-013 — Public `Result`-returning fns lack `# Errors` rustdoc sections
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/math/summations.rs:119](crates/synergismforkd_logic/src/math/summations.rs:119) (`solve_quadratic`), [crates/synergismforkd_logic/src/math/summations.rs:155](crates/synergismforkd_logic/src/math/summations.rs:155) (`calculate_cubic_sum_data`), [crates/synergismforkd_logic/src/mechanics/cube_upgrades.rs:196](crates/synergismforkd_logic/src/mechanics/cube_upgrades.rs:196) (`get_cube_cost`), [crates/synergismforkd_save/src/lib.rs:106](crates/synergismforkd_save/src/lib.rs:106) (`save`), [crates/synergismforkd_save/src/lib.rs:122](crates/synergismforkd_save/src/lib.rs:122) (`load`)
- **Why it matters**: API Guidelines **C-FAILURE** + `clippy::missing_errors_doc` (pedantic). A function that returns `Result<_, E>` should tell the caller what cases yield `Err` so they can decide on retry/recovery without reading the implementation.
- **Recommendation**: Add `# Errors` sections to each. For `save::save`: "Returns `Err(SaveError::Postcard)` if serialization fails (only on allocator exhaustion or a `Serialize` impl error)."
- **Effort**: S

#### RBP-014 — `cargo doc --no-deps` produces 10 documentation warnings
- **Dimension**: Code
- **Location**: 7 in `logic`, 3 in `save`. Examples: unresolved links `75`/`78`, references to private items `phase_cross_mechanic_precompute` and `SaveEnvelope`, name collision `crate::math::smallest_inc` (both module and function), unresolved link `SaveV1::migrate_to`.
- **Why it matters**: rustdoc lints `broken_intra_doc_links` and `private_intra_doc_links` are warnings today, but CI doesn't run `cargo doc`, so they accumulate. Public docs that link to private items confuse external readers and break docs.rs once published.
- **Recommendation**: (a) Add the rustdoc lint block from Appendix A. (b) Fix each warning — promote `SaveEnvelope` to `pub` or stop linking to it from public docs; drop the `SaveV1::migrate_to` link until that method exists; rename `crate::math::smallest_inc` (function or module) to disambiguate. (c) Add `cargo doc --workspace --no-deps -- -D warnings` to CI (or include it under the lint config).
- **Effort**: S

#### RBP-015 — `synergismforkd_common` is a 7-LoC placeholder paid for by four downstream crates
- **Dimension**: Design
- **Location**: [crates/synergismforkd_common/src/lib.rs](crates/synergismforkd_common/src/lib.rs); silencer pattern at [crates/synergismforkd_logic/src/lib.rs:25](crates/synergismforkd_logic/src/lib.rs:25), [crates/synergismforkd_ui/src/lib.rs:7](crates/synergismforkd_ui/src/lib.rs:7), [crates/synergismforkd_ui_web/src/lib.rs:7](crates/synergismforkd_ui_web/src/lib.rs:7), [crates/synergismforkd_save/src/lib.rs:34](crates/synergismforkd_save/src/lib.rs:34)
- **Current state**: `common` contains `pub fn placeholder() {}` and a doc comment promising "`PlayerId`, `Tick`, `MechanicId`, error enums, serde helpers" — none of which exist. Currency newtypes already live in `logic/currency.rs`. The four downstream crates carry `use synergismforkd_common as _;` lines to silence `unused_crate_dependencies = "warn"`.
- **Why it matters**: An empty crate depended on by four others pays compile-graph and IDE-indexing cost. The `use _;` workaround is the tell — the dep doesn't actually do anything. Per the locked plan, the project already deleted `audio`/`netcode`/`modding`/`api` for this reason.
- **Recommendation**: Delete `synergismforkd_common`. If a genuinely cross-crate type appears later (most likely candidate: a `Decimal`-aware serde helper used by both `save` and a future `api`), reinstate it then with concrete content.
- **Effort**: S

#### RBP-016 — `mechanics/` directory is flat at 86 files; group before 200
- **Dimension**: Design
- **Location**: [crates/synergismforkd_logic/src/mechanics/](crates/synergismforkd_logic/src/mechanics/)
- **Current state**: 86 files, no sub-directories. Natural clusters present in file names: 8× `ant_*`, 7× `rune_*`, 5× cube/tesseract/hypercube/platonic blessings, 4× `singularity_*`, 5× talisman/hepteract/octeract, 4× golden-quark/shop. The `mod.rs` table-of-contents is 75 lines.
- **Why it matters**: At 200 files (when Reincarnation lands), `mod.rs` becomes unreadable. Navigation cost: a developer working on rune effects has to know `rune_effects.rs`, `rune_levels.rs`, `rune_spirit_effects.rs`, etc. are related but `talisman_effects.rs` (which interacts with rune levels) is alphabetically elsewhere. Per [[feedback_port_structure_pragmatism]], structure is negotiable when grouping produces cleaner Rust.
- **Recommendation**: Group into `mechanics/{ants, runes, blessings, hepteracts, singularity, golden_quark, talismans, producers, ascensions}/` with each sub-module re-exporting its public surface from `mod.rs`. Leave the orchestrators (`global_multipliers`, `update_all_multiplier`, `update_all_tick`, `resource_gain`, `tax`, `reset_currency`) flat at the top level. Do this *before* the Reincarnation slice. Pairs with RBP-001 — sub-modules let `pub(crate)` work properly (sibling files share helpers without exposing workspace-wide).
- **Effort**: M (mechanical moves, noisy diff)

#### RBP-017 — `*Pre` aggregator migration is type-invisible
- **Dimension**: Design
- **Location**: [crates/synergismforkd_logic/src/tick/mod.rs](crates/synergismforkd_logic/src/tick/mod.rs); four `*Pre` structs in `mechanics/`
- **Current state**: `tack` builds a `CrossMechanicCache` that copies the four `*Pre` from `TackInput`, then overrides state-derivable fields via `compute_*_pre(state, &fallback)`. Comments in `tick/mod.rs` label which fields are still caller-provided vs state-derived. A forgotten "// Forwarded" field silently reads the identity-default and silently produces a wrong tick — exactly the failure mode [[feedback_port_structure_pragmatism]] flagged.
- **Why it matters**: This is a porting hazard. The test `cross_mechanic_cache_forwards_pre_bundles_from_input` pins the forwarding shape, but you can pass it while still silently identity-defaulting a field that should be state-derived.
- **Recommendation**: Replace each `*Pre` field type with `enum FieldOrigin<T> { Derived(T), Forwarded(T) }`. A code-review reviewer can grep `Forwarded(` to see remaining migration debt. End-state: every `*Pre` is fully `Derived`, then the `*Pre` struct is deleted and the aggregator reads from `&CrossMechanicCache`/`&GameState` directly. Don't refactor today; record the end-state so each port PR shaves one field.
- **Effort**: M

#### RBP-018 — Workspace crates lack `publish = false`
- **Dimension**: Supply chain / Config
- **Location**: All 8 `crates/*/Cargo.toml`
- **Current state**: No crate sets `publish`. Path deps like `synergismforkd_bignum = { path = "../synergismforkd_bignum" }` carry no `version`. `cargo deny check bans` reports 6 wildcard-dependency errors as a result.
- **Why it matters**: Functionally a false positive (crates.io rejects path-only deps anyway), but it blocks any CI integration of `cargo deny check bans` until fixed. OWASP ASVS V14.2.4.
- **Recommendation**: Add `publish = false` to every workspace `[package]` section. Once committed, `allow-wildcard-paths = true` in `deny.toml` cleanly suppresses the warnings.
- **Effort**: S

#### RBP-019 — Lossy `as` casts in production hot paths
- **Dimension**: Code
- **Location**: 12× `u32 → i32` in [crates/synergismforkd_logic/src/mechanics/blueberry_upgrades.rs](crates/synergismforkd_logic/src/mechanics/blueberry_upgrades.rs); 1× `f64 → u8` at [crates/synergismforkd_logic/src/mechanics/talisman_levels.rs:83](crates/synergismforkd_logic/src/mechanics/talisman_levels.rs:83); 3× `f64 → usize` in `mechanics/` (e.g., [octeracts.rs:323](crates/synergismforkd_logic/src/mechanics/octeracts.rs:323))
- **Why it matters**: `clippy::cast_possible_wrap`, `cast_possible_truncation`, `cast_sign_loss` (all pedantic). For `f64 → integer`, behavior is *defined* (Rust 1.45+ saturates) but not always desired — silent truncation of a negative or fractional level produces a wrong cube count.
- **Recommendation**: Replace `level as usize` with `usize::try_from(level as u64).expect("level fits")` for fail-loud behavior, or `debug_assert!(level >= 0.0 && level < usize::MAX as f64)` guard before the cast. For the `u32 → i32` powi cases, use `i32::try_from(level).expect(...)`.
- **Effort**: M

#### RBP-020 — Currency types derive `PartialEq` without `Eq`, intentional but undocumented
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/currency.rs:53](crates/synergismforkd_logic/src/currency.rs:53) (macro), expanded to 14 instances
- **Current state**: All 5 currency newtypes derive `PartialEq, PartialOrd` without `Eq, Ord`. Clippy `derive_partial_eq_without_eq` fires.
- **Why it matters**: API Guidelines **C-COMMON-TRAITS** wants `Eq` where reflexivity holds. Reflexivity *doesn't* hold here because `Decimal` carries NaN (see `from_finite` constructor naming). The lack of `Eq` is correct — but the macro should document *why*.
- **Recommendation**: Add `#[allow(clippy::derive_partial_eq_without_eq)]` to the macro with a comment: "Decimal carries NaN; reflexivity not guaranteed; intentional non-Eq."
- **Effort**: S

#### RBP-021 — Currency types lack `Display`
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/currency.rs](crates/synergismforkd_logic/src/currency.rs)
- **Current state**: Newtypes implement `Debug`, `PartialEq`, `Add`, `Sub`, `Mul<Decimal>`, `Div<Decimal>`, `Serialize`, `Deserialize`. No `Display`.
- **Why it matters**: API Guidelines **C-COMMON-TRAITS**. The UI will eventually render `Coins`; without `Display`, every render site reaches for `coins.raw().to_string()` — the type system stops helping at the boundary. CLAUDE.md correctly notes i18n is a UI concern; a debug-friendly `Display` is still appropriate (UI overrides for player rendering).
- **Recommendation**: Add `impl Display for $name` in the macro: `write!(f, "{}", self.0)`. Use cases stay typed; UI overrides via its own formatter.
- **Effort**: S

#### RBP-022 — `BuyRequest` rejection contract is undocumented
- **Dimension**: Design
- **Location**: [crates/synergismforkd_logic/src/tick/mod.rs](crates/synergismforkd_logic/src/tick/mod.rs)
- **Current state**: `tack()` returns events only; `dispatch_buy` matches `BuyRequest` and calls one of eight `buy_*` mutators. An unaffordable or malformed buy produces zero events and no state change. The contract is implicit — nothing on `BuyRequest` or `tack` documents it.
- **Why it matters**: A naive caller building `BuyRequest::Upgrade { pos: 999, … }` gets a silent no-op. The TS version threw or set an error modal; Rust goes the opposite direction (defensible) but should document the choice. The no-`Result` shape is right — every fail-mode in an incremental game is a *gameplay event*, not a programming error.
- **Recommendation**: (a) Add a doc-section to `BuyRequest` and `tack` that states "All malformed inputs produce zero events and no state change — no `Result` is returned." (b) Add `CoreEvent::BuyRequestRejected { request: BuyRequest, reason: BuyRejection }` with `BuyRejection = Unaffordable | OutOfBounds | PrerequisiteNotMet | NotYetVisible` so UIs can render the failure.
- **Effort**: S

#### RBP-023 — Public function names redundantly include their module path
- **Dimension**: Code
- **Location**: 229 of 769 public functions in `mechanics/` (29.8%). Example: [mechanics/blueberry_upgrades.rs](crates/synergismforkd_logic/src/mechanics/blueberry_upgrades.rs) `pub fn ambrosia_luck_3_cost_formula(...)`. Caller sees `mechanics::blueberry_upgrades::ambrosia_luck_3_cost_formula` — "ambrosia" appears twice.
- **Why it matters**: rustdoc style guide flags this as "stutter." The pattern recurs for `rune_*`/`ant_*`/`cube_*`/`singularity_*`/`talisman_*`/`octeract_*`/`hepteract_*`/`red_ambrosia_*`. Per [[feedback_port_structure_pragmatism]], this is in scope to fix even though it diverges from TS naming.
- **Recommendation**: Strip the module-name prefix at the function level. `mechanics::blueberry_upgrades::luck_3_cost_formula` reads cleanly. Do this incrementally during natural mechanic-touch PRs; not a sweep.
- **Effort**: M (~229 renames, but distributable across PRs)

#### RBP-024 — 6 structs with >3 `bool` fields (input bags)
- **Dimension**: Design
- **Location**: [mechanics/gq_upgrade_levels.rs:80](crates/synergismforkd_logic/src/mechanics/gq_upgrade_levels.rs:80) (`ActualGQUpgradeTotalLevelsInput`), [mechanics/platonic_upgrade_costs.rs:384](crates/synergismforkd_logic/src/mechanics/platonic_upgrade_costs.rs:384), [mechanics/tax.rs:29](crates/synergismforkd_logic/src/mechanics/tax.rs:29) (`CalculateTaxInput`), [state/ants.rs:65](crates/synergismforkd_logic/src/state/ants.rs:65) (`AntsToggles`), [state/reset_counters.rs:13](crates/synergismforkd_logic/src/state/reset_counters.rs:13), [state/upgrades.rs:18](crates/synergismforkd_logic/src/state/upgrades.rs:18)
- **Why it matters**: `clippy::struct_excessive_bools` (pedantic). Call sites read `Foo { a: true, b: false, c: true, d: true }` worse than `Foo { mode: Mode::AB, ...}`. Boolean blindness is the same hazard the currency newtypes already fix.
- **Recommendation**: For the three input bags (`tax`, `platonic_upgrade_costs`, `gq_upgrade_levels`), group related bools into bitflag-style enums. For the three state bags (`AntsToggles`, `reset_counters`, `upgrades`), they mirror the TS player object — defer until the slice is reworked for another reason.
- **Effort**: M

#### RBP-025 — `Coins::raw(self) -> Decimal` violates API Guidelines C-CONV
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/currency.rs:83](crates/synergismforkd_logic/src/currency.rs:83)
- **Current state**: `pub const fn raw(self) -> Decimal { self.0 }` on each currency type.
- **Why it matters**: API Guidelines **C-CONV** says owned→owned cheap conversions should be `into_*`. `raw` is the call-site hazard — it's the escape hatch where type safety drops, and grep-finding all such sites is harder when the name doesn't match convention.
- **Recommendation**: Rename `raw` → `into_decimal` on each currency type (and `Multiplier`). Mechanical rename, ~50 call sites. The name "into_decimal" makes the escape hatch greppable.
- **Effort**: S

#### RBP-026 — 25 match arms have identical bodies in `ant_masteries.rs`
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/mechanics/ant_masteries.rs:315-330](crates/synergismforkd_logic/src/mechanics/ant_masteries.rs:315) (200-line `match (producer, level) -> (f64, f64)`)
- **Why it matters**: `clippy::match_same_arms` (pedantic). Identical bodies in distinct arms usually mean either combine with `|`, or the table is deliberately enumerated for clarity.
- **Recommendation**: This match mirrors the TS truth table; **do not combine arms** — keep them as a 1:1 mirror so future TS-side bug fixes can be mirrored. Add `#[allow(clippy::match_same_arms)]` at the match site with a comment: "Each arm mirrors a distinct legacy case; consolidation would obscure parity."
- **Effort**: S

#### RBP-027 — `#[allow(dead_code)]` should be `#[expect(dead_code)]`
- **Dimension**: Code
- **Location**: [crates/synergismforkd_logic/src/tick/mod.rs:177-180](crates/synergismforkd_logic/src/tick/mod.rs:177)
- **Current state**:
  ```rust
  #[allow(dead_code)]
  update_all_multiplier: UpdateAllMultiplierResult,
  #[allow(dead_code)]
  update_all_tick: UpdateAllTickResult,
  ```
- **Why it matters**: `#[allow]` is permanent. `#[expect]` (stable since 1.81; project MSRV is 1.95) flips the lint on as soon as the field is actually read — the migration debt becomes visible.
- **Recommendation**: `#[expect(dead_code, reason = "captured for downstream phase migration")]`.
- **Effort**: S

### LOW

#### RBP-028 — `bignum` should expose a `serde` feature flag instead of forcing it
- **Dimension**: Config
- **Location**: [crates/synergismforkd_bignum/Cargo.toml:11](crates/synergismforkd_bignum/Cargo.toml:11)
- **Recommendation**:
  ```toml
  [features]
  default = ["serde"]
  serde = ["break-eternity-rs/serde"]
  ```
  Then `break-eternity-rs = { version = "=0.3.0", default-features = false }`. Every current consumer keeps serde via `default`. Reversible, costs ~5 lines.
- **Effort**: S

#### RBP-029 — Workspace deps incomplete
- **Dimension**: Config
- **Location**: [Cargo.toml](Cargo.toml) (root); per-crate manifests
- **Current state**: Root `[workspace.dependencies]` only declares `serde`, `serde-big-array`, `postcard`. `logic` declares `rand`, `rand_xoshiro`, `smallvec` directly; `bignum` declares `break-eternity-rs` directly.
- **Recommendation**: Move all four to `[workspace.dependencies]` with `{ workspace = true }` referencing from per-crate manifests. Pairs cleanly with RBP-012 (pin `=0.3.0`) and RBP-028 (feature flag).
- **Effort**: S

#### RBP-030 — No `[profile.dev]` tuning; bignum-heavy tests are slow
- **Dimension**: Config
- **Location**: [Cargo.toml](Cargo.toml)
- **Recommendation**: Add:
  ```toml
  [profile.dev.package."*"]
  opt-level = 1

  [profile.test]
  inherits = "dev"
  ```
  Compiles dependencies (including `break-eternity-rs`) at `opt-level = 1` while keeping workspace crates at 0 for fast incremental rebuild. Test suite often goes from 30s to 5s.
- **Effort**: S

#### RBP-031 — `cargo doc` first paragraphs are too long (77 instances)
- **Dimension**: Code
- **Location**: [currency.rs:204](crates/synergismforkd_logic/src/currency.rs:204), [events/mod.rs:36,152](crates/synergismforkd_logic/src/events/mod.rs:36), [math/sigmoid.rs:8](crates/synergismforkd_logic/src/math/sigmoid.rs:8), 73 others
- **Recommendation**: Insert a paragraph break after the first sentence. The first paragraph becomes the docs.rs summary; long summaries wrap poorly.
- **Effort**: S (mechanical)

#### RBP-032 — 40 doc comments reference identifiers without backticks
- **Dimension**: Code
- **Location**: [mechanics/ant_reborn_elo.rs:8](crates/synergismforkd_logic/src/mechanics/ant_reborn_elo.rs:8), [mechanics/ant_upgrades.rs:18,19,189,244,271](crates/synergismforkd_logic/src/mechanics/ant_upgrades.rs:18), and more
- **Recommendation**: Backtick. `clippy::doc_markdown` catches these once enabled (Appendix A).
- **Effort**: S (mechanical)

#### RBP-033 — 14 `pub fn` could carry `#[must_use]`
- **Dimension**: Code
- **Location**: [math/sigmoid.rs:12,20](crates/synergismforkd_logic/src/math/sigmoid.rs:12), [math/smallest_inc.rs:20](crates/synergismforkd_logic/src/math/smallest_inc.rs:20), [math/summations.rs:60,102](crates/synergismforkd_logic/src/math/summations.rs:60), others
- **Why it matters**: Pure-math fns returning `f64` qualify. Many `mechanics/` fns already carry `#[must_use]` — this is straggler cleanup.
- **Effort**: S

#### RBP-034 — 70 functions could be `const fn`
- **Dimension**: Code
- **Location**: Per `clippy::missing_const_for_fn` (nursery)
- **Why it matters**: Widens the contract — callers can use them in const contexts. Zero runtime cost.
- **Recommendation**: Audit the 70-entry list; add `const` where called functions are also `const`. Burn down before promoting `clippy::missing_const_for_fn = "warn"`.
- **Effort**: M

#### RBP-035 — `unused_crate_dependencies = "warn"` is mostly silenced
- **Dimension**: Config
- **Location**: Six crates carry `use foo as _;` silencers
- **Why it matters**: The lint is doing the opposite of its intent — every silencer signals a stub-state dep, not real misuse. Once stubs become non-stubs, silencers come out and the lint earns its keep.
- **Recommendation**: Don't change the lint. Add a one-line comment next to each `use foo as _;` explaining what real usage replaces it. When all silencers are gone, that's a milestone. Pairs with RBP-015 — deleting `common` removes ~half the silencers in one PR.
- **Effort**: S

#### RBP-036 — `getrandom 0.2.x` is on the maintenance track
- **Dimension**: Supply chain
- **Location**: [crates/synergismforkd_ui_web/Cargo.toml:25](crates/synergismforkd_ui_web/Cargo.toml:25); transitive via `rand 0.8 → rand_chacha → rand_core → getrandom 0.2`
- **Current state**: Lock resolves `getrandom 0.2.17` (Jan 2026). `getrandom 0.4.2` exists, but `rand 0.8` requires `getrandom 0.2`. `rand 0.9` (Jan 2025) reworked `Rng`/`SeedableRng` — mechanical port. `0.2.x` is still maintained in parallel.
- **Recommendation**: Defer. Upgrade `rand` + `getrandom` together when (a) a RustSec advisory hits 0.8, or (b) the RNG layer is refactored for another reason. Expect mechanical breakage in [crates/synergismforkd_logic/src/state/rng.rs](crates/synergismforkd_logic/src/state/rng.rs).
- **Effort**: M (when undertaken)

#### RBP-037 — `desktop-release.yml` over-grants `contents: write` to build matrix
- **Dimension**: Supply chain
- **Location**: [.github/workflows/desktop-release.yml:16-17](.github/workflows/desktop-release.yml:16)
- **Current state**: `permissions: contents: write` applied to all three OS matrix jobs. Tauri integration is deferred, so the permission is over-granted today.
- **Recommendation**: Downgrade to `contents: read` until release-publishing step lands. When Tauri ships, split into a single follow-up job (one runner, `contents: write`) that gathers per-OS artifacts via `actions/download-artifact` and publishes.
- **Effort**: S

#### RBP-038 — `claude-auto-review.yml` triggers on fork PRs
- **Dimension**: Config
- **Location**: [.github/workflows/claude-auto-review.yml:4-5](.github/workflows/claude-auto-review.yml:4)
- **Current state**: `on: pull_request: types: [opened, synchronize]` without an `if:` filter on author or fork. Secrets aren't exposed to forks (GitHub default), so the action fails on hostile PRs rather than leaking — but it does run, consuming runner minutes.
- **Recommendation**:
  ```yaml
  jobs:
    auto-review:
      if: github.event.pull_request.head.repo.full_name == github.repository
  ```
- **Effort**: S

#### RBP-039 — Unbalanced backticks and numeric literal style nits
- **Dimension**: Code
- **Location**: 4× unbalanced backticks in doc comments (`clippy::doc_markdown`); [crates/synergismforkd_logic/src/state/rng.rs:100-101](crates/synergismforkd_logic/src/state/rng.rs:100) `0xC0FFEE` without separators; [crates/synergismforkd_logic/src/mechanics/cube_upgrades.rs:18-50](crates/synergismforkd_logic/src/mechanics/cube_upgrades.rs:18) mixes `1e4` and `10_000.0` in the same table
- **Effort**: S (each)

#### RBP-040 — `SummationError` and `SaveError` use manual `Display`/`Error` impls
- **Dimension**: Code (style)
- **Location**: [crates/synergismforkd_logic/src/math/summations.rs:29-40](crates/synergismforkd_logic/src/math/summations.rs:29), [crates/synergismforkd_save/src/lib.rs:75-100](crates/synergismforkd_save/src/lib.rs:75)
- **Why it matters**: Not a violation — manual impls are correct. `thiserror` reduces boilerplate (~12 lines saved per error type) while preserving semantics. Optional refactor.
- **Effort**: S (if undertaken)

---

## Forward-Looking Notes (for the stub crates)

When the five stub crates fill in, these are the design choices to land *first*, before the code grows around the wrong default:

#### FL-01 — `synergismforkd_ui`: start with `Signal<GameState>`, promote to `Signal<Arc<GameState>>` only if measured
- The locked plan specifies "one `Signal<GameState>` at root, per-slice `use_memo` selectors, per-leaf fine-grained signals." `GameState` is ~30 slice fields totaling ~10s of KB. Naive `signal.read()` clones the entire state.
- Start without `Arc`. Add `criterion` benches measuring per-tick clone+render cost. Promote to `Signal<Arc<GameState>>` (with `Arc::make_mut` for writes) only if the benchmark justifies it. Don't speculate.

#### FL-02 — `synergismforkd_api`: `thiserror` for `ApiError`, `anyhow` only in `main()`
- `api` will accumulate ~15 error variants (auth, DB, payload validation, axum extractor failure, save load/persist, rate limit). Hand-rolled `Display`/`Error` (the `save` pattern) doesn't scale; use `thiserror`.
- `anyhow` erases variants — wrong for the public error type (axum status mapping becomes string-matching). Use `anyhow::Result<()>` only at the `main()` entry point.

#### FL-03 — `synergismforkd_ui_desktop`: keep the Tauri webview as the only logic host
- Per the locked plan and CLAUDE.md, game logic runs in-process in WASM alongside the UI to avoid per-tick IPC. Tauri commands are reserved for native-only operations (file pickers, Steam SDK, Discord RPC).
- When Tauri integration lands, also wire: `actions/attest-build-provenance` for SLSA L2 provenance (free with GitHub OIDC), CycloneDX SBOM per artifact, and split `contents: write` permission per RBP-037.

#### FL-04 — `synergismforkd_testkit`: when fixtures land, extract the `synergismforkd-sim` bin
- Today `testkit` is a `[lib]` + `[[bin]]`. Once parity fixtures + sim runner land, `testkit` will (a) be imported by every other crate's `[dev-dependencies]`, and (b) building the bin pulls every dev-only dep into the release path.
- Extract the bin into a sibling `synergismforkd_sim_cli` crate that depends on `testkit` from `[dependencies]`. Other workspace members depend on `testkit` from `[dev-dependencies]` only.

#### FL-05 — Compile-time `Send + Sync` assertion on `GameState`
- `axum` will need `State<Arc<Mutex<GameState>>>` or per-connection lock. Today `GameState` *is* `Send + Sync` because every field is — but no test asserts this. Adding any `Rc` field later silently breaks the property.
- Add to [crates/synergismforkd_logic/src/state/mod.rs](crates/synergismforkd_logic/src/state/mod.rs) now:
  ```rust
  const _: () = {
      const fn assert_send_sync<T: Send + Sync>() {}
      assert_send_sync::<GameState>();
      assert_send_sync::<TickOutput>();
      assert_send_sync::<CoreEvent>();
  };
  ```
  Same pattern as `bignum`'s `assert_copy::<Decimal>()`.

#### FL-06 — Property testing where it earns its keep
- 982 single-`assert` `#[test]` fns is solid unit-level coverage but shallow. Add `proptest` for:
  - **Currency arithmetic round-trips** — `Coins + Coins == Coins` with arbitrary `Decimal`; reflexivity (`a + Coins::zero() == a`).
  - **`solve_quadratic`** — `a > 0, disc > 0 → roots satisfy the original equation` within epsilon. Would have caught `det == 0.0` strict-equality (RBP-003).
  - **Sigmoid bounds** — `0 ≤ sigmoid(x) ≤ asymptote` for all `x`.
  - **Save round-trip** — *arbitrary* `GameState → save → load → equal-by-slice` (currently tested with two fixed states).
  - **Tick determinism** — identical `TackInput`s produce equal `TickOutput`s; would catch hidden RNG draws.
- `cargo-fuzz` only justified for the save loader (untrusted input).

#### FL-07 — When releases ship, emit signed SBOM
- Add to [.github/workflows/desktop-release.yml](.github/workflows/desktop-release.yml) when the Tauri build step lands:
  - `actions/attest-build-provenance@<SHA>` for SLSA L2 (free with GH OIDC).
  - `cargo sbom` per artifact (`cyclone_dx_json_1_5` format).
  - Document the verification recipe in README: `gh attestation verify <file> --repo MaddisonM79/synergism_forkd`.

---

## Appendix A — Drop-in `[workspace.lints]` block

Replace the existing `[workspace.lints]` block in the root `Cargo.toml` with:

```toml
[workspace.lints.rust]
unused_crate_dependencies = "warn"
unsafe_code = "forbid"                  # CLAUDE.md mandates no unsafe; promote to forbid
missing_debug_implementations = "warn"  # API Guidelines C-DEBUG
missing_docs = "warn"                   # Promote to deny per-crate when burndown complete
rust_2018_idioms = "warn"
unreachable_pub = "warn"                # Enforces pub(crate) discipline (RBP-001)
elided_lifetimes_in_paths = "warn"
trivial_casts = "warn"
trivial_numeric_casts = "warn"

[workspace.lints.clippy]
# Existing baseline
all = { level = "warn", priority = -1 }

# Pedantic + nursery as default-warn; suppress the parity-incompatible ones
pedantic = { level = "warn", priority = -1 }
nursery  = { level = "warn", priority = -1 }

# RBP-006: TS parity prohibits mul_add rewrites
suboptimal_flops = "allow"
# RBP-020: Decimal carries NaN; reflexivity not guaranteed
derive_partial_eq_without_eq = "allow"
# Style lints not backed by API Guidelines
module_name_repetitions = "allow"
similar_names = "allow"

# Restriction lints — opt in, do not bundle
unwrap_used = "warn"      # logic crate already denies non-test; promote to deny once burned down
expect_used = "warn"      # RBP-002 covers the one production case
panic = "warn"
unreachable = "warn"
todo = "warn"
unimplemented = "warn"
indexing_slicing = "warn" # RBP-019: prefer .get() over [idx]

[workspace.lints.rustdoc]
broken_intra_doc_links = "deny"      # RBP-014
private_intra_doc_links = "deny"     # RBP-014
missing_crate_level_docs = "warn"
```

## Appendix B — Drop-in `clippy.toml`

Commit at workspace root:

```toml
# Per-tick fns are necessarily complex; default 25 is too tight here.
cognitive-complexity-threshold = 40

# Tick orchestrator funnels parameters; default 7 is too tight.
too-many-arguments-threshold = 10

# Long-form numeric literals are parity artifacts.
literal-representation-threshold = 16384

# Disallowed methods — formalize the no-panic convention.
disallowed-methods = [
    { path = "std::option::Option::unwrap", reason = "use .expect(\"why\") or ? — see unwrap_used lint" },
    { path = "std::result::Result::unwrap", reason = "use .expect(\"why\") or ? — see unwrap_used lint" },
]

# MSRV must mirror workspace.package.rust-version.
msrv = "1.95"
```

## Appendix C — Drop-in `rustfmt.toml`

Commit at workspace root. Stable-only — `imports_granularity` and `group_imports` remain nightly:

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
newline_style = "Unix"
hard_tabs = false
tab_spaces = 4
```

## Appendix D — Drop-in `deny.toml`

Commit at workspace root. Prereq: every workspace crate must have `publish = false` (RBP-018).

```toml
# cargo-deny configuration for synergismforkd.
# Run: cargo deny check all

[graph]
all-features = true

[advisories]
version = 2
yanked = "deny"
ignore = []

[licenses]
version = 2
# Allow-list tuned for an MIT-licensed game shipping a static desktop binary.
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "Zlib",
    "CC0-1.0",
    "Unlicense",
    "MPL-2.0",  # File-level copyleft; OK for static-linked binary
                # provided MPL'd source files are not modified.
]
confidence-threshold = 0.93
exceptions = []

# Excluded — do not add without review:
#   GPL-*, AGPL-* (viral; incompatible with MIT redistribution)
#   LGPL-* (workable for dynamic linking; awkward for a single static binary)
#   OpenSSL (advertising clause; legacy openssl-sys chain indicator)
#   SSPL, BSL-*, Commons-Clause (non-OSI; commercial-use restrictions)

[bans]
multiple-versions = "warn"   # Cargo.lock currently has 0 duplicates; warn surfaces drift.
wildcards = "deny"
allow-wildcard-paths = true  # Requires `publish = false` on workspace crates (RBP-018).
highlight = "all"
workspace-default-features = "allow"
external-default-features = "allow"
allow = []
deny = [
    # Add specific bad-actor or deprecated crates here as discovered.
]
skip = []
skip-tree = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

## Appendix E — Drop-in CI `supply-chain` job

Append to `.github/workflows/rust-ci.yml`. Pin every action SHA at adoption (Dependabot will keep them current):

```yaml
  supply-chain:
    name: cargo-audit + cargo-deny
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          persist-credentials: false

      # cargo-audit — RustSec advisory scan
      - uses: rustsec/audit-check@69366f33c96575abad1ee0dba8212993eecbe998 # v2.0.0
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      # cargo-deny — license + bans + sources + advisories
      - uses: EmbarkStudios/cargo-deny-action@e2f4ede4a4e60ea15ff31bc0647485d80c66cfba # v2.0.4
        with:
          command: check all
          arguments: --all-features
```

## Appendix F — Tooling output summary

| Tool | Result |
|---|---|
| `cargo build --workspace` | OK |
| `cargo test --workspace` | OK |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK (clean against `clippy::all`) |
| `cargo clippy --workspace --all-targets -- -W clippy::pedantic -W clippy::nursery` | 1530 warnings (~779 unique after dedup) — see RBP-003 / RBP-009 |
| `cargo doc --workspace --no-deps` | 10 warnings — see RBP-014 |
| `cargo tree --workspace --duplicates --target all` | "nothing to print" — no duplicate dep versions |
| `cargo audit` | 0/1098 advisories — clean |
| `cargo deny check advisories` | OK |
| `cargo deny check licenses` (no `deny.toml`) | FAILED (`zerocopy` BSD-2/Apache-2/MIT triple-license rejected) — fixed by Appendix D |
| `cargo deny check bans` (with `deny.toml`) | FAILED on 6 wildcard-path errors — fixed by RBP-018 |
| `cargo deny check sources` | OK |
| Production `unwrap()` / `panic!()` count in `logic` | 1 (`state/rng.rs:65 expect(...)` — RBP-002) — all other instances in `#[cfg(test)]` |
| `unsafe` blocks workspace-wide | 0 |
| Public items workspace-wide | ~1145 |
| Tests workspace-wide | 982 `#[test]` annotations |

---

## Findings Index

| ID | Severity | Dimension | Title |
|---|---|---|---|
| RBP-001 | HIGH | Design | `logic` has 1,169 `pub` items, zero `pub(crate)` |
| RBP-002 | HIGH | Design+Code | `OsRng` syscall in `logic`; `Default` panics |
| RBP-003 | HIGH | Code | 775 `float_cmp` warnings, including production hot loops |
| RBP-004 | — | — | (Reserved — see exec summary) |
| RBP-005 | — | — | (Reserved — see exec summary) |
| RBP-006 | MED | Code | 366 `mul_add` candidates; suppress for TS parity |
| RBP-007 | MED | Code | Three float-condition `while` loops without progress guarantee |
| RBP-008 | MED | Code | `unreachable!()` on const-data invariant |
| RBP-009 | MED | Config | Workspace lint policy is the bare minimum |
| RBP-010 | MED | Supply chain | No `cargo-audit`/`cargo-deny` in CI |
| RBP-011 | MED | Supply chain | GitHub Actions pinned by floating tag |
| RBP-012 | MED | Supply chain | `break-eternity-rs` is project-controlled; CLAUDE.md implies otherwise |
| RBP-013 | MED | Code | Public `Result`-returning fns lack `# Errors` docs |
| RBP-014 | MED | Code | `cargo doc` produces 10 warnings |
| RBP-015 | MED | Design | `synergismforkd_common` is a 7-LoC placeholder |
| RBP-016 | MED | Design | `mechanics/` flat at 86 files |
| RBP-017 | MED | Design | `*Pre` aggregator migration is type-invisible |
| RBP-018 | MED | Supply chain | Workspace crates lack `publish = false` |
| RBP-019 | MED | Code | Lossy `as` casts in production hot paths |
| RBP-020 | MED | Code | Currency types derive `PartialEq` without `Eq`; intentional but undocumented |
| RBP-021 | MED | Code | Currency types lack `Display` |
| RBP-022 | MED | Design | `BuyRequest` rejection contract undocumented |
| RBP-023 | MED | Code | Public fn names redundantly include module path |
| RBP-024 | MED | Design | 6 structs with >3 `bool` fields |
| RBP-025 | MED | Code | `Coins::raw(self) -> Decimal` violates C-CONV |
| RBP-026 | MED | Code | 25 match arms with identical bodies (parity-intentional) |
| RBP-027 | MED | Code | `#[allow(dead_code)]` should be `#[expect(dead_code)]` |
| RBP-028 | LOW | Config | `bignum` should expose a `serde` feature flag |
| RBP-029 | LOW | Config | Workspace deps incomplete |
| RBP-030 | LOW | Config | No `[profile.dev]` tuning |
| RBP-031 | LOW | Code | 77 `cargo doc` first paragraphs too long |
| RBP-032 | LOW | Code | 40 doc identifier references without backticks |
| RBP-033 | LOW | Code | 14 `pub fn` could carry `#[must_use]` |
| RBP-034 | LOW | Code | 70 fns could be `const fn` |
| RBP-035 | LOW | Config | `unused_crate_dependencies` mostly silenced |
| RBP-036 | LOW | Supply chain | `getrandom 0.2.x` on maintenance track |
| RBP-037 | LOW | Supply chain | `desktop-release.yml` over-grants `contents: write` |
| RBP-038 | LOW | Config | `claude-auto-review.yml` triggers on fork PRs |
| RBP-039 | LOW | Code | Unbalanced backticks; literal style nits |
| RBP-040 | LOW | Code | Manual `Display`/`Error` impls; consider `thiserror` |
| FL-01 | — | Forward | `ui`: start with `Signal<GameState>` |
| FL-02 | — | Forward | `api`: `thiserror` for errors, `anyhow` only in `main()` |
| FL-03 | — | Forward | `ui_desktop`: Tauri webview as logic host |
| FL-04 | — | Forward | `testkit`: extract `synergismforkd-sim` bin to sibling crate |
| FL-05 | — | Forward | `assert_send_sync::<GameState>()` compile-time guard |
| FL-06 | — | Forward | Property testing for currency, summations, save round-trip |
| FL-07 | — | Forward | Signed SBOM on release |

**Totals**: 3 High + 22 Medium + 13 Low + 7 Forward-looking = 45 items.
