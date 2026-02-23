use std::collections::{BTreeMap, BTreeSet};

use super::distributed_dht::{DistributedDht, ProviderRecord};
use super::error::WorldError;
use super::provider_selection::ProviderSelectionPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplicaMaintenancePolicy {
    pub target_replicas_per_blob: usize,
    pub max_repairs_per_round: usize,
    pub max_rebalances_per_round: usize,
    pub rebalance_source_load_min_per_mille: u16,
    pub rebalance_target_load_max_per_mille: u16,
}

impl Default for ReplicaMaintenancePolicy {
    fn default() -> Self {
        Self {
            target_replicas_per_blob: 3,
            max_repairs_per_round: 32,
            max_rebalances_per_round: 32,
            rebalance_source_load_min_per_mille: 850,
            rebalance_target_load_max_per_mille: 450,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicaTransferKind {
    Repair,
    Rebalance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicaTransferTask {
    pub kind: ReplicaTransferKind,
    pub content_hash: String,
    pub source_provider_id: String,
    pub target_provider_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplicaMaintenancePlan {
    pub repair_tasks: Vec<ReplicaTransferTask>,
    pub rebalance_tasks: Vec<ReplicaTransferTask>,
    pub warnings: Vec<String>,
}

pub fn plan_replica_maintenance(
    dht: &impl DistributedDht,
    world_id: &str,
    content_hashes: &[String],
    policy: ReplicaMaintenancePolicy,
) -> Result<ReplicaMaintenancePlan, WorldError> {
    validate_policy(policy)?;

    let required_hashes = normalize_hashes(content_hashes);
    if required_hashes.is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "replica maintenance requires at least one content hash".to_string(),
        });
    }

    let mut providers_by_hash: BTreeMap<String, Vec<ProviderRecord>> = BTreeMap::new();
    for content_hash in &required_hashes {
        let providers = dedupe_providers(dht.get_providers(world_id, content_hash)?);
        providers_by_hash.insert(content_hash.clone(), providers);
    }

    let mut plan = ReplicaMaintenancePlan::default();
    plan_repair_tasks(&providers_by_hash, policy, &mut plan);
    plan_rebalance_tasks(&providers_by_hash, policy, &mut plan);
    Ok(plan)
}

fn validate_policy(policy: ReplicaMaintenancePolicy) -> Result<(), WorldError> {
    if policy.target_replicas_per_blob == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "replica maintenance policy requires target_replicas_per_blob > 0".to_string(),
        });
    }
    Ok(())
}

fn plan_repair_tasks(
    providers_by_hash: &BTreeMap<String, Vec<ProviderRecord>>,
    policy: ReplicaMaintenancePolicy,
    plan: &mut ReplicaMaintenancePlan,
) {
    if policy.max_repairs_per_round == 0 {
        return;
    }

    let all_candidates = collect_global_candidates(providers_by_hash);
    let selector = ProviderSelectionPolicy::default();

    for (content_hash, providers) in providers_by_hash {
        if plan.repair_tasks.len() >= policy.max_repairs_per_round {
            return;
        }

        let current_replica_count = providers.len();
        if current_replica_count >= policy.target_replicas_per_blob {
            continue;
        }

        let Some(source) = selector
            .rank_providers(providers, selection_now_ms(providers))
            .into_iter()
            .next()
        else {
            plan.warnings.push(format!(
                "repair planning skipped for content_hash={content_hash}: no source provider"
            ));
            continue;
        };

        let mut selected_targets: BTreeSet<String> =
            providers.iter().map(|p| p.provider_id.clone()).collect();
        let target_candidates = selector.rank_providers(
            &all_candidates
                .iter()
                .filter(|candidate| !selected_targets.contains(&candidate.provider_id))
                .cloned()
                .collect::<Vec<_>>(),
            selection_now_ms(&all_candidates),
        );

        let needed = policy
            .target_replicas_per_blob
            .saturating_sub(current_replica_count);
        let mut produced = 0usize;
        for target in target_candidates {
            if produced >= needed || plan.repair_tasks.len() >= policy.max_repairs_per_round {
                break;
            }
            if !selected_targets.insert(target.provider_id.clone()) {
                continue;
            }
            plan.repair_tasks.push(ReplicaTransferTask {
                kind: ReplicaTransferKind::Repair,
                content_hash: content_hash.clone(),
                source_provider_id: source.provider_id.clone(),
                target_provider_id: target.provider_id,
            });
            produced = produced.saturating_add(1);
        }

        if produced < needed {
            plan.warnings.push(format!(
                "repair planning insufficient targets for content_hash={content_hash}: needed={needed}, planned={produced}"
            ));
        }
    }
}

