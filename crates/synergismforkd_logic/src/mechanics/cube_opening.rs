//! Cube opening — distribute Wow! Cube / Tesseract / Hypercube / Platonic
//! "blessings" by opening the matching cube currency.
//!
//! Port of the legacy `CubeExperimental.ts` `open()` methods. Opening `n`
//! cubes splits into a **deterministic bulk** grant (`weight × floor(n / base)`
//! per blessing, where the weights sum to the modulo `base`, so a full set
//! yields exactly the expected counts) and a **stochastic remainder** (`n %
//! base` per-cube rolls against the blessing pdf bands).
//!
//! The RNG draws from [`RngPurpose::CubeOpen`]. Like the rest of the port's
//! randomness, this stream is a deterministic `Xoshiro256PlusPlus` rather than
//! the legacy unseeded `Math.random()`, so opens are reproducible but **not**
//! TS-bit-equal; tests assert the deterministic bulk exactly and the stochastic
//! remainder by total/range.
//!
//! Three tiers cascade: opening tesseracts grants free cube opens
//! (`researches[153]`), hypercubes grant free tesseract opens
//! (`researches[183]`). The platonic→hypercube cascade needs the unported
//! `platonicToHypercubes` achievement reward and is inert (neutral 0) until it
//! lands. The platonic `scoreBonus` blessing is written-but-never-read in the
//! legacy schema, so this port computes its distribution share (for faithful
//! remainder accounting) and discards it (see [`PlatonicBlessings::add_from_eight`]).

use rand::Rng;
use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::{CoreEvent, CubeTier};
use crate::math::rng::next_f64;
use crate::state::rng::RngPurpose;
use crate::state::GameState;

// ─── Distribution tables ──────────────────────────────────────────────────────

/// Per-blessing weights for cube / tesseract / hypercube opening, in the
/// canonical `cubeBlessings` key order. Sum to 20 — the bulk modulo base.
const CUBE_WEIGHTS: [f64; 10] = [4.0, 4.0, 2.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0];

/// Upper bound of each cube-blessing pdf band over `(0, 100]`. A per-cube roll
/// `num = 100·rand()` grants `+1` to the first blessing whose band it does not
/// exceed (`num <= band`), so each blessing's probability is `weight / 20`.
const CUBE_BANDS: [f64; 10] = [20.0, 40.0, 50.0, 60.0, 70.0, 80.0, 85.0, 90.0, 95.0, 100.0];

/// Platonic-blessing weights, 8 logical slots (incl. the dead `scoreBonus` at
/// index 6) in the legacy `platonicBlessings` key order. Sum to 40000.
const PLATONIC_WEIGHTS: [f64; 8] = [13200.0, 13200.0, 13200.0, 396.0, 1.0, 1.0, 1.0, 1.0];

/// Upper bound of each platonic pdf band over `(0, 100]` for the final per-unit
/// roll.
const PLATONIC_BANDS: [f64; 8] = [33.0, 66.0, 99.0, 99.99, 99.9925, 99.995, 99.9975, 100.0];

// ─── Shared helpers ───────────────────────────────────────────────────────────

/// Resolve the cube count to consume: `max` opens the whole balance, `free`
/// opens exactly `value` without spending, otherwise `min(balance, value)`.
fn resolve_to_spend(balance: f64, value: f64, max: bool, free: bool) -> f64 {
    if max {
        balance
    } else if free {
        value
    } else {
        balance.min(value)
    }
}

