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
//! ## Implemented since slice 1
//!
//! - `challengeAchievementCheck` (the 14 challenge groups) — slice 2;
//!   progressive achievements (the `Math.max` cache) — slice 3a; the
//!   `sacMult` / `sacCount` and ungrouped (`seeingRed`, `oneCubeOfMany`)
//!   sacrifice & cube achievements.
//! - The per-achievement quark reward (`player.worlds.add(getAchievementQuarks())`):
//!   `award_achievement` returns whether it newly awarded, the check helpers sum
//!   that into a count, and each call site credits it via
//!   `credit_achievement_quarks`.
//!
//! ## Implemented since slice 1 (continued)
//!
//! - The ungrouped no-reset achievements (indices 57–62, 64–70) — awarded by
//!   [`reset_achievement_check`] from the per-run "didn't buy X" flags
//!   ([`ResetNoBuyFlags`]). Those flags are already plumbed false-on-buy /
//!   true-on-reset across the multiplier/accelerator/upgrade slices, so a flag
//!   still `true` at the (pre-reset) check means the run avoided that purchase.
//!
//! ## Still deferred (faithful: those bits stay `0`)
//!
//! - The per-challenge ungrouped extras (`chalNNoGen`, `diamondSearch` #63,
//!   `extraChallenging` #247, `sadisticAch` #252) — the ungrouped tail of
//!   `challengeAchievementCheck`.
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
/// Returns the count of achievements newly awarded (for the per-achievement
/// quark reward).
fn award_threshold_group(ach: &mut AchievementsState, value: f64, table: &[ThresholdRow]) -> usize {
    let mut awarded = 0;
    for &(index, threshold, point_value) in table {
        if award_achievement(ach, index, value >= threshold, point_value) {
            awarded += 1;
        }
    }
    awarded
}

/// Award every row in a point-gain group whose `10^threshold` the `gain`
/// meets, compared in log10 space. A non-positive gain (a zero-gain reset)
/// awards nothing — every row requires `gain >= 1`. Returns the count newly
/// awarded.
fn award_log10_group(ach: &mut AchievementsState, gain: Decimal, table: &[Log10Row]) -> usize {
    if gain <= Decimal::zero() {
        return 0;
    }
    let log10 = gain.log10().to_number();
    let mut awarded = 0;
    for &(index, threshold, point_value) in table {
        if award_achievement(ach, index, log10 >= threshold, point_value) {
            awarded += 1;
        }
    }
    awarded
}

/// `buildingAchievementCheck()` — award the five `*OwnedCoin` groups from the
/// owned coin-producer counts (`coin_owned[0..5]` = first..fifth owned coin).
/// Called after a coin-producer buy.
pub fn building_achievement_check(ach: &mut AchievementsState, coin_owned: &[f64; 5]) -> usize {
    award_threshold_group(ach, coin_owned[0], FIRST_OWNED_COIN)
        + award_threshold_group(ach, coin_owned[1], SECOND_OWNED_COIN)
        + award_threshold_group(ach, coin_owned[2], THIRD_OWNED_COIN)
        + award_threshold_group(ach, coin_owned[3], FOURTH_OWNED_COIN)
        + award_threshold_group(ach, coin_owned[4], FIFTH_OWNED_COIN)
}

// ─── Reset-count groups (ascension / prestige / transcend / reincarnation) ──
//
// Awarded from the lifetime reset counts. Thresholds + pointValues are verbatim
// from `legacy/original/src/Achievements.ts` (extracted in array order; the
// 0-indexed positions align with the achievement bitmap). The counts only grow,
// so a per-tick sweep awards each row the tick its threshold is first crossed.

/// `ascensionCount` group.
const ASCENSION_COUNT: &[ThresholdRow] = &[
    (183, 1.0, 5.0),
    (184, 2.0, 10.0),
    (185, 10.0, 15.0),
    (186, 100.0, 20.0),
    (187, 1_000.0, 25.0),
    (188, 14_142.0, 30.0),
    (189, 141_421.0, 35.0),
    (260, 1e7, 40.0),
    (261, 1e8, 45.0),
    (262, 2e9, 50.0),
    (263, 4e10, 55.0),
    (264, 8e11, 60.0),
    (265, 1.6e13, 65.0),
    (266, 1e14, 70.0),
    (350, 1e16, 75.0),
    (351, 1e20, 80.0),
    (352, 1e25, 85.0),
    (353, 1e35, 90.0),
    (354, 1e50, 95.0),
    (355, 1e75, 100.0),
];

/// `prestigeCount` group.
const PRESTIGE_COUNT: &[ThresholdRow] = &[
    (436, 1.0, 2.0),
    (437, 10.0, 4.0),
    (438, 100.0, 6.0),
    (439, 1_000.0, 8.0),
    (440, 10_000.0, 10.0),
    (441, 100_000.0, 12.0),
    (442, 1e6, 14.0),
    (443, 1e7, 16.0),
    (444, 1e8, 18.0),
    (445, 1e9, 20.0),
    (446, 1e11, 22.0),
    (447, 1e13, 24.0),
    (448, 1e15, 26.0),
    (449, 1e17, 28.0),
    (450, 1e20, 30.0),
];

/// `transcensionCount` group.
const TRANSCENSION_COUNT: &[ThresholdRow] = &[
    (451, 1.0, 3.0),
    (452, 10.0, 6.0),
    (453, 100.0, 9.0),
    (454, 1_000.0, 12.0),
    (455, 10_000.0, 15.0),
    (456, 100_000.0, 18.0),
    (457, 1e6, 21.0),
    (458, 1e7, 24.0),
    (459, 1e8, 27.0),
    (460, 1e9, 30.0),
    (461, 3e10, 33.0),
    (462, 9e11, 36.0),
    (463, 2.7e13, 39.0),
    (464, 8.1e14, 42.0),
    (465, 1e17, 45.0),
];

/// `reincarnationCount` group.
const REINCARNATION_COUNT: &[ThresholdRow] = &[
    (466, 1.0, 4.0),
    (467, 10.0, 8.0),
    (468, 100.0, 12.0),
    (469, 1_000.0, 16.0),
    (470, 10_000.0, 20.0),
    (471, 100_000.0, 24.0),
    (472, 1e6, 28.0),
    (473, 1e7, 32.0),
    (474, 1e8, 36.0),
    (475, 1e9, 40.0),
    (476, 8e9, 44.0),
    (477, 1e11, 48.0),
    (478, 1e12, 52.0),
    (479, 13_131_313_131_313.0, 56.0),
    (480, 2e14, 60.0),
];

