//! Achievement awarding — the write path for `player.achievements`.
//!
//! Port of the legacy awarding machinery in `Achievements.ts`:
//! `awardAchievement` (set the bit + accumulate `achievementPoints`),
//! `buildingAchievementCheck`, and the point-gain side of
//! `resetAchievementCheck`. Until this module existed the `[u8; 509]`
//! bitmap was read at ~12 sites but written nowhere, so
//! `achievement_points` was frozen at `0` — which zeroes the crystal
//! `(1 + 0.01·cu[0])^points` and mythos `1.01^points·(points/5 + 1)`
//! exponents ([`crate::mechanics::global_multipliers`]) and the whole
//! `achievement_level` cascade.
//!
//! ## Slice 1 scope (P3.1)
//!
//! The two cleanest, highest-traffic check families with uniform-threshold
//! conditions:
//! - `buildingAchievementCheck` → the five `*OwnedCoin` groups (owned
//!   coin-producer counts), indices 1–35 + 281–295.
//! - `resetAchievementCheck` → the `{prestige,transcend,reincarnation}`
//!   `PointGain` groups (the just-computed reset gain), indices 36–56 +
//!   296–304.
//!
//! Each in-scope achievement's `(index, threshold, pointValue)` was
//! extracted programmatically from the legacy `achievements` array (a
//! brace-matched parse, not hand-counted) to avoid index-transcription
//! drift.
//!
//! ## Deferred (faithful: those bits stay `0`, as before)
//!
//! - The ungrouped no-accelerator / no-mult / no-upgrade reset
//!   achievements (indices 57–74) — they need the timing-sensitive
//!   `*no*` flags + a name→index map; they land with the ungrouped tail.
//! - `challengeAchievementCheck` (the 14 challenge groups) — slice 2.
//! - Progressive achievements (the `Math.max` cache system) — slice 3.
//! - The quark reward on award (`player.worlds.add(getAchievementQuarks())`)
//!   — a separable currency effect; see [`super::achievement_points`].
//!
//! Points accumulate **incrementally** here (`achievement_points +=
//! pointValue`), mirroring `awardAchievement`. The full-recompute path
//! ([`super::achievement_points::compute_achievement_points`]) needs the
//! caller-provided 509-entry value table and lands with save-load.

use synergismforkd_bignum::Decimal;

use crate::events::AutoResetTier;
use crate::state::achievements::AchievementsState;

/// `(achievement index, owned-count threshold, point value)` — a building
/// group row. Condition: `player.{tier}OwnedCoin >= threshold`.
type ThresholdRow = (usize, f64, f64);

/// `(achievement index, log10(gain) threshold, point value)` — a point-gain
/// group row. Condition: `G.{tier}PointGain.gte(10^threshold)`, evaluated in
/// log10 space so the `1e1000`+ thresholds stay representable.
type Log10Row = (usize, f64, f64);

// ─── Building groups (Achievements.ts, the `*OwnedCoin` groups) ─────────────

const FIRST_OWNED_COIN: &[ThresholdRow] = &[
    (1, 1.0, 5.0),
    (2, 10.0, 10.0),
    (3, 100.0, 15.0),
    (4, 1_000.0, 20.0),
    (5, 5_000.0, 25.0),
    (6, 10_000.0, 30.0),
    (7, 20_000.0, 35.0),
    (281, 1e5, 40.0),
    (282, 1e6, 45.0),
    (283, 1e8, 50.0),
];

const SECOND_OWNED_COIN: &[ThresholdRow] = &[
    (8, 1.0, 5.0),
    (9, 10.0, 10.0),
    (10, 100.0, 15.0),
    (11, 500.0, 20.0),
    (12, 5_000.0, 25.0),
    (13, 10_000.0, 30.0),
    (14, 20_000.0, 35.0),
    (284, 1e6, 40.0),
    (285, 1e8, 45.0),
    (286, 1e9, 50.0),
];

const THIRD_OWNED_COIN: &[ThresholdRow] = &[
    (15, 1.0, 5.0),
    (16, 10.0, 10.0),
    (17, 100.0, 15.0),
    (18, 333.0, 20.0),
    (19, 5_000.0, 25.0),
    (20, 10_000.0, 30.0),
    (21, 20_000.0, 35.0),
    (287, 1e7, 40.0),
    (288, 1e8, 45.0),
    (289, 5e9, 50.0),
];