/// `calculateCubeQuarkMultiplier()` assembled from `&GameState`. The
/// `autoWarpCheck` / `dailyPowderResetUses` warp bonus is an unported
/// auto-feature → neutral (`auto_warp_check = false`).
fn cube_quark_multiplier(state: &GameState) -> f64 {
    use crate::mechanics::overflux_bonuses::{
        calculate_cube_quark_multiplier, CalculateCubeQuarkMultiplierInput,
    };
    use crate::mechanics::shop_upgrades::cube_to_quark_all_effect;
    use crate::state::shop::SHOP_CUBE_TO_QUARK_ALL;

    calculate_cube_quark_multiplier(&CalculateCubeQuarkMultiplierInput {
        overflux_orbs: state.hepteracts.overflux_orbs,
        highest_singularity_count: state.singularity.highest_singularity_count,
        cube_to_quark_all_mult: cube_to_quark_all_effect(
            state.shop.upgrades[SHOP_CUBE_TO_QUARK_ALL],
        ),
        auto_warp_check: false,
        daily_powder_reset_uses: 0.0,
    })
}

/// `checkQuarkGain(base, mult, cubesOpenedDaily)` — quarks earned from the
/// cumulative cubes opened today: `floor(applyBonus(log10(cubes) · base · mult ·
/// cubeMult))`, where `applyBonus(x) = x · (1 + quark_bonus / 100)`.
fn check_quark_gain(
    base: f64,
    mult: f64,
    cube_mult: f64,
    quark_bonus: f64,
    cubes_opened: f64,
) -> f64 {
    if cubes_opened < 1.0 {
        return 0.0;
    }
    let multiplier = mult * cube_mult;
    ((1.0 + quark_bonus / 100.0) * (cubes_opened.log10() * base * multiplier)).floor()
}

/// The shared 20-bucket cube/tesseract/hypercube distribution: deterministic
/// `weight × div20` bulk, then `modulo` stochastic per-cube pdf rolls. Returns
/// the per-blessing increments in canonical order.
fn distribute_cube_blessings(div20: f64, modulo: f64, rng: &mut impl Rng) -> [f64; 10] {
    let mut acc = [0.0_f64; 10];
    for (slot, &weight) in acc.iter_mut().zip(CUBE_WEIGHTS.iter()) {
        *slot = weight * div20;
    }
    for _ in 0..(modulo as u64) {
        let num = 100.0 * next_f64(rng);
        let idx = CUBE_BANDS.iter().position(|&band| num <= band).unwrap_or(9);
        acc[idx] += 1.0;
    }
    acc
}

/// Credit the incremental quark gift from opening (`worlds.add(actual, false,
/// true)`): the delta over the running daily total. Pushes [`CoreEvent::QuarksAwarded`].
fn credit_open_quarks(
    state: &mut GameState,
    gain: f64,
    daily_awarded: f64,
    events: &mut SmallVec<[CoreEvent; 2]>,
) -> f64 {
    let actual = (gain - daily_awarded).max(0.0);
    if actual > 0.0 {
        state.quarks.worlds += Decimal::from_finite(actual);
        state.golden_quarks.quarks_this_singularity += actual;
        events.push(CoreEvent::QuarksAwarded { quarks: actual });
    }
    actual
}

// ─── Per-tier open ────────────────────────────────────────────────────────────

