use super::*;
use crate::runtime::{
    CapabilityAuthorizationEvent, EffectIntent, EffectReceipt, Manifest, ManifestUpdate,
    ModuleEvent, ModuleRegistry,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) enum PreparedEventStateDelta {
    NoState,
    ModuleMarketplace(
        super::super::super::state::module_marketplace_transition::PreparedModuleMarketplace,
    ),
    ModuleRelease(super::super::super::state::module_release_transition::PreparedModuleRelease),
    ModuleInstance {
        prepared: super::super::super::state::module_instance_transition::PreparedModuleInstance,
        schedule: Option<(String, Option<WorldTime>)>,
    },
    ModuleStateUpdated {
        module_states: BTreeMap<String, Vec<u8>>,
    },
    ModuleRuntimeCharged(super::super::module_runtime_metering::PreparedModuleRuntimeCharge),
    EffectQueued {
        intent_id: String,
        pending_effects: VecDeque<EffectIntent>,
        pending_effects_evicted: u64,
    },
    ReceiptAppended {
        intent_id: String,
        pending_effects: VecDeque<EffectIntent>,
        inflight_effects: BTreeMap<String, EffectIntent>,
    },
    ModuleEvent {
        event: ModuleEvent,
        module_registry: ModuleRegistry,
        module_artifacts: BTreeSet<String>,
        module_tick_schedule: BTreeMap<String, WorldTime>,
        cache_invalidations: BTreeSet<String>,
    },
    ManifestUpdated {
        update: ManifestUpdate,
        manifest: Manifest,
    },
    GovernanceRegistry(
        super::super::governance_registry_publication::PreparedGovernanceRegistryEvent,
    ),
    CapabilityAuthorization(
        super::super::capability_authorization_publication::PreparedCapabilityAuthorizationEvent,
    ),
    CapabilityCommandCommit(
        super::super::capability_authorization_command_projection::PreparedCapabilityCommandCommit,
    ),
    CapabilityEffectReceipt(
        super::super::capability_effect_receipt_projection::PreparedCapabilityEffectReceipt,
    ),
    AgentIntent(super::super::agent_intent_publication::PreparedAgentIntent),
    EconomyData(super::super::economy_data_publication::PreparedEconomyDataEvent),
    PowerRedemption(super::super::power_redemption_publication::PreparedPowerRedemptionEvent),
    NodePointsSettlement(
        super::super::node_points_settlement_publication::PreparedNodePointsSettlement,
    ),
    MainTokenMonetary(
        super::super::main_token_monetary_publication::PreparedMainTokenMonetaryEvent,
    ),
    MainTokenGovernanceMonetary(
        super::super::main_token_governance_monetary_publication::PreparedMainTokenGovernanceMonetaryEvent,
    ),
    MainTokenRestrictedClaim(
        super::super::main_token_restricted_claim_publication::PreparedMainTokenRestrictedClaimEvent,
    ),
    Body(PreparedBodyAttributesUpdate),
    RouteOnly {
        agent_id: String,
    },
    GovernanceEmergencyBrake {
        next_until_tick: Option<WorldTime>,
    },
    GovernanceFinalityEpochSnapshot {
        epoch_id: u64,
        next: Option<super::super::super::GovernanceFinalityEpochSnapshot>,
    },
    GovernanceEmergencyVeto {
        proposal_id: ProposalId,
        next: super::super::super::Proposal,
    },
    GovernanceProposal {
        proposal_id: ProposalId,
        next: super::super::super::Proposal,
        next_proposal_id: ProposalId,
        next_proposal_id_era: u64,
    },
    GovernanceProposalShadow {
        proposal_id: ProposalId,
        next: super::super::super::Proposal,
    },
    GovernanceProposalStatus {
        event: GovernanceEvent,
        proposal_id: ProposalId,
        next: super::super::super::Proposal,
    },
    GovernanceIdentityPenaltyAppeal {
        penalty_id: u64,
        next: GovernanceIdentityPenaltyRecord,
    },
    GovernanceIdentityPenaltyApplication {
        penalty_id: u64,
        target_agent_id: String,
        next: GovernanceIdentityPenaltyRecord,
        next_profile: GovernanceIdentityProfileState,
        profile_was_present: bool,
        next_penalty_id: u64,
    },
    GovernanceIdentityPenaltyResolution {
        penalty_id: u64,
        target_agent_id: String,
        next: GovernanceIdentityPenaltyRecord,
        next_profile: GovernanceIdentityProfileState,
    },
}

impl PreparedEventStateDelta {
    fn for_body(body: &WorldEventBody) -> Option<Self> {
        match body {
            WorldEventBody::PolicyDecisionRecorded(_)
            | WorldEventBody::RuleDecisionRecorded(_)
            | WorldEventBody::ActionOverridden(_)
            | WorldEventBody::ModuleCallFailed(_)
            | WorldEventBody::ModuleEmitted(_)
            | WorldEventBody::SnapshotCreated(_)
            | WorldEventBody::RollbackApplied(_) => Some(Self::NoState),
            WorldEventBody::Governance(GovernanceEvent::EmergencyBrakeActivated {
                active_until_tick,
                ..
            }) => Some(Self::GovernanceEmergencyBrake {
                next_until_tick: Some(*active_until_tick),
            }),
            WorldEventBody::Governance(GovernanceEvent::EmergencyBrakeReleased { .. }) => {
                Some(Self::GovernanceEmergencyBrake {
                    next_until_tick: None,
                })
            }
            _ => None,
        }
    }