/// The four reset-count achievement groups (`ascensionCount`, `prestigeCount`,
/// `transcensionCount`, `reincarnationCount`) — awarded from the lifetime reset
/// counters. Returns the count newly awarded (for the per-achievement quark
/// reward). Safe to call every tick: the counts are monotonic and
/// [`award_achievement`] is idempotent.
pub fn reset_count_achievement_check(
    ach: &mut AchievementsState,
    prestige_count: f64,
    transcend_count: f64,
    reincarnation_count: f64,
    ascension_count: f64,
) -> usize {
    award_threshold_group(ach, prestige_count, PRESTIGE_COUNT)
        + award_threshold_group(ach, transcend_count, TRANSCENSION_COUNT)
        + award_threshold_group(ach, reincarnation_count, REINCARNATION_COUNT)
        + award_threshold_group(ach, ascension_count, ASCENSION_COUNT)
}

// ─── Accelerator / multiplier / boost groups (lifetime bought counts) ───────

/// `accelerators` group — `player.acceleratorBought`.
const ACCELERATORS: &[ThresholdRow] = &[
    (148, 5.0, 5.0),
    (149, 25.0, 10.0),
    (150, 100.0, 15.0),
    (151, 666.0, 20.0),
    (152, 2_000.0, 25.0),
    (153, 12_500.0, 30.0),
    (154, 100_000.0, 35.0),
    (335, 1e6, 40.0),
    (336, 1e7, 45.0),
    (337, 1e8, 50.0),
];

/// `multipliers` group — `player.multiplierBought`.
const MULTIPLIERS: &[ThresholdRow] = &[
    (155, 2.0, 5.0),
    (156, 20.0, 10.0),
    (157, 100.0, 15.0),
    (158, 500.0, 20.0),
    (159, 2_000.0, 25.0),
    (160, 12_500.0, 30.0),
    (161, 100_000.0, 35.0),
    (338, 3e6, 40.0),
    (339, 3e7, 45.0),
    (340, 3e8, 50.0),
];

/// `acceleratorBoosts` group — `player.acceleratorBoostBought`.
const ACCELERATOR_BOOSTS: &[ThresholdRow] = &[
    (162, 2.0, 5.0),
    (163, 10.0, 10.0),
    (164, 50.0, 15.0),
    (165, 200.0, 20.0),
    (166, 1_000.0, 25.0),
    (167, 5_000.0, 30.0),
    (168, 15_000.0, 35.0),
    (341, 1e5, 40.0),
    (342, 1e6, 45.0),
    (343, 1e7, 50.0),
];

/// The accelerator / multiplier / accelerator-boost achievement groups —
/// awarded from the lifetime bought counts. Returns the count newly awarded.
pub fn accelerator_achievement_check(
    ach: &mut AchievementsState,
    accelerator_bought: f64,
    multiplier_bought: f64,
    accelerator_boost_bought: f64,
) -> usize {
    award_threshold_group(ach, accelerator_bought, ACCELERATORS)
        + award_threshold_group(ach, multiplier_bought, MULTIPLIERS)
        + award_threshold_group(ach, accelerator_boost_bought, ACCELERATOR_BOOSTS)
}

// ─── Speed-rune groups (level / free levels / blessing / spirit) ────────────

/// `runeLevel` group — `runes.speed.level`.
const RUNE_LEVEL: &[ThresholdRow] = &[
    (396, 100.0, 2.0),
    (397, 250.0, 4.0),
    (398, 500.0, 6.0),
    (399, 1_000.0, 8.0),
    (400, 2_000.0, 10.0),
    (401, 5_000.0, 12.0),
    (402, 10_000.0, 14.0),
    (403, 20_000.0, 16.0),
    (404, 50_000.0, 18.0),
    (405, 100_000.0, 20.0),
    (406, 200_000.0, 22.0),
    (407, 300_000.0, 24.0),
    (408, 500_000.0, 26.0),
    (409, 750_000.0, 28.0),
    (410, 1_000_000.0, 30.0),
];

/// `runeFreeLevel` group — `runes.speed.freeLevels()`.
const RUNE_FREE_LEVEL: &[ThresholdRow] = &[
    (411, 10.0, 2.0),
    (412, 40.0, 4.0),
    (413, 125.0, 6.0),
    (414, 250.0, 8.0),
    (415, 500.0, 10.0),
    (416, 1_000.0, 12.0),
    (417, 2_000.0, 14.0),
    (418, 4_000.0, 16.0),
    (419, 7_500.0, 18.0),
    (420, 12_500.0, 20.0),
    (421, 25_000.0, 22.0),
    (422, 37_500.0, 24.0),
    (423, 50_000.0, 26.0),
    (424, 75_000.0, 28.0),
    (425, 100_000.0, 30.0),
];

/// `speedBlessing` group — `runeBlessings.speed.level`.
const SPEED_BLESSING: &[ThresholdRow] = &[
    (232, 20.0, 10.0),
    (233, 40.0, 20.0),
    (234, 80.0, 30.0),
    (382, 200.0, 40.0),
    (383, 400.0, 50.0),
    (384, 800.0, 60.0),
    (385, 1_000.0, 70.0),
    (386, 1_200.0, 80.0),
    (387, 1_500.0, 90.0),
    (388, 2_000.0, 100.0),
];

/// `speedSpirit` group — `runeSpirits.speed.level`.
const SPEED_SPIRIT: &[ThresholdRow] = &[
    (235, 20.0, 10.0),
    (236, 40.0, 20.0),
    (237, 80.0, 30.0),
    (389, 160.0, 40.0),
    (390, 320.0, 50.0),
    (391, 640.0, 60.0),
    (392, 960.0, 70.0),
    (393, 1_280.0, 80.0),
    (394, 1_600.0, 90.0),
    (395, 2_000.0, 100.0),
];