/// `WowCubes.open` — the most involved tier: research multipliers, the
/// `taxmanLastStand` divisor, the `cubeUpgrades[13/23/33]` modulo boosts, the
/// `1e300` tribute cap, and the `oneCubeOfMany` achievement.
pub(crate) fn open_cubes(
    state: &mut GameState,
    value: f64,
    max: bool,
    free: bool,
) -> SmallVec<[CoreEvent; 2]> {
    use crate::mechanics::achievement_awards::award_ungrouped_achievement;
    use crate::mechanics::shop_upgrades::cube_to_quark_effect;
    use crate::state::shop::SHOP_CUBE_TO_QUARK;
    const ONE_CUBE_OF_MANY: usize = 246;

    let mut events: SmallVec<[CoreEvent; 2]> = SmallVec::new();
    let balance = state.cube_balances.wow_cubes.to_number();
    let mut to_spend = resolve_to_spend(balance, value, max, free);
    let spent = to_spend;

    // oneCubeOfMany — opening exactly one cube at >= 2e11 accelerator blessing.
    if value == 1.0 && state.cube_blessings.accelerator >= 2e11 {
        award_ungrouped_achievement(&mut state.achievements, ONE_CUBE_OF_MANY, 50.0, true);
    }

    if !free {
        state.cube_balances.wow_cubes = Decimal::from_finite((balance - to_spend).max(0.0));
    }
    state.cube_balances.cube_opened_daily += to_spend;

    let cube_mult = cube_quark_multiplier(state);
    let shop_mult = cube_to_quark_effect(state.shop.upgrades[SHOP_CUBE_TO_QUARK]);
    let gain = check_quark_gain(
        5.0,
        shop_mult,
        cube_mult,
        state.quarks.quark_bonus,
        state.cube_balances.cube_opened_daily,
    );
    let actual = credit_open_quarks(
        state,
        gain,
        state.cube_balances.cube_quark_daily,
        &mut events,
    );
    state.cube_balances.cube_quark_daily += actual;

    // `sumContents(cubeBlessings) >= 1e300` → no more tributes awarded.
    let sum_of_tributes = state.cube_blessings.sum();
    if sum_of_tributes >= 1e300 {
        events.push(CoreEvent::CubesOpened {
            tier: CubeTier::Cubes,
            spent,
        });
        return events;
    }

    // Cubes-only research multipliers (researches 138 / 168 / 198), then floor.
    let r138 = state.researches.researches[138];
    let r168 = state.researches.researches[168];
    let r198 = state.researches.researches[198];
    to_spend *= 1.0 + r138 / 1000.0;
    to_spend *= 1.0 + 0.8 * r168 / 1000.0;
    to_spend *= 1.0 + 0.6 * r198 / 1000.0;
    to_spend = to_spend.floor();
    to_spend = to_spend.min(1e300 - sum_of_tributes);

    // Exalt-8 taxmanLastStand divisor at >= 5 completions.
    if state.singularity.taxman_last_stand.enabled
        && state.singularity.taxman_last_stand.completions >= 5.0
    {
        to_spend /= (1.0 + (1.0 + sum_of_tributes + to_spend).ln()).powi(3);
    }

    // div/modulo split + the cubeUpgrade[13/23/33] modulo boosts, then re-fold.
    let mut modulo = to_spend % 20.0;
    let mut div20 = (to_spend / 20.0).floor();
    let cu13 = state.cube_upgrade_levels.cube_upgrades[13];
    let cu23 = state.cube_upgrade_levels.cube_upgrades[23];
    let cu33 = state.cube_upgrade_levels.cube_upgrades[33];
    if div20 > 0.0 && cu13 == 1.0 {
        modulo += div20;
    }
    if div20 > 0.0 && cu23 == 1.0 {
        modulo += div20;
    }
    if div20 > 0.0 && cu33 == 1.0 {
        modulo += div20;
    }
    div20 += (modulo / 20.0).floor();
    modulo %= 20.0;

    let acc = distribute_cube_blessings(div20, modulo, state.rng.draw(RngPurpose::CubeOpen));
    state.cube_blessings.add_in_order(&acc);

    events.push(CoreEvent::CubesOpened {
        tier: CubeTier::Cubes,
        spent,
    });
    events
}

