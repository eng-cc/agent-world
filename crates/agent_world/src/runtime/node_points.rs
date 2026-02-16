use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

const DEFAULT_WEIGHT_COMPUTE: f64 = 0.45;
const DEFAULT_WEIGHT_STORAGE: f64 = 0.35;
const DEFAULT_WEIGHT_UPTIME: f64 = 0.10;
const DEFAULT_WEIGHT_RELIABILITY: f64 = 0.10;
const BYTES_PER_GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Node points settlement configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePointsConfig {
    pub epoch_duration_seconds: u64,
    pub epoch_pool_points: u64,
    pub min_self_sim_compute_units: u64,
    pub delegated_compute_multiplier: f64,
    pub maintenance_compute_multiplier: f64,
    pub weight_compute: f64,
    pub weight_storage: f64,
    pub weight_uptime: f64,
    pub weight_reliability: f64,
    pub obligation_penalty_points: f64,
}

impl Default for NodePointsConfig {
    fn default() -> Self {
        Self {
            epoch_duration_seconds: 3600,
            epoch_pool_points: 1000,
            min_self_sim_compute_units: 1,
            delegated_compute_multiplier: 1.0,
            maintenance_compute_multiplier: 1.2,
            weight_compute: DEFAULT_WEIGHT_COMPUTE,
            weight_storage: DEFAULT_WEIGHT_STORAGE,
            weight_uptime: DEFAULT_WEIGHT_UPTIME,
            weight_reliability: DEFAULT_WEIGHT_RELIABILITY,
            obligation_penalty_points: 5.0,
        }
    }
}

impl NodePointsConfig {
    fn normalized_weights(&self) -> (f64, f64, f64, f64) {
        let wc = self.weight_compute.max(0.0);
        let ws = self.weight_storage.max(0.0);
        let wu = self.weight_uptime.max(0.0);
        let wr = self.weight_reliability.max(0.0);
        let sum = wc + ws + wu + wr;
        if sum <= f64::EPSILON {
            return (
                DEFAULT_WEIGHT_COMPUTE,
                DEFAULT_WEIGHT_STORAGE,
                DEFAULT_WEIGHT_UPTIME,
                DEFAULT_WEIGHT_RELIABILITY,
            );
        }
        (wc / sum, ws / sum, wu / sum, wr / sum)
    }
}

/// A node contribution sample collected within one epoch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeContributionSample {
    pub node_id: String,
    pub self_sim_compute_units: u64,
    pub delegated_sim_compute_units: u64,
    pub world_maintenance_compute_units: u64,
    pub effective_storage_bytes: u64,
    pub uptime_seconds: u64,
    pub verify_pass_ratio: f64,
    pub availability_ratio: f64,
    pub explicit_penalty_points: f64,
}

/// Per-node settlement result for one epoch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeSettlement {
    pub node_id: String,
    pub obligation_met: bool,
    pub compute_score: f64,
    pub storage_score: f64,
    pub uptime_score: f64,
    pub reliability_score: f64,
    pub penalty_score: f64,
    pub total_score: f64,
    pub awarded_points: u64,
    pub cumulative_points: u64,
}

/// A full epoch settlement report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpochSettlementReport {
    pub epoch_index: u64,
    pub pool_points: u64,
    pub distributed_points: u64,
    pub settlements: Vec<NodeSettlement>,
}

#[derive(Debug, Clone)]
struct RemainderEntry {
    settlement_index: usize,
    node_id: String,
    fractional: f64,
}

/// In-memory node points ledger.
#[derive(Debug, Clone)]
pub struct NodePointsLedger {
    config: NodePointsConfig,
    epoch_index: u64,
    cumulative_points: BTreeMap<String, u64>,
}

impl NodePointsLedger {
    pub fn new(config: NodePointsConfig) -> Self {
        Self {
            config,
            epoch_index: 0,
            cumulative_points: BTreeMap::new(),
        }
    }

    pub fn config(&self) -> &NodePointsConfig {
        &self.config
    }