    fn matches_body(&self, body: &WorldEventBody) -> bool {
        match self {
            Self::ModuleMarketplace(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::ModuleRelease(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::ModuleInstance { prepared, .. } => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::ModuleStateUpdated { module_states } => matches!(body,
                WorldEventBody::ModuleStateUpdated(update)
                if module_states.len() == 1 && module_states.get(&update.module_id) == Some(&update.state)),
            Self::ModuleRuntimeCharged(prepared) => matches!(body,
                WorldEventBody::ModuleRuntimeCharged(charge)
                if prepared.agents.contains_key(&charge.payer_agent_id)),
            Self::EffectQueued { intent_id, .. } => matches!(
                body,
                WorldEventBody::EffectQueued(intent) if &intent.intent_id == intent_id
            ),
            Self::ReceiptAppended { intent_id, .. } => matches!(
                body,
                WorldEventBody::ReceiptAppended(receipt) if &receipt.intent_id == intent_id
            ),
            Self::ModuleEvent { event, .. } => {
                matches!(body, WorldEventBody::ModuleEvent(body_event) if body_event == event)
            }
            Self::ManifestUpdated { update, .. } => {
                matches!(body, WorldEventBody::ManifestUpdated(body_update) if body_update == update)
            }
            Self::GovernanceRegistry(prepared) => {
                matches!(body, WorldEventBody::Governance(event) if prepared.matches_event(event))
            }
            Self::CapabilityAuthorization(prepared) => matches!(
                body,
                WorldEventBody::CapabilityAuthorization(event) if prepared.matches_event(event)
            ),
            Self::CapabilityCommandCommit(prepared) => matches!(
                body,
                WorldEventBody::CapabilityAuthorization(event) if prepared.matches_event(event)
            ),
            Self::CapabilityEffectReceipt(prepared) => matches!(
                body,
                WorldEventBody::CapabilityAuthorization(event) if prepared.matches_event(event)
            ),
            Self::AgentIntent(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::EconomyData(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::PowerRedemption(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::NodePointsSettlement(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::MainTokenMonetary(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::MainTokenGovernanceMonetary(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::MainTokenRestrictedClaim(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::NoState => matches!(Self::for_body(body), Some(Self::NoState)),
            Self::Body(prepared) => {
                matches!(body, WorldEventBody::Domain(event) if prepared.matches_event(event))
            }
            Self::RouteOnly { agent_id } => matches!(
                body,
                WorldEventBody::Domain(DomainEvent::BodyAttributesRejected {
                    agent_id: event_agent_id,
                    ..
                }) if event_agent_id == agent_id
            ),
            Self::GovernanceEmergencyBrake { next_until_tick } => match body {
                WorldEventBody::Governance(GovernanceEvent::EmergencyBrakeActivated {
                    active_until_tick,
                    ..
                }) => next_until_tick == &Some(*active_until_tick),
                WorldEventBody::Governance(GovernanceEvent::EmergencyBrakeReleased { .. }) => {
                    next_until_tick.is_none()
                }
                _ => false,
            },
            Self::GovernanceFinalityEpochSnapshot { epoch_id, next } => match body {
                WorldEventBody::Governance(GovernanceEvent::FinalityEpochSnapshotSet {
                    snapshot,
                    ..
                }) => *epoch_id == snapshot.epoch_id && next.as_ref() == Some(snapshot),
                WorldEventBody::Governance(GovernanceEvent::FinalityEpochSnapshotRemoved {
                    epoch_id: event_epoch_id,
                    ..
                }) => *epoch_id == *event_epoch_id && next.is_none(),
                _ => false,
            },
            Self::GovernanceEmergencyVeto { proposal_id, .. } => matches!(
                body,
                WorldEventBody::Governance(GovernanceEvent::EmergencyVetoed {
                    proposal_id: event_proposal_id,
                    ..
                }) if proposal_id == event_proposal_id
            ),
            Self::GovernanceProposal { proposal_id, .. } => matches!(
                body,
                WorldEventBody::Governance(GovernanceEvent::Proposed {
                    proposal_id: event_proposal_id,
                    ..
                }) if proposal_id == event_proposal_id
            ),
            Self::GovernanceProposalShadow { proposal_id, .. } => matches!(
                body,
                WorldEventBody::Governance(GovernanceEvent::ShadowReport {
                    proposal_id: event_proposal_id,
                    ..
                }) if proposal_id == event_proposal_id
            ),
            Self::GovernanceProposalStatus { event, .. } => {
                matches!(body, WorldEventBody::Governance(body_event) if body_event == event)
            }
            Self::GovernanceIdentityPenaltyAppeal { penalty_id, .. } => matches!(
                body,
                WorldEventBody::Governance(GovernanceEvent::IdentityPenaltyAppealed {
                    penalty_id: event_penalty_id,
                    ..
                }) if penalty_id == event_penalty_id
            ),
            Self::GovernanceIdentityPenaltyApplication { penalty_id, .. } => matches!(
                body,
                WorldEventBody::Governance(GovernanceEvent::IdentityPenaltyApplied {
                    penalty_id: event_penalty_id,
                    ..
                }) if penalty_id == event_penalty_id
            ),
            Self::GovernanceIdentityPenaltyResolution { penalty_id, .. } => matches!(
                body,
                WorldEventBody::Governance(GovernanceEvent::IdentityPenaltyResolved {
                    penalty_id: event_penalty_id,
                    ..
                }) if penalty_id == event_penalty_id
            ),
        }
    }

    fn state_overlay(&self, event: DomainEvent) -> super::super::super::BodyOverlay {
        match self {
            Self::ModuleMarketplace(_) => unreachable!("marketplace uses a sparse overlay"),
            Self::ModuleRelease(_) => unreachable!("release uses a sparse release overlay"),
            Self::ModuleInstance { .. }
            | Self::ModuleStateUpdated { .. }
            | Self::ModuleRuntimeCharged(_) => {
                unreachable!("module output uses a command overlay")
            }
            Self::EffectQueued { .. } | Self::ReceiptAppended { .. } => {
                unreachable!("effect sidecars do not have a state overlay")
            }
            Self::ModuleEvent { .. } | Self::ManifestUpdated { .. } => {
                unreachable!("module metadata does not have a body state overlay")
            }
            Self::GovernanceRegistry(_) => {
                unreachable!("governance registry uses a state projection")
            }
            Self::CapabilityAuthorization(_) => {
                unreachable!("capability authorization uses sidecar state")
            }
            Self::CapabilityCommandCommit(_) => {
                unreachable!("capability command commit uses sidecar state")
            }
            Self::CapabilityEffectReceipt(_) => {
                unreachable!("capability effect receipt uses sidecar state")
            }
            Self::AgentIntent(_) => unreachable!("agent intent uses a sparse state projection"),
            Self::EconomyData(_) => {
                unreachable!("economy/data events use a sparse state projection")
            }
            Self::PowerRedemption(_) => {
                unreachable!("power redemption uses a sparse state projection")
            }
            Self::NodePointsSettlement(_) => {
                unreachable!("node points settlement uses a sparse state projection")
            }
            Self::MainTokenMonetary(_) => {
                unreachable!("main-token monetary events use a sparse state projection")
            }
            Self::MainTokenGovernanceMonetary(_) => {
                unreachable!("main-token governance monetary events use a sparse state projection")
            }
            Self::MainTokenRestrictedClaim(_) => {
                unreachable!("restricted-claim events use a sparse state projection")
            }
            Self::NoState => unreachable!("NoState does not have a state overlay"),
            Self::Body(prepared) => prepared.body_overlay().with_routed_domain_event(event),
            Self::RouteOnly { agent_id } => {
                { super::super::super::BodyOverlay::route_only(agent_id.clone()) }
                    .with_routed_domain_event(event)
            }
            Self::GovernanceEmergencyBrake { .. } => {
                unreachable!("governance emergency brake does not have a state overlay")
            }
            Self::GovernanceFinalityEpochSnapshot { .. } => {
                unreachable!("governance finality snapshot does not have a state overlay")
            }
            Self::GovernanceEmergencyVeto { .. } => {
                unreachable!("governance emergency veto does not have a state overlay")
            }
            Self::GovernanceProposal { .. } => {
                unreachable!("governance proposal does not have a state overlay")
            }
            Self::GovernanceProposalShadow { .. } => {
                unreachable!("governance proposal shadow does not have a state overlay")
            }
            Self::GovernanceProposalStatus { .. } => {
                unreachable!("governance proposal status does not have a state overlay")
            }
            Self::GovernanceIdentityPenaltyAppeal { .. } => {
                unreachable!("identity penalty appeal does not have a state overlay")
            }
            Self::GovernanceIdentityPenaltyApplication { .. } => {
                unreachable!("identity penalty application does not have a state overlay")
            }
            Self::GovernanceIdentityPenaltyResolution { .. } => {
                unreachable!("identity penalty resolution does not have a state overlay")
            }
        }
    }

    fn install_infallible(self, world: &mut World) {
        match self {
            Self::ModuleMarketplace(prepared) => prepared.install_infallible(&mut world.state),
            Self::ModuleRelease(prepared) => prepared.install_infallible(&mut world.state),
            Self::ModuleInstance { prepared, schedule } => {
                prepared.install_infallible(&mut world.state);
                world.install_prepared_module_instance_schedule(schedule);
            }
            Self::ModuleStateUpdated { module_states } => {
                world.state.module_states.extend(module_states)
            }
            Self::ModuleRuntimeCharged(prepared) => prepared.install_infallible(world),
            Self::EffectQueued {
                pending_effects,
                pending_effects_evicted,
                ..
            } => {
                world.pending_effects = pending_effects;
                world.runtime_backpressure_stats.pending_effects_evicted = world
                    .runtime_backpressure_stats
                    .pending_effects_evicted
                    .saturating_add(pending_effects_evicted);
            }
            Self::ReceiptAppended {
                pending_effects,
                inflight_effects,
                ..
            } => {
                world.pending_effects = pending_effects;
                world.inflight_effects = inflight_effects;
            }
            Self::ModuleEvent {
                module_registry,
                module_artifacts,
                module_tick_schedule,
                cache_invalidations,
                ..
            } => {
                world.module_registry = module_registry;
                world.module_artifacts = module_artifacts;
                world.module_tick_schedule = module_tick_schedule;
                for record_key in cache_invalidations {
                    let prefix = format!("{record_key}|");
                    world
                        .prepared_subscription_cache
                        .retain(|key, _| !key.starts_with(prefix.as_str()));
                }
            }
            Self::ManifestUpdated { manifest, .. } => world.manifest = manifest,
            Self::GovernanceRegistry(prepared) => prepared.install(world),
            Self::CapabilityAuthorization(prepared) => prepared.install(world),
            Self::CapabilityCommandCommit(prepared) => prepared.install(world),
            Self::CapabilityEffectReceipt(prepared) => prepared.install(world),
            Self::AgentIntent(prepared) => prepared.install_infallible(&mut world.state),
            Self::EconomyData(prepared) => prepared.install_infallible(&mut world.state),
            Self::PowerRedemption(prepared) => prepared.install_infallible(&mut world.state),
            Self::NodePointsSettlement(prepared) => prepared.install_infallible(&mut world.state),
            Self::MainTokenMonetary(prepared) => prepared.install_infallible(&mut world.state),
            Self::MainTokenGovernanceMonetary(prepared) => {
                prepared.install_infallible(&mut world.state)
            }
            Self::MainTokenRestrictedClaim(prepared) => {
                prepared.install_infallible(&mut world.state)
            }
            Self::Body(prepared) => prepared.install_infallible(world),
            Self::GovernanceEmergencyBrake { next_until_tick } => {
                let next_until_tick = next_until_tick.map(|next| {
                    world
                        .governance_emergency_brake_until_tick
                        .map_or(next, |current| current.max(next))
                });
                world.governance_emergency_brake_until_tick = next_until_tick;
            }
            Self::GovernanceFinalityEpochSnapshot { epoch_id, next } => match next {
                Some(snapshot) => {
                    world
                        .governance_finality_epoch_snapshots
                        .insert(epoch_id, snapshot);
                }
                None => {
                    world.governance_finality_epoch_snapshots.remove(&epoch_id);
                }
            },
            Self::GovernanceEmergencyVeto { proposal_id, next } => {
                world.proposals.insert(proposal_id, next);
            }
            Self::GovernanceProposal {
                proposal_id,
                next,
                next_proposal_id,
                next_proposal_id_era,
            } => {
                world.proposals.insert(proposal_id, next);
                world.next_proposal_id = next_proposal_id;
                world.next_proposal_id_era = next_proposal_id_era;
            }
            Self::GovernanceProposalShadow { proposal_id, next } => {
                world.proposals.insert(proposal_id, next);
            }
            Self::GovernanceProposalStatus {
                proposal_id, next, ..
            } => {
                world.proposals.insert(proposal_id, next);
            }
            Self::GovernanceIdentityPenaltyAppeal { penalty_id, next } => {
                world.governance_identity_penalties.insert(penalty_id, next);
            }
            Self::GovernanceIdentityPenaltyApplication {
                penalty_id,
                target_agent_id,
                next,
                next_profile,
                next_penalty_id,
                ..
            } => {
                world.governance_identity_penalties.insert(penalty_id, next);
                world
                    .state
                    .governance_identity_profiles
                    .insert(target_agent_id, next_profile);
                world.next_governance_identity_penalty_id = next_penalty_id;
            }
            Self::GovernanceIdentityPenaltyResolution {
                penalty_id,
                target_agent_id,
                next,
                next_profile,
            } => {
                world.governance_identity_penalties.insert(penalty_id, next);
                world
                    .state
                    .governance_identity_profiles
                    .insert(target_agent_id, next_profile);
            }
            Self::NoState | Self::RouteOnly { .. } => {}
        }
    }
}

struct PreparedEventPublication {
    event: WorldEvent,
    next_event_id: WorldEventId,
    next_event_id_era: u64,
    journal_events: Vec<WorldEvent>,
    journal_events_evicted: u64,
    consensus_record: TickConsensusRecord,
    state_delta: PreparedEventStateDelta,
}

impl World {
    pub(in crate::runtime::world) fn append_event(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        let state_delta = match &body {
            WorldEventBody::Domain(
                event @ (DomainEvent::AgentIntentProposed { .. }
                | DomainEvent::AgentIntentSubmitted { .. }
                | DomainEvent::AgentIntentAccepted { .. }
                | DomainEvent::AgentIntentReplaced { .. }
                | DomainEvent::AgentIntentTransitioned { .. }),
            ) => Some(PreparedEventStateDelta::AgentIntent(
                self.prepare_raw_agent_intent_event(event)?,
            )),
            WorldEventBody::Domain(
                event @ (DomainEvent::ModuleArtifactDeployed { .. }
                | DomainEvent::ModuleArtifactListed { .. }
                | DomainEvent::ModuleArtifactDelisted { .. }
                | DomainEvent::ModuleArtifactDestroyed { .. }
                | DomainEvent::ModuleArtifactBidPlaced { .. }
                | DomainEvent::ModuleArtifactBidCancelled { .. }
                | DomainEvent::ModuleArtifactSaleCompleted { .. }),
            ) => Some(PreparedEventStateDelta::ModuleMarketplace(
                self.state
                    .prepare_module_marketplace_event(event, self.state.time)?,
            )),
            WorldEventBody::Domain(
                event @ (DomainEvent::ModuleReleaseRequested { .. }
                | DomainEvent::ModuleReleaseAttested { .. }
                | DomainEvent::ModuleReleaseRolesBound { .. }
                | DomainEvent::ModuleReleaseShadowed { .. }
                | DomainEvent::ModuleReleaseRoleApproved { .. }
                | DomainEvent::ModuleReleaseRejected { .. }
                | DomainEvent::ProductProfileGoverned { .. }
                | DomainEvent::RecipeProfileGoverned { .. }
                | DomainEvent::FactoryProfileGoverned { .. }
                | DomainEvent::ModuleReleaseApplied { .. }),
            ) => Some(PreparedEventStateDelta::ModuleRelease(
                self.state
                    .prepare_module_release_event(event, self.state.time)?,
            )),
            WorldEventBody::Domain(
                event @ (DomainEvent::ModuleInstalled { .. }
                | DomainEvent::ModuleUpgraded { .. }
                | DomainEvent::ModuleRollbackApplied { .. }),
            ) => {
                let prepared = self
                    .state
                    .prepare_module_instance_event(event, self.state.time)?;
                let schedule = self.prepare_module_instance_schedule(event, self.state.time)?;
                Some(PreparedEventStateDelta::ModuleInstance { prepared, schedule })
            }
            WorldEventBody::Domain(
                event @ (DomainEvent::ResourceTransferred { .. }
                | DomainEvent::DataCollected { .. }
                | DomainEvent::DataCollectedAuthenticated { .. }
                | DomainEvent::DataAccessGranted { .. }
                | DomainEvent::DataAccessRevoked { .. }),
            ) => Some(PreparedEventStateDelta::EconomyData(
                super::super::economy_data_publication::PreparedEconomyDataEvent::prepare(
                    &self.state,
                    event,
                    self.state.time,
                )?,
            )),
            WorldEventBody::Domain(
                event @ (DomainEvent::PowerRedeemed { .. }
                | DomainEvent::PowerRedeemRejected { .. }),
            ) => Some(PreparedEventStateDelta::PowerRedemption(
                super::super::power_redemption_publication::PreparedPowerRedemptionEvent::prepare(
                    &self.state,
                    event,
                    self.state.time,
                )?,
            )),
            WorldEventBody::Domain(event @ DomainEvent::NodePointsSettlementApplied { .. }) => {
                Some(PreparedEventStateDelta::NodePointsSettlement(
                    super::super::node_points_settlement_publication::PreparedNodePointsSettlement::prepare(
                        &self.state,
                        event,
                    )?,
                ))
            }
            WorldEventBody::Domain(event @ (DomainEvent::MainTokenGenesisInitialized { .. }
                | DomainEvent::MainTokenVestingClaimed { .. }
                | DomainEvent::MainTokenTransferred { .. }
                | DomainEvent::MainTokenEpochIssued { .. }
                | DomainEvent::MainTokenFeeSettled { .. })) => {
                Some(PreparedEventStateDelta::MainTokenMonetary(
                    super::super::main_token_monetary_publication::PreparedMainTokenMonetaryEvent::prepare(&self.state, event, self.state.time)?,
                ))
            }
            WorldEventBody::Domain(event @ (DomainEvent::MainTokenPolicyUpdateScheduled { .. }
                | DomainEvent::MainTokenTreasuryDistributed { .. })) => {
                Some(PreparedEventStateDelta::MainTokenGovernanceMonetary(
                    super::super::main_token_governance_monetary_publication::PreparedMainTokenGovernanceMonetaryEvent::prepare(&self.state, event, self.state.time)?,
                ))
            }
            WorldEventBody::Domain(event @ (DomainEvent::RestrictedStarterClaimLiveopsPoolToppedUp { .. }
                | DomainEvent::RestrictedStarterClaimGrantIssued { .. }
                | DomainEvent::RestrictedStarterClaimGrantExpired { .. }
                | DomainEvent::RestrictedStarterClaimGrantRevoked { .. })) => {
                Some(PreparedEventStateDelta::MainTokenRestrictedClaim(
                    super::super::main_token_restricted_claim_publication::PreparedMainTokenRestrictedClaimEvent::prepare(&self.state, event)?,
                ))
            }
            WorldEventBody::ModuleStateUpdated(update) => {
                Some(PreparedEventStateDelta::ModuleStateUpdated {
                    module_states: BTreeMap::from([(
                        update.module_id.clone(),
                        update.state.clone(),
                    )]),
                })
            }
            WorldEventBody::ModuleRuntimeCharged(charge) => {
                Some(PreparedEventStateDelta::ModuleRuntimeCharged(
                    self.prepare_module_runtime_charge_event(charge, self.state.time)?,
                ))
            }
            WorldEventBody::EffectQueued(intent) => {
                Some(self.prepare_raw_effect_queue_delta(intent)?)
            }
            WorldEventBody::ReceiptAppended(receipt) => {
                Some(self.prepare_raw_receipt_delta(receipt)?)
            }
            WorldEventBody::ModuleEvent(event) => Some(self.prepare_raw_module_event_delta(event)?),
            WorldEventBody::ManifestUpdated(update) => {
                Some(PreparedEventStateDelta::ManifestUpdated {
                    update: update.clone(),
                    manifest: update.manifest.clone(),
                })
            }
            WorldEventBody::Governance(
                event @ (GovernanceEvent::RestrictedStarterClaimAdminRegistryUpdated { .. }
                | GovernanceEvent::ValidatorAdmissionSubmitted { .. }
                | GovernanceEvent::ValidatorAdmissionApproved { .. }
                | GovernanceEvent::ValidatorAdmissionActivated { .. }
                | GovernanceEvent::ValidatorAdmissionRevoked { .. }),
            ) => Some(PreparedEventStateDelta::GovernanceRegistry(
                self.prepare_governance_registry_event(event)?,
            )),
            WorldEventBody::CapabilityAuthorization(
                event @ (CapabilityAuthorizationEvent::AuthorityInstalledWithProof { .. }
                | CapabilityAuthorizationEvent::AgentIdentityInstalled { .. }
                | CapabilityAuthorizationEvent::SystemIdentityInstalled { .. }
                | CapabilityAuthorizationEvent::InvocationContextInstalled { .. }
                | CapabilityAuthorizationEvent::BudgetAccountInstalled { .. }
                | CapabilityAuthorizationEvent::GrantRegistered { .. }),
            ) => Some(PreparedEventStateDelta::CapabilityAuthorization(
                self.prepare_raw_capability_authorization_event(event)?,
            )),
            WorldEventBody::CapabilityAuthorization(
                event @ CapabilityAuthorizationEvent::CommandCommitted { .. },
            ) => Some(PreparedEventStateDelta::CapabilityCommandCommit(
                self.prepare_raw_command_commit(event, self.state.time)?,
            )),
            WorldEventBody::CapabilityAuthorization(
                event @ CapabilityAuthorizationEvent::EffectReceiptCommitted { .. },
            ) => Some(PreparedEventStateDelta::CapabilityEffectReceipt(
                self.prepare_raw_effect_receipt_commit(event)?,
            )),
            _ => prepared_governance_events::prepare(self, &body)?
                .or_else(|| PreparedEventStateDelta::for_body(&body)),
        };
        self.append_event_internal(body, caused_by, state_delta)
    }

    pub(in crate::runtime::world) fn append_event_with_prepared_body(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
        prepared: PreparedBodyAttributesUpdate,
    ) -> Result<WorldEventId, WorldError> {
        self.append_event_internal(
            body,
            caused_by,
            Some(PreparedEventStateDelta::Body(prepared)),
        )
    }

    pub(in crate::runtime::world) fn append_event_with_route_only_domain_event(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
        agent_id: String,
    ) -> Result<WorldEventId, WorldError> {
        self.append_event_internal(
            body,
            caused_by,
            Some(PreparedEventStateDelta::RouteOnly { agent_id }),
        )
    }

    fn append_event_internal(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
        state_delta: Option<PreparedEventStateDelta>,
    ) -> Result<WorldEventId, WorldError> {
        if let Some(state_delta) = state_delta {
            return self.append_prepared_event(body, caused_by, state_delta);
        }

        // Domain intent payloads carry the journal position as part of their
        // authority identity. Validate against the id before mutating state;
        // this keeps the payload and its envelope inseparable on replay.
        let expected_event_id = self.next_event_id.max(1);
        self.apply_event_body_at_with_prepared_body(
            &body,
            self.state.time,
            Some(expected_event_id),
            None,
        )?;
        let event_id = self.allocate_next_event_id();
        debug_assert_eq!(event_id, expected_event_id);
        self.journal.append(WorldEvent {
            id: event_id,
            time: self.state.time,
            caused_by,
            body,
        });
        self.enforce_journal_event_limit();
        self.record_tick_consensus_for_tick(self.state.time)?;
        Ok(event_id)
    }

    fn prepare_raw_effect_queue_delta(
        &self,
        intent: &EffectIntent,
    ) -> Result<PreparedEventStateDelta, WorldError> {
        let (pending_effects, pending_effects_evicted) =
            self.prepare_pending_effect_queue(intent.clone())?;
        Ok(PreparedEventStateDelta::EffectQueued {
            intent_id: intent.intent_id.clone(),
            pending_effects,
            pending_effects_evicted,
        })
    }

    fn prepare_raw_receipt_delta(
        &self,
        receipt: &EffectReceipt,
    ) -> Result<PreparedEventStateDelta, WorldError> {
        let mut pending_effects = self.pending_effects.clone();
        let mut inflight_effects = self.inflight_effects.clone();
        let mut removed = inflight_effects.remove(&receipt.intent_id).is_some();
        let before = pending_effects.len();
        pending_effects.retain(|intent| intent.intent_id != receipt.intent_id);
        removed |= before != pending_effects.len();
        if !removed {
            return Err(WorldError::ReceiptUnknownIntent {
                intent_id: receipt.intent_id.clone(),
            });
        }
        Ok(PreparedEventStateDelta::ReceiptAppended {
            intent_id: receipt.intent_id.clone(),
            pending_effects,
            inflight_effects,
        })
    }

    fn prepare_raw_module_event_delta(
        &self,
        event: &ModuleEvent,
    ) -> Result<PreparedEventStateDelta, WorldError> {
        let mut module_registry = self.module_registry.clone();
        let mut module_artifacts = self.module_artifacts.clone();
        let mut module_tick_schedule = self.module_tick_schedule.clone();
        let mut cache_invalidations = BTreeSet::new();
        super::super::governance_publication::project_module_event(
            self.state.time,
            event,
            &mut module_registry,
            &mut module_artifacts,
            &mut module_tick_schedule,
            &mut cache_invalidations,
        )?;
        Ok(PreparedEventStateDelta::ModuleEvent {
            event: event.clone(),
            module_registry,
            module_artifacts,
            module_tick_schedule,
            cache_invalidations,
        })
    }

    fn append_prepared_event(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
        state_delta: PreparedEventStateDelta,
    ) -> Result<WorldEventId, WorldError> {
        let prepared = self.prepare_event_publication(body, caused_by, state_delta)?;
        if self.take_fail_next_append_after_publication_prepare_for_test() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "injected append_event failure after publication preparation".to_string(),
            });
        }

        self.install_event_publication(prepared, None)
    }

    fn install_event_publication(
        &mut self,
        prepared: PreparedEventPublication,
        release: Option<
            super::super::super::state::module_release_transition::PreparedModuleRelease,
        >,
    ) -> Result<WorldEventId, WorldError> {
        let PreparedEventPublication {
            event,
            next_event_id,
            next_event_id_era,
            journal_events,
            journal_events_evicted,
            consensus_record,
            state_delta,
        } = prepared;
        state_delta.install_infallible(self);
        self.state.time = event.time;
        self.next_event_id = next_event_id;
        self.next_event_id_era = next_event_id_era;
        self.journal.events = journal_events;
        self.runtime_backpressure_stats.journal_events_evicted = self
            .runtime_backpressure_stats
            .journal_events_evicted
            .saturating_add(journal_events_evicted);
        self.install_prepared_tick_consensus_record(consensus_record);
        if let Some(release) = release {
            release.install_routed(&mut self.state);
        } else if let WorldEventBody::Domain(domain_event) = &event.body {
            self.state.route_domain_event(domain_event);
        }
        Ok(event.id)
    }

    pub(in crate::runtime::world) fn append_module_artifact_deployment(
        &mut self,
        event: DomainEvent,
        caused_by: Option<CausedBy>,
        registration: super::super::module_runtime::PreparedModuleArtifactRegistration,
    ) -> Result<WorldEventId, WorldError> {
        let event_wasm_hash = match &event {
            DomainEvent::ModuleArtifactDeployed { wasm_hash, .. } => wasm_hash,
            _ => {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: "artifact registration requires a deployment event".to_string(),
                });
            }
        };
        if event_wasm_hash != registration.wasm_hash() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "artifact registration hash {} does not match deployment hash {event_wasm_hash}",
                    registration.wasm_hash()
                ),
            });
        }
        let state_delta = PreparedEventStateDelta::ModuleMarketplace(
            self.state
                .prepare_module_marketplace_event(&event, self.state.time)?,
        );
        let prepared =
            self.prepare_event_publication(WorldEventBody::Domain(event), caused_by, state_delta)?;
        if self.take_fail_next_append_after_publication_prepare_for_test() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "injected append_event failure after publication preparation".to_string(),
            });
        }
        let event_id = self.install_event_publication(prepared, None)?;
        registration.install(self);
        Ok(event_id)
    }

    pub(in crate::runtime::world) fn append_module_artifact_retirement(
        &mut self,
        event: DomainEvent,
        caused_by: Option<CausedBy>,
        retirement: super::super::module_artifact_retirement::PreparedModuleArtifactRetirement,
    ) -> Result<WorldEventId, WorldError> {
        if !retirement.matches_event(&event) {
            return Err(WorldError::ModuleChangeInvalid {
                reason: "artifact retirement requires a matching destroyed event".to_string(),
            });
        }
        let state_delta = PreparedEventStateDelta::ModuleMarketplace(
            self.state
                .prepare_module_marketplace_event(&event, self.state.time)?,
        );
        let prepared =
            self.prepare_event_publication(WorldEventBody::Domain(event), caused_by, state_delta)?;
        if self.take_fail_next_append_after_publication_prepare_for_test() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "injected append_event failure after publication preparation".to_string(),
            });
        }
        let event_id = self.install_event_publication(prepared, None)?;
        retirement.install(self);
        Ok(event_id)
    }

    pub(in crate::runtime::world) fn append_module_install_with_release(
        &mut self,
        event: DomainEvent,
        caused_by: Option<CausedBy>,
        completion: super::super::module_release_publication::ModuleReleaseCompletion,
    ) -> Result<(), WorldError> {
        let instance = self
            .state
            .prepare_module_instance_event(&event, self.state.time)?;
        let schedule = self.prepare_module_instance_schedule(&event, self.state.time)?;
        let mut prepared = self.prepare_event_publication(
            WorldEventBody::Domain(event.clone()),
            caused_by.clone(),
            PreparedEventStateDelta::ModuleInstance {
                prepared: instance,
                schedule,
            },
        )?;
        if self.take_fail_next_append_after_publication_prepare_for_test() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "injected append_event failure after publication preparation".to_string(),
            });
        }
        let PreparedEventStateDelta::ModuleInstance {
            prepared: instance, ..
        } = &prepared.state_delta
        else {
            unreachable!()
        };
        let manifest_hash = self.current_manifest_hash()?;
        let tail = self.prepare_module_release_tail(
            instance,
            &event,
            completion,
            super::super::module_release_publication::ReleasePublicationJournal {
                next_event_id: prepared.next_event_id,
                next_event_id_era: prepared.next_event_id_era,
                events: std::mem::take(&mut prepared.journal_events),
                evicted: prepared.journal_events_evicted,
            },
            &manifest_hash,
            caused_by,
        )?;
        prepared.next_event_id = tail.journal.next_event_id;
        prepared.next_event_id_era = tail.journal.next_event_id_era;
        prepared.journal_events = tail.journal.events;
        prepared.journal_events_evicted = tail.journal.evicted;
        prepared.consensus_record = tail.consensus_record;
        self.install_event_publication(prepared, Some(tail.state))?;
        Ok(())
    }

    fn prepare_event_publication(
        &self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
        state_delta: PreparedEventStateDelta,
    ) -> Result<PreparedEventPublication, WorldError> {
        if !state_delta.matches_body(&body) {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "prepared body delta does not match body event".to_string(),
            });
        }
        let domain_event = match &body {
            WorldEventBody::Domain(domain_event) => Some(domain_event.clone()),
            _ => None,
        };
        let (event_id, next_event_id, next_event_id_era) =
            Self::preview_next_event_id(self.next_event_id, self.next_event_id_era);
        if let Some(domain_event) = domain_event.as_ref() {
            self.validate_agent_intent_receipt_reference(domain_event, Some(event_id))?;
        }
        let event = WorldEvent {
            id: event_id,
            time: self.state.time,
            caused_by,
            body,
        };

        let mut journal_events = self.journal.events.clone();
        journal_events.push(event.clone());
        let max_len = self.runtime_memory_limits.max_journal_events.max(1);
        let overflow = journal_events.len().saturating_sub(max_len);
        if overflow > 0 {
            journal_events.drain(0..overflow);
        }
        let tick_events: Vec<WorldEvent> = journal_events
            .iter()
            .filter(|journal_event| journal_event.time == event.time)
            .cloned()
            .collect();
        let state_root = match &state_delta {
            PreparedEventStateDelta::ModuleMarketplace(prepared) => {
                self.state_root_hash_with_module_marketplace_overlay(prepared)?
            }
            PreparedEventStateDelta::ModuleRelease(prepared) => self
                .state_root_hash_with_module_release_overlay(
                    None,
                    prepared,
                    &self.current_manifest_hash()?,
                )?,
            PreparedEventStateDelta::ModuleInstance { prepared, .. } => {
                self.state_root_hash_with_module_instance_overlay(prepared)?
            }
            PreparedEventStateDelta::ModuleStateUpdated { module_states } => self
                .state_root_hash_with_command_overlay(
                    super::super::super::state::CommandStateOverlay {
                        module_states,
                        resources: &BTreeMap::new(),
                        agents: &BTreeMap::new(),
                    },
                )?,
            PreparedEventStateDelta::ModuleRuntimeCharged(prepared) => self
                .state_root_hash_with_command_overlay(
                    super::super::super::state::CommandStateOverlay {
                        module_states: &BTreeMap::new(),
                        resources: &prepared.resources,
                        agents: &prepared.agents,
                    },
                )?,
            PreparedEventStateDelta::EffectQueued { .. }
            | PreparedEventStateDelta::ReceiptAppended { .. } => {
                // Effect queues are persisted World sidecars outside the
                // canonical WorldState root schema.
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::ModuleEvent { .. } => self.current_state_root_hash()?,
            PreparedEventStateDelta::ManifestUpdated { manifest, .. } => {
                super::super::governance_publication::state_root_hash_with_manifest(self, manifest)?
            }
            PreparedEventStateDelta::GovernanceRegistry(prepared) => {
                self.state_root_hash_with_governance_registry_overlay(prepared)?
            }
            PreparedEventStateDelta::CapabilityAuthorization(_)
            | PreparedEventStateDelta::CapabilityCommandCommit(_)
            | PreparedEventStateDelta::CapabilityEffectReceipt(_) => {
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::AgentIntent(prepared) => {
                self.state_root_hash_with_agent_intent_overlay(prepared)?
            }
            PreparedEventStateDelta::EconomyData(prepared) => {
                self.state_root_hash_with_economy_data_overlay(prepared)?
            }
            PreparedEventStateDelta::PowerRedemption(prepared) => {
                self.state_root_hash_with_power_redemption_overlay(prepared)?
            }
            PreparedEventStateDelta::NodePointsSettlement(prepared) => {
                self.state_root_hash_with_node_points_settlement_overlay(prepared)?
            }
            PreparedEventStateDelta::MainTokenMonetary(prepared) => {
                self.state_root_hash_with_main_token_monetary_overlay(prepared)?
            }
            PreparedEventStateDelta::MainTokenGovernanceMonetary(prepared) => {
                self.state_root_hash_with_main_token_governance_monetary_overlay(prepared)?
            }
            PreparedEventStateDelta::MainTokenRestrictedClaim(prepared) => {
                self.state_root_hash_with_main_token_restricted_claim_overlay(prepared)?
            }
            PreparedEventStateDelta::NoState => self.current_state_root_hash()?,
            PreparedEventStateDelta::Body(_) | PreparedEventStateDelta::RouteOnly { .. } => {
                let Some(domain_event) = domain_event.as_ref() else {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "prepared body delta requires a domain event".to_string(),
                    });
                };
                let body_overlay = state_delta.state_overlay(domain_event.clone());
                self.state_root_hash_with_body_overlay(&body_overlay)?
            }
            PreparedEventStateDelta::GovernanceEmergencyBrake { .. } => {
                // The emergency-brake gate is a World sidecar and is intentionally
                // outside the canonical WorldState root schema.
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::GovernanceFinalityEpochSnapshot { .. } => {
                // Finality epoch snapshots are persisted World sidecar data and
                // intentionally remain outside the canonical WorldState root schema.
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::GovernanceEmergencyVeto { .. } => {
                // Governance proposals are persisted World sidecar data and
                // intentionally remain outside the canonical WorldState root schema.
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::GovernanceProposal { .. }
            | PreparedEventStateDelta::GovernanceProposalShadow { .. }
            | PreparedEventStateDelta::GovernanceProposalStatus { .. } => {
                // Governance proposals are persisted World sidecar data and
                // intentionally remain outside the canonical WorldState root schema.
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::GovernanceIdentityPenaltyAppeal { .. } => {
                // Identity penalty records are persisted World sidecar data and
                // intentionally remain outside the canonical WorldState root schema.
                self.current_state_root_hash()?
            }
            PreparedEventStateDelta::GovernanceIdentityPenaltyApplication {
                target_agent_id,
                next_profile,
                profile_was_present,
                ..
            } => self.state_root_hash_with_governance_identity_profile_overlay(
                target_agent_id.as_str(),
                next_profile,
                !profile_was_present,
            )?,
            PreparedEventStateDelta::GovernanceIdentityPenaltyResolution {
                target_agent_id,
                next_profile,
                ..
            } => self.state_root_hash_with_governance_identity_profile_overlay(
                target_agent_id.as_str(),
                next_profile,
                false,
            )?,
        };
        let consensus_record = self.build_tick_consensus_record_for_prepared_events(
            event.time,
            tick_events.as_slice(),
            state_root.clone(),
        )?;
        self.validate_tick_consensus_candidate_for_prepared_publication(
            &consensus_record,
            tick_events.as_slice(),
            state_root.as_str(),
        )?;

        Ok(PreparedEventPublication {
            event,
            next_event_id,
            next_event_id_era,
            journal_events,
            journal_events_evicted: overflow as u64,
            consensus_record,
            state_delta,
        })
    }
}