/// `WowTesseracts.open` — the plain 20-bucket distribution + the cube cascade
/// (`researches[153]` free cube opens).
pub(crate) fn open_tesseracts(
    state: &mut GameState,
    value: f64,
    max: bool,
    free: bool,
) -> SmallVec<[CoreEvent; 2]> {
    use crate::mechanics::shop_upgrades::tesseract_to_quark_effect;
    use crate::state::shop::SHOP_TESSERACT_TO_QUARK;

    let mut events: SmallVec<[CoreEvent; 2]> = SmallVec::new();
    let balance = state.cube_balances.wow_tesseracts.to_number();
    let to_spend = resolve_to_spend(balance, value, max, free);

    if !free {
        state.cube_balances.wow_tesseracts = Decimal::from_finite((balance - to_spend).max(0.0));
    }
    state.cube_balances.tesseract_opened_daily += to_spend;

    let cube_mult = cube_quark_multiplier(state);
    let shop_mult = tesseract_to_quark_effect(state.shop.upgrades[SHOP_TESSERACT_TO_QUARK]);
    let gain = check_quark_gain(
        7.0,
        shop_mult,
        cube_mult,
        state.quarks.quark_bonus,
        state.cube_balances.tesseract_opened_daily,
    );
    let actual = credit_open_quarks(
        state,
        gain,
        state.cube_balances.tesseract_quark_daily,
        &mut events,
    );
    state.cube_balances.tesseract_quark_daily += actual;

    let modulo = to_spend % 20.0;
    let div20 = (to_spend / 20.0).floor();
    let acc = distribute_cube_blessings(div20, modulo, state.rng.draw(RngPurpose::CubeOpen));
    state.tesseract_blessings.add_in_order(&acc);
    events.push(CoreEvent::CubesOpened {
        tier: CubeTier::Tesseracts,
        spent: to_spend,
    });

    // Cascade: research 6x33 grants free cube opens.
    let extra = (12.0 * to_spend * state.researches.researches[153]).floor();
    if extra > 0.0 {
        events.extend(open_cubes(state, extra, false, true));
    }
    events
}

/// `WowHypercubes.open` — 20-bucket distribution + the tesseract cascade
/// (`researches[183]` free tesseract opens).
pub(crate) fn open_hypercubes(
    state: &mut GameState,
    value: f64,
    max: bool,
    free: bool,
) -> SmallVec<[CoreEvent; 2]> {
    use crate::mechanics::shop_upgrades::hypercube_to_quark_effect;
    use crate::state::shop::SHOP_HYPERCUBE_TO_QUARK;

    let mut events: SmallVec<[CoreEvent; 2]> = SmallVec::new();
    let balance = state.cube_balances.wow_hypercubes.to_number();
    let to_spend = resolve_to_spend(balance, value, max, free);

    if !free {
        state.cube_balances.wow_hypercubes = Decimal::from_finite((balance - to_spend).max(0.0));
    }
    state.cube_balances.hypercube_opened_daily += to_spend;

    let cube_mult = cube_quark_multiplier(state);
    let shop_mult = hypercube_to_quark_effect(state.shop.upgrades[SHOP_HYPERCUBE_TO_QUARK]);
    let gain = check_quark_gain(
        10.0,
        shop_mult,
        cube_mult,
        state.quarks.quark_bonus,
        state.cube_balances.hypercube_opened_daily,
    );
    let actual = credit_open_quarks(
        state,
        gain,
        state.cube_balances.hypercube_quark_daily,
        &mut events,
    );
    state.cube_balances.hypercube_quark_daily += actual;

    let modulo = to_spend % 20.0;
    let div20 = (to_spend / 20.0).floor();
    let acc = distribute_cube_blessings(div20, modulo, state.rng.draw(RngPurpose::CubeOpen));
    state.hypercube_blessings.add_in_order(&acc);
    events.push(CoreEvent::CubesOpened {
        tier: CubeTier::Hypercubes,
        spent: to_spend,
    });

    // Cascade: research 8x33 grants free tesseract opens.
    let extra = (100.0 * to_spend * state.researches.researches[183]).floor();
    if extra > 0.0 {
        events.extend(open_tesseracts(state, extra, false, true));
    }
    events
}

