//! Campaign runner data + helpers (`Campaign.ts`).
//!
//! The 50-entry `campaignDatas` const: per-campaign corruption loadouts and
//! unlock requirements, in `campaignDatas` key order (`first` = 0 …
//! `fiftieth` = 49). Token limits / meta flags already live in
//! [`crate::mechanics::campaign_token_rewards`]; i18n names, descriptions
//! and icons are UI-tier and not ported. Completion recording is the
//! verbatim only-increase clamp of the `Campaign.c10Completions` setter
//! (Campaign.ts:523-527).

use crate::mechanics::campaign_token_rewards::CAMPAIGN_TOKEN_LIMITS;
use crate::mechanics::corruptions::corruption_loadout_difficulty_score;
pub use crate::state::campaigns::CAMPAIGNS_LEN;

/// Per-campaign `campaignCorruptions` (Campaign.ts:562-1356), each row in
/// corruption-index order (viscosity = 0, dilation = 1, hyperchallenge = 2,
/// illiteracy = 3, deflation = 4, extinction = 5, drought = 6,
/// recession = 7 — the `*_INDEX` constants in [`crate::state`]). Keys the
/// legacy entry omits are `0` (the `corruptionsSchema` default).
pub const CAMPAIGN_CORRUPTION_LOADOUTS: [[u32; 8]; CAMPAIGNS_LEN] = [
    //  v  dil hyp ill def ext dro rec      cardinal
    [1, 0, 0, 0, 0, 0, 0, 0],         // 1  first
    [0, 0, 0, 0, 0, 0, 1, 0],         // 2  second
    [1, 0, 0, 0, 0, 0, 1, 0],         // 3  third
    [2, 0, 0, 0, 0, 0, 2, 0],         // 4  fourth
    [3, 0, 0, 0, 0, 0, 3, 0],         // 5  fifth
    [4, 0, 0, 0, 0, 0, 4, 0],         // 6  sixth
    [5, 0, 0, 0, 0, 0, 5, 0],         // 7  seventh
    [1, 0, 0, 0, 1, 0, 1, 0],         // 8  eighth
    [3, 0, 0, 0, 1, 0, 1, 0],         // 9  ninth
    [5, 0, 0, 0, 0, 1, 1, 0],         // 10 tenth
    [7, 0, 0, 0, 0, 3, 1, 0],         // 11 eleventh
    [7, 0, 0, 0, 3, 3, 1, 0],         // 12 twelfth
    [7, 0, 0, 0, 3, 3, 3, 0],         // 13 thirteenth
    [7, 0, 0, 0, 3, 3, 5, 0],         // 14 fourteenth
    [7, 0, 0, 0, 3, 3, 7, 0],         // 15 fifteenth
    [9, 0, 0, 1, 0, 0, 4, 1],         // 16 sixteenth
    [9, 0, 0, 3, 0, 0, 4, 1],         // 17 seventeenth
    [9, 0, 0, 3, 1, 3, 4, 1],         // 18 eighteenth
    [9, 0, 0, 6, 1, 3, 4, 1],         // 19 nineteenth
    [9, 0, 0, 9, 1, 3, 4, 1],         // 20 twentieth
    [9, 0, 0, 3, 9, 3, 4, 1],         // 21 twentyFirst
    [9, 0, 0, 6, 9, 3, 6, 1],         // 22 twentySecond
    [9, 0, 0, 9, 9, 3, 6, 1],         // 23 twentyThird
    [9, 0, 0, 9, 9, 3, 9, 1],         // 24 twentyFourth
    [9, 0, 0, 9, 9, 5, 9, 3],         // 25 twentyFifth
    [9, 1, 0, 9, 9, 3, 9, 3],         // 26 twentySixth
    [9, 0, 1, 9, 9, 3, 9, 3],         // 27 twentySeventh
    [9, 1, 1, 9, 9, 3, 9, 3],         // 28 twentyEighth
    [9, 3, 3, 9, 9, 3, 9, 3],         // 29 twentyNinth
    [0, 4, 1, 0, 4, 11, 5, 11],       // 30 thirtieth
    [0, 4, 2, 0, 4, 11, 5, 11],       // 31 thirtyFirst
    [1, 5, 3, 1, 4, 11, 5, 11],       // 32 thirtySecond
    [1, 4, 3, 2, 4, 11, 5, 11],       // 33 thirtyThird
    [1, 4, 5, 2, 4, 11, 9, 11],       // 34 thirtyFourth
    [2, 4, 4, 2, 4, 11, 9, 11],       // 35 thirtyFifth
    [2, 5, 5, 2, 4, 11, 9, 11],       // 36 thirtySixth
    [2, 5, 5, 2, 4, 11, 10, 11],      // 37 thirtySeventh
    [3, 5, 4, 2, 4, 11, 10, 11],      // 38 thirtyEighth
    [0, 6, 9, 2, 4, 11, 10, 11],      // 39 thirtyNinth
    [3, 6, 9, 11, 4, 11, 11, 11],     // 40 fortieth
    [3, 7, 11, 11, 4, 11, 11, 11],    // 41 fortyFirst
    [3, 9, 11, 12, 4, 12, 12, 12],    // 42 fortySecond
    [5, 9, 11, 12, 4, 12, 12, 12],    // 43 fortyThird
    [5, 11, 11, 12, 12, 12, 12, 12],  // 44 fortyFourth
    [5, 11, 8, 13, 13, 12, 13, 13],   // 45 fortyFifth
    [6, 12, 11, 13, 13, 13, 13, 13],  // 46 fortySixth
    [6, 13, 7, 13, 13, 13, 13, 13],   // 47 fortySeventh
    [6, 13, 11, 13, 13, 13, 13, 13],  // 48 fortyEighth
    [11, 11, 11, 11, 11, 11, 11, 11], // 49 fortyNinth
    [13, 13, 13, 13, 13, 13, 13, 13], // 50 fiftieth
];

