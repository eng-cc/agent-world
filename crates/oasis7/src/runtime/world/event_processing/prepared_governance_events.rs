use super::super::super::{
    GovernanceEvent, GovernanceFinalityEpochSnapshot, Proposal, ProposalId, ProposalStatus,
    WorldError, WorldEventBody,
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
        WorldEventBody::Governance(GovernanceEvent::EmergencyVetoed {
            proposal_id,
            reason,
            threshold,
            signer_node_ids,
            ..
        }) => {
            let next = world.prepare_governance_emergency_veto(
                *proposal_id,
                reason,
                *threshold,
                signer_node_ids,
            )?;
            Ok(Some(PreparedEventStateDelta::GovernanceEmergencyVeto {
                proposal_id: *proposal_id,
                next,
            }))
        }
        _ => Ok(None),
    }
}

impl World {
    pub(crate) fn prepare_governance_emergency_veto(
        &self,
        proposal_id: ProposalId,
        reason: &str,
        threshold: u16,
        signer_node_ids: &[String],
    ) -> Result<Proposal, WorldError> {
        self.validate_guardian_signers(signer_node_ids, threshold)?;
        let mut proposal = self
            .proposals
            .get(&proposal_id)
            .cloned()
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        if !matches!(proposal.status, ProposalStatus::Approved { .. }) {
            return Err(WorldError::ProposalInvalidState {
                proposal_id,
                expected: "approved".to_string(),
                found: proposal.status.label(),
            });
        }
        if proposal.not_before_tick.is_none() || proposal.activate_epoch.is_none() {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!("proposal_id={} is not queued for activation", proposal_id),
            });
        }
        proposal.queued_at_tick = None;
        proposal.not_before_tick = None;
        proposal.activate_epoch = None;
        proposal.timelock_ticks = 0;
        proposal.status = ProposalStatus::Rejected {
            reason: format!("emergency_veto: {reason}"),
        };
        Ok(proposal)
    }

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
