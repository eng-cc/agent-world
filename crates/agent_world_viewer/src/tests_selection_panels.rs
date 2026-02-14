use super::*;

#[test]
fn update_ui_populates_asset_selection_details() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().spawn((Text::new(""), SelectionDetailsText));

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().insert_resource(ViewerSelection {
        current: Some(SelectionInfo {
            entity,
            kind: SelectionKind::Asset,
            id: "asset-1".to_string(),
            name: None,
        }),
    });

    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.assets.insert(
        "asset-1".to_string(),
        agent_world::simulator::Asset {
            id: "asset-1".to_string(),
            owner: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            kind: agent_world::simulator::AssetKind::Resource {
                kind: agent_world::simulator::ResourceKind::Electricity,
            },
            quantity: 25,
        },
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 8,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 2,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 1,
        time: 8,
        kind: agent_world::simulator::WorldEventKind::ResourceTransferred {
            from: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            to: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            kind: agent_world::simulator::ResourceKind::Electricity,
            amount: 3,
        },
    }];

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events,
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let world = app.world_mut();
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: asset asset-1"));
    assert!(details_text.0.contains("Owner: location::loc-1"));
    assert!(details_text.0.contains("Recent Owner Events"));
}

#[test]
fn update_ui_populates_power_plant_selection_details() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().spawn((Text::new(""), SelectionDetailsText));

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().insert_resource(ViewerSelection {
        current: Some(SelectionInfo {
            entity,
            kind: SelectionKind::PowerPlant,
            id: "plant-1".to_string(),
            name: None,
        }),
    });

    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.power_plants.insert(
        "plant-1".to_string(),
        agent_world::simulator::PowerPlant {
            id: "plant-1".to_string(),
            location_id: "loc-1".to_string(),
            owner: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            capacity_per_tick: 30,
            current_output: 12,
            fuel_cost_per_pu: 2,
            maintenance_cost: 1,
            status: agent_world::simulator::PlantStatus::Running,
            efficiency: 0.9,
            degradation: 0.1,
        },
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 9,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 3,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 2,
        time: 9,
        kind: agent_world::simulator::WorldEventKind::Power(
            agent_world::simulator::PowerEvent::PowerGenerated {
                plant_id: "plant-1".to_string(),
                location_id: "loc-1".to_string(),
                amount: 7,
            },
        ),
    }];

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events,
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let world = app.world_mut();
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: power_plant plant-1"));
    assert!(details_text.0.contains("Output: current=12"));
    assert!(details_text.0.contains("generated 7"));
}

#[test]
fn update_ui_populates_chunk_selection_details() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().spawn((Text::new(""), SelectionDetailsText));

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().insert_resource(ViewerSelection {
        current: Some(SelectionInfo {
            entity,
            kind: SelectionKind::Chunk,
            id: "0,0,0".to_string(),
            name: Some("generated".to_string()),
        }),
    });

    let mut model = agent_world::simulator::WorldModel::default();
    model.chunks.insert(
        agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
        agent_world::simulator::ChunkState::Generated,
    );

    let mut budget = agent_world::simulator::ChunkResourceBudget::default();
    budget
        .total_by_element_g
        .insert(agent_world::simulator::FragmentElementKind::Iron, 120);
    budget
        .remaining_by_element_g
        .insert(agent_world::simulator::FragmentElementKind::Iron, 90);
    model.chunk_resource_budgets.insert(
        agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
        budget,
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 12,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 3,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 2,
        time: 12,
        kind: agent_world::simulator::WorldEventKind::ChunkGenerated {
            coord: agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
            seed: 11,
            fragment_count: 4,
            block_count: 18,
            chunk_budget: agent_world::simulator::ChunkResourceBudget::default(),
            cause: agent_world::simulator::ChunkGenerationCause::Action,
        },
    }];

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events,
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let world = app.world_mut();
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: chunk 0,0,0"));
    assert!(details_text.0.contains("State: generated"));
    assert!(details_text.0.contains("Budget (remaining top):"));
    assert!(details_text.0.contains("generated fragments=4 blocks=18"));
}