/// `WowPlatonicCubes.open` — the bespoke 40000-bucket distribution: bulk grant
/// (with the `cubeUpgrades[64]` weight-1 doubling), the "RNGesus" rare-drop
/// loop, proportional common drops, then a final per-unit pdf roll. The
/// platonic→hypercube cascade (`platonicToHypercubes` achievement reward) is
/// unported → inert.
pub(crate) fn open_platonic(
    state: &mut GameState,
    value: f64,
    max: bool,
    free: bool,
) -> SmallVec<[CoreEvent; 2]> {
    let mut events: SmallVec<[CoreEvent; 2]> = SmallVec::new();
    let balance = state.cube_balances.wow_platonic_cubes.to_number();
    let to_spend = resolve_to_spend(balance, value, max, free);

    if !free {
        state.cube_balances.wow_platonic_cubes =
            Decimal::from_finite((balance - to_spend).max(0.0));
    }
    state.cube_balances.platonic_cube_opened_daily += to_spend;

    // Base 15, fixed mult 1.5 (no platonic-to-quark shop upgrade).
    let cube_mult = cube_quark_multiplier(state);
    let gain = check_quark_gain(
        15.0,
        1.5,
        cube_mult,
        state.quarks.quark_bonus,
        state.cube_balances.platonic_cube_opened_daily,
    );
    let actual = credit_open_quarks(
        state,
        gain,
        state.cube_balances.platonic_cube_quark_daily,
        &mut events,
    );
    state.cube_balances.platonic_cube_quark_daily += actual;

    let div40000 = (to_spend / 40_000.0).floor();
    let mut modulo = to_spend % 40_000.0;

    // Bulk grant; the four weight-1 rare slots double under cubeUpgrades[64].
    let double_rares = state.cube_upgrade_levels.cube_upgrades[64] > 0.0;
    let mut acc = [0.0_f64; 8];
    for (slot, &weight) in acc.iter_mut().zip(PLATONIC_WEIGHTS.iter()) {
        *slot = weight * div40000;
        if weight == 1.0 && double_rares {
            *slot += div40000;
        }
    }

    // "RNGesus" rare-drop loop over the four weight-1 slots (indices 4..8).
    for slot in acc[4..8].iter_mut() {
        let num = next_f64(state.rng.draw(RngPurpose::CubeOpen));
        if modulo / 40_000.0 >= num && modulo != 0.0 {
            *slot += 1.0;
            modulo -= 1.0;
        }
    }

    // Proportional common drops over the four common slots (indices 0..4), each
    // computed against the same post-RNGesus remainder.
    let common = [
        (33.0 * modulo / 100.0).floor(),
        (33.0 * modulo / 100.0).floor(),
        (33.0 * modulo / 100.0).floor(),
        (396.0 * modulo / 40_000.0).floor(),
    ];
    for (slot, &gained) in acc[..4].iter_mut().zip(common.iter()) {
        *slot += gained;
        modulo -= gained;
    }

    // Final per-unit pdf roll on the leftover.
    for _ in 0..(modulo as u64) {
        let num = 100.0 * next_f64(state.rng.draw(RngPurpose::CubeOpen));
        let idx = PLATONIC_BANDS
            .iter()
            .position(|&band| num <= band)
            .unwrap_or(7);
        acc[idx] += 1.0;
    }

    state.platonic_blessings.add_from_eight(&acc);
    events.push(CoreEvent::CubesOpened {
        tier: CubeTier::Platonic,
        spent: to_spend,
    });
    events
}