    pub fn epoch_index(&self) -> u64 {
        self.epoch_index
    }

    pub fn cumulative_points(&self, node_id: &str) -> u64 {
        *self.cumulative_points.get(node_id).unwrap_or(&0)
    }

    pub fn settle_epoch(&mut self, samples: &[NodeContributionSample]) -> EpochSettlementReport {
        let mut settlements = samples
            .iter()
            .map(|sample| self.build_settlement(sample))
            .collect::<Vec<_>>();
        let total_score = settlements
            .iter()
            .map(|settlement| settlement.total_score)
            .sum();

        let distributed_points =
            allocate_awards(self.config.epoch_pool_points, total_score, &mut settlements);

        for settlement in &mut settlements {
            let cumulative = self
                .cumulative_points
                .entry(settlement.node_id.clone())
                .or_insert(0);
            *cumulative = cumulative.saturating_add(settlement.awarded_points);
            settlement.cumulative_points = *cumulative;
        }

        let report = EpochSettlementReport {
            epoch_index: self.epoch_index,
            pool_points: self.config.epoch_pool_points,
            distributed_points,
            settlements,
        };
        self.epoch_index = self.epoch_index.saturating_add(1);
        report
    }

    fn build_settlement(&self, sample: &NodeContributionSample) -> NodeSettlement {
        let verify_pass_ratio = clamp_ratio(sample.verify_pass_ratio);
        let availability_ratio = clamp_ratio(sample.availability_ratio);
        let compute_units = sample.delegated_sim_compute_units as f64
            * self.config.delegated_compute_multiplier.max(0.0)
            + sample.world_maintenance_compute_units as f64
                * self.config.maintenance_compute_multiplier.max(0.0);
        let compute_score = compute_units.max(0.0) * verify_pass_ratio;

        let storage_gib = sample.effective_storage_bytes as f64 / BYTES_PER_GIB;
        let storage_score = storage_gib.max(0.0).sqrt() * availability_ratio;

        let uptime_score = if self.config.epoch_duration_seconds == 0 {
            0.0
        } else {
            (sample.uptime_seconds as f64 / self.config.epoch_duration_seconds as f64).min(1.0)
        };

        let reliability_score = (verify_pass_ratio + availability_ratio) / 2.0;
        let obligation_met =
            sample.self_sim_compute_units >= self.config.min_self_sim_compute_units;
        let mut penalty_score = sample.explicit_penalty_points.max(0.0);
        if !obligation_met {
            penalty_score += self.config.obligation_penalty_points.max(0.0);
        }

        let (weight_compute, weight_storage, weight_uptime, weight_reliability) =
            self.config.normalized_weights();
        let total_score = (weight_compute * compute_score
            + weight_storage * storage_score
            + weight_uptime * uptime_score
            + weight_reliability * reliability_score
            - penalty_score)
            .max(0.0);

        NodeSettlement {
            node_id: sample.node_id.clone(),
            obligation_met,
            compute_score,
            storage_score,
            uptime_score,
            reliability_score,
            penalty_score,
            total_score,
            awarded_points: 0,
            cumulative_points: 0,
        }
    }
}

impl Default for NodePointsLedger {
    fn default() -> Self {
        Self::new(NodePointsConfig::default())
    }
}

