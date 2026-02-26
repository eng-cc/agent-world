use super::super::util::{hash_json, sha256_hex};
use super::super::{
    apply_manifest_patch, GovernanceEvent, GovernanceFinalityCertificate, Manifest, ManifestPatch,
    ManifestUpdate, Proposal, ProposalDecision, ProposalId, ProposalStatus, WorldError,
    WorldEventBody,
};
use super::World;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

const LOCAL_GOVERNANCE_FINALITY_SIGNERS: [(&str, &str); 2] = [
    (
        "governance.local.finality.signer.1",
        "agent-world-governance-local-finality-signer-1-v1",
    ),
    (
        "governance.local.finality.signer.2",
        "agent-world-governance-local-finality-signer-2-v1",
    ),
];

pub(super) fn local_governance_finality_signer_public_keys() -> Vec<(String, String)> {
    let mut keys = Vec::with_capacity(LOCAL_GOVERNANCE_FINALITY_SIGNERS.len());
    for (node_id, seed_label) in LOCAL_GOVERNANCE_FINALITY_SIGNERS {
        let signing_key = local_governance_finality_signing_key(seed_label);
        keys.push((
            node_id.to_string(),
            hex::encode(signing_key.verifying_key().to_bytes()),
        ));
    }
    keys
}

fn local_governance_finality_signing_key(seed_label: &str) -> SigningKey {
    let seed = sha256_hex(seed_label.as_bytes());
    let seed_bytes = hex::decode(seed).expect("decode governance finality seed");
    let private_key_bytes: [u8; 32] = seed_bytes
        .as_slice()
        .try_into()
        .expect("governance finality seed is 32 bytes");
    SigningKey::from_bytes(&private_key_bytes)
}