/// Open `value` cubes of `tier` (or the whole balance when `max`). The
/// player-initiated entry point — never `free`; the free re-opens are internal
/// to the tier cascades.
pub(crate) fn open(
    state: &mut GameState,
    tier: CubeTier,
    value: f64,
    max: bool,
) -> SmallVec<[CoreEvent; 2]> {
    match tier {
        CubeTier::Cubes => open_cubes(state, value, max, false),
        CubeTier::Tesseracts => open_tesseracts(state, value, max, false),
        CubeTier::Hypercubes => open_hypercubes(state, value, max, false),
        CubeTier::Platonic => open_platonic(state, value, max, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_cubes_bulk_is_deterministic_at_a_full_set() {
        let mut state = GameState::default();
        state.cube_balances.wow_cubes = Decimal::from_finite(100.0);
        // 20 cubes → div20 = 1, modulo = 0 → exactly the weights, RNG-free.
        open_cubes(&mut state, 20.0, false, false);
        let b = &state.cube_blessings;
        assert_eq!(b.accelerator, 4.0);
        assert_eq!(b.multiplier, 4.0);
        assert_eq!(b.offering, 2.0);
        assert_eq!(b.ant_sacrifice, 1.0);
        assert_eq!(b.global_speed, 1.0);
        // Currency spent.
        assert_eq!(state.cube_balances.wow_cubes.to_number(), 80.0);
    }

    #[test]
    fn open_cubes_remainder_grants_exactly_modulo_blessings() {
        let mut state = GameState::default();
        state.cube_balances.wow_cubes = Decimal::from_finite(50.0);
        // 5 cubes → div20 = 0, modulo = 5 → 5 RNG +1s total (which blessings
        // depends on the seed, but the total is exact).
        open_cubes(&mut state, 5.0, false, false);
        assert_eq!(state.cube_blessings.sum(), 5.0);
    }

    #[test]
    fn open_most_spends_whole_balance() {
        let mut state = GameState::default();
        state.cube_balances.wow_cubes = Decimal::from_finite(123.0);
        open_cubes(&mut state, 0.0, true, false);
        assert_eq!(state.cube_balances.wow_cubes.to_number(), 0.0);
        // 123 cubes → 6 full sets (div20=6) + 3 remainder = 6*20 + 3 = 123 blessings.
        assert_eq!(state.cube_blessings.sum(), 123.0);
    }

    #[test]
    fn tesseract_open_cascades_into_cube_blessings() {
        let mut state = GameState::default();
        state.cube_balances.wow_tesseracts = Decimal::from_finite(100.0);
        state.researches.researches[153] = 1.0; // enables the free cube cascade

        open_tesseracts(&mut state, 20.0, false, false);

        // Tesseract's own full set.
        assert_eq!(state.tesseract_blessings.accelerator, 4.0);
        // Cascade: floor(12 * 20 * 1) = 240 free cube opens → div20 = 12 → 48.
        assert_eq!(state.cube_blessings.accelerator, 48.0);
    }

    #[test]
    fn platonic_open_bulk_is_deterministic_at_a_full_set() {
        let mut state = GameState::default();
        state.cube_balances.wow_platonic_cubes = Decimal::from_finite(40_000.0);
        // 40000 → div40000 = 1, modulo = 0 → exactly the weights, RNG-free.
        open_platonic(&mut state, 40_000.0, false, false);
        let p = &state.platonic_blessings;
        assert_eq!(p.cubes, 13_200.0);
        assert_eq!(p.platonics, 396.0);
        assert_eq!(p.hypercube_bonus, 1.0);
        assert_eq!(p.taxes, 1.0);
        assert_eq!(p.global_speed, 1.0); // the scoreBonus slot is discarded
        assert_eq!(state.cube_balances.wow_platonic_cubes.to_number(), 0.0);
    }

    #[test]
    fn one_cube_of_many_awards_at_high_accelerator_blessing() {
        let mut state = GameState::default();
        state.cube_balances.wow_cubes = Decimal::from_finite(10.0);
        state.cube_blessings.accelerator = 2e11;
        open_cubes(&mut state, 1.0, false, false);
        assert_ne!(state.achievements.achievements[246], 0); // oneCubeOfMany
    }

    #[test]
    fn opening_awards_quarks_with_overflux_orbs() {
        let mut state = GameState::default();
        state.cube_balances.wow_cubes = Decimal::from_finite(1e6);
        state.hepteracts.overflux_orbs = 1e6; // drives the cube-quark multiplier

        let events = open_cubes(&mut state, 1e6, false, false);

        assert!(state.quarks.worlds.to_number() > 0.0);
        assert!(state.cube_balances.cube_quark_daily > 0.0);
        assert!(events
            .iter()
            .any(|e| matches!(e, CoreEvent::QuarksAwarded { .. })));
    }
}
