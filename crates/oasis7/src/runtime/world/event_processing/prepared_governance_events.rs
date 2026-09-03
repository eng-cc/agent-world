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
        WorldEventBody::Governance(GovernanceEvent::Proposed {
            proposal_id,
            author,
            base_manifest_hash,
            manifest,
            patch,
        }) => {
            let (next, next_proposal_id, next_proposal_id_era) = world.prepare_governance_proposal(
                *proposal_id,
                author,
                base_manifest_hash,
                manifest,
                patch,
            );
            Ok(Some(PreparedEventStateDelta::GovernanceProposal {
                proposal_id: *proposal_id,
                next,
                next_proposal_id,
                next_proposal_id_era,
            }))
        }
        WorldEventBody::Governance(GovernanceEvent::ShadowReport {
            proposal_id,
            manifest_hash,
        }) => {
            let next = world.prepare_governance_proposal_shadow(*proposal_id, manifest_hash)?;
            Ok(Some(PreparedEventStateDelta::GovernanceProposalShadow {
                proposal_id: *proposal_id,
                next,
            }))
        }
        WorldEventBody::Governance(GovernanceEvent::IdentityPenaltyApplied {
            penalty_id,
            target_agent_id,
            evidence_hash,
            initiator,
            reason,
            slash_stake,
            appeal_deadline_tick,
            threshold,
            signer_node_ids,
        }) => {
            let (target_agent_id, next, next_profile, profile_was_present, next_penalty_id) = world
                .prepare_governance_identity_penalty_application(
                    *penalty_id,
                    target_agent_id,
                    evidence_hash,
                    initiator,
                    reason,
                    *slash_stake,
                    *appeal_deadline_tick,
                    *threshold,
                    signer_node_ids,
                )?;
            Ok(Some(
                PreparedEventStateDelta::GovernanceIdentityPenaltyApplication {
                    penalty_id: *penalty_id,
                    target_agent_id,
                    next,
                    next_profile,
                    profile_was_present,
                    next_penalty_id,
                },
            ))
        }
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
    pub(super) fn prepare_governance_proposal(
        &self,
        proposal_id: ProposalId,
        author: &str,
        base_manifest_hash: &str,
        manifest: &super::super::super::Manifest,
        patch: &Option<super::super::super::ManifestPatch>,
    ) -> (Proposal, ProposalId, u64) {
        let proposal = Proposal {
            id: proposal_id,
            author: author.to_string(),
            base_manifest_hash: base_manifest_hash.to_string(),
            manifest: manifest.clone(),
            patch: patch.clone(),
            queued_at_tick: None,
            not_before_tick: None,
            activate_epoch: None,
            timelock_ticks: 0,
            status: ProposalStatus::Proposed,
        };
        let (allocated, next_proposal_id, next_proposal_id_era) =
            Self::preview_next_proposal_id(self.next_proposal_id, self.next_proposal_id_era);
        let (next_proposal_id, next_proposal_id_era) = if allocated == proposal_id {
            (next_proposal_id, next_proposal_id_era)
        } else {
            (
                self.next_proposal_id.max(proposal_id.saturating_add(1)),
                self.next_proposal_id_era,
            )
        };
        (proposal, next_proposal_id, next_proposal_id_era)
    }

    pub(super) fn prepare_governance_proposal_shadow(
        &self,
        proposal_id: ProposalId,
        manifest_hash: &str,
    ) -> Result<Proposal, WorldError> {
        let mut proposal = self
            .proposals
            .get(&proposal_id)
            .cloned()
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        proposal.status = ProposalStatus::Shadowed {
            manifest_hash: manifest_hash.to_string(),
        };
        Ok(proposal)
    }

    pub(crate) fn prepare_governance_identity_penalty_application(
        &self,
        penalty_id: u64,
        target_agent_id: &str,
        evidence_hash: &str,
        initiator: &str,
        reason: &str,
        slash_stake: u64,
        appeal_deadline_tick: u64,
        threshold: u16,
        signer_node_ids: &[String],
    ) -> Result<
        (
            String,
            GovernanceIdentityPenaltyRecord,
            GovernanceIdentityProfileState,
            bool,
            u64,
        ),
        WorldError,
    > {
        self.validate_guardian_signers(signer_node_ids, threshold)?;
        if !self.state.agents.contains_key(target_agent_id) {
            return Err(WorldError::AgentNotFound {
                agent_id: target_agent_id.to_string(),
            });
        }
        Self::validate_governance_identity_evidence_hash(evidence_hash)?;
        Self::validate_governance_identity_field("identity penalty reason", reason)?;
        Self::validate_governance_identity_field("identity penalty initiator", initiator)?;
        if self.governance_identity_penalties.contains_key(&penalty_id) {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!("duplicate identity penalty id: penalty_id={penalty_id}"),
            });
        }
        let detection_incident_id =
            Self::build_identity_penalty_incident_id(target_agent_id, evidence_hash);
        if self
            .governance_identity_penalties
            .values()
            .any(|record| record.detection_incident_id == detection_incident_id)
        {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "duplicate identity penalty incident: incident_id={detection_incident_id}"
                ),
            });
        }
        let detection_risk_score = self
            .threat_heatmap
            .get(target_agent_id)
            .copied()
            .unwrap_or_default();
        let evidence_chain_hash = Self::build_identity_penalty_chain_hash(
            penalty_id,
            target_agent_id,
            evidence_hash,
            reason,
            detection_incident_id.as_str(),
        );
        let profile_was_present = self
            .state
            .governance_identity_profiles
            .contains_key(target_agent_id);
        let mut profile = self
            .state
            .governance_identity_profiles
            .get(target_agent_id)
            .cloned()
            .unwrap_or_else(|| GovernanceIdentityProfileState {
                agent_id: target_agent_id.to_string(),
                ..GovernanceIdentityProfileState::default()
            });
        if slash_stake > profile.stake_locked {
            return Err(WorldError::GovernancePolicyInvalid {
                reason: format!(
                    "identity penalty slash exceeds locked stake: penalty_id={} slash={} stake_locked={}",
                    penalty_id, slash_stake, profile.stake_locked
                ),
            });
        }
        let identity_status_before = profile.status;
        profile.stake_locked = profile.stake_locked.saturating_sub(slash_stake);
        profile.status = super::super::super::GovernanceIdentityStatus::Frozen;
        profile.slash_count = profile.slash_count.saturating_add(1);
        profile.updated_at = self.state.time;
        let next = GovernanceIdentityPenaltyRecord {
            penalty_id,
            target_agent_id: target_agent_id.to_string(),
            evidence_hash: evidence_hash.to_string(),
            reason: reason.to_string(),
            slash_stake,
            appeal_deadline_tick,
            status: super::super::super::GovernanceIdentityPenaltyStatus::Applied,
            identity_status_before,
            detection_source: "world.threat_heatmap.v1".to_string(),
            detection_risk_score,
            detection_incident_id,
            evidence_chain_hash,
            appeal_evidence_hash: None,
            resolution_evidence_hash: None,
            appellant: None,
            appeal_reason: None,
            resolved_by: None,
            resolution_reason: None,
            resolved_at_tick: None,
        };
        let next_penalty_id = self
            .next_governance_identity_penalty_id
            .max(penalty_id.saturating_add(1));
        Ok((
            target_agent_id.to_string(),
            next,
            profile,
            profile_was_present,
            next_penalty_id,
        ))
    }

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