fn decode_hex_array<const N: usize>(raw: &str, label: &str) -> Result<[u8; N], WorldError> {
    let bytes = hex::decode(raw).map_err(|_| WorldError::GovernanceFinalityInvalid {
        reason: format!("{label} is not valid hex"),
    })?;
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| WorldError::GovernanceFinalityInvalid {
            reason: format!("{label} has invalid length"),
        })
}

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
        let proposal_id = self.allocate_next_proposal_id();
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
        let proposal_id = self.allocate_next_proposal_id();
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

    pub fn build_local_finality_certificate(
        &self,
        proposal_id: ProposalId,
    ) -> Result<GovernanceFinalityCertificate, WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        let manifest_hash = match &proposal.status {
            ProposalStatus::Approved { manifest_hash, .. } => manifest_hash.clone(),
            other => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "approved".to_string(),
                    found: other.label(),
                })
            }
        };
        let consensus_height = self.journal.events.len() as u64 + 1;
        let threshold = 2_u16;
        let mut signatures = std::collections::BTreeMap::new();
        for (node_id, seed_label) in LOCAL_GOVERNANCE_FINALITY_SIGNERS {
            let payload = GovernanceFinalityCertificate::signing_payload_v1(
                proposal_id,
                manifest_hash.as_str(),
                consensus_height,
                threshold,
                node_id,
            );
            let signing_key = local_governance_finality_signing_key(seed_label);
            let signature = signing_key.sign(payload.as_slice());
            signatures.insert(
                node_id.to_string(),
                format!(
                    "{}{}",
                    GovernanceFinalityCertificate::SIGNATURE_PREFIX_ED25519_V1,
                    hex::encode(signature.to_bytes())
                ),
            );
        }
        Ok(GovernanceFinalityCertificate {
            proposal_id,
            manifest_hash,
            consensus_height,
            threshold,
            signatures,
        })
    }

    pub fn apply_proposal(&mut self, proposal_id: ProposalId) -> Result<String, WorldError> {
        let finality_certificate = self.build_local_finality_certificate(proposal_id)?;
        self.apply_proposal_with_finality(proposal_id, &finality_certificate)
    }

    pub fn apply_proposal_with_finality(
        &mut self,
        proposal_id: ProposalId,
        finality_certificate: &GovernanceFinalityCertificate,
    ) -> Result<String, WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        let (manifest, actor, approved_manifest_hash) = match &proposal.status {
            ProposalStatus::Approved { manifest_hash, .. } => (
                proposal.manifest.clone(),
                proposal.author.clone(),
                manifest_hash.clone(),
            ),
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
        let proposal_manifest_hash = hash_json(&manifest)?;
        if proposal_manifest_hash != approved_manifest_hash {
            return Err(WorldError::GovernanceFinalityInvalid {
                reason: "approved manifest hash drift".to_string(),
            });
        }
        let applied_hash = hash_json(&applied_manifest)?;
        self.validate_governance_finality_certificate(
            proposal_id,
            approved_manifest_hash.as_str(),
            finality_certificate,
        )?;

        if let Some(changes) = module_changes {
            self.apply_module_changes(proposal_id, &changes, &actor)?;
        }
        let update = ManifestUpdate {
            manifest: applied_manifest,
            manifest_hash: applied_hash.clone(),
        };
        self.append_event(WorldEventBody::ManifestUpdated(update), None)?;
        let event = GovernanceEvent::Applied {
            proposal_id,
            manifest_hash: Some(applied_hash.clone()),
            consensus_height: Some(finality_certificate.consensus_height),
            threshold: Some(finality_certificate.threshold),
            signer_node_ids: finality_certificate.signatures.keys().cloned().collect(),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(applied_hash)
    }

    fn validate_governance_finality_certificate(
        &self,
        proposal_id: ProposalId,
        manifest_hash: &str,
        certificate: &GovernanceFinalityCertificate,
    ) -> Result<(), WorldError> {
        if certificate.proposal_id != proposal_id {
            return Err(WorldError::GovernanceFinalityInvalid {
                reason: format!(
                    "proposal_id mismatch: expected={} found={}",
                    proposal_id, certificate.proposal_id
                ),
            });
        }
        if certificate.manifest_hash != manifest_hash {
            return Err(WorldError::GovernanceFinalityInvalid {
                reason: "manifest_hash mismatch".to_string(),
            });
        }
        if certificate.consensus_height == 0 {
            return Err(WorldError::GovernanceFinalityInvalid {
                reason: "consensus_height must be > 0".to_string(),
            });
        }
        if certificate.threshold < 2 {
            return Err(WorldError::GovernanceFinalityInvalid {
                reason: "threshold must be >= 2".to_string(),
            });
        }
        if certificate.signatures.len() < certificate.threshold as usize {
            return Err(WorldError::GovernanceFinalityInvalid {
                reason: format!(
                    "signatures below threshold: signatures={} threshold={}",
                    certificate.signatures.len(),
                    certificate.threshold
                ),
            });
        }
        for (node_id, signature_with_prefix) in &certificate.signatures {
            let signer_public_key = self.node_identity_public_key(node_id).ok_or_else(|| {
                WorldError::GovernanceFinalityInvalid {
                    reason: format!("untrusted signer node_id: {node_id}"),
                }
            })?;
            let signature_hex = signature_with_prefix
                .strip_prefix(GovernanceFinalityCertificate::SIGNATURE_PREFIX_ED25519_V1)
                .ok_or_else(|| WorldError::GovernanceFinalityInvalid {
                    reason: format!("signature prefix mismatch for signer {node_id}"),
                })?;
            let payload = GovernanceFinalityCertificate::signing_payload_v1(
                certificate.proposal_id,
                certificate.manifest_hash.as_str(),
                certificate.consensus_height,
                certificate.threshold,
                node_id.as_str(),
            );
            let public_key_bytes =
                decode_hex_array::<32>(signer_public_key, "governance finality signer public key")?;
            let signature_bytes =
                decode_hex_array::<64>(signature_hex, "governance finality signature")?;
            let verifying_key = VerifyingKey::from_bytes(&public_key_bytes).map_err(|_| {
                WorldError::GovernanceFinalityInvalid {
                    reason: format!("invalid signer public key for {node_id}"),
                }
            })?;
            let signature = Signature::from_bytes(&signature_bytes);
            verifying_key
                .verify(payload.as_slice(), &signature)
                .map_err(|error| WorldError::GovernanceFinalityInvalid {
                    reason: format!("signature verification failed for {node_id}: {error}"),
                })?;
        }
        Ok(())
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
                let proposal =
                    self.proposals
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
                let proposal =
                    self.proposals
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
                ..
            } => {
                let proposal =
                    self.proposals
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
