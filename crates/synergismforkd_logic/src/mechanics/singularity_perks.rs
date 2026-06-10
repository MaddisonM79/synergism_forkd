//! Singularity perk table (`singularity.ts`, `singularityPerks`).
//!
//! The 53-perk roster: each perk's `levels` array — the singularity counts at
//! which it unlocks and upgrades — plus the [`getLastUpgradeInfo`] level /
//! next-threshold helper. This is the data model the singularity-perk panel
//! renders from; the i18n `name` / `description` closures are UI-tier and not
//! ported.
//!
//! ## Effect wiring
//!
//! A perk's *effect* is almost never applied at the perk record — it lives as a
//! scattered `highestSingularityCount >= N` (occasionally `singularityCount >=
//! N`) comparison inside the consuming system. Each perk below is tagged with
//! its current Rust status:
//!
//! - **WIRED** — the effect is already ported at the cited site.
//! - **DEFERRED(assembly)** — the perk's host stat-assembly is itself a neutral
//!   placeholder in Rust (e.g. `calculateActualAntSpeedMult` → the tick's
//!   `ant_speed_mult` input defaults to `1.0`), so the term has nowhere to land
//!   yet. Not a dropped term — the whole assembly is unported.
//! - **DEFERRED(export)** — GQ-per-second export generation, claimed in the
//!   export/offline reward flow (the `goldenRevolution` family).
//! - **UI** — display, automation-cadence, or unlock-gate only; no logic effect.
//! - **EXTERNAL** — depends on a service outside the fork (PseudoCoins, add-codes).
//!
//! The big multiplicative perks (`goldenCoins`, `skrauQ`, `goldenRevolution2`,
//! `primalPower`, `derpSmithsCornucopia`, `immaculateAlchemy`, the salvage /
//! token / ELO / blueberry perks) are all WIRED — most by the meta-economy
//! sweep. What remains DEFERRED is gated on a *consumer* that isn't ported yet,
//! not on the perk itself.

/// A singularity perk, in `singularityPerks` declaration order
/// (`welcometoSingularity` = 0 … `taxReduction` = 52). `ID` order is stable
/// and matches the legacy htmlID used for save/UI cross-reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum SingularityPerkId {
    WelcomeToSingularity = 0,
    UnlimitedGrowth = 1,
    GoldenCoins = 2,
    Xyz = 3,
    GenerousOrbs = 4,
    ResearchDummies = 5,
    RecycledContent = 6,
    AntGodsCornucopia = 7,
    BringToLife = 8,
    TokenInheritance = 9,
    Sweepomatic = 10,
    SuperStart = 11,
    InvigoratedSpirits = 12,
    EloBonus = 13,
    NotSoChallenging = 14,
    AutoCampaigns = 15,
    AutomationUpgrades = 16,
    EvenMoreQuarks = 17,
    PotionAutogenerator = 18,
    PersistentGlobalResets = 19,
    ShopSpecialOffer = 20,
    ForTheLoveOfTheAntGod = 21,
    ItAllAddsUp = 22,
    AutomagicalRunes = 23,
    FirstClearTokens = 24,
    DerpSmithsCornucopia = 25,
    EternalAscensions = 26,
    ExaltedAchievements = 27,
    CoolQolCubes = 28,
    InfiniteRecycling = 29,
    IrishAnt = 30,
    BonusTokens = 31,
    ImmaculateAlchemy = 32,
    Overclocked = 33,
    WowCubeAutomatedShipping = 34,
    PlatonicClones = 35,
    CongealedBlueberries = 36,
    LastClearTokens = 37,
    RecyclistsDesktop = 38,
    GoldenRevolution = 39,
    GoldenRevolution2 = 40,
    GoldenRevolution3 = 41,
    IrishAnt2 = 42,
    PlatSigma = 43,
    PrimalPower = 44,
    MidasMilleniumAgedGold = 45,
    GoldenRevolution4 = 46,
    OcteractMetagenesis = 47,
    SkrauQ = 48,
    DemeterHarvest = 49,
    PermanentBenefaction = 50,
    InfiniteShopUpgrades = 51,
    TaxReduction = 52,
}