fn plan_rebalance_tasks(
    providers_by_hash: &BTreeMap<String, Vec<ProviderRecord>>,
    policy: ReplicaMaintenancePolicy,
    plan: &mut ReplicaMaintenancePlan,
) {
    if policy.max_rebalances_per_round == 0 {
        return;
    }

    let all_candidates = collect_global_candidates(providers_by_hash);
    let underloaded: Vec<ProviderRecord> = all_candidates
        .iter()
        .filter(|record| {
            record
                .load_ratio_per_mille
                .map(|load| load <= policy.rebalance_target_load_max_per_mille)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let mut existing_tasks: BTreeSet<(String, String)> = plan
        .repair_tasks
        .iter()
        .map(|task| (task.content_hash.clone(), task.target_provider_id.clone()))
        .collect();

    for (content_hash, providers) in providers_by_hash {
        if plan.rebalance_tasks.len() >= policy.max_rebalances_per_round {
            return;
        }

        let source = providers
            .iter()
            .filter(|record| {
                record
                    .load_ratio_per_mille
                    .map(|load| load >= policy.rebalance_source_load_min_per_mille)
                    .unwrap_or(false)
            })
            .cloned()
            .max_by_key(|record| {
                (
                    record.load_ratio_per_mille.unwrap_or(0),
                    record.last_seen_ms,
                    std::cmp::Reverse(record.provider_id.clone()),
                )
            });
        let Some(source) = source else {
            continue;
        };

        let occupied: BTreeSet<String> = providers.iter().map(|p| p.provider_id.clone()).collect();
        let target = underloaded
            .iter()
            .filter(|candidate| !occupied.contains(&candidate.provider_id))
            .cloned()
            .min_by_key(|record| {
                (
                    record.load_ratio_per_mille.unwrap_or(u16::MAX),
                    std::cmp::Reverse(record.last_seen_ms),
                    record.provider_id.clone(),
                )
            });
        let Some(target) = target else {
            continue;
        };

        let task_key = (content_hash.clone(), target.provider_id.clone());
        if existing_tasks.contains(&task_key) {
            continue;
        }

        existing_tasks.insert(task_key);
        plan.rebalance_tasks.push(ReplicaTransferTask {
            kind: ReplicaTransferKind::Rebalance,
            content_hash: content_hash.clone(),
            source_provider_id: source.provider_id,
            target_provider_id: target.provider_id,
        });
    }
}

fn collect_global_candidates(
    providers_by_hash: &BTreeMap<String, Vec<ProviderRecord>>,
) -> Vec<ProviderRecord> {
    let mut by_id: BTreeMap<String, ProviderRecord> = BTreeMap::new();
    for providers in providers_by_hash.values() {
        for record in providers {
            by_id
                .entry(record.provider_id.clone())
                .and_modify(|existing| {
                    if record.last_seen_ms > existing.last_seen_ms {
                        *existing = record.clone();
                    }
                })
                .or_insert_with(|| record.clone());
        }
    }
    by_id.into_values().collect()
}

fn dedupe_providers(providers: Vec<ProviderRecord>) -> Vec<ProviderRecord> {
    let mut by_id: BTreeMap<String, ProviderRecord> = BTreeMap::new();
    for record in providers {
        by_id
            .entry(record.provider_id.clone())
            .and_modify(|existing| {
                if record.last_seen_ms > existing.last_seen_ms {
                    *existing = record.clone();
                }
            })
            .or_insert(record);
    }
    by_id.into_values().collect()
}

fn normalize_hashes(content_hashes: &[String]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for content_hash in content_hashes {
        set.insert(content_hash.clone());
    }
    set.into_iter().collect()
}

fn selection_now_ms(providers: &[ProviderRecord]) -> i64 {
    providers
        .iter()
        .map(|record| record.last_seen_ms)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use agent_world_proto::distributed as proto_distributed;

    use super::*;
    use crate::proto_dht;

    #[derive(Clone, Default)]
    struct StaticProvidersDht {
        providers_by_hash: HashMap<String, Vec<ProviderRecord>>,
    }

    impl StaticProvidersDht {
        fn with_providers_by_hash(providers_by_hash: HashMap<String, Vec<ProviderRecord>>) -> Self {
            Self { providers_by_hash }
        }
    }

    impl proto_dht::DistributedDht<WorldError> for StaticProvidersDht {
        fn publish_provider(
            &self,
            _world_id: &str,
            _content_hash: &str,
            _provider_id: &str,
        ) -> Result<(), WorldError> {
            Ok(())
        }

        fn get_providers(
            &self,
            _world_id: &str,
            content_hash: &str,
        ) -> Result<Vec<ProviderRecord>, WorldError> {
            Ok(self
                .providers_by_hash
                .get(content_hash)
                .cloned()
                .unwrap_or_default())
        }

        fn put_world_head(
            &self,
            _world_id: &str,
            _head: &proto_distributed::WorldHeadAnnounce,
        ) -> Result<(), WorldError> {
            Ok(())
        }

        fn get_world_head(
            &self,
            _world_id: &str,
        ) -> Result<Option<proto_distributed::WorldHeadAnnounce>, WorldError> {
            Ok(None)
        }

        fn put_membership_directory(
            &self,
            _world_id: &str,
            _snapshot: &super::super::distributed_dht::MembershipDirectorySnapshot,
        ) -> Result<(), WorldError> {
            Ok(())
        }

        fn get_membership_directory(
            &self,
            _world_id: &str,
        ) -> Result<Option<super::super::distributed_dht::MembershipDirectorySnapshot>, WorldError>
        {
            Ok(None)
        }
    }

    fn provider(provider_id: &str, load_ratio_per_mille: Option<u16>) -> ProviderRecord {
        ProviderRecord {
            provider_id: provider_id.to_string(),
            last_seen_ms: 1_000,
            storage_total_bytes: Some(1_000),
            storage_available_bytes: Some(500),
            uptime_ratio_per_mille: Some(990),
            challenge_pass_ratio_per_mille: Some(980),
            load_ratio_per_mille,
            p50_read_latency_ms: Some(20),
        }
    }

    fn map(entries: &[(&str, Vec<ProviderRecord>)]) -> HashMap<String, Vec<ProviderRecord>> {
        let mut out = HashMap::new();
        for (hash, providers) in entries {
            out.insert((*hash).to_string(), providers.clone());
        }
        out
    }

    #[test]
    fn plan_replica_maintenance_creates_repair_tasks_for_under_replicated_blob() {
        let dht = StaticProvidersDht::with_providers_by_hash(map(&[
            (
                "hash-a",
                vec![provider("peer-1", Some(300)), provider("peer-2", Some(400))],
            ),
            ("hash-b", vec![provider("peer-1", Some(300))]),
        ]));
        let hashes = vec!["hash-a".to_string(), "hash-b".to_string()];

        let plan = plan_replica_maintenance(
            &dht,
            "w1",
            &hashes,
            ReplicaMaintenancePolicy {
                target_replicas_per_blob: 2,
                max_repairs_per_round: 8,
                max_rebalances_per_round: 0,
                ..ReplicaMaintenancePolicy::default()
            },
        )
        .expect("plan");

        assert!(!plan.repair_tasks.is_empty());
        assert!(plan
            .repair_tasks
            .iter()
            .any(|task| task.content_hash == "hash-b"));
        assert!(plan.rebalance_tasks.is_empty());
    }

    #[test]
    fn plan_replica_maintenance_creates_rebalance_tasks_for_overloaded_provider() {
        let dht = StaticProvidersDht::with_providers_by_hash(map(&[
            (
                "hash-a",
                vec![
                    provider("peer-hot", Some(950)),
                    provider("peer-cool", Some(200)),
                ],
            ),
            (
                "hash-b",
                vec![
                    provider("peer-hot", Some(940)),
                    provider("peer-warm", Some(300)),
                ],
            ),
            (
                "hash-c",
                vec![
                    provider("peer-hot", Some(930)),
                    provider("peer-cool", Some(220)),
                ],
            ),
        ]));
        let hashes = vec![
            "hash-a".to_string(),
            "hash-b".to_string(),
            "hash-c".to_string(),
        ];

        let plan = plan_replica_maintenance(
            &dht,
            "w1",
            &hashes,
            ReplicaMaintenancePolicy {
                target_replicas_per_blob: 2,
                max_repairs_per_round: 0,
                max_rebalances_per_round: 8,
                rebalance_source_load_min_per_mille: 900,
                rebalance_target_load_max_per_mille: 350,
            },
        )
        .expect("plan");

        assert!(plan.repair_tasks.is_empty());
        assert!(!plan.rebalance_tasks.is_empty());
        assert!(plan
            .rebalance_tasks
            .iter()
            .all(|task| task.kind == ReplicaTransferKind::Rebalance));
    }

    #[test]
    fn plan_replica_maintenance_writes_warning_when_no_target_candidate() {
        let dht = StaticProvidersDht::with_providers_by_hash(map(&[(
            "hash-a",
            vec![provider("peer-only", Some(500))],
        )]));
        let hashes = vec!["hash-a".to_string()];

        let plan = plan_replica_maintenance(
            &dht,
            "w1",
            &hashes,
            ReplicaMaintenancePolicy {
                target_replicas_per_blob: 3,
                max_repairs_per_round: 8,
                max_rebalances_per_round: 0,
                ..ReplicaMaintenancePolicy::default()
            },
        )
        .expect("plan");

        assert!(plan.repair_tasks.is_empty());
        assert!(!plan.warnings.is_empty());
    }
}