const FOURTH_OWNED_COIN: &[ThresholdRow] = &[
    (22, 1.0, 5.0),
    (23, 10.0, 10.0),
    (24, 100.0, 15.0),
    (25, 333.0, 20.0),
    (26, 5_000.0, 25.0),
    (27, 10_000.0, 30.0),
    (28, 20_000.0, 35.0),
    (290, 1e8, 40.0),
    (291, 1e9, 45.0),
    (292, 2e10, 50.0),
];

const FIFTH_OWNED_COIN: &[ThresholdRow] = &[
    (29, 1.0, 5.0),
    (30, 10.0, 10.0),
    (31, 66.0, 15.0),
    (32, 200.0, 20.0),
    (33, 6_666.0, 25.0),
    (34, 17_777.0, 30.0),
    (35, 42_777.0, 35.0),
    (293, 1e9, 40.0),
    (294, 2e10, 45.0),
    (295, 1e12, 50.0),
];

// ─── Point-gain groups (Achievements.ts, the `*PointGain` groups) ───────────

const PRESTIGE_POINT_GAIN: &[Log10Row] = &[
    (36, 0.0, 5.0),
    (37, 6.0, 10.0),
    (38, 100.0, 15.0),
    (39, 1_000.0, 20.0),
    (40, 10_000.0, 25.0),
    (41, 77_777.0, 30.0),
    (42, 250_000.0, 35.0),
    (296, 1e7, 40.0),
    (297, 1e10, 45.0),
    (298, 1e13, 50.0),
];

const TRANSCEND_POINT_GAIN: &[Log10Row] = &[
    (43, 0.0, 5.0),
    (44, 6.0, 10.0),
    (45, 50.0, 15.0),
    (46, 308.0, 20.0),
    (47, 1_500.0, 25.0),
    (48, 25_000.0, 30.0),
    (49, 100_000.0, 35.0),
    (299, 2.5e6, 40.0),
    (300, 2.5e9, 45.0),
    (301, 2.5e12, 50.0),
];

const REINCARNATION_POINT_GAIN: &[Log10Row] = &[
    (50, 0.0, 5.0),
    (51, 5.0, 10.0),
    (52, 30.0, 15.0),
    (53, 200.0, 20.0),
    (54, 1_500.0, 25.0),
    (55, 5_000.0, 30.0),
    (56, 7_777.0, 35.0),
    (302, 100_000.0, 40.0),
    (303, 1e8, 45.0),
    (304, 1e11, 50.0),
];

// ─── Challenge groups (Achievements.ts, the `challengeN` groups) ────────────
//
// Each row's condition is `player.challengecompletions[N] >= threshold` for
// the matching challenge `N`. Transcension/reincarnation challenges (1–10)
// carry 10 rows; ascension challenges (11–14) carry 12.

const CHALLENGE_1: &[ThresholdRow] = &[
    (78, 1.0, 5.0),
    (79, 3.0, 10.0),
    (80, 5.0, 15.0),
    (81, 10.0, 20.0),
    (82, 20.0, 25.0),
    (83, 50.0, 30.0),
    (84, 75.0, 35.0),
    (305, 1_000.0, 40.0),
    (306, 9_000.0, 45.0),
    (307, 9_001.0, 50.0),
];

const CHALLENGE_2: &[ThresholdRow] = &[
    (85, 1.0, 5.0),
    (86, 3.0, 10.0),
    (87, 5.0, 15.0),
    (88, 10.0, 20.0),
    (89, 20.0, 25.0),
    (90, 50.0, 30.0),
    (91, 75.0, 35.0),
    (308, 1_000.0, 40.0),
    (309, 9_000.0, 45.0),
    (310, 9_001.0, 50.0),
];

const CHALLENGE_3: &[ThresholdRow] = &[
    (92, 1.0, 5.0),
    (93, 3.0, 10.0),
    (94, 5.0, 15.0),
    (95, 10.0, 20.0),
    (96, 20.0, 25.0),
    (97, 50.0, 30.0),
    (98, 75.0, 35.0),
    (311, 1_000.0, 40.0),
    (312, 9_000.0, 45.0),
    (313, 9_001.0, 50.0),
];

const CHALLENGE_4: &[ThresholdRow] = &[
    (99, 1.0, 5.0),
    (100, 3.0, 10.0),
    (101, 5.0, 15.0),
    (102, 10.0, 20.0),
    (103, 20.0, 25.0),
    (104, 50.0, 30.0),
    (105, 75.0, 35.0),
    (314, 1_000.0, 40.0),
    (315, 9_000.0, 45.0),
    (316, 9_001.0, 50.0),
];

