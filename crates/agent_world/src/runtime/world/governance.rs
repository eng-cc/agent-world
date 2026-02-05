use super::World;
use super::super::{
    apply_manifest_patch, GovernanceEvent, Manifest, ManifestPatch, ManifestUpdate, Proposal,
    ProposalDecision, ProposalId, ProposalStatus, WorldError, WorldEventBody, DistributedClient,
    DistributedDht,
};
use super::super::util::hash_json;

impl World {
    // ---------------------------------------------------------------------
    // Manifest and governance
    // ---------------------------------------------------------------------

    pub fn current_manifest_hash(&self) -> Result<String, WorldError> {
        hash_json(&self.manifest)
    }

    pub fn propose_manifest_update(
        &mut self,
        manifest: Manifest,
        author: impl Into<String>,
    ) -> Result<ProposalId, WorldError> {
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;
        let base_manifest_hash = self.current_manifest_hash()?;
        let event = GovernanceEvent::Proposed {
            proposal_id,
            author: author.into(),
            base_manifest_hash,
            manifest,
            patch: None,
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(proposal_id)
    }

    pub fn propose_manifest_patch(
        &mut self,
        patch: ManifestPatch,
        author: impl Into<String>,
    ) -> Result<ProposalId, WorldError> {
        let base_manifest_hash = self.current_manifest_hash()?;
        if patch.base_manifest_hash != base_manifest_hash {
            return Err(WorldError::PatchBaseMismatch {
                expected: base_manifest_hash,
                found: patch.base_manifest_hash,
            });
        }

        let manifest = apply_manifest_patch(&self.manifest, &patch)?;
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;
        let event = GovernanceEvent::Proposed {
            proposal_id,
            author: author.into(),
            base_manifest_hash,
            manifest,
            patch: Some(patch),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(proposal_id)
    }

    pub fn shadow_proposal(&mut self, proposal_id: ProposalId) -> Result<String, WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        if !matches!(proposal.status, ProposalStatus::Proposed) {
            return Err(WorldError::ProposalInvalidState {
                proposal_id,
                expected: "proposed".to_string(),
                found: proposal.status.label(),
            });
        }
        if let Some(changes) = proposal.manifest.module_changes()? {
            self.validate_module_changes(&changes)?;
            self.shadow_validate_module_changes(&changes)?;
        }
        let manifest_hash = hash_json(&proposal.manifest)?;
        let event = GovernanceEvent::ShadowReport {
            proposal_id,
            manifest_hash: manifest_hash.clone(),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(manifest_hash)
    }

    pub fn shadow_proposal_with_fetch(
        &mut self,
        proposal_id: ProposalId,
        world_id: &str,
        client: &DistributedClient,
        dht: &impl DistributedDht,
    ) -> Result<String, WorldError> {
        let (status, manifest) = {
            let proposal = self
                .proposals
                .get(&proposal_id)
                .ok_or(WorldError::ProposalNotFound { proposal_id })?;
            (proposal.status.clone(), proposal.manifest.clone())
        };
        if !matches!(status, ProposalStatus::Proposed) {
            return Err(WorldError::ProposalInvalidState {
                proposal_id,
                expected: "proposed".to_string(),
                found: status.label(),
            });
        }
        if let Some(changes) = manifest.module_changes()? {
            self.ensure_module_changes_with_fetch(world_id, &changes, client, dht)?;
            self.validate_module_changes(&changes)?;
            self.shadow_validate_module_changes(&changes)?;
        }
        let manifest_hash = hash_json(&manifest)?;
        let event = GovernanceEvent::ShadowReport {
            proposal_id,
            manifest_hash: manifest_hash.clone(),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(manifest_hash)
    }

    pub fn approve_proposal(
        &mut self,
        proposal_id: ProposalId,
        approver: impl Into<String>,
        decision: ProposalDecision,
    ) -> Result<(), WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;

        match (&decision, &proposal.status) {
            (ProposalDecision::Approve, ProposalStatus::Shadowed { .. }) => {}
            (ProposalDecision::Reject { .. }, ProposalStatus::Applied { .. })
            | (ProposalDecision::Reject { .. }, ProposalStatus::Rejected { .. }) => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "proposed".to_string(),
                    found: proposal.status.label(),
                });
            }
            (ProposalDecision::Approve, _) => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "shadowed".to_string(),
                    found: proposal.status.label(),
                });
            }
            _ => {}
        }

        let event = GovernanceEvent::Approved {
            proposal_id,
            approver: approver.into(),
            decision,
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(())
    }

    pub fn apply_proposal(&mut self, proposal_id: ProposalId) -> Result<String, WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        let (manifest, actor) = match &proposal.status {
            ProposalStatus::Approved { .. } => (proposal.manifest.clone(), proposal.author.clone()),
            other => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "approved".to_string(),
                    found: other.label(),
                })
            }
        };

        let module_changes = manifest.module_changes()?;
        if let Some(changes) = &module_changes {
            self.validate_module_changes(changes)?;
        }
        let applied_manifest = if module_changes.is_some() {
            manifest.without_module_changes()?
        } else {
            manifest.clone()
        };
        let applied_hash = hash_json(&applied_manifest)?;

        let event = GovernanceEvent::Applied {
            proposal_id,
            manifest_hash: Some(applied_hash.clone()),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        if let Some(changes) = module_changes {
            self.apply_module_changes(proposal_id, &changes, &actor)?;
        }
        let update = ManifestUpdate {
            manifest: applied_manifest,
            manifest_hash: applied_hash.clone(),
        };
        self.append_event(WorldEventBody::ManifestUpdated(update), None)?;
        Ok(applied_hash)
    }

    pub fn apply_proposal_with_fetch(
        &mut self,
        proposal_id: ProposalId,
        world_id: &str,
        client: &DistributedClient,
        dht: &impl DistributedDht,
    ) -> Result<String, WorldError> {
        let (status, manifest, actor) = {
            let proposal = self
                .proposals
                .get(&proposal_id)
                .ok_or(WorldError::ProposalNotFound { proposal_id })?;
            (
                proposal.status.clone(),
                proposal.manifest.clone(),
                proposal.author.clone(),
            )
        };
        let (manifest, actor) = match &status {
            ProposalStatus::Approved { .. } => (manifest, actor),
            other => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "approved".to_string(),
                    found: other.label(),
                })
            }
        };

        let module_changes = manifest.module_changes()?;
        if let Some(changes) = &module_changes {
            self.ensure_module_changes_with_fetch(world_id, changes, client, dht)?;
            self.validate_module_changes(changes)?;
        }
        let applied_manifest = if module_changes.is_some() {
            manifest.without_module_changes()?
        } else {
            manifest.clone()
        };
        let applied_hash = hash_json(&applied_manifest)?;

        let event = GovernanceEvent::Applied {
            proposal_id,
            manifest_hash: Some(applied_hash.clone()),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        if let Some(changes) = module_changes {
            self.apply_module_changes(proposal_id, &changes, &actor)?;
        }
        let update = ManifestUpdate {
            manifest: applied_manifest,
            manifest_hash: applied_hash.clone(),
        };
        self.append_event(WorldEventBody::ManifestUpdated(update), None)?;
        Ok(applied_hash)
    }

    pub(super) fn apply_governance_event(
        &mut self,
        event: &GovernanceEvent,
    ) -> Result<(), WorldError> {
        match event {
            GovernanceEvent::Proposed {
                proposal_id,
                author,
                base_manifest_hash,
                manifest,
                patch,
            } => {
                let proposal = Proposal {
                    id: *proposal_id,
                    author: author.clone(),
                    base_manifest_hash: base_manifest_hash.clone(),
                    manifest: manifest.clone(),
                    patch: patch.clone(),
                    status: ProposalStatus::Proposed,
                };
                self.proposals.insert(*proposal_id, proposal);
                self.next_proposal_id = self.next_proposal_id.max(proposal_id.saturating_add(1));
            }
            GovernanceEvent::ShadowReport {
                proposal_id,
                manifest_hash,
            } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                proposal.status = ProposalStatus::Shadowed {
                    manifest_hash: manifest_hash.clone(),
                };
            }
            GovernanceEvent::Approved {
                proposal_id,
                approver,
                decision,
            } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                match decision {
                    ProposalDecision::Approve => {
                        let ProposalStatus::Shadowed { manifest_hash } = &proposal.status else {
                            return Err(WorldError::ProposalInvalidState {
                                proposal_id: *proposal_id,
                                expected: "shadowed".to_string(),
                                found: proposal.status.label(),
                            });
                        };
                        proposal.status = ProposalStatus::Approved {
                            manifest_hash: manifest_hash.clone(),
                            approver: approver.clone(),
                        };
                    }
                    ProposalDecision::Reject { reason } => {
                        proposal.status = ProposalStatus::Rejected {
                            reason: reason.clone(),
                        };
                    }
                }
            }
            GovernanceEvent::Applied {
                proposal_id,
                manifest_hash,
            } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                let ProposalStatus::Approved {
                    manifest_hash: approved_hash,
                    ..
                } = &proposal.status
                else {
                    return Err(WorldError::ProposalInvalidState {
                        proposal_id: *proposal_id,
                        expected: "approved".to_string(),
                        found: proposal.status.label(),
                    });
                };
                let applied_hash = manifest_hash
                    .clone()
                    .unwrap_or_else(|| approved_hash.clone());
                proposal.status = ProposalStatus::Applied {
                    manifest_hash: applied_hash,
                };
            }
        }
        Ok(())
    }
}