/// The four speed-rune achievement groups (`runeLevel`, `runeFreeLevel`,
/// `speedBlessing`, `speedSpirit`) — all gated on the **speed** rune's
/// level / free-levels / blessing-level / spirit-level. Returns the count
/// newly awarded.
pub fn rune_achievement_check(
    ach: &mut AchievementsState,
    speed_rune_level: f64,
    speed_rune_free_level: f64,
    speed_blessing_level: f64,
    speed_spirit_level: f64,
) -> usize {
    award_threshold_group(ach, speed_rune_level, RUNE_LEVEL)
        + award_threshold_group(ach, speed_rune_free_level, RUNE_FREE_LEVEL)
        + award_threshold_group(ach, speed_blessing_level, SPEED_BLESSING)
        + award_threshold_group(ach, speed_spirit_level, SPEED_SPIRIT)
}

// ─── Decimal-currency groups (constant = ascendShards, antCrumbs) ───────────
//
// Compared in log10 space — the gates run past f64 (`ascendShards.gte('1e1e8')`,
// `crumbs.gte('1e1000000')`). The threshold column is `log10(gate)` at full f64
// precision (so a fractional gate like `3.14` keeps its exact boundary).

/// `constant` group — `player.ascendShards`.
const CONSTANT: &[Log10Row] = &[
    (190, 0.496_929_648_073_214_94, 5.0), // gate 3.14
    (191, 6.0, 10.0),                     // gate 1e6
    (192, 10.635_483_746_814_913, 15.0),  // gate 4.32e10
    (193, 21.838_849_090_737_256, 20.0),  // gate 6.9e21
    (194, 33.178_689_239_775_586, 25.0),  // gate 1.509e33
    (195, 66.0, 30.0),                    // gate 1e66
    (196, 308.255_272_505_103_3, 35.0),   // gate 1.8e308
    (267, 1_000.0, 40.0),                 // gate 1e1000
    (268, 5_000.0, 45.0),                 // gate 1e5000
    (269, 15_000.0, 50.0),                // gate 1e15000
    (270, 50_000.0, 55.0),                // gate 1e50000
    (271, 100_000.0, 60.0),               // gate 1e100000
    (272, 300_000.0, 65.0),               // gate 1e300000
    (273, 1_000_000.0, 70.0),             // gate 1e1000000
    (356, 2_000_000.0, 75.0),             // gate 1e2000000
    (357, 5_000_000.0, 80.0),             // gate 1e5000000
    (358, 10_000_000.0, 85.0),            // gate 1e10000000
    (359, 25_000_000.0, 90.0),            // gate 1e25000000
    (360, 50_000_000.0, 95.0),            // gate 1e50000000
    (361, 100_000_000.0, 100.0),          // gate 1e100000000
];

/// `antCrumbs` group — `player.ants.crumbs`.
const ANT_CRUMBS: &[Log10Row] = &[
    (169, 0.477_121_254_719_662_44, 5.0), // gate 3
    (170, 5.0, 10.0),                     // gate 1e5
    (171, 8.823_908_740_510_024, 15.0),   // gate 666666666
    (172, 20.0, 20.0),                    // gate 1e20
    (173, 40.0, 25.0),                    // gate 1e40
    (174, 250.0, 30.0),                   // gate 1e250
    (175, 2_500.0, 35.0),                 // gate 1e2500
    (344, 25_000.0, 40.0),                // gate 1e25000
    (345, 125_000.0, 45.0),               // gate 1e125000
    (346, 1_000_000.0, 50.0),             // gate 1e1000000
];

/// The two Decimal-currency achievement groups (`constant` = `ascendShards`,
/// `antCrumbs` = ant crumbs) — compared in log10 space. Returns the count newly
/// awarded.
pub fn decimal_currency_achievement_check(
    ach: &mut AchievementsState,
    ascend_shards: Decimal,
    crumbs: Decimal,
) -> usize {
    award_log10_group(ach, ascend_shards, CONSTANT) + award_log10_group(ach, crumbs, ANT_CRUMBS)
}

// ─── Ascension-score group ──────────────────────────────────────────────────

/// `ascensionScore` group — `CalcCorruptionStuff().effectiveScore`. The score
/// is `f64` (softcapped at `1e23`), so the gates are compared directly.
const ASCENSION_SCORE: &[ThresholdRow] = &[
    (225, 1e5, 5.0),
    (226, 1e6, 10.0),
    (227, 1e7, 15.0),
    (228, 1e8, 20.0),
    (229, 1e9, 25.0),
    (230, 5e9, 30.0),
    (231, 2.5e10, 35.0),
    (253, 1e12, 40.0),
    (254, 1e14, 45.0),
    (255, 1e17, 50.0),
    (256, 2e18, 55.0),
    (257, 4e19, 60.0),
    (258, 1e21, 65.0),
    (259, 1e23, 70.0),
];

/// `ascensionScore` group — awarded from the effective ascension score. The
/// caller supplies the score (and may skip the score computation while
/// ascension is locked, where it stays below `1e5`). Returns the count newly
/// awarded.
pub fn ascension_score_achievement_check(
    ach: &mut AchievementsState,
    effective_score: f64,
) -> usize {
    award_threshold_group(ach, effective_score, ASCENSION_SCORE)
}

// ─── Singularity-count group ────────────────────────────────────────────────

/// `singularityCount` group — `player.highestSingularityCount`. Pure
/// point-value achievements (no rewards). The seven indices sit between the
/// `constant` group (…273) and the `firstOwnedCoin` second tier (281…). Now
/// reachable: the singularity layer is live, so `highestSingularityCount`
/// climbs in play.
const SINGULARITY_COUNT: &[ThresholdRow] = &[
    (274, 1.0, 10.0),
    (275, 2.0, 20.0),
    (276, 3.0, 30.0),
    (277, 4.0, 40.0),
    (278, 5.0, 50.0),
    (279, 7.0, 60.0),
    (280, 10.0, 70.0),
];

/// `singularityCount` achievement group — awarded from the highest singularity
/// count reached. Returns the count newly awarded.
pub fn singularity_achievement_check(
    ach: &mut AchievementsState,
    highest_singularity_count: f64,
) -> usize {
    award_threshold_group(ach, highest_singularity_count, SINGULARITY_COUNT)
}

// ─── Campaign-token group ───────────────────────────────────────────────────

