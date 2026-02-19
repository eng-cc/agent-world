use std::collections::BTreeMap;

use agent_world::simulator::{
    ModuleVisualEntity, PowerEvent, ResourceKind, ResourceOwner, WorldEvent, WorldEventKind,
    WorldSnapshot,
};

const INDUSTRIAL_EVENT_WINDOW: usize = 96;
const INDUSTRIAL_TOP_ROUTE_LIMIT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndustrialVisualRole {
    Factory,
    Recipe,
    Product,
    Logistics,
    Other,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ProductionVisualCounts {
    factory: usize,
    recipe: usize,
    product: usize,
    logistics: usize,
}

impl ProductionVisualCounts {
    fn add_role(&mut self, role: IndustrialVisualRole) {
        match role {
            IndustrialVisualRole::Factory => self.factory += 1,
            IndustrialVisualRole::Recipe => self.recipe += 1,
            IndustrialVisualRole::Product => self.product += 1,
            IndustrialVisualRole::Logistics => self.logistics += 1,
            IndustrialVisualRole::Other => {}
        }
    }

    fn total(&self) -> usize {
        self.factory + self.recipe + self.product + self.logistics
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct LogisticsRouteStats {
    transfer_events: usize,
    electricity: i64,
    data: i64,
    power: i64,
    power_loss: i64,
}

pub(super) fn industrial_ops_summary(
    snapshot: Option<&WorldSnapshot>,
    events: &[WorldEvent],
) -> Option<String> {
    let mut visuals = ProductionVisualCounts::default();
    if let Some(snapshot) = snapshot {
        for entity in snapshot.model.module_visual_entities.values() {
            visuals.add_role(classify_visual_role(entity));
        }
    }

    let mut routes = BTreeMap::<(String, String), LogisticsRouteStats>::new();
    let mut recent_refine_events = 0_usize;
    let mut recent_line_updates = 0_usize;
    let mut recent_hardware_output = 0_i64;
    let mut transfer_events = 0_usize;
    let mut total_power_moved = 0_i64;
    let mut total_power_loss = 0_i64;

    for event in events.iter().rev().take(INDUSTRIAL_EVENT_WINDOW) {
        match &event.kind {
            WorldEventKind::CompoundRefined {
                hardware_output, ..
            } => {
                recent_refine_events += 1;
                recent_hardware_output = recent_hardware_output.saturating_add(*hardware_output);
            }
            WorldEventKind::ModuleVisualEntityUpserted { entity } => {
                if classify_visual_role(entity) != IndustrialVisualRole::Other {
                    recent_line_updates += 1;
                }
            }
            WorldEventKind::ResourceTransferred {
                from,
                to,
                kind,
                amount,
            } => {
                transfer_events += 1;
                let entry = routes
                    .entry((owner_label(from), owner_label(to)))
                    .or_default();
                entry.transfer_events += 1;
                match kind {
                    ResourceKind::Electricity => {
                        entry.electricity = entry.electricity.saturating_add(*amount);
                    }
                    ResourceKind::Data => {
                        entry.data = entry.data.saturating_add(*amount);
                    }
                }
            }
            WorldEventKind::Power(PowerEvent::PowerTransferred {
                from,
                to,
                amount,
                loss,
                ..
            }) => {
                transfer_events += 1;
                total_power_moved = total_power_moved.saturating_add(*amount);
                total_power_loss = total_power_loss.saturating_add(*loss);

                let entry = routes
                    .entry((owner_label(from), owner_label(to)))
                    .or_default();
                entry.transfer_events += 1;
                entry.power = entry.power.saturating_add(*amount);
                entry.power_loss = entry.power_loss.saturating_add(*loss);
            }
            _ => {}
        }
    }

    let has_production_signals =
        visuals.total() > 0 || recent_refine_events > 0 || recent_line_updates > 0;
    let has_logistics_signals = !routes.is_empty() || transfer_events > 0 || total_power_moved > 0;
    if !has_production_signals && !has_logistics_signals {
        return None;
    }

    let mut lines = vec!["Industrial Ops:".to_string()];

    lines.push("Production Lines:".to_string());
    lines.push(format!("- Factory Visuals: {}", visuals.factory));
    lines.push(format!("- Recipe Visuals: {}", visuals.recipe));
    lines.push(format!("- Product Visuals: {}", visuals.product));
    lines.push(format!("- Logistics Visuals: {}", visuals.logistics));
    lines.push(format!("- Recent Refine Events: {recent_refine_events}"));
    lines.push(format!("- Recent Line Updates: {recent_line_updates}"));
    lines.push(format!("- Refine Output(Recent): {recent_hardware_output}"));

    lines.push("".to_string());
    lines.push("Logistics Routes:".to_string());
    lines.push(format!("- Active Routes: {}", routes.len()));
    lines.push(format!("- Transfer Events: {transfer_events}"));
    lines.push(format!(
        "- Power Moved: {} (loss={})",
        total_power_moved, total_power_loss
    ));

    let mut top_routes: Vec<_> = routes.into_iter().collect();
    top_routes.sort_by(|left, right| {
        route_weight(&right.1)
            .cmp(&route_weight(&left.1))
            .then_with(|| right.1.transfer_events.cmp(&left.1.transfer_events))
            .then_with(|| left.0.cmp(&right.0))
    });

    for ((from, to), stats) in top_routes.into_iter().take(INDUSTRIAL_TOP_ROUTE_LIMIT) {
        lines.push(format!(
            "- Route {} -> {} moves={} electricity={} data={} power={} loss={}",
            from,
            to,
            stats.transfer_events,
            stats.electricity,
            stats.data,
            stats.power,
            stats.power_loss
        ));
    }

    Some(lines.join("\n"))
}

fn classify_visual_role(entity: &ModuleVisualEntity) -> IndustrialVisualRole {
    let module_id = entity.module_id.to_ascii_lowercase();
    let kind = entity.kind.to_ascii_lowercase();

    if module_id.contains("recipe") || kind.contains("recipe") {
        return IndustrialVisualRole::Recipe;
    }

    if module_id.contains("product") || kind.contains("product") {
        return IndustrialVisualRole::Product;
    }

    if module_id.contains("logistics")
        || kind.contains("logistics")
        || module_id.contains("transport")
        || kind.contains("relay")
    {
        return IndustrialVisualRole::Logistics;
    }

    if module_id.contains("factory")
        || kind.contains("factory")
        || module_id.contains("miner")
        || module_id.contains("smelter")
        || module_id.contains("assembler")
    {
        return IndustrialVisualRole::Factory;
    }

    IndustrialVisualRole::Other
}

fn owner_label(owner: &ResourceOwner) -> String {
    match owner {
        ResourceOwner::Agent { agent_id } => format!("agent::{agent_id}"),
        ResourceOwner::Location { location_id } => format!("location::{location_id}"),
    }
}

fn route_weight(stats: &LogisticsRouteStats) -> i64 {
    stats
        .electricity
        .abs()
        .saturating_add(stats.data.abs())
        .saturating_add(stats.power.abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        ChunkRuntimeConfig, ModuleVisualAnchor, ModuleVisualEntity, WorldConfig, WorldModel,
        WorldSnapshot, CHUNK_GENERATION_SCHEMA_VERSION, SNAPSHOT_VERSION,
    };

    #[test]
    fn industrial_ops_summary_returns_none_without_industrial_signals() {
        assert!(industrial_ops_summary(None, &[]).is_none());
    }

    #[test]
    fn industrial_ops_summary_aggregates_production_and_routes() {
        let mut model = WorldModel::default();
        model.module_visual_entities.insert(
            "factory-1".to_string(),
            ModuleVisualEntity {
                entity_id: "factory-1".to_string(),
                module_id: "m4.factory.smelter.mk1".to_string(),
                kind: "factory".to_string(),
                label: Some("Smelter".to_string()),
                anchor: ModuleVisualAnchor::Location {
                    location_id: "loc-a".to_string(),
                },
            },
        );
        model.module_visual_entities.insert(
            "recipe-1".to_string(),
            ModuleVisualEntity {
                entity_id: "recipe-1".to_string(),
                module_id: "m4.recipe.smelter.iron_ingot".to_string(),
                kind: "recipe".to_string(),
                label: Some("Iron Ingot".to_string()),
                anchor: ModuleVisualAnchor::Location {
                    location_id: "loc-a".to_string(),
                },
            },
        );
        model.module_visual_entities.insert(
            "product-1".to_string(),
            ModuleVisualEntity {
                entity_id: "product-1".to_string(),
                module_id: "m4.product.component.motor_mk1".to_string(),
                kind: "product".to_string(),
                label: Some("Motor".to_string()),
                anchor: ModuleVisualAnchor::Location {
                    location_id: "loc-b".to_string(),
                },
            },
        );

        let snapshot = WorldSnapshot {
            version: SNAPSHOT_VERSION,
            chunk_generation_schema_version: CHUNK_GENERATION_SCHEMA_VERSION,
            time: 18,
            config: WorldConfig::default(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: 5,
            next_action_id: 3,
            pending_actions: Vec::new(),
            journal_len: 4,
        };

        let events = vec![
            WorldEvent {
                id: 1,
                time: 12,
                kind: WorldEventKind::CompoundRefined {
                    owner: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    compound_mass_g: 14,
                    electricity_cost: 2,
                    hardware_output: 9,
                },
            },
            WorldEvent {
                id: 2,
                time: 13,
                kind: WorldEventKind::ModuleVisualEntityUpserted {
                    entity: ModuleVisualEntity {
                        entity_id: "factory-2".to_string(),
                        module_id: "m4.factory.assembler.mk1".to_string(),
                        kind: "factory".to_string(),
                        label: Some("Assembler".to_string()),
                        anchor: ModuleVisualAnchor::Absolute {
                            pos: GeoPos::new(10.0, 0.0, 10.0),
                        },
                    },
                },
            },
            WorldEvent {
                id: 3,
                time: 14,
                kind: WorldEventKind::ResourceTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    to: ResourceOwner::Location {
                        location_id: "loc-b".to_string(),
                    },
                    kind: ResourceKind::Data,
                    amount: 5,
                },
            },
            WorldEvent {
                id: 4,
                time: 15,
                kind: WorldEventKind::Power(PowerEvent::PowerTransferred {
                    from: ResourceOwner::Location {
                        location_id: "loc-a".to_string(),
                    },
                    to: ResourceOwner::Location {
                        location_id: "loc-b".to_string(),
                    },
                    amount: 11,
                    loss: 1,
                    price_per_pu: 0,
                }),
            },
        ];

        let summary =
            industrial_ops_summary(Some(&snapshot), &events).expect("industrial summary exists");
        assert!(summary.contains("Production Lines:"));
        assert!(summary.contains("- Factory Visuals: 1"));
        assert!(summary.contains("- Recipe Visuals: 1"));
        assert!(summary.contains("- Product Visuals: 1"));
        assert!(summary.contains("- Recent Refine Events: 1"));
        assert!(summary.contains("- Refine Output(Recent): 9"));
        assert!(summary.contains("Logistics Routes:"));
        assert!(summary.contains("- Active Routes: 1"));
        assert!(summary.contains("- Power Moved: 11 (loss=1)"));
        assert!(summary.contains("location::loc-a -> location::loc-b"));
    }
}
