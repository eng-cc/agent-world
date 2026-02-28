use crate::industry_graph_view_model::{
    IndustryGraphViewModel, IndustryNodeKind, IndustrySemanticZoomLevel,
};

const INDUSTRIAL_TOP_ROUTE_LIMIT: usize = 3;
const INDUSTRIAL_NODE_DETAIL_LIMIT: usize = 4;
const INDUSTRIAL_WORLD_HOTSPOT_LIMIT: usize = 3;

#[allow(dead_code)]
pub(super) fn industrial_ops_summary(
    snapshot: Option<&agent_world::simulator::WorldSnapshot>,
    events: &[agent_world::simulator::WorldEvent],
) -> Option<String> {
    let graph = IndustryGraphViewModel::build(snapshot, events);
    industrial_ops_summary_with_zoom(&graph, IndustrySemanticZoomLevel::Node)
}

pub(super) fn industrial_ops_summary_with_zoom(
    graph: &IndustryGraphViewModel,
    zoom: IndustrySemanticZoomLevel,
) -> Option<String> {
    if !graph.has_industrial_signals() {
        return None;
    }

    let mut lines = vec!["Industrial Ops:".to_string()];
    lines.push(format!("- Semantic Zoom: {}", zoom.key()));
    lines.push("Production Lines:".to_string());
    lines.push(format!(
        "- Factory Visuals: {}",
        graph.rollup.factory_visuals
    ));
    lines.push(format!("- Recipe Visuals: {}", graph.rollup.recipe_visuals));
    lines.push(format!(
        "- Product Visuals: {}",
        graph.rollup.product_visuals
    ));
    lines.push(format!(
        "- Logistics Visuals: {}",
        graph.rollup.logistics_visuals
    ));
    lines.push(format!(
        "- Recent Refine Events: {}",
        graph.rollup.recent_refine_events
    ));
    lines.push(format!(
        "- Recent Line Updates: {}",
        graph.rollup.recent_line_updates
    ));
    lines.push(format!(
        "- Refine Output(Recent): {}",
        graph.rollup.recent_hardware_output
    ));

    lines.push("".to_string());
    lines.push("Logistics Routes:".to_string());
    let routes = graph.routes_for_zoom(zoom);
    lines.push(format!("- Active Routes: {}", routes.len()));
    lines.push(format!(
        "- Transfer Events: {}",
        graph.rollup.transfer_events
    ));
    lines.push(format!(
        "- Power Moved: {} (loss={})",
        graph.rollup.total_power_moved, graph.rollup.total_power_loss
    ));

    for route in routes.into_iter().take(INDUSTRIAL_TOP_ROUTE_LIMIT) {
        lines.push(format!(
            "- Route {} -> {} moves={} material={} electricity={} data={} power={} loss={}",
            route.from,
            route.to,
            route.transfer_events,
            route.material,
            route.electricity,
            route.data,
            route.power,
            route.power_loss,
        ));
    }

    lines.push("".to_string());
    match zoom {
        IndustrySemanticZoomLevel::World => {
            lines.push("World Lens: Hotspots & Trunk Flow".to_string());
            if graph.region_hotspots.is_empty() {
                lines.push("- Hotspots: (none)".to_string());
            } else {
                for hotspot in graph
                    .region_hotspots
                    .iter()
                    .take(INDUSTRIAL_WORLD_HOTSPOT_LIMIT)
                {
                    lines.push(format!(
                        "- chunk({}, {}, {}): events={} alerts={}",
                        hotspot.coord.x,
                        hotspot.coord.y,
                        hotspot.coord.z,
                        hotspot.events,
                        hotspot.alerts,
                    ));
                }
            }
        }
        IndustrySemanticZoomLevel::Region => {
            lines.push("Region Lens: Cluster Nodes".to_string());
            let slice = graph.graph_for_zoom(zoom);
            let factories = slice
                .nodes
                .iter()
                .filter(|node| node.kind == IndustryNodeKind::Factory)
                .count();
            let recipes = slice
                .nodes
                .iter()
                .filter(|node| node.kind == IndustryNodeKind::Recipe)
                .count();
            lines.push(format!("- Cluster Nodes: {}", slice.nodes.len()));
            lines.push(format!("- Cluster Edges: {}", slice.edges.len()));
            lines.push(format!("- Factory/Recipe: {factories}/{recipes}"));
        }
        IndustrySemanticZoomLevel::Node => {
            lines.push("Node Lens: Recipe & Inventory State".to_string());
            let mut detailed = graph.nodes.clone();
            detailed.sort_by(|left, right| {
                right
                    .throughput
                    .cmp(&left.throughput)
                    .then_with(|| left.id.cmp(&right.id))
            });
            for node in detailed
                .into_iter()
                .filter(|node| {
                    matches!(
                        node.kind,
                        IndustryNodeKind::Factory
                            | IndustryNodeKind::Recipe
                            | IndustryNodeKind::Product
                            | IndustryNodeKind::LogisticsStation
                    )
                })
                .take(INDUSTRIAL_NODE_DETAIL_LIMIT)
            {
                lines.push(format!(
                    "- {} kind={:?} tier={:?} stage={:?} throughput={} stock(E/D)={}/{} flags(b/c/a)={}/{}/{}",
                    node.id,
                    node.kind,
                    node.tier,
                    node.stage,
                    node.throughput,
                    node.stock_electricity,
                    node.stock_data,
                    yes_no(node.status.bottleneck),
                    yes_no(node.status.congestion),
                    yes_no(node.status.alert),
                ));
            }
        }
    }

    Some(lines.join("\n"))
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "Y"
    } else {
        "N"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        ChunkRuntimeConfig, ModuleVisualAnchor, ModuleVisualEntity, PowerEvent, ResourceKind,
        ResourceOwner, WorldConfig, WorldEvent, WorldEventKind, WorldModel, WorldSnapshot,
        CHUNK_GENERATION_SCHEMA_VERSION, SNAPSHOT_VERSION,
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
                    quoted_price_per_pu: 0,
                    price_per_pu: 0,
                    settlement_amount: 0,
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
        assert!(summary.contains("- Active Routes: 2"));
        assert!(summary.contains("- Power Moved: 11 (loss=1)"));
        assert!(summary.contains("location::loc-a -> location::loc-b"));
    }

    #[test]
    fn industrial_ops_summary_world_zoom_includes_hotspot_lens() {
        let graph = IndustryGraphViewModel::build(None, &[]);
        assert!(
            industrial_ops_summary_with_zoom(&graph, IndustrySemanticZoomLevel::World).is_none()
        );
    }
}
