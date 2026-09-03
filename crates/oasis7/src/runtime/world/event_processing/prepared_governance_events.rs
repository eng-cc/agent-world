use super::super::super::{
    GovernanceEvent, GovernanceFinalityEpochSnapshot, GovernanceIdentityPenaltyRecord,
    GovernanceIdentityProfileState, Proposal, ProposalId, ProposalStatus, WorldError,
    WorldEventBody,
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
        WorldEventBody::Governance(GovernanceEvent::IdentityPenaltyAppealed {
            penalty_id,
            appellant,
            reason,
        }) => {
            let next =
                world.prepare_governance_identity_penalty_appeal(*penalty_id, appellant, reason)?;
            Ok(Some(
                PreparedEventStateDelta::GovernanceIdentityPenaltyAppeal {
                    penalty_id: *penalty_id,
                    next,
                },
            ))
        }
        WorldEventBody::Governance(GovernanceEvent::IdentityPenaltyResolved {
            penalty_id,
            resolver,
            accepted,
            reason,
        }) => {
            let (target_agent_id, next, next_profile) = world
                .prepare_governance_identity_penalty_resolution(
                    *penalty_id,
                    resolver,
                    *accepted,
                    reason,
                )?;
            Ok(Some(
                PreparedEventStateDelta::GovernanceIdentityPenaltyResolution {
                    penalty_id: *penalty_id,
                    target_agent_id,
                    next,
                    next_profile,
                },
            ))
        }
        _ => Ok(None),
    }
}

impl World {
    pub(crate) fn prepare_governance_identity_penalty_appeal(
        &self,
        penalty_id: u64,
        appellant: &str,
        reason: &str,
    ) -> Result<GovernanceIdentityPenaltyRecord, WorldError> {
        Self::validate_governance_identity_field("identity penalty appeal appellant", appellant)?;
        Self::validate_governance_identity_field("identity penalty appeal reason", reason)?;
        let appeal_evidence_hash =
            Self::build_identity_penalty_stage_evidence_hash("appeal", appellant, reason);
        let mut penalty = self
            .governance_identity_penalties
            .get(&penalty_id)
            .cloned()
            .ok_or(WorldError::GovernancePolicyInvalid {
                reason: format!("identity penalty not found: penalty_id={penalty_id}"),
            })?;
        if penalty.status != super::super::super::GovernanceIdentityPenaltyStatus::Applied {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "identity penalty is not appealable: penalty_id={} status={:?}",
                    penalty_id, penalty.status
                ),
            });
        }
        if self.state.time > penalty.appeal_deadline_tick {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "identity penalty appeal window closed: penalty_id={} deadline_tick={}",
                    penalty_id, penalty.appeal_deadline_tick
                ),
            });
        }
        if penalty.detection_source.trim().is_empty() {
            penalty.detection_source = "world.threat_heatmap.v1".to_string();
        }
        if penalty.detection_incident_id.trim().is_empty() {
            penalty.detection_incident_id = Self::build_identity_penalty_incident_id(
                penalty.target_agent_id.as_str(),
                penalty.evidence_hash.as_str(),
            );
        }
        if penalty.evidence_chain_hash.trim().is_empty() {
            penalty.evidence_chain_hash = Self::build_identity_penalty_chain_hash(
                penalty.penalty_id,
                penalty.target_agent_id.as_str(),
                penalty.evidence_hash.as_str(),
                penalty.reason.as_str(),
                penalty.detection_incident_id.as_str(),
            );
        }
        penalty.status = super::super::super::GovernanceIdentityPenaltyStatus::Appealed;
        penalty.appellant = Some(appellant.to_string());
        penalty.appeal_reason = Some(reason.to_string());
        penalty.appeal_evidence_hash = Some(appeal_evidence_hash.clone());
        penalty.evidence_chain_hash = Self::extend_identity_penalty_chain_hash(
            penalty.evidence_chain_hash.as_str(),
            "appeal",
            appeal_evidence_hash.as_str(),
        );
        Ok(penalty)
    }

    pub(crate) fn prepare_governance_identity_penalty_resolution(
        &self,
        penalty_id: u64,
        resolver: &str,
        accepted: bool,
        reason: &str,
    ) -> Result<
        (
            String,
            GovernanceIdentityPenaltyRecord,
            GovernanceIdentityProfileState,
        ),
        WorldError,
    > {
        Self::validate_governance_identity_field("identity penalty appeal resolver", resolver)?;
        Self::validate_governance_identity_field("identity penalty appeal resolution", reason)?;
        let resolution_evidence_hash = Self::build_identity_penalty_stage_evidence_hash(
            if accepted {
                "resolve_accept"
            } else {
                "resolve_reject"
            },
            resolver,
            reason,
        );
        let mut penalty = self
            .governance_identity_penalties
            .get(&penalty_id)
            .cloned()
            .ok_or(WorldError::GovernancePolicyInvalid {
                reason: format!("identity penalty not found: penalty_id={penalty_id}"),
            })?;
        if penalty.status != super::super::super::GovernanceIdentityPenaltyStatus::Appealed {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "identity penalty appeal is not pending: penalty_id={} status={:?}",
                    penalty_id, penalty.status
                ),
            });
        }
        let target_agent_id = penalty.target_agent_id.clone();
        let mut profile = self
            .state
            .governance_identity_profiles
            .get(target_agent_id.as_str())
            .cloned()
            .ok_or(WorldError::AgentNotFound {
                agent_id: target_agent_id.clone(),
            })?;
        if penalty.detection_source.trim().is_empty() {
            penalty.detection_source = "world.threat_heatmap.v1".to_string();
        }
        if penalty.detection_incident_id.trim().is_empty() {
            penalty.detection_incident_id = Self::build_identity_penalty_incident_id(
                penalty.target_agent_id.as_str(),
                penalty.evidence_hash.as_str(),
            );
        }
        if penalty.evidence_chain_hash.trim().is_empty() {
            penalty.evidence_chain_hash = Self::build_identity_penalty_chain_hash(
                penalty.penalty_id,
                penalty.target_agent_id.as_str(),
                penalty.evidence_hash.as_str(),
                penalty.reason.as_str(),
                penalty.detection_incident_id.as_str(),
            );
        }
        penalty.status = if accepted {
            super::super::super::GovernanceIdentityPenaltyStatus::AppealAccepted
        } else {
            super::super::super::GovernanceIdentityPenaltyStatus::AppealRejected
        };
        penalty.resolved_by = Some(resolver.to_string());
        penalty.resolution_reason = Some(reason.to_string());
        penalty.resolved_at_tick = Some(self.state.time);
        penalty.resolution_evidence_hash = Some(resolution_evidence_hash.clone());
        penalty.evidence_chain_hash = Self::extend_identity_penalty_chain_hash(
            penalty.evidence_chain_hash.as_str(),
            "resolve",
            resolution_evidence_hash.as_str(),
        );
        if accepted {
            profile.stake_locked = profile.stake_locked.saturating_add(penalty.slash_stake);
            profile.status = penalty.identity_status_before;
        }
        profile.updated_at = self.state.time;
        Ok((target_agent_id, penalty, profile))
    }

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