/// One perk record: its stable string ID (the legacy htmlID, used for
/// save/UI cross-reference) and its monotonically-increasing `levels`
/// thresholds.
#[derive(Debug, Clone, Copy)]
pub struct SingularityPerk {
    /// The enum tag.
    pub id: SingularityPerkId,
    /// The legacy `ID` string (htmlID).
    pub html_id: &'static str,
    /// The singularity counts at which the perk unlocks (index 0) and each
    /// subsequent level activates. Always non-empty and ascending.
    pub levels: &'static [u32],
}

/// The full perk roster, in declaration order. Verified 1:1 against
/// `singularityPerks` (53 perks; level arrays extracted mechanically).
pub const SINGULARITY_PERKS: [SingularityPerk; 53] = {
    use SingularityPerkId::*;
    macro_rules! perk {
        ($id:expr, $html:literal, $levels:expr) => {
            SingularityPerk {
                id: $id,
                html_id: $html,
                levels: &$levels,
            }
        };
    }
    [
        perk!(WelcomeToSingularity, "welcometoSingularity", [1]),
        perk!(UnlimitedGrowth, "unlimitedGrowth", [1]),
        perk!(GoldenCoins, "goldenCoins", [1]),
        perk!(Xyz, "xyz", [1, 20, 200]),
        perk!(
            GenerousOrbs,
            "generousOrbs",
            [1, 2, 5, 10, 15, 20, 25, 30, 35]
        ),
        perk!(ResearchDummies, "researchDummies", [1, 11]),
        perk!(
            RecycledContent,
            "recycledContent",
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        ),
        perk!(AntGodsCornucopia, "antGodsCornucopia", [1, 30, 70, 100]),
        perk!(
            BringToLife,
            "bringToLife",
            [1, 9, 25, 49, 81, 121, 169, 196, 225, 256, 289]
        ),
        perk!(
            TokenInheritance,
            "tokenInheritance",
            [2, 5, 10, 17, 26, 37, 50, 65, 82, 101, 220, 240, 260, 270, 277]
        ),
        perk!(Sweepomatic, "sweepomatic", [2, 101]),
        perk!(SuperStart, "superStart", [2, 3, 4, 7, 15]),
        perk!(
            InvigoratedSpirits,
            "invigoratedSpirits",
            [2, 10, 26, 50, 82, 122, 170, 197, 226, 257, 290]
        ),
        perk!(
            EloBonus,
            "eloBonus",
            [3, 11, 27, 51, 83, 123, 171, 198, 227, 258, 291]
        ),
        perk!(NotSoChallenging, "notSoChallenging", [4, 7, 10, 15, 20]),
        perk!(AutoCampaigns, "autoCampaigns", [4]),
        perk!(
            AutomationUpgrades,
            "automationUpgrades",
            [5, 10, 15, 25, 30, 100]
        ),
        perk!(
            EvenMoreQuarks,
            "evenMoreQuarks",
            [
                5, 7, 10, 20, 35, 50, 65, 80, 90, 100, 121, 144, 150, 160, 166, 169, 170, 175, 180,
                190, 196, 200, 201, 202, 203, 204, 205, 210, 213, 216, 219, 225, 228, 231, 234,
                237, 240, 244, 248, 252, 256, 260, 264, 268, 272, 276, 280, 284, 288, 290
            ]
        ),
        perk!(PotionAutogenerator, "potionAutogenerator", [6]),
        perk!(PersistentGlobalResets, "persistentGlobalResets", [8]),
        perk!(ShopSpecialOffer, "shopSpecialOffer", [10, 50]),
        perk!(ForTheLoveOfTheAntGod, "forTheLoveOfTheAntGod", [10, 15, 20]),
        perk!(
            ItAllAddsUp,
            "itAllAddsUp",
            [10, 16, 25, 36, 49, 64, 81, 100, 121, 144, 169, 196, 225, 235, 240]
        ),
        perk!(AutomagicalRunes, "automagicalRunes", [15, 30, 40, 50]),
        perk!(FirstClearTokens, "firstClearTokens", [16]),
        perk!(
            DerpSmithsCornucopia,
            "derpSmithsCornucopia",
            [18, 38, 58, 78, 88, 98, 118, 148, 178, 188, 198, 208, 218, 228, 238, 248]
        ),
        perk!(EternalAscensions, "eternalAscensions", [25]),
        perk!(ExaltedAchievements, "exaltedAchievements", [25]),
        perk!(CoolQolCubes, "coolQOLCubes", [25, 35]),
        perk!(
            InfiniteRecycling,
            "infiniteRecycling",
            [30, 40, 61, 81, 111, 131, 161, 191, 236, 260]
        ),
        perk!(
            IrishAnt,
            "irishAnt",
            [35, 42, 49, 56, 63, 70, 77, 135, 142, 149, 156, 163, 170, 177]
        ),
        perk!(BonusTokens, "bonusTokens", [41, 58, 113, 163, 229]),
        perk!(ImmaculateAlchemy, "immaculateAlchemy", [50]),
        perk!(
            Overclocked,
            "overclocked",
            [50, 60, 75, 100, 125, 150, 175, 200, 225, 250]
        ),
        perk!(WowCubeAutomatedShipping, "wowCubeAutomatedShipping", [50]),
        perk!(PlatonicClones, "platonicClones", [50]),
        perk!(
            CongealedBlueberries,
            "congealedblueberries",
            [64, 128, 192, 256, 270]
        ),
        perk!(LastClearTokens, "lastClearTokens", [69]),
        perk!(
            RecyclistsDesktop,
            "recyclistsDesktop",
            [75, 85, 105, 125, 155, 185, 215, 245, 260, 275]
        ),
        perk!(GoldenRevolution, "goldenRevolution", [100]),
        perk!(GoldenRevolution2, "goldenRevolution2", [100]),
        perk!(GoldenRevolution3, "goldenRevolution3", [100]),
        perk!(
            IrishAnt2,
            "irishAnt2",
            [100, 150, 200, 225, 250, 255, 260, 265, 269, 272]
        ),
        perk!(PlatSigma, "platSigma", [125, 200]),
        perk!(PrimalPower, "primalPower", [131, 269]),
        perk!(MidasMilleniumAgedGold, "midasMilleniumAgedGold", [150]),
        perk!(
            GoldenRevolution4,
            "goldenRevolution4",
            [160, 173, 185, 194, 204, 210, 219, 229, 240, 249]
        ),
        perk!(OcteractMetagenesis, "octeractMetagenesis", [200, 205]),
        perk!(SkrauQ, "skrauQ", [200]),
        perk!(DemeterHarvest, "demeterHarvest", [230, 245, 260, 275, 290]),
        perk!(PermanentBenefaction, "permanentBenefaction", [244]),
        perk!(InfiniteShopUpgrades, "infiniteShopUpgrades", [250, 280]),
        perk!(TaxReduction, "taxReduction", [281]),
    ]
};