const CHALLENGE_5: &[ThresholdRow] = &[
    (106, 1.0, 5.0),
    (107, 3.0, 10.0),
    (108, 5.0, 15.0),
    (109, 10.0, 20.0),
    (110, 20.0, 25.0),
    (111, 50.0, 30.0),
    (112, 75.0, 35.0),
    (317, 1_000.0, 40.0),
    (318, 9_000.0, 45.0),
    (319, 9_001.0, 50.0),
];

const CHALLENGE_6: &[ThresholdRow] = &[
    (113, 1.0, 5.0),
    (114, 2.0, 10.0),
    (115, 3.0, 15.0),
    (116, 5.0, 20.0),
    (117, 10.0, 25.0),
    (118, 15.0, 30.0),
    (119, 25.0, 35.0),
    (320, 40.0, 40.0),
    (321, 80.0, 45.0),
    (322, 120.0, 50.0),
];

const CHALLENGE_7: &[ThresholdRow] = &[
    (120, 1.0, 5.0),
    (121, 2.0, 10.0),
    (122, 3.0, 15.0),
    (123, 5.0, 20.0),
    (124, 10.0, 25.0),
    (125, 15.0, 30.0),
    (126, 25.0, 35.0),
    (323, 40.0, 40.0),
    (324, 80.0, 45.0),
    (325, 125.0, 50.0),
];

const CHALLENGE_8: &[ThresholdRow] = &[
    (127, 1.0, 5.0),
    (128, 2.0, 10.0),
    (129, 3.0, 15.0),
    (130, 5.0, 20.0),
    (131, 10.0, 25.0),
    (132, 15.0, 30.0),
    (133, 25.0, 35.0),
    (326, 40.0, 40.0),
    (327, 80.0, 45.0),
    (328, 130.0, 50.0),
];

const CHALLENGE_9: &[ThresholdRow] = &[
    (134, 1.0, 5.0),
    (135, 2.0, 10.0),
    (136, 3.0, 15.0),
    (137, 5.0, 20.0),
    (138, 10.0, 25.0),
    (139, 20.0, 30.0),
    (140, 25.0, 35.0),
    (329, 40.0, 40.0),
    (330, 80.0, 45.0),
    (331, 135.0, 50.0),
];

const CHALLENGE_10: &[ThresholdRow] = &[
    (141, 1.0, 5.0),
    (142, 2.0, 10.0),
    (143, 3.0, 15.0),
    (144, 5.0, 20.0),
    (145, 10.0, 25.0),
    (146, 20.0, 30.0),
    (147, 25.0, 35.0),
    (332, 40.0, 40.0),
    (333, 80.0, 45.0),
    (334, 140.0, 50.0),
];

const CHALLENGE_11: &[ThresholdRow] = &[
    (197, 1.0, 10.0),
    (198, 2.0, 20.0),
    (199, 3.0, 30.0),
    (200, 5.0, 40.0),
    (201, 10.0, 50.0),
    (202, 20.0, 60.0),
    (203, 30.0, 70.0),
    (362, 40.0, 80.0),
    (363, 50.0, 90.0),
    (364, 60.0, 100.0),
    (365, 65.0, 110.0),
    (366, 70.0, 120.0),
];

const CHALLENGE_12: &[ThresholdRow] = &[
    (204, 1.0, 10.0),
    (205, 2.0, 20.0),
    (206, 3.0, 30.0),
    (207, 5.0, 40.0),
    (208, 10.0, 50.0),
    (209, 20.0, 60.0),
    (210, 30.0, 70.0),
    (367, 40.0, 80.0),
    (368, 50.0, 90.0),
    (369, 60.0, 100.0),
    (370, 65.0, 110.0),
    (371, 70.0, 120.0),
];

const CHALLENGE_13: &[ThresholdRow] = &[
    (211, 1.0, 10.0),
    (212, 2.0, 20.0),
    (213, 3.0, 30.0),
    (214, 5.0, 40.0),
    (215, 10.0, 50.0),
    (216, 20.0, 60.0),
    (217, 30.0, 70.0),
    (372, 40.0, 80.0),
    (373, 50.0, 90.0),
    (374, 60.0, 100.0),
    (375, 70.0, 110.0),
    (376, 72.0, 120.0),
];