/// `campaignTokens` group — the derived campaign-token total (the legacy
/// `updateTokens()` awards this group right after recomputing the total).
const CAMPAIGN_TOKENS: &[ThresholdRow] = &[
    (426, 10.0, 5.0),
    (427, 20.0, 10.0),
    (428, 40.0, 15.0),
    (429, 80.0, 20.0),
    (430, 160.0, 25.0),
    (431, 320.0, 30.0),
    (432, 1_000.0, 35.0),
    (433, 2_000.0, 40.0),
    (434, 4_000.0, 45.0),
    (435, 9_000.0, 50.0),
];

/// `campaignTokens` achievement group — awarded from the campaign-token
/// total (`compute_campaign_tokens`). Returns the count newly awarded.
pub fn campaign_tokens_achievement_check(
    ach: &mut AchievementsState,
    campaign_tokens: f64,
) -> usize {
    award_threshold_group(ach, campaign_tokens, CAMPAIGN_TOKENS)
}

/// Per-run "didn't buy X this run" flags read by the ungrouped no-reset
/// achievements (the `awardUngroupedAchievement` calls in the legacy
/// `resetAchievementCheck`). Each starts `true`, is cleared on the matching
/// purchase, and restored on the matching reset — so a flag still `true` at the
/// check (which runs *before* the reset body) means the run reached this reset
/// without that purchase. The caller fills the current tier's tier-prefixed
/// flags; cross-tier fields stay `false` (each tier branch reads only its own).
/// Synergism naming: "diamond" = the prestige currency, "mythos" = the
/// transcend currency.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResetNoBuyFlags {
    /// `{tier}nomultiplier`.
    pub no_multiplier: bool,
    /// `{tier}noaccelerator`.
    pub no_accelerator: bool,
    /// `{tier}nocoinupgrades`.
    pub no_coin_upgrades: bool,
    /// `{tier}nocoinorprestigeupgrades` — "no coin or diamond upgrade"
    /// (transcend #66, reincarnate #68).
    pub no_coin_or_prestige_upgrades: bool,
    /// `reincarnatenocoinprestigeortranscendupgrades` — "no coin/diamond/mythos
    /// upgrade" (#69).
    pub no_coin_prestige_or_transcend_upgrades: bool,
    /// `reincarnatenocoinprestigetranscendorgeneratorupgrades` — the "minimum
    /// upgrades" achievement (#70).
    pub no_coin_prestige_transcend_or_generator_upgrades: bool,
}

/// The ungrouped no-reset achievements awarded by `resetAchievementCheck`'s
/// `awardUngroupedAchievement` calls, gated on the per-run [`ResetNoBuyFlags`].
/// Each `(index, pointValue)` is taken verbatim from the legacy `achievements`
/// array, where the unlock condition is exactly the matching flag. Returns the
/// count newly awarded.
fn award_no_buy_achievements(
    ach: &mut AchievementsState,
    tier: AutoResetTier,
    f: &ResetNoBuyFlags,
) -> usize {
    match tier {
        AutoResetTier::Prestige => {
            usize::from(award_achievement(ach, 57, f.no_multiplier, 5.0))
                + usize::from(award_achievement(ach, 60, f.no_accelerator, 20.0))
                + usize::from(award_achievement(ach, 64, f.no_coin_upgrades, 5.0))
        }
        AutoResetTier::Transcension => {
            usize::from(award_achievement(ach, 58, f.no_multiplier, 10.0))
                + usize::from(award_achievement(ach, 61, f.no_accelerator, 25.0))
                + usize::from(award_achievement(ach, 65, f.no_coin_upgrades, 10.0))
                + usize::from(award_achievement(
                    ach,
                    66,
                    f.no_coin_or_prestige_upgrades,
                    15.0,
                ))
        }
        AutoResetTier::Reincarnation => {
            usize::from(award_achievement(ach, 59, f.no_multiplier, 15.0))
                + usize::from(award_achievement(ach, 62, f.no_accelerator, 30.0))
                + usize::from(award_achievement(ach, 67, f.no_coin_upgrades, 15.0))
                + usize::from(award_achievement(
                    ach,
                    68,
                    f.no_coin_or_prestige_upgrades,
                    20.0,
                ))
                + usize::from(award_achievement(
                    ach,
                    69,
                    f.no_coin_prestige_or_transcend_upgrades,
                    30.0,
                ))
                + usize::from(award_achievement(
                    ach,
                    70,
                    f.no_coin_prestige_transcend_or_generator_upgrades,
                    40.0,
                ))
        }
        AutoResetTier::Ascension => 0,
    }
}

/// `resetAchievementCheck(reset)` — the ungrouped no-reset achievements (gated
/// on [`ResetNoBuyFlags`]) plus the per-tier point-gain group (from the
/// just-computed `gain`, `G.{tier}PointGain`). Must run **before** the reset
/// proper (the legacy trigger calls it ahead of `reset()`), so the
/// offering/obtainium awards inside the reset see the updated
/// `achievement_points`.
///
/// `Ascension` awards nothing (faithful to `resetAchievementCheck('ascension')`).
pub fn reset_achievement_check(
    ach: &mut AchievementsState,
    tier: AutoResetTier,
    gain: Decimal,
    no_buy: &ResetNoBuyFlags,
) -> usize {
    let mut awarded = award_no_buy_achievements(ach, tier, no_buy);
    let table = match tier {
        AutoResetTier::Prestige => PRESTIGE_POINT_GAIN,
        AutoResetTier::Transcension => TRANSCEND_POINT_GAIN,
        AutoResetTier::Reincarnation => REINCARNATION_POINT_GAIN,
        AutoResetTier::Ascension => return awarded,
    };
    awarded += award_log10_group(ach, gain, table);
    awarded
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
) -> usize {
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
        _ => return 0,
    };
    award_threshold_group(ach, challenge_completions[challenge], table)
}

/// Per-completion context for the ungrouped per-challenge achievements (the
/// `awardUngroupedAchievement` calls in `challengeAchievementCheck`). All fields
/// are existing state reads; coins are carried as `log10(coins + 1)` because the
/// thresholds (`1e1000`, `1e99999`, `1e120000`) far exceed `f64`.
#[derive(Debug, Clone, Copy)]
pub struct ChallengeUngroupedContext {
    /// `log10(player.coinsThisTranscension + 1)`.
    pub coins_this_transcension_log10: f64,
    /// `player.currentChallenge.transcension`.
    pub current_transcension_challenge: u32,
    /// `sumContents(player.upgrades.slice(101, 106))` — generator upgrades
    /// (101..=105) owned (`0` = none bought this run).
    pub generator_upgrades_owned: u32,
    /// `player.acceleratorBought`.
    pub accelerator_bought: f64,
    /// `player.acceleratorBoostBought`.
    pub accelerator_boost_bought: f64,
    /// `player.corruptions.used.extinction` level.
    pub extinction_level: f64,
}

