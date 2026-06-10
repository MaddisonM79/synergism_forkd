//! Campaigns + constants state slice.
//!
//! Mirrors `player.campaigns` and `player.constantUpgrades` from
//! the legacy schema. Backs [`crate::mechanics::campaign_token_rewards`]
//! and the constant-upgrade reads scattered across the tick layer.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use synergismforkd_bignum::Decimal;

/// Fixed cardinality of the campaign-completions array — one slot per key
/// of the legacy `campaignDatas` const (`Campaign.ts`, `first` through
/// `fiftieth`), verified against both legacy snapshots (50 keys, identical
/// order and `limit`/`isMeta` values). The earlier `10` was a latent sizing
/// bug (the octeract-42→47 class).
pub const CAMPAIGNS_LEN: usize = 50;

/// Fixed cardinality of the constant-upgrade array — `10 + 1` for the
/// legacy 1-indexed convention. Tier B item 12.
pub const CONSTANT_UPGRADES_LEN: usize = 11;

/// Slice of `GameState` for campaigns + constant upgrades.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CampaignsState {
    /// Per-campaign `c10Completions` count, in `campaignDatas` key order
    /// (`first` = 0 … `fiftieth` = 49). Written by the ascension layer's
    /// campaign sweep (auto-complete at `highestSingularityCount >= 4` +
    /// the active-campaign bank, Reset.ts:762-784); the token total derives
    /// from them via [`crate::mechanics::campaign_token_rewards`].
    #[serde(with = "BigArray")]
    pub campaign_completions: [f64; CAMPAIGNS_LEN],
    /// `player.campaigns.currentCampaign` — the active campaign's index
    /// (`campaignDatas` key order), or `None` outside a campaign. Set by
    /// `PlayerAction::SelectCampaign`; cleared — after banking completions —
    /// by the ascension layer (Reset.ts:782-784).
    pub current_campaign: Option<u8>,
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
            current_campaign: None,
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