const CHALLENGE_14: &[ThresholdRow] = &[
    (218, 1.0, 10.0),
    (219, 2.0, 20.0),
    (220, 3.0, 30.0),
    (221, 5.0, 40.0),
    (222, 10.0, 50.0),
    (223, 20.0, 60.0),
    (224, 30.0, 70.0),
    (377, 40.0, 80.0),
    (378, 50.0, 90.0),
    (379, 60.0, 100.0),
    (380, 70.0, 110.0),
    (381, 72.0, 120.0),
];

/// `awardAchievement(index)` — award `index` when `unlocked` and not already
/// owned: set the bit and accumulate its `point_value`
/// (`achievementPoints += achievements[index].pointValue`). Returns whether
/// it was newly awarded.
///
/// The unlock predicate is evaluated by the caller (mirroring
/// `achievements[index].unlockCondition()` in the legacy body). The quark
/// reward, notification, and DOM refresh are UI-tier and deferred.
fn award_achievement(
    ach: &mut AchievementsState,
    index: usize,
    unlocked: bool,
    point_value: f64,
) -> bool {
    if !unlocked || ach.achievements[index] != 0 {
        return false;
    }
    ach.achievements[index] = 1;
    ach.achievement_points += point_value;
    true
}

/// Award every row in a building group whose owned threshold `value` meets.
fn award_threshold_group(ach: &mut AchievementsState, value: f64, table: &[ThresholdRow]) {
    for &(index, threshold, point_value) in table {
        award_achievement(ach, index, value >= threshold, point_value);
    }
}

/// Award every row in a point-gain group whose `10^threshold` the `gain`
/// meets, compared in log10 space. A non-positive gain (a zero-gain reset)
/// awards nothing — every row requires `gain >= 1`.
fn award_log10_group(ach: &mut AchievementsState, gain: Decimal, table: &[Log10Row]) {
    if gain <= Decimal::zero() {
        return;
    }
    let log10 = gain.log10().to_number();
    for &(index, threshold, point_value) in table {
        award_achievement(ach, index, log10 >= threshold, point_value);
    }
}

/// `buildingAchievementCheck()` — award the five `*OwnedCoin` groups from the
/// owned coin-producer counts (`coin_owned[0..5]` = first..fifth owned coin).
/// Called after a coin-producer buy.
pub fn building_achievement_check(ach: &mut AchievementsState, coin_owned: &[f64; 5]) {
    award_threshold_group(ach, coin_owned[0], FIRST_OWNED_COIN);
    award_threshold_group(ach, coin_owned[1], SECOND_OWNED_COIN);
    award_threshold_group(ach, coin_owned[2], THIRD_OWNED_COIN);
    award_threshold_group(ach, coin_owned[3], FOURTH_OWNED_COIN);
    award_threshold_group(ach, coin_owned[4], FIFTH_OWNED_COIN);
}

/// `resetAchievementCheck(reset)` — slice-1 subset: the per-tier point-gain
/// group, awarded from the just-computed `gain` (`G.{tier}PointGain`). Must
/// run **before** the reset proper (the legacy trigger calls it ahead of
/// `reset()`), so the offering/obtainium awards inside the reset see the
/// updated `achievement_points`.
///
/// `Ascension` awards no point-gain group (faithful to
/// `resetAchievementCheck('ascension')`). The ungrouped no-* reset
/// achievements are deferred (see module docs).
pub fn reset_achievement_check(ach: &mut AchievementsState, tier: AutoResetTier, gain: Decimal) {
    let table = match tier {
        AutoResetTier::Prestige => PRESTIGE_POINT_GAIN,
        AutoResetTier::Transcension => TRANSCEND_POINT_GAIN,
        AutoResetTier::Reincarnation => REINCARNATION_POINT_GAIN,
        AutoResetTier::Ascension => return,
    };
    award_log10_group(ach, gain, table);
}

/// `challengeAchievementCheck(i)` — award the `challengeI` group from the
/// current `challengecompletions[i]`. Called when challenge `i`'s completion
/// count rises. `challenge` is 1..=14; challenge 15 (only the ungrouped
/// `sadisticAch`) and the per-challenge ungrouped extras (`chalNNoGen`,
/// `diamondSearch`, `extraChallenging`) are deferred to the ungrouped tail.
pub fn challenge_achievement_check(
    ach: &mut AchievementsState,
    challenge: usize,
    challenge_completions: &[f64],
) {
    let table: &[ThresholdRow] = match challenge {
        1 => CHALLENGE_1,
        2 => CHALLENGE_2,
        3 => CHALLENGE_3,
        4 => CHALLENGE_4,
        5 => CHALLENGE_5,
        6 => CHALLENGE_6,
        7 => CHALLENGE_7,
        8 => CHALLENGE_8,
        9 => CHALLENGE_9,
        10 => CHALLENGE_10,
        11 => CHALLENGE_11,
        12 => CHALLENGE_12,
        13 => CHALLENGE_13,
        14 => CHALLENGE_14,
        _ => return,
    };
    award_threshold_group(ach, challenge_completions[challenge], table);
}

