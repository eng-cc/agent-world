use super::super::super::{
    GovernanceEvent, GovernanceFinalityEpochSnapshot, WorldError, WorldEventBody,
};
use super::super::World;
use super::PreparedEventStateDelta;

pub(super) fn prepare(
    world: &World,
    body: &WorldEventBody,
) -> Result<Option<PreparedEventStateDelta>, WorldError> {
    match body {
        WorldEventBody::Governance(GovernanceEvent::FinalityEpochSnapshotSet {
            snapshot,
            previous,
        }) => {
            world.validate_governance_finality_epoch_snapshot_set(snapshot, previous)?;
            Ok(Some(
                PreparedEventStateDelta::GovernanceFinalityEpochSnapshot {
                    epoch_id: snapshot.epoch_id,
                    next: Some(snapshot.clone()),
                },
            ))
        }
        WorldEventBody::Governance(GovernanceEvent::FinalityEpochSnapshotRemoved {
            epoch_id,
            snapshot,
        }) => {
            world.validate_governance_finality_epoch_snapshot_removal(*epoch_id, snapshot)?;
            Ok(Some(
                PreparedEventStateDelta::GovernanceFinalityEpochSnapshot {
                    epoch_id: *epoch_id,
                    next: None,
                },
            ))
        }
        _ => Ok(None),
    }
}

impl World {
    pub(crate) fn validate_governance_finality_epoch_snapshot_set(
        &self,
        snapshot: &GovernanceFinalityEpochSnapshot,
        previous: &Option<GovernanceFinalityEpochSnapshot>,
    ) -> Result<(), WorldError> {
        let current = self
            .governance_finality_epoch_snapshots
            .get(&snapshot.epoch_id)
            .cloned();
        if current != *previous {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "governance finality snapshot predecessor drift: epoch_id={}",
                    snapshot.epoch_id
                ),
            });
        }
        let mut normalized = snapshot.clone();
        self.normalize_governance_finality_epoch_snapshot(&mut normalized)?;
        if normalized != *snapshot {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "governance finality snapshot normalization drift: epoch_id={}",
                    snapshot.epoch_id
                ),
            });
        }
        Ok(())
    }

    pub(crate) fn validate_governance_finality_epoch_snapshot_removal(
        &self,
        epoch_id: u64,
        snapshot: &GovernanceFinalityEpochSnapshot,
    ) -> Result<(), WorldError> {
        if snapshot.epoch_id != epoch_id
            || self.governance_finality_epoch_snapshots.get(&epoch_id) != Some(snapshot)
        {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!("governance finality snapshot removal drift: epoch_id={epoch_id}"),
            });
        }
        Ok(())
    }
}
