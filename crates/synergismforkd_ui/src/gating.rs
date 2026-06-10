//! Grouped-nav information architecture + unlock gating.
//!
//! The legacy 15-tab strip regroups into 5 play groups + Settings. A group
//! is visible iff any of its sections is; sections read the unlock flags
//! the logic tier already maintains (mostly `reset_counters`). Early game
//! therefore shows exactly Production + Settings — no empty groups.
//!
//! Sections the vertical slice doesn't implement yet still appear here
//! (rendered as placeholders once their gate opens) so the IA is complete
//! from day one and each later milestone only fills in content.

use synergismforkd_logic::GameState;

/// Top-level nav groups, in rail order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Group {
    #[default]
    Production,
    Mystic,
    Ascension,
    Singularity,
    Shop,
    Settings,
}

impl Group {
    pub const ALL: [Group; 6] = [
        Group::Production,
        Group::Mystic,
        Group::Ascension,
        Group::Singularity,
        Group::Shop,
        Group::Settings,
    ];

    /// Translation key for the rail button.
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Group::Production => "nav.group.production",
            Group::Mystic => "nav.group.mystic",
            Group::Ascension => "nav.group.ascension",
            Group::Singularity => "nav.group.singularity",
            Group::Shop => "nav.group.shop",
            Group::Settings => "nav.group.settings",
        }
    }

    /// CSS accent hook (`--accent-<value>` token family).
    #[must_use]
    pub fn css_value(self) -> &'static str {
        match self {
            Group::Production => "production",
            Group::Mystic => "mystic",
            Group::Ascension => "ascension",
            Group::Singularity => "singularity",
            Group::Shop => "shop",
            Group::Settings => "settings",
        }
    }

    /// Sections inside this group, in sub-nav order.
    #[must_use]
    pub fn sections(self) -> &'static [Section] {
        match self {
            Group::Production => &[Section::Buildings, Section::Upgrades, Section::Achievements],
            Group::Mystic => &[
                Section::Runes,
                Section::Challenges,
                Section::Research,
                Section::AntHill,
            ],
            Group::Ascension => &[Section::WowCubes, Section::Corruption, Section::Campaign],
            Group::Singularity => &[
                Section::SingularityOverview,
                Section::GoldenQuarkUpgrades,
                Section::Elevator,
                Section::Octeracts,
                Section::Exalts,
                Section::Ambrosia,
            ],
            Group::Shop => &[Section::Shop],
            Group::Settings => &[
                Section::SettingsGeneral,
                Section::SettingsSaves,
                Section::SettingsThemes,
            ],
        }
    }

    /// A group shows iff any child section does.
    #[must_use]
    pub fn visible(self, state: &GameState) -> bool {
        self.sections().iter().any(|s| s.visible(state))
    }

    /// First visible section (fallback target when routing into a group or
    /// when an import regresses visibility).
    #[must_use]
    pub fn first_visible_section(self, state: &GameState) -> Option<Section> {
        self.sections().iter().copied().find(|s| s.visible(state))
    }
}

/// One content panel. `← legacy tab` mappings per the ratified IA;
/// Exalts moved from Challenges to Singularity (it's singularity content —
/// the legacy placement was a DOM convenience).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    // Production
    #[default]
    Buildings,
    Upgrades,
    Achievements,
    // Mystic
    Runes,
    Challenges,
    Research,
    AntHill,
    // Ascension
    WowCubes,
    Corruption,
    Campaign,
    // Singularity
    SingularityOverview,
    GoldenQuarkUpgrades,
    Elevator,
    Octeracts,
    Exalts,
    Ambrosia,
    // Shop
    Shop,
    // Settings
    SettingsGeneral,
    SettingsSaves,
    SettingsThemes,
}

impl Section {
    /// The group this section lives in.
    #[must_use]
    pub fn group(self) -> Group {
        match self {
            Section::Buildings | Section::Upgrades | Section::Achievements => Group::Production,
            Section::Runes | Section::Challenges | Section::Research | Section::AntHill => {
                Group::Mystic
            }
            Section::WowCubes | Section::Corruption | Section::Campaign => Group::Ascension,
            Section::SingularityOverview
            | Section::GoldenQuarkUpgrades
            | Section::Elevator
            | Section::Octeracts
            | Section::Exalts
            | Section::Ambrosia => Group::Singularity,
            Section::Shop => Group::Shop,
            Section::SettingsGeneral | Section::SettingsSaves | Section::SettingsThemes => {
                Group::Settings
            }
        }
    }