impl SingularityPerkId {
    /// This perk's record (its `html_id` + `levels`).
    #[must_use]
    pub fn def(self) -> &'static SingularityPerk {
        &SINGULARITY_PERKS[self as usize]
    }

    /// This perk's `levels` thresholds.
    #[must_use]
    pub fn levels(self) -> &'static [u32] {
        self.def().levels
    }

    /// The perk's current level at `count` singularities — the number of
    /// thresholds crossed (`getLastUpgradeInfo(...).level`, `singularity.ts:
    /// 3194-3204`). `0` until the first threshold is reached, then the count
    /// of `levels[i] <= count`.
    ///
    /// Effects read `player.highestSingularityCount` (the all-time max), so
    /// callers should pass that — *not* the current `singularityCount`, which
    /// only the panel display uses.
    #[must_use]
    pub fn level_at(self, count: f64) -> u32 {
        self.levels()
            .iter()
            .filter(|&&threshold| count >= f64::from(threshold))
            .count() as u32
    }

    /// Whether the perk has unlocked at all (`level_at(count) >= 1`).
    #[must_use]
    pub fn is_active(self, count: f64) -> bool {
        self.levels()
            .first()
            .is_some_and(|&first| count >= f64::from(first))
    }

    /// The next threshold strictly above `count` (`getLastUpgradeInfo(...).
    /// next`), or `None` once the perk is maxed.
    #[must_use]
    pub fn next_threshold(self, count: f64) -> Option<u32> {
        self.levels()
            .iter()
            .copied()
            .find(|&threshold| f64::from(threshold) > count)
    }
}