/// A campaign's `unlockRequirement` closure, flattened to data — the legacy
/// closures only ever test `maxCorruptionLevel()` thresholds or
/// `player.cubeUpgrades[50] > 99999` (Campaign.ts:562-1356).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignUnlock {
    /// `() => true`.
    Always,
    /// `maxCorruptionLevel() >= n`.
    MaxCorruption(u32),
    /// `player.cubeUpgrades[50] > 99999` (campaigns 40-41).
    CubeUpgrade50Over99999,
}

/// Per-campaign unlock gate, same key order as
/// [`CAMPAIGN_CORRUPTION_LOADOUTS`].
pub const CAMPAIGN_UNLOCK_REQUIREMENTS: [CampaignUnlock; CAMPAIGNS_LEN] = {
    let mut reqs = [CampaignUnlock::Always; CAMPAIGNS_LEN];
    let mut i = 7;
    while i < 15 {
        reqs[i] = CampaignUnlock::MaxCorruption(7); // 8-15
        i += 1;
    }
    while i < 25 {
        reqs[i] = CampaignUnlock::MaxCorruption(9); // 16-25
        i += 1;
    }
    while i < 39 {
        reqs[i] = CampaignUnlock::MaxCorruption(11); // 26-39
        i += 1;
    }
    reqs[39] = CampaignUnlock::CubeUpgrade50Over99999; // 40
    reqs[40] = CampaignUnlock::CubeUpgrade50Over99999; // 41
    i = 41;
    while i < 44 {
        reqs[i] = CampaignUnlock::MaxCorruption(12); // 42-44
        i += 1;
    }
    while i < 50 {
        reqs[i] = CampaignUnlock::MaxCorruption(13); // 45-50
        i += 1;
    }
    reqs
};

/// Evaluate a campaign's `unlockRequirement()` against the live inputs.
#[must_use]
pub fn campaign_unlocked(index: usize, max_corruption_level: f64, cube_upgrade_50: f64) -> bool {
    match CAMPAIGN_UNLOCK_REQUIREMENTS[index] {
        CampaignUnlock::Always => true,
        CampaignUnlock::MaxCorruption(n) => max_corruption_level >= f64::from(n),
        CampaignUnlock::CubeUpgrade50Over99999 => cube_upgrade_50 > 99999.0,
    }
}

/// `usableLoadout.totalCorruptionDifficultyScore` for a campaign
/// (Reset.ts:766-767): the difficulty of its stored loadout with the live
/// `bonusLevels` added to every corruption. The legacy loadout is built
/// straight from `campaignDatas` without clamping (the `CorruptionLoadout`
/// constructor copies levels verbatim), so this is player-independent
/// except for `bonus_levels`.
#[must_use]
pub fn campaign_loadout_difficulty(index: usize, bonus_levels: f64) -> f64 {
    corruption_loadout_difficulty_score(&CAMPAIGN_CORRUPTION_LOADOUTS[index], bonus_levels)
}