/// `challengeAchievementCheck`'s ungrouped per-challenge achievements, awarded
/// alongside the `challengeN` group: `chal{1,2,3}NoGen` (reach the coin
/// threshold in that transcension challenge with no generator upgrades),
/// `diamondSearch` (challenge 5: `1e120000` coins, no accelerators/boosts), and
/// `extraChallenging` (challenge 11: `c10 > 50 && extinction >= 5 && c11 >= 20`).
/// `sadisticAch` (challenge 15) is awarded from the c15 path instead. Each
/// `(index, pointValue)` and unlock condition is taken verbatim from the legacy
/// `achievements` array. Returns the count newly awarded.
pub fn challenge_ungrouped_achievement_check(
    ach: &mut AchievementsState,
    challenge: usize,
    challenge_completions: &[f64],
    ctx: &ChallengeUngroupedContext,
) -> usize {
    let tc = ctx.current_transcension_challenge as usize;
    let no_generators = ctx.generator_upgrades_owned == 0;
    match challenge {
        1 => usize::from(award_achievement(
            ach,
            75,
            tc == 1 && ctx.coins_this_transcension_log10 >= 1_000.0 && no_generators,
            25.0,
        )),
        2 => usize::from(award_achievement(
            ach,
            76,
            tc == 2 && ctx.coins_this_transcension_log10 >= 1_000.0 && no_generators,
            25.0,
        )),
        3 => usize::from(award_achievement(
            ach,
            77,
            tc == 3 && ctx.coins_this_transcension_log10 >= 99_999.0 && no_generators,
            50.0,
        )),
        5 => usize::from(award_achievement(
            ach,
            63,
            ctx.coins_this_transcension_log10 >= 120_000.0
                && ctx.accelerator_bought == 0.0
                && ctx.accelerator_boost_bought == 0.0,
            35.0,
        )),
        11 => usize::from(award_achievement(
            ach,
            247,
            challenge_completions[10] > 50.0
                && ctx.extinction_level >= 5.0
                && challenge_completions[11] >= 20.0,
            50.0,
        )),
        _ => 0,
    }
}

/// `sacCount` achievement group — `antSacrificeCount >= threshold`. Awarded
/// after every ant sacrifice (legacy `awardAchievementGroup('sacCount')` inside
/// `resetPlayerAntSacrificeCounts`).
const SAC_COUNT: &[ThresholdRow] = &[
    (481, 1.0, 3.0),
    (482, 10.0, 6.0),
    (483, 50.0, 9.0),
    (484, 250.0, 12.0),
    (485, 1_250.0, 15.0),
    (486, 5_000.0, 17.0),
    (487, 20_000.0, 19.0),
    (488, 80_000.0, 21.0),
    (489, 250_000.0, 23.0),
    (490, 1_000_000.0, 25.0),
    (491, 3_000_000.0, 40.0),
    (492, 10_000_000.0, 45.0),
    (493, 100_000_000.0, 55.0),
];

/// `awardAchievementGroup('sacCount')` — the ant-sacrifice-count milestones.
pub fn sac_count_achievement_check(ach: &mut AchievementsState, ant_sacrifice_count: f64) -> usize {
    award_threshold_group(ach, ant_sacrifice_count, SAC_COUNT)
}

/// `sacMult` achievement group — the compound condition
/// `immortalELO >= elo_req && producers[tier].purchased > 0`; the late #347/#348
/// are immortal-ELO-only (`tier = None`), so [`award_threshold_group`] can't
/// express them. Rows: `(index, elo_req, producer_tier, point_value)`.
const SAC_MULT: &[(usize, f64, Option<usize>, f64)] = &[
    (176, 50.0, Some(1), 5.0),
    (177, 200.0, Some(2), 10.0),
    (178, 500.0, Some(3), 15.0),
    (179, 1_000.0, Some(4), 20.0),
    (180, 2_500.0, Some(5), 25.0),
    (181, 20_000.0, Some(6), 30.0),
    (182, 100_000.0, Some(7), 35.0),
    (347, 400_000.0, None, 40.0),
    (348, 1_500_000.0, None, 45.0),
    (349, 5_000_000.0, Some(8), 50.0),
];

/// `awardAchievementGroup('sacMult')` — awarded after a sacrifice, *before* the
/// ants reset (so the producers are still owned). `producer_owned[t]` is
/// `producers[t].purchased > 0` for tier `t` (Workers=0 .. HolySpirit=8).
pub fn sac_mult_achievement_check(
    ach: &mut AchievementsState,
    immortal_elo: f64,
    producer_owned: &[bool; 9],
) -> usize {
    let mut awarded = 0;
    for &(index, elo_req, tier, point_value) in SAC_MULT {
        let owned = tier.is_none_or(|t| producer_owned[t]);
        if award_achievement(ach, index, immortal_elo >= elo_req && owned, point_value) {
            awarded += 1;
        }
    }
    awarded
}

/// The two ungrouped mythical-fragment achievements checked after a sacrifice:
/// `seeingRed` (#239, `mythicalFragments >= 1e25`) and `seeingRedNoBlue` (#248,
/// `mythicalFragments >= 1e11` inside ascension challenge 14), each worth 50
/// points. Mirrors the trailing `awardUngroupedAchievement` calls in
/// `sacrificeAnts` (`awardUngroupedAchievement(name)` →
/// `awardAchievement(achievementID)`).
pub fn ant_sacrifice_fragment_achievement_check(
    ach: &mut AchievementsState,
    mythical_fragments: f64,
    in_ascension_challenge_14: bool,
) -> usize {
    const SEEING_RED: usize = 239;
    const SEEING_RED_NO_BLUE: usize = 248;
    usize::from(award_achievement(
        ach,
        SEEING_RED,
        mythical_fragments >= 1e25,
        50.0,
    )) + usize::from(award_achievement(
        ach,
        SEEING_RED_NO_BLUE,
        mythical_fragments >= 1e11 && in_ascension_challenge_14,
        50.0,
    ))
}

