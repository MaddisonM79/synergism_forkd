//! Campaigns + constants state slice.
//!
//! Mirrors `player.campaigns` and `player.constantUpgrades` from
//! the legacy schema. Backs [`crate::mechanics::campaign_token_rewards`]
//! and the constant-upgrade reads scattered across the tick layer.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Fixed cardinality of the campaign-completions array. Tier B item 12.
pub const CAMPAIGNS_LEN: usize = 10;

/// Fixed cardinality of the constant-upgrade array — `10 + 1` for the
/// legacy 1-indexed convention. Tier B item 12.
pub const CONSTANT_UPGRADES_LEN: usize = 11;

/// Slice of `GameState` for campaigns + constant upgrades.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CampaignsState {
    /// Per-campaign completion count. UI maintains the name ↔
    /// index mapping. Both arrays fit inside serde's default
    /// 0..=32-length window so no `BigArray` attribute needed.
    pub campaign_completions: [f64; CAMPAIGNS_LEN],
    /// `player.campaigns.tokensSpent` — total tokens spent across
    /// all campaigns.
    pub tokens_spent: f64,
    /// `player.campaigns.ascensionScoreMultiplier` — cached
    /// derived value from campaign rewards.
    pub ascension_score_multiplier: f64,
    /// `player.constantUpgrades` — per-constant-upgrade level.
    /// 1-indexed (slot 0 unused).
    pub constant_upgrades: [f64; CONSTANT_UPGRADES_LEN],
    /// `player.ascendShards` — Decimal balance for ascend-shards.
    pub ascend_shards: Decimal,
}

impl Default for CampaignsState {
    fn default() -> Self {
        Self {
            campaign_completions: [0.0; CAMPAIGNS_LEN],
            tokens_spent: 0.0,
            ascension_score_multiplier: 1.0,
            constant_upgrades: [0.0; CONSTANT_UPGRADES_LEN],
            ascend_shards: Decimal::zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_widths_match_legacy() {
        let s = CampaignsState::default();
        assert_eq!(s.campaign_completions.len(), CAMPAIGNS_LEN);
        assert_eq!(s.constant_upgrades.len(), CONSTANT_UPGRADES_LEN);
        assert_eq!(s.ascension_score_multiplier, 1.0);
    }
}
