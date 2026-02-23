use std::cmp::Ordering;
use std::collections::HashSet;

use super::distributed_dht::ProviderRecord;

const DEFAULT_LATENCY_WORST_MS: u32 = 1_000;
const NEUTRAL_SCORE: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct ProviderSelectionPolicy {
    pub freshness_ttl_ms: i64,
    pub weight_freshness: f64,
    pub weight_uptime: f64,
    pub weight_challenge: f64,
    pub weight_capacity: f64,
    pub weight_load: f64,
    pub weight_latency: f64,
    pub max_candidates: usize,
}

impl Default for ProviderSelectionPolicy {
    fn default() -> Self {
        Self {
            freshness_ttl_ms: 10 * 60 * 1_000,
            weight_freshness: 0.20,
            weight_uptime: 0.20,
            weight_challenge: 0.20,
            weight_capacity: 0.20,
            weight_load: 0.10,
            weight_latency: 0.10,
            max_candidates: 8,
        }
    }
}

impl ProviderSelectionPolicy {
    pub fn score_provider(&self, provider: &ProviderRecord, now_ms: i64) -> f64 {
        let freshness = self.freshness_score(provider.last_seen_ms, now_ms);
        let uptime = provider
            .uptime_ratio_per_mille
            .map(normalize_ratio_per_mille)
            .unwrap_or(NEUTRAL_SCORE);
        let challenge = provider
            .challenge_pass_ratio_per_mille
            .map(normalize_ratio_per_mille)
            .unwrap_or(NEUTRAL_SCORE);
        let capacity = capacity_score(
            provider.storage_total_bytes,
            provider.storage_available_bytes,
        );
        let load = provider
            .load_ratio_per_mille
            .map(normalize_ratio_per_mille)
            .map(|ratio| clamp01(1.0 - ratio))
            .unwrap_or(NEUTRAL_SCORE);
        let latency = provider
            .p50_read_latency_ms
            .map(latency_score)
            .unwrap_or(NEUTRAL_SCORE);

        let total_weight = self.weight_freshness
            + self.weight_uptime
            + self.weight_challenge
            + self.weight_capacity
            + self.weight_load
            + self.weight_latency;

        if total_weight <= f64::EPSILON {
            return freshness;
        }

        let score = self.weight_freshness * freshness
            + self.weight_uptime * uptime
            + self.weight_challenge * challenge
            + self.weight_capacity * capacity
            + self.weight_load * load
            + self.weight_latency * latency;
        clamp01(score / total_weight)
    }

    pub fn rank_providers(&self, providers: &[ProviderRecord], now_ms: i64) -> Vec<ProviderRecord> {
        let mut scored: Vec<(ProviderRecord, f64)> = providers
            .iter()
            .cloned()
            .map(|provider| {
                let score = self.score_provider(&provider, now_ms);
                (provider, score)
            })
            .collect();
        scored.sort_by(
            |(left_provider, left_score), (right_provider, right_score)| {
                right_score
                    .partial_cmp(left_score)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| right_provider.last_seen_ms.cmp(&left_provider.last_seen_ms))
                    .then_with(|| left_provider.provider_id.cmp(&right_provider.provider_id))
            },
        );

        let mut deduplicated = Vec::with_capacity(scored.len());
        let mut seen = HashSet::new();
        for (provider, _) in scored {
            if !seen.insert(provider.provider_id.clone()) {
                continue;
            }
            deduplicated.push(provider);
            if self.max_candidates > 0 && deduplicated.len() >= self.max_candidates {
                break;
            }
        }
        deduplicated
    }

    fn freshness_score(&self, last_seen_ms: i64, now_ms: i64) -> f64 {
        if self.freshness_ttl_ms <= 0 {
            return NEUTRAL_SCORE;
        }
        let age_ms = now_ms.saturating_sub(last_seen_ms).max(0);
        if age_ms >= self.freshness_ttl_ms {
            return 0.0;
        }
        clamp01(1.0 - (age_ms as f64 / self.freshness_ttl_ms as f64))
    }
}

fn normalize_ratio_per_mille(ratio: u16) -> f64 {
    clamp01((ratio.min(1_000) as f64) / 1_000.0)
}

fn capacity_score(total_bytes: Option<u64>, available_bytes: Option<u64>) -> f64 {
    let (Some(total), Some(available)) = (total_bytes, available_bytes) else {
        return NEUTRAL_SCORE;
    };
    if total == 0 {
        return 0.0;
    }
    clamp01((available.min(total) as f64) / (total as f64))
}

fn latency_score(latency_ms: u32) -> f64 {
    let bounded = latency_ms.min(DEFAULT_LATENCY_WORST_MS) as f64;
    clamp01(1.0 - bounded / DEFAULT_LATENCY_WORST_MS as f64)
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(provider_id: &str, last_seen_ms: i64) -> ProviderRecord {
        ProviderRecord {
            provider_id: provider_id.to_string(),
            last_seen_ms,
            storage_total_bytes: None,
            storage_available_bytes: None,
            uptime_ratio_per_mille: None,
            challenge_pass_ratio_per_mille: None,
            load_ratio_per_mille: None,
            p50_read_latency_ms: None,
        }
    }

    #[test]
    fn rank_providers_prefers_stronger_capability_profile() {
        let now_ms = 10_500;
        let policy = ProviderSelectionPolicy::default();

        let mut strong = provider("peer-strong", 10_000);
        strong.storage_total_bytes = Some(100);
        strong.storage_available_bytes = Some(90);
        strong.uptime_ratio_per_mille = Some(990);
        strong.challenge_pass_ratio_per_mille = Some(980);
        strong.load_ratio_per_mille = Some(100);
        strong.p50_read_latency_ms = Some(50);

        let legacy = provider("peer-legacy", 10_100);

        let mut weak = provider("peer-weak", 9_000);
        weak.storage_total_bytes = Some(100);
        weak.storage_available_bytes = Some(10);
        weak.uptime_ratio_per_mille = Some(700);
        weak.challenge_pass_ratio_per_mille = Some(650);
        weak.load_ratio_per_mille = Some(900);
        weak.p50_read_latency_ms = Some(900);

        let ranked = policy.rank_providers(&[weak, legacy, strong], now_ms);
        assert_eq!(ranked[0].provider_id, "peer-strong");
        assert_eq!(ranked[1].provider_id, "peer-legacy");
        assert_eq!(ranked[2].provider_id, "peer-weak");
    }

    #[test]
    fn rank_providers_supports_legacy_records_without_capabilities() {
        let now_ms = 10_000;
        let policy = ProviderSelectionPolicy::default();
        let fresh = provider("peer-fresh", 9_990);
        let stale = provider("peer-stale", 8_000);

        let ranked = policy.rank_providers(&[stale.clone(), fresh.clone()], now_ms);
        assert_eq!(ranked[0].provider_id, "peer-fresh");
        assert_eq!(ranked[1].provider_id, "peer-stale");

        let fresh_score = policy.score_provider(&fresh, now_ms);
        let stale_score = policy.score_provider(&stale, now_ms);
        assert!(fresh_score.is_finite());
        assert!(stale_score.is_finite());
        assert!(fresh_score > stale_score);
    }
}
