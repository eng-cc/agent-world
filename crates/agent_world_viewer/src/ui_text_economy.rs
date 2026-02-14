use std::collections::BTreeMap;

use agent_world::simulator::{
    AssetKind, PowerEvent, RejectReason, ResourceKind, WorldEvent, WorldEventKind, WorldSnapshot,
};

const ECONOMY_EVENT_WINDOW: usize = 120;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct EconomyRollup {
    transfer_events: usize,
    power_trade_events: usize,
    refine_events: usize,
    insufficient_rejects: usize,
    flow_by_kind: BTreeMap<ResourceKind, i64>,
    shortfall_by_kind: BTreeMap<ResourceKind, i64>,
    stock_by_kind: BTreeMap<ResourceKind, i64>,
    power_trade_settlement: i64,
    power_loss: i64,
    refine_electricity_cost: i64,
}

pub(super) fn economy_dashboard_summary(
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> Option<String> {
    let mut rollup = EconomyRollup::default();
    let mut has_economy_signals = false;

    for event in events.iter().rev().take(ECONOMY_EVENT_WINDOW) {
        match &event.kind {
            WorldEventKind::ResourceTransferred { kind, amount, .. } => {
                has_economy_signals = true;
                rollup.transfer_events += 1;
                add_amount(&mut rollup.flow_by_kind, *kind, amount.abs());
            }
            WorldEventKind::Power(PowerEvent::PowerTransferred {
                amount,
                loss,
                price_per_pu,
                ..
            }) => {
                has_economy_signals = true;
                rollup.transfer_events += 1;
                rollup.power_trade_events += 1;
                add_amount(
                    &mut rollup.flow_by_kind,
                    ResourceKind::Electricity,
                    amount.abs(),
                );
                rollup.power_loss = rollup.power_loss.saturating_add((*loss).max(0));
                let settlement = amount.abs().saturating_mul((*price_per_pu).max(0));
                rollup.power_trade_settlement =
                    rollup.power_trade_settlement.saturating_add(settlement);
            }
            WorldEventKind::CompoundRefined {
                electricity_cost, ..
            } => {
                has_economy_signals = true;
                rollup.refine_events += 1;
                rollup.refine_electricity_cost = rollup
                    .refine_electricity_cost
                    .saturating_add((*electricity_cost).max(0));
            }
            WorldEventKind::ActionRejected {
                reason:
                    RejectReason::InsufficientResource {
                        kind,
                        requested,
                        available,
                        ..
                    },
            } => {
                has_economy_signals = true;
                rollup.insufficient_rejects += 1;
                let shortfall = requested.saturating_sub(*available).max(0);
                add_amount(&mut rollup.shortfall_by_kind, *kind, shortfall);
            }
            _ => {}
        }
    }

    if !has_economy_signals {
        return None;
    }

    if let Some(snapshot) = snapshot {
        collect_resource_stocks(snapshot, &mut rollup.stock_by_kind);
    }

    let electricity_flow = amount_or_zero(&rollup.flow_by_kind, ResourceKind::Electricity);
    let hardware_flow = amount_or_zero(&rollup.flow_by_kind, ResourceKind::Hardware);
    let data_flow = amount_or_zero(&rollup.flow_by_kind, ResourceKind::Data);
    let outbound_value_proxy = electricity_flow
        .saturating_add(hardware_flow.saturating_mul(4))
        .saturating_add(data_flow.saturating_mul(2));
    let total_cost_proxy = rollup
        .power_trade_settlement
        .saturating_add(rollup.refine_electricity_cost)
        .saturating_add(rollup.power_loss);
    let margin_proxy = outbound_value_proxy.saturating_sub(total_cost_proxy);

    let mut lines = vec!["Economy Dashboard:".to_string()];
    lines.push("Supply & Demand:".to_string());
    for kind in [
        ResourceKind::Electricity,
        ResourceKind::Hardware,
        ResourceKind::Data,
    ] {
        let stock = amount_or_zero(&rollup.stock_by_kind, kind);
        let flow = amount_or_zero(&rollup.flow_by_kind, kind);
        let shortfall = amount_or_zero(&rollup.shortfall_by_kind, kind);
        lines.push(format!(
            "- {:?}: stock={} flow={} shortfall={} health={}",
            kind,
            stock,
            flow,
            shortfall,
            inventory_health_label(stock, flow, shortfall)
        ));
    }
    lines.push(format!(
        "- Insufficient Rejects(Recent): {}",
        rollup.insufficient_rejects
    ));

    lines.push("".to_string());
    lines.push("Cost & Revenue Proxy:".to_string());
    lines.push(format!(
        "- Transfer Events(Recent): {}",
        rollup.transfer_events
    ));
    lines.push(format!(
        "- Power Trades(Recent): {}",
        rollup.power_trade_events
    ));
    lines.push(format!(
        "- Power Trade Settlement(Recent): {}",
        rollup.power_trade_settlement
    ));
    lines.push(format!(
        "- Refine Electricity Cost(Recent): {}",
        rollup.refine_electricity_cost
    ));
    lines.push(format!("- Power Loss(Recent): {}", rollup.power_loss));
    lines.push(format!(
        "- Outbound Value Proxy(Recent): {outbound_value_proxy}"
    ));
    lines.push(format!("- Margin Proxy(Recent): {margin_proxy}"));

    Some(lines.join("\n"))
}

fn add_amount(map: &mut BTreeMap<ResourceKind, i64>, kind: ResourceKind, amount: i64) {
    let entry = map.entry(kind).or_insert(0);
    *entry = entry.saturating_add(amount.max(0));
}

fn amount_or_zero(map: &BTreeMap<ResourceKind, i64>, kind: ResourceKind) -> i64 {
    *map.get(&kind).unwrap_or(&0)
}

fn collect_resource_stocks(snapshot: &WorldSnapshot, out: &mut BTreeMap<ResourceKind, i64>) {
    for agent in snapshot.model.agents.values() {
        for (kind, amount) in &agent.resources.amounts {
            add_amount(out, *kind, *amount);
        }
    }

    for location in snapshot.model.locations.values() {
        for (kind, amount) in &location.resources.amounts {
            add_amount(out, *kind, *amount);
        }
    }

    for asset in snapshot.model.assets.values() {
        let AssetKind::Resource { kind } = asset.kind;
        add_amount(out, kind, asset.quantity);
    }
}

fn inventory_health_label(stock: i64, flow: i64, shortfall: i64) -> &'static str {
    if stock <= 0 {
        return "critical";
    }
    let pressure = flow.saturating_add(shortfall).max(1);
    let ratio = stock as f64 / pressure as f64;
    if ratio < 0.5 {
        "critical"
    } else if ratio < 2.0 {
        "warn"
    } else {
        "stable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        Agent, ChunkRuntimeConfig, Location, ResourceOwner, WorldConfig, WorldModel, WorldSnapshot,
        CHUNK_GENERATION_SCHEMA_VERSION, SNAPSHOT_VERSION,
    };

    #[test]
    fn economy_dashboard_summary_returns_none_without_economy_signals() {
        assert!(economy_dashboard_summary(None, &[]).is_none());
    }

    #[test]
    fn economy_dashboard_summary_reports_supply_demand_and_cost_proxy() {
        let mut model = WorldModel::default();
        let mut agent = Agent::new("agent-1", "loc-a", GeoPos::new(0.0, 0.0, 0.0));
        agent.resources.set(ResourceKind::Electricity, 25).ok();
        model.agents.insert("agent-1".to_string(), agent);

        let mut location = Location::new("loc-a", "Alpha", GeoPos::new(0.0, 0.0, 0.0));
        location.resources.set(ResourceKind::Hardware, 9).ok();
        model.locations.insert("loc-a".to_string(), location);

        let snapshot = WorldSnapshot {
            version: SNAPSHOT_VERSION,
            chunk_generation_schema_version: CHUNK_GENERATION_SCHEMA_VERSION,
            time: 30,
            config: WorldConfig::default(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: 8,
            next_action_id: 2,
            pending_actions: Vec::new(),
            journal_len: 4,
        };

        let events = vec![
            WorldEvent {
                id: 1,
                time: 21,
                kind: WorldEventKind::ResourceTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    to: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    kind: ResourceKind::Hardware,
                    amount: 6,
                },
            },
            WorldEvent {
                id: 2,
                time: 22,
                kind: WorldEventKind::Power(PowerEvent::PowerTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    to: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    amount: 10,
                    loss: 2,
                    price_per_pu: 3,
                }),
            },
            WorldEvent {
                id: 3,
                time: 23,
                kind: WorldEventKind::CompoundRefined {
                    owner: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    compound_mass_g: 12,
                    electricity_cost: 4,
                    hardware_output: 3,
                },
            },
            WorldEvent {
                id: 4,
                time: 24,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InsufficientResource {
                        owner: ResourceOwner::Agent {
                            agent_id: "agent-1".to_string(),
                        },
                        kind: ResourceKind::Data,
                        requested: 8,
                        available: 3,
                    },
                },
            },
        ];

        let summary =
            economy_dashboard_summary(Some(&snapshot), &events).expect("economy summary exists");
        assert!(summary.contains("Economy Dashboard:"));
        assert!(summary.contains("Supply & Demand:"));
        assert!(summary.contains("Electricity: stock=25"));
        assert!(summary.contains("Hardware: stock=9"));
        assert!(summary.contains("Insufficient Rejects(Recent): 1"));
        assert!(summary.contains("Cost & Revenue Proxy:"));
        assert!(summary.contains("Power Trade Settlement(Recent): 30"));
        assert!(summary.contains("Refine Electricity Cost(Recent): 4"));
        assert!(summary.contains("Power Loss(Recent): 2"));
    }
}
