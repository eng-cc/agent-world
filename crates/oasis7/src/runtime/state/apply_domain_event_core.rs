use super::*;

use crate::runtime::AgentActivityV1;

#[path = "apply_domain_event_core_late.rs"]
mod late;

impl WorldState {
    pub(super) fn apply_domain_event_core(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        match event {
            DomainEvent::AgentRegistered { agent_id, pos } => {
                let state = AgentState::new(agent_id, *pos);
                let mut cell = AgentCell::new(state, now);
                cell.activity = Some(AgentActivityV1::idle(now));
                self.agents.insert(agent_id.clone(), cell);
            }
            DomainEvent::AgentMoved { agent_id, to, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.state.pos = *to;
                    cell.last_active = now;
                }
            }
            DomainEvent::ActionAccepted { .. } => {}
            DomainEvent::ActionRejected { .. } => {}
            DomainEvent::AgentIntentProposed { .. }
            | DomainEvent::AgentIntentSubmitted { .. }
            | DomainEvent::AgentIntentAccepted { .. }
            | DomainEvent::AgentIntentReplaced { .. }
            | DomainEvent::AgentIntentTransitioned { .. } => {
                unreachable!("Agent Intent events are handled by apply_domain_event_intent")
            }
            DomainEvent::Observation { .. } => {}
            DomainEvent::BodyAttributesUpdated { agent_id, view, .. } => {
                let cell =
                    self.agents
                        .get_mut(agent_id)
                        .ok_or_else(|| WorldError::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                cell.state.body_view = view.clone();
                cell.last_active = now;
            }
            DomainEvent::BodyAttributesRejected { agent_id, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: agent_id.clone(),
                    });
                }
            }
            DomainEvent::BodyInterfaceExpanded {
                agent_id,
                slot_capacity,
                expansion_level,
                consumed_item_id,
                new_slot_id,
                slot_type,
                ..
            } => {
                let cell =
                    self.agents
                        .get_mut(agent_id)
                        .ok_or_else(|| WorldError::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                cell.state
                    .body_state
                    .consume_interface_module_item(consumed_item_id)
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "consume interface module item failed for {agent_id}: {reason}"
                        ),
                    })?;
                cell.state.body_state.slot_capacity = *slot_capacity;
                cell.state.body_state.expansion_level = *expansion_level;
                if !cell
                    .state
                    .body_state
                    .slots
                    .iter()
                    .any(|slot| slot.slot_id == *new_slot_id)
                {
                    cell.state
                        .body_state
                        .slots
                        .push(crate::models::BodyModuleSlot {
                            slot_id: new_slot_id.clone(),
                            slot_type: *slot_type,
                            installed_module: None,
                            locked: false,
                        });
                }
                cell.last_active = now;
            }
            DomainEvent::BodyInterfaceExpandRejected { agent_id, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: agent_id.clone(),
                    });
                }
            }
            DomainEvent::ModuleArtifactDeployed { .. } => {
                self.prepare_module_marketplace_event(event, now)?
                    .install_infallible(self);
            }
            DomainEvent::ModuleInstalled { .. } | DomainEvent::ModuleUpgraded { .. } => {
                self.prepare_module_instance_event(event, now)?
                    .install_infallible(self);
            }
            DomainEvent::ModuleReleaseRequested {
                request_id,
                requester_agent_id,
                manifest,
                activate,
                install_target,
                required_roles,
                profile_changes,
            } => {
                if *request_id == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "module release request_id must be > 0".to_string(),
                    });
                }
                if !self.agents.contains_key(requester_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: requester_agent_id.clone(),
                    });
                }
                if self.module_release_requests.contains_key(request_id) {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release request already exists: request_id={request_id}"
                        ),
                    });
                }

                let mut normalized_roles: Vec<String> = required_roles
                    .iter()
                    .map(|role| role.trim().to_ascii_lowercase())
                    .filter(|role| !role.is_empty())
                    .collect();
                normalized_roles.sort();
                normalized_roles.dedup();
                if normalized_roles.is_empty() {
                    normalized_roles = default_module_release_required_roles();
                }

                self.module_release_requests.insert(
                    *request_id,
                    ModuleReleaseRequestState {
                        request_id: *request_id,
                        requester_agent_id: requester_agent_id.clone(),
                        manifest: manifest.clone(),
                        activate: *activate,
                        install_target: install_target.clone(),
                        profile_changes: profile_changes.clone(),
                        required_roles: normalized_roles,
                        role_approvals: BTreeMap::new(),
                        attestations: BTreeMap::new(),
                        status: ModuleReleaseRequestStatus::Requested,
                        shadow_manifest_hash: None,
                        applied_manifest_hash: None,
                        applied_proposal_id: None,
                        rejected_reason: None,
                        created_at: now,
                        updated_at: now,
                    },
                );
                self.module_release_manifest_mappings.insert(
                    *request_id,
                    ModuleReleaseManifestMappingState {
                        request_id: *request_id,
                        release_id: format!("release-{request_id}"),
                        module_id: manifest.module_id.clone(),
                        attestation_count: 0,
                        release_wasm_hash: None,
                        release_source_hash: None,
                        release_build_manifest_hash: None,
                        release_builder_image_digest: None,
                        release_container_platform: None,
                        release_canonicalizer_version: None,
                        attestation_platforms: Vec::new(),
                        attestation_proof_cids: Vec::new(),
                        receipt_evidence_conflict: false,
                        shadow_manifest_hash: None,
                        applied_manifest_hash: None,
                        applied_proposal_id: None,
                        status: ModuleReleaseRequestStatus::Requested,
                        created_at: now,
                        updated_at: now,
                    },
                );
                self.next_module_release_request_id = self
                    .next_module_release_request_id
                    .max(request_id.saturating_add(1));
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ModuleReleaseShadowed { .. } => {
                self.prepare_module_release_event(event, now)?
                    .install_infallible(self);
            }
            DomainEvent::ModuleReleaseAttested {
                request_id,
                operator_agent_id,
                signer_node_id,
                platform,
                build_manifest_hash,
                source_hash,
                wasm_hash,
                proof_cid,
                builder_image_digest,
                container_platform,
                canonicalizer_version,
            } => {
                if !self.agents.contains_key(operator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
                if signer_node_id.trim().is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release attestation signer_node_id cannot be empty (request_id={request_id})"
                        ),
                    });
                }
                if !self
                    .node_identity_bindings
                    .contains_key(signer_node_id.trim())
                {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release attestation signer_node_id is untrusted: {}",
                            signer_node_id
                        ),
                    });
                }
                let normalized_platform = platform.trim().to_ascii_lowercase();
                if normalized_platform.is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release attestation platform cannot be empty (request_id={request_id})"
                        ),
                    });
                }
                let request = self
                    .module_release_requests
                    .get_mut(request_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release attestation rejected: request not found ({request_id})"
                        ),
                    })?;
                if matches!(
                    request.status,
                    ModuleReleaseRequestStatus::Rejected | ModuleReleaseRequestStatus::Applied
                ) {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release attestation invalid status for request {}: {:?}",
                            request_id, request.status
                        ),
                    });
                }
                if request.manifest.wasm_hash != *wasm_hash {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release attestation wasm hash mismatch: request_id={} expected={} found={}",
                            request_id, request.manifest.wasm_hash, wasm_hash
                        ),
                    });
                }

                let attestation_key = format!("{}|{}", signer_node_id.trim(), normalized_platform);
                let next_attestation = ModuleReleaseAttestationState {
                    request_id: *request_id,
                    signer_node_id: signer_node_id.trim().to_string(),
                    platform: normalized_platform,
                    submitted_by_agent_id: operator_agent_id.clone(),
                    build_manifest_hash: build_manifest_hash.clone(),
                    source_hash: source_hash.clone(),
                    wasm_hash: wasm_hash.clone(),
                    proof_cid: proof_cid.clone(),
                    builder_image_digest: builder_image_digest.clone(),
                    container_platform: container_platform.clone(),
                    canonicalizer_version: canonicalizer_version.clone(),
                    submitted_at: now,
                };
                if let Some(existing) = request.attestations.get(attestation_key.as_str()) {
                    let same_payload = existing.request_id == *request_id
                        && existing.signer_node_id == next_attestation.signer_node_id
                        && existing.platform == next_attestation.platform
                        && existing.build_manifest_hash == next_attestation.build_manifest_hash
                        && existing.source_hash == next_attestation.source_hash
                        && existing.wasm_hash == next_attestation.wasm_hash
                        && existing.proof_cid == next_attestation.proof_cid
                        && existing.builder_image_digest == next_attestation.builder_image_digest
                        && existing.container_platform == next_attestation.container_platform
                        && existing.canonicalizer_version == next_attestation.canonicalizer_version;
                    if !same_payload {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "module release attestation conflict: request_id={} signer={} platform={}",
                                request_id, signer_node_id, platform
                            ),
                        });
                    }
                } else {
                    request
                        .attestations
                        .insert(attestation_key, next_attestation.clone());
                }
                request.updated_at = now;
                let mapping = self
                    .module_release_manifest_mappings
                    .get_mut(request_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module release mapping missing for attestation request_id={request_id}"
                        ),
                    })?;
                mapping.attestation_count = request.attestations.len() as u32;
                if !mapping
                    .attestation_platforms
                    .iter()
                    .any(|platform_name| platform_name == &next_attestation.platform)
                {
                    mapping
                        .attestation_platforms
                        .push(next_attestation.platform.clone());
                    mapping.attestation_platforms.sort();
                }
                if !mapping
                    .attestation_proof_cids
                    .iter()
                    .any(|existing_cid| existing_cid == &next_attestation.proof_cid)
                {
                    mapping
                        .attestation_proof_cids
                        .push(next_attestation.proof_cid.clone());
                    mapping.attestation_proof_cids.sort();
                }
                match mapping.release_wasm_hash.as_ref() {
                    None => {
                        mapping.release_wasm_hash = Some(next_attestation.wasm_hash.clone());
                        mapping.release_source_hash = Some(next_attestation.source_hash.clone());
                        mapping.release_build_manifest_hash =
                            Some(next_attestation.build_manifest_hash.clone());
                        mapping.release_builder_image_digest =
                            Some(next_attestation.builder_image_digest.clone());
                        mapping.release_container_platform =
                            Some(next_attestation.container_platform.clone());
                        mapping.release_canonicalizer_version =
                            Some(next_attestation.canonicalizer_version.clone());
                    }
                    Some(existing_wasm_hash) => {
                        let same_release_evidence = existing_wasm_hash
                            == &next_attestation.wasm_hash
                            && mapping.release_source_hash.as_ref()
                                == Some(&next_attestation.source_hash)
                            && mapping.release_build_manifest_hash.as_ref()
                                == Some(&next_attestation.build_manifest_hash)
                            && mapping.release_builder_image_digest.as_ref()
                                == Some(&next_attestation.builder_image_digest)
                            && mapping.release_container_platform.as_ref()
                                == Some(&next_attestation.container_platform)
                            && mapping.release_canonicalizer_version.as_ref()
                                == Some(&next_attestation.canonicalizer_version);
                        if !same_release_evidence {
                            mapping.receipt_evidence_conflict = true;
                        }
                    }
                }
                mapping.updated_at = now;
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ModuleReleaseRoleApproved { .. } => {
                self.prepare_module_release_event(event, now)?
                    .install_infallible(self);
            }
            DomainEvent::ModuleReleaseRolesBound {
                operator_agent_id,
                target_agent_id,
                roles,
            } => {
                if !self.agents.contains_key(operator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
                if !self.agents.contains_key(target_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: target_agent_id.clone(),
                    });
                }

                let normalized_roles: BTreeSet<String> = roles
                    .iter()
                    .map(|role| role.trim().to_ascii_lowercase())
                    .filter(|role| !role.is_empty())
                    .collect();
                if normalized_roles.is_empty() {
                    self.module_release_role_bindings.remove(target_agent_id);
                } else {
                    self.module_release_role_bindings
                        .insert(target_agent_id.clone(), normalized_roles);
                }
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
                if operator_agent_id != target_agent_id {
                    if let Some(cell) = self.agents.get_mut(target_agent_id) {
                        cell.last_active = now;
                    }
                }
            }
            DomainEvent::ModuleReleaseRejected { .. } => {
                self.prepare_module_release_event(event, now)?
                    .install_infallible(self);
            }
            DomainEvent::ModuleReleaseApplied { .. } => {
                self.prepare_module_release_event(event, now)?
                    .install_infallible(self);
            }
            DomainEvent::ModuleRollbackApplied { .. } => {
                self.prepare_module_instance_event(event, now)?
                    .install_infallible(self);
            }
            event @ DomainEvent::ModuleArtifactListed { .. }
            | event @ DomainEvent::ModuleArtifactDelisted { .. }
            | event @ DomainEvent::ModuleArtifactDestroyed { .. }
            | event @ DomainEvent::ModuleArtifactBidPlaced { .. }
            | event @ DomainEvent::ModuleArtifactBidCancelled { .. }
            | event @ DomainEvent::ModuleArtifactSaleCompleted { .. }
            | event @ DomainEvent::ResourceTransferred { .. }
            | event @ DomainEvent::DataCollected { .. }
            | event @ DomainEvent::DataCollectedAuthenticated { .. }
            | event @ DomainEvent::DataAccessGranted { .. }
            | event @ DomainEvent::DataAccessRevoked { .. }
            | event @ DomainEvent::PowerRedeemed { .. }
            | event @ DomainEvent::PowerRedeemRejected { .. }
            | event @ DomainEvent::NodePointsSettlementApplied { .. }
            | event @ DomainEvent::MainTokenGenesisInitialized { .. }
            | event @ DomainEvent::MainTokenVestingClaimed { .. }
            | event @ DomainEvent::MainTokenTransferred { .. }
            | event @ DomainEvent::MainTokenEpochIssued { .. }
            | event @ DomainEvent::MainTokenFeeSettled { .. }
            | event @ DomainEvent::MainTokenPolicyUpdateScheduled { .. }
            | event @ DomainEvent::MainTokenTreasuryDistributed { .. }
            | event @ DomainEvent::RestrictedStarterClaimLiveopsPoolToppedUp { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantIssued { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantExpired { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantRevoked { .. }
            | event @ DomainEvent::LogisticsRouteRegistered { .. }
            | event @ DomainEvent::LogisticsRouteAvailabilityChanged { .. }
            | event @ DomainEvent::LogisticsPathRerouted { .. }
            | event @ DomainEvent::MaterialTransferred { .. }
            | event @ DomainEvent::MaterialTransitStarted { .. }
            | event @ DomainEvent::MaterialTransitCompleted { .. }
            | event @ DomainEvent::FactoryBuildStarted { .. }
            | event @ DomainEvent::FactoryBuilt { .. }
            | event @ DomainEvent::FactoryDurabilityChanged { .. }
            | event @ DomainEvent::FactoryMaintained { .. }
            | event @ DomainEvent::FactoryRecycled { .. }
            | event @ DomainEvent::RecipeStarted { .. }
            | event @ DomainEvent::RecipeCompleted { .. }
            | event @ DomainEvent::FactoryProductionBlocked { .. }
            | event @ DomainEvent::FactoryProductionResumed { .. }
            | event @ DomainEvent::FactoryProductionPaused { .. }
            | event @ DomainEvent::MaterialProfileGoverned { .. }
            | event @ DomainEvent::ProductProfileGoverned { .. }
            | event @ DomainEvent::RecipeProfileGoverned { .. }
            | event @ DomainEvent::FactoryProfileGoverned { .. } => {
                self.apply_domain_event_core_late(event, now)?;
            }
            _ => unreachable!("apply_domain_event_core received unsupported event variant"),
        }
        Ok(())
    }
}
