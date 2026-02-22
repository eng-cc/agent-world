use super::*;

pub(super) fn event_matches_agent(event: &WorldEvent, agent_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::AgentRegistered { agent_id: id, .. }
        | WorldEventKind::AgentMoved { agent_id: id, .. }
        | WorldEventKind::RadiationHarvested { agent_id: id, .. }
        | WorldEventKind::LlmEffectQueued { agent_id: id, .. }
        | WorldEventKind::LlmReceiptAppended { agent_id: id, .. } => id == agent_id,
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            owner_is_agent(from, agent_id) || owner_is_agent(to, agent_id)
        }
        WorldEventKind::DebugResourceGranted { owner, .. }
        | WorldEventKind::CompoundMined { owner, .. }
        | WorldEventKind::CompoundRefined { owner, .. }
        | WorldEventKind::FactoryBuilt { owner, .. }
        | WorldEventKind::RecipeScheduled { owner, .. } => owner_is_agent(owner, agent_id),
        WorldEventKind::ActionRejected { reason } => reject_reason_matches_agent(reason, agent_id),
        WorldEventKind::Power(power_event) => match power_event {
            PowerEvent::PowerConsumed { agent_id: id, .. }
            | PowerEvent::PowerStateChanged { agent_id: id, .. }
            | PowerEvent::PowerCharged { agent_id: id, .. } => id == agent_id,
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn event_matches_location(event: &WorldEvent, location_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::LocationRegistered {
            location_id: id, ..
        } => id == location_id,
        WorldEventKind::AgentRegistered {
            location_id: id, ..
        }
        | WorldEventKind::RadiationHarvested {
            location_id: id, ..
        } => id == location_id,
        WorldEventKind::AgentMoved { from, to, .. } => from == location_id || to == location_id,
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            owner_is_location(from, location_id) || owner_is_location(to, location_id)
        }
        WorldEventKind::CompoundMined {
            owner,
            location_id: event_location_id,
            ..
        } => event_location_id == location_id || owner_is_location(owner, location_id),
        WorldEventKind::DebugResourceGranted { owner, .. }
        | WorldEventKind::CompoundRefined { owner, .. }
        | WorldEventKind::RecipeScheduled { owner, .. } => owner_is_location(owner, location_id),
        WorldEventKind::FactoryBuilt {
            owner,
            location_id: event_location_id,
            ..
        } => event_location_id == location_id || owner_is_location(owner, location_id),
        WorldEventKind::ActionRejected { reason } => {
            reject_reason_matches_location(reason, location_id)
        }
        WorldEventKind::Power(power_event) => match power_event {
            PowerEvent::PowerGenerated {
                location_id: id, ..
            }
            | PowerEvent::PowerStored {
                location_id: id, ..
            }
            | PowerEvent::PowerDischarged {
                location_id: id, ..
            } => id == location_id,
            PowerEvent::PowerPlantRegistered { plant } => plant.location_id == location_id,
            PowerEvent::PowerStorageRegistered { storage } => storage.location_id == location_id,
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn event_matches_power_plant(event: &WorldEvent, facility_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerPlantRegistered { plant }) => {
            plant.id == facility_id
        }
        WorldEventKind::Power(PowerEvent::PowerGenerated { plant_id, .. }) => {
            plant_id == facility_id
        }
        WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityAlreadyExists { facility_id: id },
        }
        | WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityNotFound { facility_id: id },
        } => id == facility_id,
        _ => false,
    }
}

pub(super) fn event_matches_power_storage(event: &WorldEvent, facility_id: &str) -> bool {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerStorageRegistered { storage }) => {
            storage.id == facility_id
        }
        WorldEventKind::Power(PowerEvent::PowerStored { storage_id, .. })
        | WorldEventKind::Power(PowerEvent::PowerDischarged { storage_id, .. }) => {
            storage_id == facility_id
        }
        WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityAlreadyExists { facility_id: id },
        }
        | WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityNotFound { facility_id: id },
        } => id == facility_id,
        _ => false,
    }
}

pub(super) fn event_matches_chunk(event: &WorldEvent, coord: ChunkCoord) -> bool {
    match &event.kind {
        WorldEventKind::ChunkGenerated {
            coord: event_coord, ..
        } => *event_coord == coord,
        WorldEventKind::ActionRejected {
            reason: RejectReason::ChunkGenerationFailed { x, y, z },
        } => *x == coord.x && *y == coord.y && *z == coord.z,
        _ => false,
    }
}

pub(super) fn event_matches_owner(event: &WorldEvent, owner: &ResourceOwner) -> bool {
    match &event.kind {
        WorldEventKind::ResourceTransferred { from, to, .. }
        | WorldEventKind::Power(PowerEvent::PowerTransferred { from, to, .. }) => {
            from == owner || to == owner
        }
        WorldEventKind::CompoundMined {
            owner: event_owner, ..
        }
        | WorldEventKind::DebugResourceGranted {
            owner: event_owner, ..
        }
        | WorldEventKind::CompoundRefined {
            owner: event_owner, ..
        }
        | WorldEventKind::FactoryBuilt {
            owner: event_owner, ..
        }
        | WorldEventKind::RecipeScheduled {
            owner: event_owner, ..
        } => event_owner == owner,
        WorldEventKind::ActionRejected {
            reason:
                RejectReason::InsufficientResource {
                    owner: event_owner, ..
                },
        } => event_owner == owner,
        _ => false,
    }
}

fn reject_reason_matches_agent(reason: &RejectReason, agent_id: &str) -> bool {
    match reason {
        RejectReason::AgentAlreadyExists { agent_id: id }
        | RejectReason::AgentNotFound { agent_id: id }
        | RejectReason::AgentAlreadyAtLocation { agent_id: id, .. }
        | RejectReason::AgentNotAtLocation { agent_id: id, .. }
        | RejectReason::AgentShutdown { agent_id: id } => id == agent_id,
        RejectReason::AgentsNotCoLocated {
            agent_id: id,
            other_agent_id,
        } => id == agent_id || other_agent_id == agent_id,
        RejectReason::InsufficientResource { owner, .. } => owner_is_agent(owner, agent_id),
        _ => false,
    }
}

fn reject_reason_matches_location(reason: &RejectReason, location_id: &str) -> bool {
    match reason {
        RejectReason::LocationAlreadyExists { location_id: id }
        | RejectReason::LocationNotFound { location_id: id }
        | RejectReason::RadiationUnavailable { location_id: id } => id == location_id,
        RejectReason::LocationTransferNotAllowed { from, to } => {
            from == location_id || to == location_id
        }
        RejectReason::AgentAlreadyAtLocation {
            location_id: id, ..
        }
        | RejectReason::AgentNotAtLocation {
            location_id: id, ..
        } => id == location_id,
        RejectReason::InsufficientResource { owner, .. } => owner_is_location(owner, location_id),
        _ => false,
    }
}

fn owner_is_agent(owner: &ResourceOwner, agent_id: &str) -> bool {
    matches!(owner, ResourceOwner::Agent { agent_id: id } if id == agent_id)
}

fn owner_is_location(owner: &ResourceOwner, location_id: &str) -> bool {
    matches!(owner, ResourceOwner::Location { location_id: id } if id == location_id)
}
