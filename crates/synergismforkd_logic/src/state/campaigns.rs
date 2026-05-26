//! Campaigns + constants state slice.
//!
//! Mirrors `player.campaigns` and `player.constantUpgrades` from
//! the legacy schema. Backs [`crate::mechanics::campaign_token_rewards`]
//! and the constant-upgrade reads scattered across the tick layer.

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` for campaigns + constant upgrades.
#[derive(Debug, Clone, PartialEq)]
pub struct CampaignsState {
    /// Per-campaign completion count. UI maintains the name ↔
    /// index mapping. Legacy has ~10 campaigns.
    pub campaign_completions: Vec<f64>,
    /// `player.campaigns.tokensSpent` — total tokens spent across
    /// all campaigns.
    pub tokens_spent: f64,
    /// `player.campaigns.ascensionScoreMultiplier` — cached
    /// derived value from campaign rewards.
    pub ascension_score_multiplier: f64,
    /// `player.constantUpgrades` — per-constant-upgrade level. 1-indexed
    /// (slot 0 unused).
    pub constant_upgrades: Vec<f64>,
    /// `player.ascendShards` — Decimal balance for ascend-shards.
    pub ascend_shards: Decimal,
}

impl CampaignsState {
    /// Build with `n_campaigns` campaign slots and
    /// `n_constant_upgrades + 1` constant slots.
    #[must_use]
    pub fn new(n_campaigns: usize, n_constant_upgrades: usize) -> Self {
        Self {
            campaign_completions: vec![0.0; n_campaigns],
            tokens_spent: 0.0,
            ascension_score_multiplier: 1.0,
            constant_upgrades: vec![0.0; n_constant_upgrades + 1],
            ascend_shards: Decimal::zero(),
        }
    }
}

impl Default for CampaignsState {
    fn default() -> Self {
        Self::new(10, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_widths_match_legacy() {
        let s = CampaignsState::default();
        assert_eq!(s.campaign_completions.len(), 10);
        assert_eq!(s.constant_upgrades.len(), 11);
        assert_eq!(s.ascension_score_multiplier, 1.0);
    }
}