    /// Translation key for the sub-nav button. Sections beyond the vertical
    /// slice reuse their legacy names until their milestone lands.
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Section::Buildings => "nav.section.buildings",
            Section::Upgrades => "nav.section.upgrades",
            Section::Achievements => "nav.section.achievements",
            Section::Runes => "nav.section.runes",
            Section::Challenges => "nav.section.challenges",
            Section::Research => "nav.section.research",
            Section::AntHill => "nav.section.ant_hill",
            Section::WowCubes => "nav.section.wow_cubes",
            Section::Corruption => "nav.section.corruption",
            Section::Campaign => "nav.section.campaign",
            Section::SingularityOverview => "nav.section.singularity_overview",
            Section::GoldenQuarkUpgrades => "nav.section.gq_upgrades",
            Section::Elevator => "nav.section.elevator",
            Section::Octeracts => "nav.section.octeracts",
            Section::Exalts => "nav.section.exalts",
            Section::Ambrosia => "nav.section.ambrosia",
            Section::Shop => "nav.section.shop",
            Section::SettingsGeneral => "nav.section.settings_general",
            Section::SettingsSaves => "nav.section.settings_saves",
            Section::SettingsThemes => "nav.section.settings_themes",
        }
    }

    /// Unlock gate. Reads the flags the logic tier maintains; the
    /// singularity-internal gates are deliberately coarse (whole-group
    /// `highest_singularity_count`) until the M5 milestone wires precise
    /// per-feature unlocks (octeract gate, blueberry gate, sing ≥ 25 for
    /// Exalts).
    #[must_use]
    pub fn visible(self, state: &GameState) -> bool {
        let rc = &state.reset_counters;
        match self {
            Section::Buildings
            | Section::Upgrades
            | Section::SettingsGeneral
            | Section::SettingsSaves
            | Section::SettingsThemes => true,
            // Legacy gates the Achievements tab on the reincarnation unlock
            // (Tabs.ts:627), not the `achievements_unlocked` flag (which the
            // logic tier never writes).
            Section::Achievements => rc.reincarnate_unlocked,
            Section::Runes => rc.prestige_unlocked,
            Section::Challenges => rc.transcend_unlocked,
            Section::Research | Section::Shop => rc.reincarnate_unlocked,
            Section::AntHill => rc.anthill_unlocked,
            Section::WowCubes | Section::Corruption | Section::Campaign => rc.ascension_unlocked,
            Section::SingularityOverview
            | Section::GoldenQuarkUpgrades
            | Section::Elevator
            | Section::Octeracts
            | Section::Exalts
            | Section::Ambrosia => state.singularity.highest_singularity_count > 0.0,
        }
    }
}

/// Current nav position. `subsection` indexes into the active section's own
/// sub-tab list (only Buildings uses it in the vertical slice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Route {
    pub group: Group,
    pub section: Section,
    pub subsection: usize,
}

impl Route {
    /// Clamp a route against current visibility: an import that regresses
    /// unlocks (or a stale route) snaps to the first visible section of the
    /// first visible group.
    #[must_use]
    pub fn clamped(self, state: &GameState) -> Route {
        if self.section.visible(state) {
            return self;
        }
        if let Some(section) = self.group.first_visible_section(state) {
            return Route {
                group: self.group,
                section,
                subsection: 0,
            };
        }
        let group = Group::ALL
            .into_iter()
            .find(|g| g.visible(state))
            .unwrap_or(Group::Production);
        Route {
            group,
            section: group.first_visible_section(state).unwrap_or_default(),
            subsection: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_save_shows_exactly_production_and_settings() {
        let state = GameState::default();
        let visible: Vec<Group> = Group::ALL
            .into_iter()
            .filter(|g| g.visible(&state))
            .collect();
        assert_eq!(visible, vec![Group::Production, Group::Settings]);
        // Achievements hides inside Production until its flag flips.
        assert!(!Section::Achievements.visible(&state));
        assert!(Section::Buildings.visible(&state));
        assert!(Section::Upgrades.visible(&state));
    }

    #[test]
    fn prestige_reveals_mystic_via_runes() {
        let mut state = GameState::default();
        state.reset_counters.prestige_unlocked = true;
        assert!(Group::Mystic.visible(&state));
        assert_eq!(
            Group::Mystic.first_visible_section(&state),
            Some(Section::Runes)
        );
        assert!(!Section::Challenges.visible(&state), "needs transcend");
    }

    #[test]
    fn ascension_and_singularity_gates() {
        let mut state = GameState::default();
        assert!(!Group::Ascension.visible(&state));
        state.reset_counters.ascension_unlocked = true;
        assert!(Group::Ascension.visible(&state));

        assert!(!Group::Singularity.visible(&state));
        state.singularity.highest_singularity_count = 1.0;
        assert!(Group::Singularity.visible(&state));
    }

    #[test]
    fn every_section_belongs_to_its_group_listing() {
        for group in Group::ALL {
            for section in group.sections() {
                assert_eq!(section.group(), group, "{section:?} mislisted");
            }
        }
    }

    #[test]
    fn route_clamps_to_visible_ground() {
        let state = GameState::default();
        // A stale route into a locked section snaps to the group's first
        // visible section…
        let stale = Route {
            group: Group::Production,
            section: Section::Achievements,
            subsection: 3,
        };
        let snapped = stale.clamped(&state);
        assert_eq!(snapped.section, Section::Buildings);
        assert_eq!(snapped.subsection, 0);
        // …and a route into a fully locked group falls back to Production.
        let locked = Route {
            group: Group::Singularity,
            section: Section::Exalts,
            subsection: 0,
        };
        assert_eq!(locked.clamped(&state).group, Group::Production);
    }
}