/// `Campaign.c10Completions` setter (Campaign.ts:523-527) — only-increase,
/// clamped to the campaign's `limit`. Used both by the singularity-4
/// auto-complete (`setC10ToArbitrary`) and the active-campaign bank
/// (`resetCampaign`), Reset.ts:762-784.
pub fn record_campaign_completion(completions: &mut [f64; CAMPAIGNS_LEN], index: usize, c10: f64) {
    if c10 > completions[index] {
        completions[index] = c10.min(CAMPAIGN_TOKEN_LIMITS[index]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loadout_rows_match_legacy_spot_checks() {
        // first: viscosity 1 only.
        assert_eq!(CAMPAIGN_CORRUPTION_LOADOUTS[0], [1, 0, 0, 0, 0, 0, 0, 0]);
        // second: drought 1 only (drought is index 6).
        assert_eq!(CAMPAIGN_CORRUPTION_LOADOUTS[1], [0, 0, 0, 0, 0, 0, 1, 0]);
        // eighteenth: v9 dr4 defl1 ext3 ill3 rec1.
        assert_eq!(CAMPAIGN_CORRUPTION_LOADOUTS[17], [9, 0, 0, 3, 1, 3, 4, 1]);
        // thirtieth drops viscosity/illiteracy to 0 (explicit zeros in TS).
        assert_eq!(CAMPAIGN_CORRUPTION_LOADOUTS[29], [0, 4, 1, 0, 4, 11, 5, 11]);
        // thirtyNinth has no viscosity key at all (schema default 0).
        assert_eq!(
            CAMPAIGN_CORRUPTION_LOADOUTS[38],
            [0, 6, 9, 2, 4, 11, 10, 11]
        );
        // fiftieth: everything 13.
        assert_eq!(CAMPAIGN_CORRUPTION_LOADOUTS[49], [13; 8]);
    }

    #[test]
    fn unlock_table_matches_legacy_bands() {
        assert_eq!(CAMPAIGN_UNLOCK_REQUIREMENTS[0], CampaignUnlock::Always);
        assert_eq!(CAMPAIGN_UNLOCK_REQUIREMENTS[6], CampaignUnlock::Always);
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[7],
            CampaignUnlock::MaxCorruption(7)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[14],
            CampaignUnlock::MaxCorruption(7)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[15],
            CampaignUnlock::MaxCorruption(9)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[24],
            CampaignUnlock::MaxCorruption(9)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[25],
            CampaignUnlock::MaxCorruption(11)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[38],
            CampaignUnlock::MaxCorruption(11)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[39],
            CampaignUnlock::CubeUpgrade50Over99999
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[40],
            CampaignUnlock::CubeUpgrade50Over99999
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[41],
            CampaignUnlock::MaxCorruption(12)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[43],
            CampaignUnlock::MaxCorruption(12)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[44],
            CampaignUnlock::MaxCorruption(13)
        );
        assert_eq!(
            CAMPAIGN_UNLOCK_REQUIREMENTS[49],
            CampaignUnlock::MaxCorruption(13)
        );
    }

    #[test]
    fn campaign_unlocked_evaluates_gates() {
        assert!(campaign_unlocked(0, 0.0, 0.0));
        assert!(!campaign_unlocked(7, 6.0, 0.0));
        assert!(campaign_unlocked(7, 7.0, 0.0));
        assert!(!campaign_unlocked(39, 14.0, 99999.0));
        assert!(campaign_unlocked(39, 0.0, 100000.0));
    }

    #[test]
    fn difficulty_matches_hand_computed_scores() {
        // first at bonus 0: 400 + 16·1² over one level-1 corruption.
        assert_eq!(campaign_loadout_difficulty(0, 0.0), 416.0);
        // fiftieth at bonus 0: 400 + 8 · 16 · 13².
        assert_eq!(
            campaign_loadout_difficulty(49, 0.0),
            400.0 + 8.0 * 16.0 * 169.0
        );
        // bonus levels are added to every one of the 8 corruptions, including
        // the zero-level ones (first: 1+b plus seven at b each).
        let b = 2.0;
        let expected = 400.0 + 16.0 * (3.0_f64 * 3.0) + 7.0 * 16.0 * (b * b);
        assert_eq!(campaign_loadout_difficulty(0, b), expected);
    }

    #[test]
    fn record_completion_is_only_increase_and_clamped() {
        let mut completions = [0.0; CAMPAIGNS_LEN];
        // Plain increase.
        record_campaign_completion(&mut completions, 0, 4.0);
        assert_eq!(completions[0], 4.0);
        // Never decreases.
        record_campaign_completion(&mut completions, 0, 2.0);
        assert_eq!(completions[0], 4.0);
        // Clamps to the campaign limit (first: 10).
        record_campaign_completion(&mut completions, 0, 25.0);
        assert_eq!(completions[0], 10.0);
        // Equal value is not an increase (verbatim `>` guard).
        record_campaign_completion(&mut completions, 0, 10.0);
        assert_eq!(completions[0], 10.0);
        // fiftieth clamps at 140.
        record_campaign_completion(&mut completions, 49, 500.0);
        assert_eq!(completions[49], 140.0);
    }
}