fn allocate_awards(pool_points: u64, total_score: f64, settlements: &mut [NodeSettlement]) -> u64 {
    if pool_points == 0 || total_score <= f64::EPSILON || settlements.is_empty() {
        return 0;
    }

    let mut distributed = 0u64;
    let mut remainders = Vec::with_capacity(settlements.len());

    for (index, settlement) in settlements.iter_mut().enumerate() {
        if settlement.total_score <= 0.0 {
            remainders.push(RemainderEntry {
                settlement_index: index,
                node_id: settlement.node_id.clone(),
                fractional: 0.0,
            });
            continue;
        }

        let exact_points = (pool_points as f64) * settlement.total_score / total_score;
        let floor_points = exact_points.floor() as u64;
        settlement.awarded_points = floor_points;
        distributed = distributed.saturating_add(floor_points);
        remainders.push(RemainderEntry {
            settlement_index: index,
            node_id: settlement.node_id.clone(),
            fractional: exact_points - floor_points as f64,
        });
    }

    let mut remaining = pool_points.saturating_sub(distributed);
    remainders.sort_by(|left, right| {
        right
            .fractional
            .partial_cmp(&left.fractional)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.node_id.cmp(&right.node_id))
    });

    for entry in remainders {
        if remaining == 0 {
            break;
        }
        if settlements[entry.settlement_index].total_score <= 0.0 {
            continue;
        }
        settlements[entry.settlement_index].awarded_points = settlements[entry.settlement_index]
            .awarded_points
            .saturating_add(1);
        distributed = distributed.saturating_add(1);
        remaining = remaining.saturating_sub(1);
    }

    distributed
}

