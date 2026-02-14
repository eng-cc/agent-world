use super::*;

pub(super) fn event_activity_for_owner(
    event: &WorldEvent,
    owner: &ResourceOwner,
) -> Option<String> {
    match &event.kind {
        WorldEventKind::ResourceTransferred {
            from,
            to,
            kind,
            amount,
        } if from == owner && to == owner => Some(format!("transfer {:?} {} (self)", kind, amount)),
        WorldEventKind::ResourceTransferred {
            from,
            to: _,
            kind,
            amount,
        } if from == owner => Some(format!("transfer out {:?} {}", kind, amount)),
        WorldEventKind::ResourceTransferred {
            from: _,
            to,
            kind,
            amount,
        } if to == owner => Some(format!("transfer in {:?} {}", kind, amount)),
        WorldEventKind::CompoundRefined {
            owner: refined_owner,
            compound_mass_g,
            hardware_output,
            ..
        } if refined_owner == owner => Some(format!(
            "refine {}g -> hw {}",
            compound_mass_g, hardware_output
        )),
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from,
            to,
            amount,
            loss,
            ..
        }) if from == owner && to == owner => {
            Some(format!("trade power {} (loss {})", amount, loss))
        }
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from,
            to: _,
            amount,
            loss,
            ..
        }) if from == owner => Some(format!("sell power {} (loss {})", amount, loss)),
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from: _,
            to,
            amount,
            loss,
            ..
        }) if to == owner => Some(format!("buy power {} (loss {})", amount, loss)),
        _ => None,
    }
}

pub(super) fn event_activity_for_power_plant(
    event: &WorldEvent,
    facility_id: &str,
) -> Option<String> {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerPlantRegistered { plant })
            if plant.id == facility_id =>
        {
            Some(format!("register at {}", plant.location_id))
        }
        WorldEventKind::Power(PowerEvent::PowerGenerated {
            plant_id,
            amount,
            location_id,
        }) if plant_id == facility_id => Some(format!("generated {} at {}", amount, location_id)),
        _ => None,
    }
}

pub(super) fn event_activity_for_power_storage(
    event: &WorldEvent,
    storage_id: &str,
) -> Option<String> {
    match &event.kind {
        WorldEventKind::Power(PowerEvent::PowerStorageRegistered { storage })
            if storage.id == storage_id =>
        {
            Some(format!("register at {}", storage.location_id))
        }
        WorldEventKind::Power(PowerEvent::PowerStored {
            storage_id: id,
            input,
            stored,
            ..
        }) if id == storage_id => Some(format!("stored {} (input {})", stored, input)),
        WorldEventKind::Power(PowerEvent::PowerDischarged {
            storage_id: id,
            output,
            drawn,
            ..
        }) if id == storage_id => Some(format!("discharged {} (drawn {})", output, drawn)),
        _ => None,
    }
}

pub(super) fn owner_matches_agent(owner: &ResourceOwner, agent_id: &str) -> bool {
    matches!(owner, ResourceOwner::Agent { agent_id: id } if id == agent_id)
}

pub(super) fn owner_matches_location(owner: &ResourceOwner, location_id: &str) -> bool {
    matches!(owner, ResourceOwner::Location { location_id: id } if id == location_id)
}