/// Award a single ungrouped achievement by index when `unlocked` and not
/// already owned — the generic `awardUngroupedAchievement` →
/// `awardAchievement(index)` path. `point_value` is that achievement's
/// `pointValue`. (e.g. `oneCubeOfMany` #246, awarded on opening a single cube
/// at high accelerator blessing.)
pub fn award_ungrouped_achievement(
    ach: &mut AchievementsState,
    index: usize,
    point_value: f64,
    unlocked: bool,
) -> usize {
    usize::from(award_achievement(ach, index, unlocked, point_value))
}

/// `getAchievementQuarks()` — quarks granted per achievement award:
/// `floor(5 × applyBonus(1))`, where `applyBonus(1) = 1 + quark_bonus / 100`
/// (the cached quark multiplier), soft-capped at `100^0.6 × m^0.4` once it
/// exceeds 100.
#[must_use]
pub fn get_achievement_quarks(quark_bonus: f64) -> f64 {
    let mut multiplier = 1.0 + quark_bonus / 100.0;
    if multiplier > 100.0 {
        multiplier = 100.0_f64.powf(0.6) * multiplier.powf(0.4);
    }
    (5.0 * multiplier).floor()
}

/// Credit the per-achievement quark reward (legacy `awardAchievement`'s
/// `player.worlds.add(getAchievementQuarks(), false, true)`): `count`
/// newly-awarded achievements each grant `get_achievement_quarks(quark_bonus)`,
/// credited to `worlds` + `quarks_this_singularity`. No `QuarksAwarded` event —
/// the achievement awards already signal the gain to the UI.
pub fn credit_achievement_quarks(
    worlds: &mut Decimal,
    quarks_this_singularity: &mut f64,
    quark_bonus: f64,
    count: usize,
) {
    if count == 0 {
        return;
    }
    let total = get_achievement_quarks(quark_bonus) * count as f64;
    *worlds += Decimal::from_finite(total);
    *quarks_this_singularity += total;
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
    fn reset_count_check_awards_by_threshold_and_is_idempotent() {
        let mut ach = AchievementsState::default();
        // prestige_count 150 → prestigeCount idx 436/437/438 (>=1/10/100); 439 (>=1000) not.
        let awarded = reset_count_achievement_check(&mut ach, 150.0, 0.0, 0.0, 0.0);
        assert_eq!(awarded, 3);
        assert_eq!(ach.achievements[436], 1);
        assert_eq!(ach.achievements[438], 1);
        assert_eq!(ach.achievements[439], 0);
        assert_eq!(ach.achievement_points, 2.0 + 4.0 + 6.0);
        // Re-running at the same counts awards nothing more.
        assert_eq!(
            reset_count_achievement_check(&mut ach, 150.0, 0.0, 0.0, 0.0),
            0
        );
    }

    #[test]
    fn reset_count_check_ascension_group_thresholds() {
        let mut ach = AchievementsState::default();
        // ascension_count 5 → ascensionCount idx 183 (>=1) + 184 (>=2); 185 (>=10) not.
        let awarded = reset_count_achievement_check(&mut ach, 0.0, 0.0, 0.0, 5.0);
        assert_eq!(awarded, 2);
        assert_eq!(ach.achievements[183], 1);
        assert_eq!(ach.achievements[184], 1);
        assert_eq!(ach.achievements[185], 0);
    }

    #[test]
    fn accelerator_and_rune_checks_award_by_threshold() {
        let mut ach = AchievementsState::default();
        // accel 30 → accelerators 5/25 (148/149); mult 2 → 155; boost 0 → none.
        assert_eq!(accelerator_achievement_check(&mut ach, 30.0, 2.0, 0.0), 3);
        assert_eq!(ach.achievements[148], 1);
        assert_eq!(ach.achievements[149], 1);
        assert_eq!(ach.achievements[150], 0); // 100 not reached
        assert_eq!(ach.achievements[155], 1);
        // speed rune level 600 → runeLevel 100/250/500 (396/397/398); free 50 →
        // 10/40 (411/412); blessing 25 → 20 (232); spirit 0 → none.
        assert_eq!(rune_achievement_check(&mut ach, 600.0, 50.0, 25.0, 0.0), 6);
        assert_eq!(ach.achievements[398], 1);
        assert_eq!(ach.achievements[399], 0); // 1000 not reached
        assert_eq!(ach.achievements[412], 1);
        assert_eq!(ach.achievements[232], 1);
        assert_eq!(ach.achievements[235], 0); // spirit threshold 20 not met
    }

    #[test]
    fn decimal_currency_check_awards_in_log10_space() {
        let mut ach = AchievementsState::default();
        // ascend_shards 1e7 → constant 3.14 / 1e6 (idx 190/191); crumbs 1e6 →
        // antCrumbs 3 / 1e5 (idx 169/170).
        let awarded = decimal_currency_achievement_check(
            &mut ach,
            Decimal::from_finite(1e7),
            Decimal::from_finite(1e6),
        );
        assert_eq!(awarded, 4);
        assert_eq!(ach.achievements[191], 1); // shards >= 1e6
        assert_eq!(ach.achievements[192], 0); // 4.32e10 not reached
        assert_eq!(ach.achievements[170], 1); // crumbs >= 1e5
        assert_eq!(ach.achievements[171], 0); // 666666666 not reached
                                              // Zero balances award nothing (the log10 guard).
        let mut empty = AchievementsState::default();
        assert_eq!(
            decimal_currency_achievement_check(&mut empty, Decimal::zero(), Decimal::zero()),
            0
        );
    }

    #[test]
    fn ascension_score_check_awards_by_threshold() {
        let mut ach = AchievementsState::default();
        // score 1.5e6 → ascensionScore 1e5 / 1e6 (idx 225/226); 1e7 (227) not.
        assert_eq!(ascension_score_achievement_check(&mut ach, 1.5e6), 2);
        assert_eq!(ach.achievements[225], 1);
        assert_eq!(ach.achievements[226], 1);
        assert_eq!(ach.achievements[227], 0);
        // Below the 1e5 floor → nothing.
        let mut low = AchievementsState::default();
        assert_eq!(ascension_score_achievement_check(&mut low, 9.9e4), 0);
    }

    #[test]
    fn singularity_check_awards_by_highest_count_and_is_idempotent() {
        let mut ach = AchievementsState::default();
        // highestSingularityCount 5 → rows 274..=278 (thresholds 1/2/3/4/5);
        // the threshold-7 row (279) stays unmet.
        assert_eq!(singularity_achievement_check(&mut ach, 5.0), 5);
        assert_eq!(ach.achievements[274], 1);
        assert_eq!(ach.achievements[278], 1);
        assert_eq!(ach.achievements[279], 0);
        // Point values 10+20+30+40+50 = 150.
        assert!((ach.achievement_points - 150.0).abs() < 1e-9);
        // Monotonic + idempotent: re-checking the same count awards nothing.
        assert_eq!(singularity_achievement_check(&mut ach, 5.0), 0);
    }

    #[test]
    fn singularity_check_default_awards_nothing() {
        let mut ach = AchievementsState::default();
        assert_eq!(singularity_achievement_check(&mut ach, 0.0), 0);
        assert_eq!(ach.achievement_points, 0.0);
    }

    #[test]
    fn reset_check_awards_prestige_point_gain_by_magnitude() {
        let mut ach = AchievementsState::default();
        // gain 1e6 → log10 6 → rows at thresholds 0 and 6 (indices 36,37;
        // pv 5+10 = 15). Threshold-100 row (index 38) stays unmet.
        reset_achievement_check(
            &mut ach,
            AutoResetTier::Prestige,
            Decimal::from_finite(1e6),
            &ResetNoBuyFlags::default(),
        );
        assert_eq!(ach.achievements[36], 1);
        assert_eq!(ach.achievements[37], 1);
        assert_eq!(ach.achievements[38], 0);
        assert_eq!(ach.achievement_points, 15.0);
    }

    #[test]
    fn reset_check_zero_gain_awards_nothing() {
        let mut ach = AchievementsState::default();
        reset_achievement_check(
            &mut ach,
            AutoResetTier::Prestige,
            Decimal::zero(),
            &ResetNoBuyFlags::default(),
        );
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
            &ResetNoBuyFlags::default(),
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
            &ResetNoBuyFlags::default(),
        );
        assert_eq!(ach.achievements[50], 1);
        assert_eq!(ach.achievements[51], 1);
        assert_eq!(ach.achievements[52], 0);
        assert_eq!(ach.achievement_points, 15.0);
    }

    #[test]
    fn reset_check_awards_prestige_no_buy_achievements() {
        let mut ach = AchievementsState::default();
        // All three prestige no-buy flags still set, zero gain → only the
        // ungrouped no-reset achievements fire (57 noMult, 60 noAccel, 64 noCoinUpg).
        let flags = ResetNoBuyFlags {
            no_multiplier: true,
            no_accelerator: true,
            no_coin_upgrades: true,
            ..ResetNoBuyFlags::default()
        };
        let awarded =
            reset_achievement_check(&mut ach, AutoResetTier::Prestige, Decimal::zero(), &flags);
        assert_eq!(ach.achievements[57], 1);
        assert_eq!(ach.achievements[60], 1);
        assert_eq!(ach.achievements[64], 1);
        assert_eq!(ach.achievement_points, 30.0); // 5 + 20 + 5
        assert_eq!(awarded, 3);
    }

    #[test]
    fn reset_check_no_buy_flag_false_suppresses_its_achievement() {
        let mut ach = AchievementsState::default();
        // Bought a multiplier this run → flag cleared → #57 not awarded; the
        // other two prestige no-buy achievements still fire.
        let flags = ResetNoBuyFlags {
            no_multiplier: false,
            no_accelerator: true,
            no_coin_upgrades: true,
            ..ResetNoBuyFlags::default()
        };
        reset_achievement_check(&mut ach, AutoResetTier::Prestige, Decimal::zero(), &flags);
        assert_eq!(ach.achievements[57], 0);
        assert_eq!(ach.achievements[60], 1);
        assert_eq!(ach.achievements[64], 1);
        assert_eq!(ach.achievement_points, 25.0); // 20 + 5
    }

    #[test]
    fn reset_check_awards_all_reincarnation_no_buy_achievements() {
        let mut ach = AchievementsState::default();
        let flags = ResetNoBuyFlags {
            no_multiplier: true,
            no_accelerator: true,
            no_coin_upgrades: true,
            no_coin_or_prestige_upgrades: true,
            no_coin_prestige_or_transcend_upgrades: true,
            no_coin_prestige_transcend_or_generator_upgrades: true,
        };
        reset_achievement_check(
            &mut ach,
            AutoResetTier::Reincarnation,
            Decimal::zero(),
            &flags,
        );
        // 59,62,67,68,69,70 → pv 15 + 30 + 15 + 20 + 30 + 40 = 150.
        for idx in [59, 62, 67, 68, 69, 70] {
            assert_eq!(ach.achievements[idx], 1, "achievement {idx} should be set");
        }
        assert_eq!(ach.achievement_points, 150.0);
    }

    #[test]
    fn reset_check_no_buy_and_point_gain_combine_in_one_call() {
        // resetAchievementCheck awards both the ungrouped no-buy achievements
        // and the point-gain group in a single call.
        let mut ach = AchievementsState::default();
        let flags = ResetNoBuyFlags {
            no_multiplier: true,
            no_accelerator: true,
            no_coin_upgrades: true,
            ..ResetNoBuyFlags::default()
        };
        reset_achievement_check(
            &mut ach,
            AutoResetTier::Prestige,
            Decimal::from_finite(1e6),
            &flags,
        );
        // no-buy 57,60,64 (30) + point-gain 36 (>=1) & 37 (>=1e6) (15) = 45.
        assert_eq!(ach.achievement_points, 45.0);
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
    fn challenge_ungrouped_chal_no_gen_needs_no_generators_and_coins() {
        let completions = [0.0_f64; 16];
        let ctx = ChallengeUngroupedContext {
            coins_this_transcension_log10: 1_500.0, // >= 1e1000
            current_transcension_challenge: 1,
            generator_upgrades_owned: 0,
            accelerator_bought: 0.0,
            accelerator_boost_bought: 0.0,
            extinction_level: 0.0,
        };
        let mut ach = AchievementsState::default();
        challenge_ungrouped_achievement_check(&mut ach, 1, &completions, &ctx);
        assert_eq!(ach.achievements[75], 1); // chal1NoGen, pv 25
        assert_eq!(ach.achievement_points, 25.0);

        // Owning even one generator upgrade suppresses it.
        let mut ach2 = AchievementsState::default();
        let ctx2 = ChallengeUngroupedContext {
            generator_upgrades_owned: 1,
            ..ctx
        };
        challenge_ungrouped_achievement_check(&mut ach2, 1, &completions, &ctx2);
        assert_eq!(ach2.achievements[75], 0);
    }

    #[test]
    fn challenge_ungrouped_diamond_search_needs_coins_and_no_accelerators() {
        let completions = [0.0_f64; 16];
        let ctx = ChallengeUngroupedContext {
            coins_this_transcension_log10: 120_000.0, // >= 1e120000
            current_transcension_challenge: 5,
            generator_upgrades_owned: 0,
            accelerator_bought: 0.0,
            accelerator_boost_bought: 0.0,
            extinction_level: 0.0,
        };
        let mut ach = AchievementsState::default();
        challenge_ungrouped_achievement_check(&mut ach, 5, &completions, &ctx);
        assert_eq!(ach.achievements[63], 1); // diamondSearch, pv 35
        assert_eq!(ach.achievement_points, 35.0);

        // Any accelerator bought suppresses it.
        let mut ach2 = AchievementsState::default();
        let ctx2 = ChallengeUngroupedContext {
            accelerator_bought: 1.0,
            ..ctx
        };
        challenge_ungrouped_achievement_check(&mut ach2, 5, &completions, &ctx2);
        assert_eq!(ach2.achievements[63], 0);
    }

    #[test]
    fn challenge_ungrouped_extra_challenging_needs_all_three_conditions() {
        let mut completions = [0.0_f64; 16];
        completions[10] = 51.0; // c10 > 50
        completions[11] = 20.0; // c11 >= 20
        let ctx = ChallengeUngroupedContext {
            coins_this_transcension_log10: 0.0,
            current_transcension_challenge: 0,
            generator_upgrades_owned: 0,
            accelerator_bought: 0.0,
            accelerator_boost_bought: 0.0,
            extinction_level: 5.0, // extinction >= 5
        };
        let mut ach = AchievementsState::default();
        challenge_ungrouped_achievement_check(&mut ach, 11, &completions, &ctx);
        assert_eq!(ach.achievements[247], 1); // extraChallenging, pv 50
        assert_eq!(ach.achievement_points, 50.0);

        // extinction < 5 suppresses it.
        let mut ach2 = AchievementsState::default();
        let ctx2 = ChallengeUngroupedContext {
            extinction_level: 4.0,
            ..ctx
        };
        challenge_ungrouped_achievement_check(&mut ach2, 11, &completions, &ctx2);
        assert_eq!(ach2.achievements[247], 0);
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

    #[test]
    fn sac_count_group_awards_met_thresholds() {
        let mut ach = AchievementsState::default();
        sac_count_achievement_check(&mut ach, 60.0);
        // >=1 (#481,3), >=10 (#482,6), >=50 (#483,9) met; >=250 (#484) not.
        assert_ne!(ach.achievements[481], 0);
        assert_ne!(ach.achievements[483], 0);
        assert_eq!(ach.achievements[484], 0);
        assert_eq!(ach.achievement_points, 18.0); // 3 + 6 + 9
    }

    #[test]
    fn sac_mult_group_requires_both_elo_and_producer() {
        let mut ach = AchievementsState::default();
        // High ELO but no producers owned → the producer-gated tiers stay locked,
        // while the immortal-ELO-only tiers (#347 @400k, #348 @1.5M) can fire.
        sac_mult_achievement_check(&mut ach, 1_000_000.0, &[false; 9]);
        assert_eq!(ach.achievements[176], 0); // needs Breeders
        assert_ne!(ach.achievements[347], 0); // ELO-only, 1M >= 400k
        assert_eq!(ach.achievements[348], 0); // 1M < 1.5M

        let mut owned = [false; 9];
        owned[1] = true; // Breeders
        sac_mult_achievement_check(&mut ach, 1_000_000.0, &owned);
        assert_ne!(ach.achievements[176], 0);
    }

    #[test]
    fn seeing_red_awards_above_fragment_thresholds() {
        let mut ach = AchievementsState::default();
        ant_sacrifice_fragment_achievement_check(&mut ach, 1e25, false);
        assert_ne!(ach.achievements[239], 0); // seeingRed (>= 1e25)
        assert_eq!(ach.achievements[248], 0); // seeingRedNoBlue needs c14

        let mut c14 = AchievementsState::default();
        ant_sacrifice_fragment_achievement_check(&mut c14, 1e11, true);
        assert_ne!(c14.achievements[248], 0); // >= 1e11 inside ascension c14
        assert_eq!(c14.achievements[239], 0); // 1e11 < 1e25
    }

    #[test]
    fn achievement_quarks_default_is_five_with_soft_cap() {
        // quark_bonus 0 → multiplier 1 → floor(5 × 1) = 5.
        assert_eq!(get_achievement_quarks(0.0), 5.0);
        // multiplier exactly 100 (bonus 9900) → no cap → floor(5 × 100) = 500.
        assert_eq!(get_achievement_quarks(9900.0), 500.0);
        // multiplier 200 (> 100) → soft cap 100^0.6 × 200^0.4.
        let expected = (5.0 * (100.0_f64.powf(0.6) * 200.0_f64.powf(0.4))).floor();
        assert_eq!(get_achievement_quarks(19_900.0), expected);
    }

    #[test]
    fn credit_achievement_quarks_scales_with_count() {
        let mut worlds = Decimal::zero();
        let mut quarks_this_singularity = 0.0;
        // 3 achievements × 5 quarks each at the default bonus.
        credit_achievement_quarks(&mut worlds, &mut quarks_this_singularity, 0.0, 3);
        assert_eq!(worlds.to_number(), 15.0);
        assert_eq!(quarks_this_singularity, 15.0);
        // count 0 is a no-op.
        credit_achievement_quarks(&mut worlds, &mut quarks_this_singularity, 0.0, 0);
        assert_eq!(worlds.to_number(), 15.0);
    }
}