fn clamp_ratio(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        NodeContributionSample, NodePointsConfig, NodePointsLedger, DEFAULT_WEIGHT_COMPUTE,
        DEFAULT_WEIGHT_RELIABILITY, DEFAULT_WEIGHT_STORAGE, DEFAULT_WEIGHT_UPTIME,
    };

    fn sample(node_id: &str) -> NodeContributionSample {
        NodeContributionSample {
            node_id: node_id.to_string(),
            self_sim_compute_units: 5,
            delegated_sim_compute_units: 0,
            world_maintenance_compute_units: 0,
            effective_storage_bytes: 0,
            uptime_seconds: 0,
            verify_pass_ratio: 1.0,
            availability_ratio: 1.0,
            explicit_penalty_points: 0.0,
        }
    }

    fn gib(value: u64) -> u64 {
        value * 1024 * 1024 * 1024
    }

    fn compute_only_config(pool: u64) -> NodePointsConfig {
        NodePointsConfig {
            epoch_pool_points: pool,
            weight_compute: 1.0,
            weight_storage: 0.0,
            weight_uptime: 0.0,
            weight_reliability: 0.0,
            ..NodePointsConfig::default()
        }
    }

    #[test]
    fn rewards_extra_compute_not_self_obligation_compute() {
        let mut ledger = NodePointsLedger::new(compute_only_config(100));
        let mut high = sample("node-high");
        high.delegated_sim_compute_units = 10;
        high.self_sim_compute_units = 5;

        let mut baseline = sample("node-baseline");
        baseline.self_sim_compute_units = 100;

        let report = ledger.settle_epoch(&[high, baseline]);
        assert_eq!(report.distributed_points, 100);
        assert_eq!(report.settlements[0].awarded_points, 100);
        assert_eq!(report.settlements[1].awarded_points, 0);
        assert_eq!(report.settlements[0].compute_score, 10.0);
        assert_eq!(report.settlements[1].compute_score, 0.0);
    }

    #[test]
    fn applies_obligation_penalty_when_self_compute_is_too_low() {
        let mut config = compute_only_config(100);
        config.min_self_sim_compute_units = 3;
        config.obligation_penalty_points = 4.0;
        let mut ledger = NodePointsLedger::new(config);

        let mut weak = sample("node-weak");
        weak.self_sim_compute_units = 2;
        weak.delegated_sim_compute_units = 10;

        let mut good = sample("node-good");
        good.self_sim_compute_units = 3;
        good.delegated_sim_compute_units = 6;

        let report = ledger.settle_epoch(&[weak, good]);
        assert_eq!(report.distributed_points, 100);
        assert!(!report.settlements[0].obligation_met);
        assert!(report.settlements[1].obligation_met);
        assert_eq!(report.settlements[0].penalty_score, 4.0);
        assert_eq!(report.settlements[0].total_score, 6.0);
        assert_eq!(report.settlements[1].total_score, 6.0);
        assert_eq!(report.settlements[0].awarded_points, 50);
        assert_eq!(report.settlements[1].awarded_points, 50);
    }

    #[test]
    fn storage_score_uses_sqrt_curve_with_availability() {
        let mut config = NodePointsConfig::default();
        config.epoch_pool_points = 100;
        config.weight_compute = 0.0;
        config.weight_storage = 1.0;
        config.weight_uptime = 0.0;
        config.weight_reliability = 0.0;
        let mut ledger = NodePointsLedger::new(config);

        let mut one_gib = sample("node-a");
        one_gib.effective_storage_bytes = gib(1);

        let mut four_gib = sample("node-b");
        four_gib.effective_storage_bytes = gib(4);

        let mut nine_gib_half = sample("node-c");
        nine_gib_half.effective_storage_bytes = gib(9);
        nine_gib_half.availability_ratio = 0.5;

        let report = ledger.settle_epoch(&[one_gib, four_gib, nine_gib_half]);
        assert_eq!(report.distributed_points, 100);
        assert_eq!(report.settlements[0].storage_score, 1.0);
        assert_eq!(report.settlements[1].storage_score, 2.0);
        assert_eq!(report.settlements[2].storage_score, 1.5);
        assert!(report.settlements[1].awarded_points > report.settlements[2].awarded_points);
        assert!(report.settlements[2].awarded_points > report.settlements[0].awarded_points);
    }

    #[test]
    fn remainder_distribution_is_stable_when_scores_tie() {
        let mut ledger = NodePointsLedger::new(compute_only_config(10));

        let mut a = sample("node-a");
        a.delegated_sim_compute_units = 1;
        let mut b = sample("node-b");
        b.delegated_sim_compute_units = 1;
        let mut c = sample("node-c");
        c.delegated_sim_compute_units = 1;

        let report = ledger.settle_epoch(&[a, b, c]);
        assert_eq!(report.distributed_points, 10);
        assert_eq!(report.settlements[0].awarded_points, 4);
        assert_eq!(report.settlements[1].awarded_points, 3);
        assert_eq!(report.settlements[2].awarded_points, 3);
    }

    #[test]
    fn cumulative_points_accumulate_across_epochs() {
        let mut ledger = NodePointsLedger::new(compute_only_config(10));
        let mut a = sample("node-a");
        a.delegated_sim_compute_units = 1;

        let first = ledger.settle_epoch(&[a.clone()]);
        assert_eq!(first.epoch_index, 0);
        assert_eq!(first.settlements[0].awarded_points, 10);
        assert_eq!(first.settlements[0].cumulative_points, 10);

        let second = ledger.settle_epoch(&[a]);
        assert_eq!(second.epoch_index, 1);
        assert_eq!(second.settlements[0].awarded_points, 10);
        assert_eq!(second.settlements[0].cumulative_points, 20);
        assert_eq!(ledger.cumulative_points("node-a"), 20);
        assert_eq!(ledger.epoch_index(), 2);
    }

    #[test]
    fn uses_default_weights_when_input_weights_are_all_zero() {
        let mut config = NodePointsConfig::default();
        config.epoch_pool_points = 100;
        config.weight_compute = 0.0;
        config.weight_storage = 0.0;
        config.weight_uptime = 0.0;
        config.weight_reliability = 0.0;
        let mut ledger = NodePointsLedger::new(config);

        let mut rich_compute = sample("node-compute");
        rich_compute.delegated_sim_compute_units = 10;

        let mut rich_storage = sample("node-storage");
        rich_storage.effective_storage_bytes = gib(16);

        let report = ledger.settle_epoch(&[rich_compute, rich_storage]);
        assert_eq!(report.distributed_points, 100);
        let compute_settlement = &report.settlements[0];
        let storage_settlement = &report.settlements[1];
        assert!(compute_settlement.total_score > 0.0);
        assert!(storage_settlement.total_score > 0.0);
        assert_eq!(
            DEFAULT_WEIGHT_COMPUTE
                + DEFAULT_WEIGHT_STORAGE
                + DEFAULT_WEIGHT_UPTIME
                + DEFAULT_WEIGHT_RELIABILITY,
            1.0
        );
    }
}