/// Apply one progressive-achievement update — a port of `updateProgressiveCache`
/// and `updateProgressiveAP`. Bumps the cached value via `Math.max(cached,
/// live_value)`, recomputes its awarded points via `points_of(cached_value)`,
/// and folds the change into `achievement_points` by the delta (so repeated
/// calls converge instead of double-counting).
///
/// For the legacy `useCachedValue: false` entries — whose `pointsAwarded`
/// reads live state rather than the cached value — the caller passes a
/// `points_of` closure that ignores its argument and returns points computed
/// from live state; the cache is still `Math.max`'d for faithfulness.
pub(crate) fn update_progressive_slot(
    ach: &mut AchievementsState,
    slot: usize,
    live_value: f64,
    points_of: impl Fn(f64) -> f64,
) {
    let new_value = ach.progressive[slot].cached_value.max(live_value);
    ach.progressive[slot].cached_value = new_value;
    let points = points_of(new_value);
    let delta = points - ach.progressive[slot].cached_points;
    ach.progressive[slot].cached_points = points;
    ach.achievement_points += delta;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn award_achievement_sets_bit_and_adds_points_once() {
        let mut ach = AchievementsState::default();
        assert!(award_achievement(&mut ach, 3, true, 15.0));
        assert_eq!(ach.achievements[3], 1);
        assert_eq!(ach.achievement_points, 15.0);
        // Idempotent: a second award is a no-op.
        assert!(!award_achievement(&mut ach, 3, true, 15.0));
        assert_eq!(ach.achievement_points, 15.0);
        // Unmet condition: no award.
        assert!(!award_achievement(&mut ach, 4, false, 20.0));
        assert_eq!(ach.achievements[4], 0);
    }

    #[test]
    fn building_check_awards_crossed_thresholds_only() {
        let mut ach = AchievementsState::default();
        // 100 first-owned coins → first group rows >=1, >=10, >=100 (indices
        // 1,2,3; pv 5+10+15 = 30). The >=1000 row (index 4) stays unmet.
        building_achievement_check(&mut ach, &[100.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(ach.achievements[1], 1);
        assert_eq!(ach.achievements[2], 1);
        assert_eq!(ach.achievements[3], 1);
        assert_eq!(ach.achievements[4], 0);
        assert_eq!(ach.achievement_points, 30.0);
        // Other building groups untouched (their owned counts are 0).
        assert_eq!(ach.achievements[8], 0);
    }

    #[test]
    fn building_check_is_monotonic_across_calls() {
        let mut ach = AchievementsState::default();
        building_achievement_check(&mut ach, &[10.0, 0.0, 0.0, 0.0, 0.0]); // 1,2 → 15
        assert_eq!(ach.achievement_points, 15.0);
        building_achievement_check(&mut ach, &[100.0, 0.0, 0.0, 0.0, 0.0]); // +3 (pv15)
        assert_eq!(ach.achievement_points, 30.0);
        assert_eq!(ach.achievements[3], 1);
    }

    #[test]
    fn reset_check_awards_prestige_point_gain_by_magnitude() {
        let mut ach = AchievementsState::default();
        // gain 1e6 → log10 6 → rows at thresholds 0 and 6 (indices 36,37;
        // pv 5+10 = 15). Threshold-100 row (index 38) stays unmet.
        reset_achievement_check(&mut ach, AutoResetTier::Prestige, Decimal::from_finite(1e6));
        assert_eq!(ach.achievements[36], 1);
        assert_eq!(ach.achievements[37], 1);
        assert_eq!(ach.achievements[38], 0);
        assert_eq!(ach.achievement_points, 15.0);
    }

    #[test]
    fn reset_check_zero_gain_awards_nothing() {
        let mut ach = AchievementsState::default();
        reset_achievement_check(&mut ach, AutoResetTier::Prestige, Decimal::zero());
        assert_eq!(ach.achievement_points, 0.0);
        assert_eq!(ach.achievements[36], 0);
    }

    #[test]
    fn reset_check_ascension_is_noop() {
        let mut ach = AchievementsState::default();
        reset_achievement_check(
            &mut ach,
            AutoResetTier::Ascension,
            Decimal::from_finite(1e9),
        );
        assert_eq!(ach.achievement_points, 0.0);
    }

    #[test]
    fn reset_check_selects_tier_table() {
        let mut ach = AchievementsState::default();
        // Reincarnation gain 1e5 → log10 5 → rows at thresholds 0 and 5
        // (indices 50,51; pv 5+10 = 15). The 1e30 row (index 52) stays unmet.
        reset_achievement_check(
            &mut ach,
            AutoResetTier::Reincarnation,
            Decimal::from_finite(1e5),
        );
        assert_eq!(ach.achievements[50], 1);
        assert_eq!(ach.achievements[51], 1);
        assert_eq!(ach.achievements[52], 0);
        assert_eq!(ach.achievement_points, 15.0);
    }

    #[test]
    fn challenge_check_awards_group_by_completions() {
        let mut ach = AchievementsState::default();
        let mut completions = [0.0_f64; 16];
        completions[5] = 5.0; // challenge 5 completed 5 times
        challenge_achievement_check(&mut ach, 5, &completions);
        // challenge5 rows >=1, >=3, >=5 met (indices 106,107,108; pv 5+10+15);
        // the >=10 row (index 109) stays unmet.
        assert_eq!(ach.achievements[106], 1);
        assert_eq!(ach.achievements[107], 1);
        assert_eq!(ach.achievements[108], 1);
        assert_eq!(ach.achievements[109], 0);
        assert_eq!(ach.achievement_points, 30.0);
        // Only challenge 5's group was touched.
        assert_eq!(ach.achievements[78], 0); // challenge1
    }

    #[test]
    fn challenge_check_ascension_group_uses_high_point_values() {
        let mut ach = AchievementsState::default();
        let mut completions = [0.0_f64; 16];
        completions[11] = 3.0; // ascension challenge 11 completed 3 times
        challenge_achievement_check(&mut ach, 11, &completions);
        // challenge11 rows >=1, >=2, >=3 met (indices 197,198,199; pv 10+20+30).
        assert_eq!(ach.achievements[197], 1);
        assert_eq!(ach.achievements[199], 1);
        assert_eq!(ach.achievements[200], 0); // >=5 unmet
        assert_eq!(ach.achievement_points, 60.0);
    }

    #[test]
    fn challenge_check_15_and_out_of_range_are_noop() {
        let mut ach = AchievementsState::default();
        let completions = [9999.0_f64; 16];
        challenge_achievement_check(&mut ach, 15, &completions);
        challenge_achievement_check(&mut ach, 0, &completions);
        assert_eq!(ach.achievement_points, 0.0);
    }

    #[test]
    fn progressive_slot_caches_max_and_folds_point_delta() {
        use crate::mechanics::achievement_points::rune_level_points;
        let mut ach = AchievementsState::default();
        // 1500 → rune_level_points = floor(1500/1000) = 1.
        update_progressive_slot(&mut ach, 0, 1500.0, rune_level_points);
        assert_eq!(ach.progressive[0].cached_value, 1500.0);
        assert_eq!(ach.progressive[0].cached_points, 1.0);
        assert_eq!(ach.achievement_points, 1.0);
        // A lower live value cannot decrease the cache (Math.max) → no delta.
        update_progressive_slot(&mut ach, 0, 500.0, rune_level_points);
        assert_eq!(ach.progressive[0].cached_value, 1500.0);
        assert_eq!(ach.achievement_points, 1.0);
        // A higher value raises both: 2500 → floor(2500/1000) + floor(2500/2500) = 3.
        update_progressive_slot(&mut ach, 0, 2500.0, rune_level_points);
        assert_eq!(ach.progressive[0].cached_value, 2500.0);
        assert_eq!(ach.achievement_points, 3.0);
    }

    #[test]
    fn progressive_slot_live_points_ignore_cached_value() {
        // The `useCachedValue: false` shape: points come from a precomputed
        // live value, not the cached one. The cache still tracks the max.
        let mut ach = AchievementsState::default();
        update_progressive_slot(&mut ach, 4, 7.0, |_| 42.0);
        assert_eq!(ach.progressive[4].cached_value, 7.0);
        assert_eq!(ach.progressive[4].cached_points, 42.0);
        assert_eq!(ach.achievement_points, 42.0);
    }
}