/// `getActivePerkCount` analogue — how many distinct perks have unlocked at
/// `count` singularities (each perk counts once, regardless of level).
#[must_use]
pub fn active_perk_count(count: f64) -> usize {
    SINGULARITY_PERKS
        .iter()
        .filter(|perk| count >= f64::from(perk.levels[0]))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_is_53_perks_in_declaration_order() {
        assert_eq!(SINGULARITY_PERKS.len(), 53);
        // Tag index matches array slot for every entry.
        for (i, perk) in SINGULARITY_PERKS.iter().enumerate() {
            assert_eq!(perk.id as usize, i, "perk {} out of order", perk.html_id);
        }
        // Endpoints + a couple of mid-roster html IDs.
        assert_eq!(SINGULARITY_PERKS[0].html_id, "welcometoSingularity");
        assert_eq!(SINGULARITY_PERKS[7].html_id, "antGodsCornucopia");
        assert_eq!(SINGULARITY_PERKS[48].html_id, "skrauQ");
        assert_eq!(SINGULARITY_PERKS[52].html_id, "taxReduction");
    }

    #[test]
    fn level_arrays_are_ascending_and_nonempty() {
        for perk in &SINGULARITY_PERKS {
            assert!(!perk.levels.is_empty(), "{} has no levels", perk.html_id);
            assert!(
                perk.levels.windows(2).all(|w| w[0] < w[1]),
                "{} levels not strictly ascending",
                perk.html_id
            );
        }
    }

    #[test]
    fn evenmorequarks_has_the_full_50_entry_ladder() {
        // The longest array — a transcription-risk row, asserted by length and
        // endpoints against the mechanical extraction.
        let levels = SingularityPerkId::EvenMoreQuarks.levels();
        assert_eq!(levels.len(), 50);
        assert_eq!(levels[0], 5);
        assert_eq!(levels[49], 290);
    }

    #[test]
    fn level_at_counts_crossed_thresholds() {
        let generous = SingularityPerkId::GenerousOrbs; // [1,2,5,10,15,20,25,30,35]
        assert_eq!(generous.level_at(0.0), 0);
        assert_eq!(generous.level_at(1.0), 1);
        assert_eq!(generous.level_at(5.0), 3); // 1,2,5 crossed
        assert_eq!(generous.level_at(34.0), 8);
        assert_eq!(generous.level_at(35.0), 9); // maxed
        assert_eq!(generous.level_at(1000.0), 9); // can't exceed array length
    }

    #[test]
    fn is_active_and_next_threshold_match_legacy_getlastupgradeinfo() {
        let tax = SingularityPerkId::TaxReduction; // [281]
        assert!(!tax.is_active(280.0));
        assert!(tax.is_active(281.0));
        assert_eq!(tax.next_threshold(0.0), Some(281));
        assert_eq!(tax.next_threshold(281.0), None);

        let coolqol = SingularityPerkId::CoolQolCubes; // [25,35]
        assert_eq!(coolqol.next_threshold(25.0), Some(35));
        assert_eq!(coolqol.next_threshold(34.9), Some(35));
        assert_eq!(coolqol.next_threshold(35.0), None);
    }

    #[test]
    fn active_perk_count_tracks_unlocks() {
        // Below the first perk threshold (1) nothing is active.
        assert_eq!(active_perk_count(0.0), 0);
        // At sing 1 every perk whose levels[0] == 1 is active. From the roster:
        // welcometoSingularity, unlimitedGrowth, goldenCoins, xyz, generousOrbs,
        // researchDummies, recycledContent, antGodsCornucopia, bringToLife = 9.
        assert_eq!(active_perk_count(1.0), 9);
        // At a very high count, all 53 are active.
        assert_eq!(active_perk_count(1000.0), 53);
    }
}
