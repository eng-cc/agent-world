use super::module_release_transition::ReleaseMapProjection;
use super::*;
use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};

#[derive(Debug, Clone, PartialEq)]
enum BodyOverlayMutation {
    Body {
        body_view: crate::models::BodyKernelView,
        last_active: WorldTime,
    },
    RouteOnly,
}

/// A typed, borrowed overlay for the state fields needed while preparing a
/// domain transition. The overlay is intentionally narrow: it cannot mutate
/// the canonical [`WorldState`] and it can either update the target agent's
/// body fields or represent a route-only event with no body mutation.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyOverlay {
    agent_id: String,
    mutation: BodyOverlayMutation,
    routed_domain_event: Option<DomainEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GovernanceIdentityProfileOverlay {
    target_agent_id: String,
    next_profile: GovernanceIdentityProfileState,
    allow_insert: bool,
}

impl BodyOverlay {
    pub fn new(
        agent_id: impl Into<String>,
        body_view: crate::models::BodyKernelView,
        last_active: WorldTime,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            mutation: BodyOverlayMutation::Body {
                body_view,
                last_active,
            },
            routed_domain_event: None,
        }
    }

    pub(crate) fn route_only(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            mutation: BodyOverlayMutation::RouteOnly,
            routed_domain_event: None,
        }
    }

    pub(crate) fn with_routed_domain_event(mut self, event: DomainEvent) -> Self {
        self.routed_domain_event = Some(event);
        self
    }

    fn requires_body_target(&self) -> bool {
        matches!(self.mutation, BodyOverlayMutation::Body { .. })
    }
}

/// A serialization projection over borrowed canonical state.
///
/// `WorldStateProjection` is the reusable state-root preparation seam for
/// transition execution.  It owns no world state and applies only typed
/// overlays at serialization time.  The canonical state serializer below is
/// shared by both this projection and `WorldState`, so a projection without an
/// overlay is byte-identical to direct state serialization.
#[derive(Debug)]
pub struct WorldStateProjection<'a> {
    state: &'a WorldState,
    body_overlay: Option<BodyOverlay>,
    command_overlay: Option<CommandStateOverlay<'a>>,
    module_instance_overlay: Option<&'a module_instance_transition::PreparedModuleInstance>,
    module_release_overlay: Option<&'a module_release_transition::PreparedModuleRelease>,
    module_marketplace_overlay:
        Option<&'a module_marketplace_transition::PreparedModuleMarketplace>,
    governance_identity_profile_overlay: Option<GovernanceIdentityProfileOverlay>,
    governance_registry_overlay: Option<
        &'a crate::runtime::world::governance_registry_publication::PreparedGovernanceRegistryEvent,
    >,
    agent_intent_overlay:
        Option<&'a crate::runtime::world::agent_intent_publication::PreparedAgentIntent>,
    economy_data_overlay:
        Option<&'a crate::runtime::world::economy_data_publication::PreparedEconomyDataEvent>,
    economic_contract_overlay: Option<
        &'a crate::runtime::world::economic_contract_publication::PreparedEconomicContractEvent,
    >,
    power_redemption_overlay: Option<
        &'a crate::runtime::world::power_redemption_publication::PreparedPowerRedemptionEvent,
    >,
    node_points_settlement_overlay: Option<
        &'a crate::runtime::world::node_points_settlement_publication::PreparedNodePointsSettlement,
    >,
    main_token_monetary_overlay: Option<
        &'a crate::runtime::world::main_token_monetary_publication::PreparedMainTokenMonetaryEvent,
    >,
    main_token_governance_monetary_overlay: Option<
        &'a crate::runtime::world::main_token_governance_monetary_publication::PreparedMainTokenGovernanceMonetaryEvent,
    >,
    main_token_restricted_claim_overlay: Option<
        &'a crate::runtime::world::main_token_restricted_claim_publication::PreparedMainTokenRestrictedClaimEvent,
    >,
    starter_oc_claim_overlay: Option<
        &'a crate::runtime::world::starter_oc_claim_publication::PreparedStarterOcClaimed,
    >,
    agent_claim_light_lifecycle_overlay: Option<
        &'a crate::runtime::world::agent_claim_light_lifecycle_publication::PreparedAgentClaimLightLifecycle,
    >,
    agent_claim_economic_overlay: Option<&'a crate::runtime::world::agent_claim_economic_publication::PreparedAgentClaimEconomic>,
    agent_claim_terminal_overlay: Option<&'a crate::runtime::world::agent_claim_terminal_publication::PreparedAgentClaimTerminal>,
}

impl<'a> WorldStateProjection<'a> {
    pub fn borrowed(state: &'a WorldState) -> Self {
        Self {
            state,
            body_overlay: None,
            command_overlay: None,
            module_instance_overlay: None,
            module_release_overlay: None,
            module_marketplace_overlay: None,
            governance_identity_profile_overlay: None,
            governance_registry_overlay: None,
            agent_intent_overlay: None,
            economy_data_overlay: None,
            economic_contract_overlay: None,
            power_redemption_overlay: None,
            node_points_settlement_overlay: None,
            main_token_monetary_overlay: None,
            main_token_governance_monetary_overlay: None,
            main_token_restricted_claim_overlay: None,
            starter_oc_claim_overlay: None,
            agent_claim_light_lifecycle_overlay: None,
            agent_claim_economic_overlay: None,
            agent_claim_terminal_overlay: None,
        }
    }

    pub(crate) fn with_governance_registry_overlay(
        mut self,
        overlay: &'a crate::runtime::world::governance_registry_publication::PreparedGovernanceRegistryEvent,
    ) -> Self {
        self.governance_registry_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_agent_intent_overlay(
        mut self,
        overlay: &'a crate::runtime::world::agent_intent_publication::PreparedAgentIntent,
    ) -> Self {
        self.agent_intent_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_economy_data_overlay(
        mut self,
        overlay: &'a crate::runtime::world::economy_data_publication::PreparedEconomyDataEvent,
    ) -> Self {
        self.economy_data_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_economic_contract_overlay(
        mut self,
        overlay: &'a crate::runtime::world::economic_contract_publication::PreparedEconomicContractEvent,
    ) -> Self {
        self.economic_contract_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_power_redemption_overlay(
        mut self,
        overlay: &'a crate::runtime::world::power_redemption_publication::PreparedPowerRedemptionEvent,
    ) -> Self {
        self.power_redemption_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_node_points_settlement_overlay(
        mut self,
        overlay: &'a crate::runtime::world::node_points_settlement_publication::PreparedNodePointsSettlement,
    ) -> Self {
        self.node_points_settlement_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_main_token_monetary_overlay(
        mut self,
        overlay: &'a crate::runtime::world::main_token_monetary_publication::PreparedMainTokenMonetaryEvent,
    ) -> Self {
        self.main_token_monetary_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_main_token_governance_monetary_overlay(
        mut self,
        overlay: &'a crate::runtime::world::main_token_governance_monetary_publication::PreparedMainTokenGovernanceMonetaryEvent,
    ) -> Self {
        self.main_token_governance_monetary_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_main_token_restricted_claim_overlay(
        mut self,
        overlay: &'a crate::runtime::world::main_token_restricted_claim_publication::PreparedMainTokenRestrictedClaimEvent,
    ) -> Self {
        self.main_token_restricted_claim_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_starter_oc_claim_overlay(
        mut self,
        overlay: &'a crate::runtime::world::starter_oc_claim_publication::PreparedStarterOcClaimed,
    ) -> Self {
        self.starter_oc_claim_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_agent_claim_light_lifecycle_overlay(
        mut self,
        overlay: &'a crate::runtime::world::agent_claim_light_lifecycle_publication::PreparedAgentClaimLightLifecycle,
    ) -> Self {
        self.agent_claim_light_lifecycle_overlay = Some(overlay);
        self
    }
    pub(crate) fn with_agent_claim_economic_overlay(
        mut self,
        overlay:&'a crate::runtime::world::agent_claim_economic_publication::PreparedAgentClaimEconomic,
    ) -> Self {
        self.agent_claim_economic_overlay = Some(overlay);
        self
    }
    pub(crate) fn with_agent_claim_terminal_overlay(
        mut self,
        overlay: &'a crate::runtime::world::agent_claim_terminal_publication::PreparedAgentClaimTerminal,
    ) -> Self {
        self.agent_claim_terminal_overlay = Some(overlay);
        self
    }

    pub fn with_body_overlay(mut self, body_overlay: BodyOverlay) -> Self {
        self.body_overlay = Some(body_overlay);
        self
    }

    pub(crate) fn with_module_instance_overlay(
        mut self,
        overlay: &'a module_instance_transition::PreparedModuleInstance,
    ) -> Self {
        self.module_instance_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_module_release_overlay(
        mut self,
        overlay: &'a module_release_transition::PreparedModuleRelease,
    ) -> Self {
        self.module_release_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_module_marketplace_overlay(
        mut self,
        overlay: &'a module_marketplace_transition::PreparedModuleMarketplace,
    ) -> Self {
        self.module_marketplace_overlay = Some(overlay);
        self
    }

    pub(crate) fn with_command_overlay(mut self, command_overlay: CommandStateOverlay<'a>) -> Self {
        self.command_overlay = Some(command_overlay);
        self
    }

    pub(crate) fn with_governance_identity_profile_overlay(
        mut self,
        target_agent_id: impl Into<String>,
        next_profile: GovernanceIdentityProfileState,
    ) -> Self {
        self.governance_identity_profile_overlay = Some(GovernanceIdentityProfileOverlay {
            target_agent_id: target_agent_id.into(),
            next_profile,
            allow_insert: false,
        });
        self
    }

    pub(crate) fn with_governance_identity_profile_insert_overlay(
        mut self,
        target_agent_id: impl Into<String>,
        next_profile: GovernanceIdentityProfileState,
    ) -> Self {
        self.governance_identity_profile_overlay = Some(GovernanceIdentityProfileOverlay {
            target_agent_id: target_agent_id.into(),
            next_profile,
            allow_insert: true,
        });
        self
    }
}

impl Serialize for WorldState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_world_state(
            self, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, serializer,
        )
    }
}

impl Serialize for WorldStateProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(overlay) = self.body_overlay.as_ref() {
            if overlay.requires_body_target() && !self.state.agents.contains_key(&overlay.agent_id)
            {
                return Err(serde::ser::Error::custom(format!(
                    "body overlay target agent not found: {}",
                    overlay.agent_id
                )));
            }
        }
        serialize_world_state(
            self.state,
            self.body_overlay.as_ref(),
            self.command_overlay.as_ref(),
            self.module_instance_overlay,
            self.module_release_overlay,
            self.module_marketplace_overlay,
            self.governance_identity_profile_overlay.as_ref(),
            self.governance_registry_overlay,
            self.agent_intent_overlay,
            self.economy_data_overlay,
            self.economic_contract_overlay,
            self.power_redemption_overlay,
            self.node_points_settlement_overlay,
            self.main_token_monetary_overlay,
            self.main_token_governance_monetary_overlay,
            self.main_token_restricted_claim_overlay,
            self.starter_oc_claim_overlay,
            self.agent_claim_light_lifecycle_overlay,
            self.agent_claim_economic_overlay,
            self.agent_claim_terminal_overlay,
            serializer,
        )
    }
}

struct GovernanceIdentityProfileMapProjection<'a> {
    profiles: &'a BTreeMap<String, GovernanceIdentityProfileState>,
    overlay: &'a GovernanceIdentityProfileOverlay,
}

impl Serialize for GovernanceIdentityProfileMapProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let target_exists = self
            .profiles
            .contains_key(self.overlay.target_agent_id.as_str());
        let map_len =
            self.profiles.len() + usize::from(!target_exists && self.overlay.allow_insert);
        let mut map = serializer.serialize_map(Some(map_len))?;
        let mut inserted = false;
        for (agent_id, profile) in self.profiles {
            if agent_id == &self.overlay.target_agent_id {
                map.serialize_entry(agent_id, &self.overlay.next_profile)?;
                inserted = true;
            } else if self.overlay.allow_insert
                && !target_exists
                && !inserted
                && self.overlay.target_agent_id.as_str() < agent_id.as_str()
            {
                map.serialize_entry(
                    self.overlay.target_agent_id.as_str(),
                    &self.overlay.next_profile,
                )?;
                map.serialize_entry(agent_id, profile)?;
                inserted = true;
            } else {
                map.serialize_entry(agent_id, profile)?;
            }
        }
        if self.overlay.allow_insert && !inserted {
            map.serialize_entry(
                self.overlay.target_agent_id.as_str(),
                &self.overlay.next_profile,
            )?;
        }
        map.end()
    }
}

struct AgentMapProjection<'a> {
    agents: &'a BTreeMap<String, AgentCell>,
    body_overlay: Option<&'a BodyOverlay>,
}

impl Serialize for AgentMapProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.agents.len()))?;
        for (agent_id, cell) in self.agents {
            if let Some(overlay) = self
                .body_overlay
                .filter(|overlay| overlay.agent_id == *agent_id)
            {
                map.serialize_entry(
                    agent_id,
                    &AgentCellProjection {
                        cell,
                        body_overlay: overlay,
                    },
                )?;
            } else {
                map.serialize_entry(agent_id, cell)?;
            }
        }
        map.end()
    }
}

struct AgentCellProjection<'a> {
    cell: &'a AgentCell,
    body_overlay: &'a BodyOverlay,
}

impl Serialize for AgentCellProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let field_count =
            3 + usize::from(self.cell.activity.is_some()) + usize::from(self.cell.intent.is_some());
        let mut state = serializer.serialize_struct("AgentCell", field_count)?;
        let body_view = match &self.body_overlay.mutation {
            BodyOverlayMutation::Body { body_view, .. } => body_view,
            BodyOverlayMutation::RouteOnly => &self.cell.state.body_view,
        };
        let last_active = match &self.body_overlay.mutation {
            BodyOverlayMutation::Body { last_active, .. } => last_active,
            BodyOverlayMutation::RouteOnly => &self.cell.last_active,
        };
        state.serialize_field(
            "state",
            &AgentStateProjection {
                state: &self.cell.state,
                body_view,
            },
        )?;
        state.serialize_field(
            "mailbox",
            &MailboxProjection {
                mailbox: &self.cell.mailbox,
                appended_event: self.body_overlay.routed_domain_event.as_ref(),
            },
        )?;
        state.serialize_field("last_active", last_active)?;
        if self.cell.activity.is_some() {
            state.serialize_field("activity", &self.cell.activity)?;
        }
        if self.cell.intent.is_some() {
            state.serialize_field("intent", &self.cell.intent)?;
        }
        state.end()
    }
}

struct MailboxProjection<'a> {
    mailbox: &'a std::collections::VecDeque<DomainEvent>,
    appended_event: Option<&'a DomainEvent>,
}

impl Serialize for MailboxProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let appended_len = usize::from(self.appended_event.is_some());
        let mut sequence = serializer.serialize_seq(Some(self.mailbox.len() + appended_len))?;
        for event in self.mailbox {
            sequence.serialize_element(event)?;
        }
        if let Some(event) = self.appended_event {
            sequence.serialize_element(event)?;
        }
        sequence.end()
    }
}

struct AgentStateProjection<'a> {
    state: &'a crate::models::AgentState,
    body_view: &'a crate::models::BodyKernelView,
}

impl Serialize for AgentStateProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AgentState", 6)?;
        state.serialize_field("agent_id", &self.state.agent_id)?;
        state.serialize_field("pos", &self.state.pos)?;
        state.serialize_field("body", &self.state.body)?;
        state.serialize_field("resources", &self.state.resources)?;
        state.serialize_field("body_view", self.body_view)?;
        state.serialize_field("body_state", &self.state.body_state)?;
        state.end()
    }
}

fn serialize_world_state<S>(
    state: &WorldState,
    body_overlay: Option<&BodyOverlay>,
    command_overlay: Option<&CommandStateOverlay<'_>>,
    module_instance_overlay: Option<&module_instance_transition::PreparedModuleInstance>,
    module_release_overlay: Option<&module_release_transition::PreparedModuleRelease>,
    module_marketplace_overlay: Option<&module_marketplace_transition::PreparedModuleMarketplace>,
    governance_identity_profile_overlay: Option<&GovernanceIdentityProfileOverlay>,
    governance_registry_overlay: Option<
        &crate::runtime::world::governance_registry_publication::PreparedGovernanceRegistryEvent,
    >,
    agent_intent_overlay: Option<
        &crate::runtime::world::agent_intent_publication::PreparedAgentIntent,
    >,
    economy_data_overlay: Option<
        &crate::runtime::world::economy_data_publication::PreparedEconomyDataEvent,
    >,
    economic_contract_overlay: Option<
        &crate::runtime::world::economic_contract_publication::PreparedEconomicContractEvent,
    >,
    power_redemption_overlay: Option<
        &crate::runtime::world::power_redemption_publication::PreparedPowerRedemptionEvent,
    >,
    node_points_settlement_overlay: Option<
        &crate::runtime::world::node_points_settlement_publication::PreparedNodePointsSettlement,
    >,
    main_token_monetary_overlay: Option<
        &crate::runtime::world::main_token_monetary_publication::PreparedMainTokenMonetaryEvent,
    >,
    main_token_governance_monetary_overlay: Option<
        &crate::runtime::world::main_token_governance_monetary_publication::PreparedMainTokenGovernanceMonetaryEvent,
    >,
    main_token_restricted_claim_overlay: Option<
        &crate::runtime::world::main_token_restricted_claim_publication::PreparedMainTokenRestrictedClaimEvent,
    >,
    starter_oc_claim_overlay: Option<
        &crate::runtime::world::starter_oc_claim_publication::PreparedStarterOcClaimed,
    >,
    agent_claim_light_lifecycle_overlay: Option<
        &crate::runtime::world::agent_claim_light_lifecycle_publication::PreparedAgentClaimLightLifecycle,
    >,
    agent_claim_economic_overlay: Option<
        &crate::runtime::world::agent_claim_economic_publication::PreparedAgentClaimEconomic,
    >,
    agent_claim_terminal_overlay: Option<
        &crate::runtime::world::agent_claim_terminal_publication::PreparedAgentClaimTerminal,
    >,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Keep this destructuring exhaustive.  Adding a persisted field without
    // adding its canonical serialization below must fail at compile time.
    let WorldState {
        time: _,
        agents: _,
        agent_intent_ledger: _,
        resources: _,
        materials: _,
        material_ledgers: _,
        material_profiles: _,
        logistics_routes: _,
        completed_logistics_route_ids: _,
        completed_logistics_paths: _,
        settled_logistics_transit_ids: _,
        logistics_settlement_receipts: _,
        direct_material_transfer_receipts: _,
        product_profiles: _,
        latest_product_validation: _,
        recipe_profiles: _,
        factory_profiles: _,
        factories: _,
        retired_factory_ids: _,
        settled_factory_build_ids: _,
        pending_factory_builds: _,
        pending_recipe_jobs: _,
        settled_recipe_job_ids: _,
        pending_material_transits: _,
        industry_progress: _,
        alliances: _,
        gameplay_policy: _,
        data_access_permissions: _,
        economic_contracts: _,
        agent_claims: _,
        starter_oc_claims: _,
        authenticated_collect_data_last_nonces: _,
        agent_claim_last_processed_epoch: _,
        contract_pair_last_success_settled_at: _,
        reputation_reward_window_started_at: _,
        reputation_reward_window_accumulated: _,
        reputation_scores: _,
        wars: _,
        governance_votes: _,
        governance_proposals: _,
        governance_identity_profiles: _,
        crises: _,
        meta_progress: _,
        module_states: _,
        module_artifact_owners: _,
        module_artifact_listings: _,
        module_artifact_bids: _,
        module_instances: _,
        module_release_requests: _,
        module_release_manifest_mappings: _,
        next_module_release_request_id: _,
        module_release_role_bindings: _,
        installed_module_targets: _,
        next_module_instance_id: _,
        next_module_market_order_id: _,
        next_module_market_sale_id: _,
        main_token_config: _,
        main_token_supply: _,
        main_token_balances: _,
        restricted_starter_claim_grants: _,
        main_token_genesis_buckets: _,
        main_token_epoch_issuance_records: _,
        main_token_treasury_balances: _,
        main_token_claim_nonces: _,
        main_token_transfer_nonces: _,
        main_token_scheduled_policy_updates: _,
        main_token_node_points_bridge_records: _,
        main_token_treasury_distribution_records: _,
        restricted_starter_claim_liveops_pool_top_up_records: _,
        reward_asset_config: _,
        node_asset_balances: _,
        protocol_power_reserve: _,
        reward_mint_records: _,
        node_redeem_nonces: _,
        system_order_pool_budgets: _,
        node_identity_bindings: _,
        node_main_token_account_bindings: _,
        governance_finality_signer_registry: _,
        governance_validator_admissions: _,
        governance_main_token_controller_registry: _,
        reward_signature_governance_policy: _,
    } = state;

    let field_count = 81
        - usize::from(
            state.agent_intent_ledger.is_empty()
                && agent_intent_overlay.is_none_or(|overlay| overlay.ledger_updates.is_empty()),
        )
        - usize::from(state.latest_product_validation.is_none())
        - usize::from(state.starter_oc_claims.is_empty() && starter_oc_claim_overlay.is_none())
        - usize::from(
            state.authenticated_collect_data_last_nonces.is_empty()
                && economy_data_overlay.is_none_or(|overlay| !overlay.has_projected_nonces(state)),
        );
    let mut output = serializer.serialize_struct("WorldState", field_count)?;
    output.serialize_field("time", &state.time)?;
    if let Some(overlay) = agent_claim_terminal_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_economic_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_light_lifecycle_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = starter_oc_claim_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = main_token_restricted_claim_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = main_token_monetary_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = power_redemption_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = economy_data_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(overlay) = economic_contract_overlay {
        overlay.serialize_agents(state, &mut output)?;
    } else if let Some(command_overlay) = command_overlay {
        output.serialize_field(
            "agents",
            &CommandAgentMapProjection {
                agents: &state.agents,
                updates: command_overlay.agents,
            },
        )?;
    } else {
        output.serialize_field(
            "agents",
            &AgentMapProjection {
                agents: &state.agents,
                body_overlay,
            },
        )?;
    }
    if let Some(overlay) = agent_intent_overlay
        && (!state.agent_intent_ledger.is_empty() || !overlay.ledger_updates.is_empty())
    {
        output.serialize_field(
            "agent_intent_ledger",
            &ReleaseMapProjection {
                base: &state.agent_intent_ledger,
                updates: &overlay.ledger_updates,
            },
        )?;
    } else if !state.agent_intent_ledger.is_empty() {
        output.serialize_field("agent_intent_ledger", &state.agent_intent_ledger)?;
    }
    if let Some(command_overlay) = command_overlay {
        output.serialize_field(
            "resources",
            &CommandResourceMapProjection {
                resources: &state.resources,
                updates: command_overlay.resources,
            },
        )?;
    } else if let Some(overlay) = economic_contract_overlay {
        overlay.serialize_resources(state, &mut output)?;
    } else {
        output.serialize_field("resources", &state.resources)?;
    }
    if let Some(overlay) = agent_claim_terminal_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_economic_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_light_lifecycle_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = starter_oc_claim_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = main_token_restricted_claim_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = main_token_governance_monetary_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = main_token_monetary_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = power_redemption_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else if let Some(overlay) = economy_data_overlay {
        overlay.serialize_material_fields(state, &mut output)?;
    } else if let Some(overlay) = module_instance_overlay {
        overlay.serialize_material_fields(state, &mut output)?;
    } else if let Some(overlay) = module_release_overlay {
        overlay.serialize_material_fields(state, &mut output)?;
    } else if let Some(overlay) = module_marketplace_overlay {
        overlay.serialize_material_fields(state, &mut output)?;
    } else if let Some(overlay) = economic_contract_overlay {
        overlay.serialize_materials(state, &mut output)?;
    } else {
        output.serialize_field("materials", &state.materials)?;
        output.serialize_field("material_ledgers", &state.material_ledgers)?;
    }
    output.serialize_field("material_profiles", &state.material_profiles)?;
    output.serialize_field("logistics_routes", &state.logistics_routes)?;
    output.serialize_field(
        "completed_logistics_route_ids",
        &state.completed_logistics_route_ids,
    )?;
    output.serialize_field(
        "completed_logistics_paths",
        &state.completed_logistics_paths,
    )?;
    output.serialize_field(
        "settled_logistics_transit_ids",
        &state.settled_logistics_transit_ids,
    )?;
    output.serialize_field(
        "logistics_settlement_receipts",
        &state.logistics_settlement_receipts,
    )?;
    output.serialize_field(
        "direct_material_transfer_receipts",
        &state.direct_material_transfer_receipts,
    )?;
    if let Some(overlay) = module_release_overlay {
        output.serialize_field(
            "product_profiles",
            &ReleaseMapProjection {
                base: &state.product_profiles,
                updates: &overlay.products,
            },
        )?;
    } else {
        output.serialize_field("product_profiles", &state.product_profiles)?;
    }
    if state.latest_product_validation.is_some() {
        output.serialize_field(
            "latest_product_validation",
            &state.latest_product_validation,
        )?;
    }
    if let Some(overlay) = module_release_overlay {
        output.serialize_field(
            "recipe_profiles",
            &ReleaseMapProjection {
                base: &state.recipe_profiles,
                updates: &overlay.recipes,
            },
        )?;
        output.serialize_field(
            "factory_profiles",
            &ReleaseMapProjection {
                base: &state.factory_profiles,
                updates: &overlay.factories,
            },
        )?;
    } else {
        output.serialize_field("recipe_profiles", &state.recipe_profiles)?;
        output.serialize_field("factory_profiles", &state.factory_profiles)?;
    }
    output.serialize_field("factories", &state.factories)?;
    output.serialize_field("retired_factory_ids", &state.retired_factory_ids)?;
    output.serialize_field(
        "settled_factory_build_ids",
        &state.settled_factory_build_ids,
    )?;
    output.serialize_field("pending_factory_builds", &state.pending_factory_builds)?;
    output.serialize_field("pending_recipe_jobs", &state.pending_recipe_jobs)?;
    output.serialize_field("settled_recipe_job_ids", &state.settled_recipe_job_ids)?;
    output.serialize_field(
        "pending_material_transits",
        &state.pending_material_transits,
    )?;
    output.serialize_field("industry_progress", &state.industry_progress)?;
    output.serialize_field("alliances", &state.alliances)?;
    output.serialize_field("gameplay_policy", &state.gameplay_policy)?;
    if let Some(overlay) = economy_data_overlay {
        overlay.serialize_permissions(state, &mut output)?;
    } else {
        output.serialize_field("data_access_permissions", &state.data_access_permissions)?;
    }
    if let Some(overlay) = economic_contract_overlay {
        overlay.serialize_contracts(state, &mut output)?;
    } else {
        output.serialize_field("economic_contracts", &state.economic_contracts)?;
    }
    if let Some(overlay) = agent_claim_terminal_overlay {
        overlay.serialize_claims(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_economic_overlay {
        overlay.serialize_claims(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_light_lifecycle_overlay {
        overlay.serialize_claims(state, &mut output)?;
    } else {
        output.serialize_field("agent_claims", &state.agent_claims)?;
    }
    if let Some(overlay) = starter_oc_claim_overlay {
        overlay.serialize_claims(state, &mut output)?;
    } else if !state.starter_oc_claims.is_empty() {
        output.serialize_field("starter_oc_claims", &state.starter_oc_claims)?;
    }
    if let Some(overlay) = economy_data_overlay
        && overlay.has_projected_nonces(state)
    {
        overlay.serialize_nonces(state, &mut output)?;
    } else if !state.authenticated_collect_data_last_nonces.is_empty() {
        output.serialize_field(
            "authenticated_collect_data_last_nonces",
            &state.authenticated_collect_data_last_nonces,
        )?;
    }
    if let Some(overlay) = agent_claim_terminal_overlay {
        overlay.serialize_last_epoch(&mut output)?;
    } else if let Some(overlay) = agent_claim_economic_overlay {
        overlay.serialize_last_epoch(&mut output)?;
    } else {
        output.serialize_field(
            "agent_claim_last_processed_epoch",
            &state.agent_claim_last_processed_epoch,
        )?;
    }
    if let Some(overlay) = economic_contract_overlay {
        overlay.serialize_reputation(state, &mut output)?;
    } else {
        output.serialize_field(
            "contract_pair_last_success_settled_at",
            &state.contract_pair_last_success_settled_at,
        )?;
        output.serialize_field(
            "reputation_reward_window_started_at",
            &state.reputation_reward_window_started_at,
        )?;
        output.serialize_field(
            "reputation_reward_window_accumulated",
            &state.reputation_reward_window_accumulated,
        )?;
        output.serialize_field("reputation_scores", &state.reputation_scores)?;
    }
    output.serialize_field("wars", &state.wars)?;
    output.serialize_field("governance_votes", &state.governance_votes)?;
    output.serialize_field("governance_proposals", &state.governance_proposals)?;
    if let Some(overlay) = governance_identity_profile_overlay {
        if !overlay.allow_insert
            && !state
                .governance_identity_profiles
                .contains_key(overlay.target_agent_id.as_str())
        {
            return Err(serde::ser::Error::custom(format!(
                "governance identity profile overlay target not found: {}",
                overlay.target_agent_id
            )));
        }
        output.serialize_field(
            "governance_identity_profiles",
            &GovernanceIdentityProfileMapProjection {
                profiles: &state.governance_identity_profiles,
                overlay,
            },
        )?;
    } else {
        output.serialize_field(
            "governance_identity_profiles",
            &state.governance_identity_profiles,
        )?;
    }
    output.serialize_field("crises", &state.crises)?;
    output.serialize_field("meta_progress", &state.meta_progress)?;
    if let Some(command_overlay) = command_overlay {
        output.serialize_field(
            "module_states",
            &CommandModuleStateMapProjection {
                module_states: &state.module_states,
                updates: command_overlay.module_states,
            },
        )?;
    } else {
        output.serialize_field("module_states", &state.module_states)?;
    }
    if let Some(overlay) = module_marketplace_overlay {
        overlay.serialize_market_fields(state, &mut output)?;
    } else {
        output.serialize_field("module_artifact_owners", &state.module_artifact_owners)?;
        output.serialize_field("module_artifact_listings", &state.module_artifact_listings)?;
        output.serialize_field("module_artifact_bids", &state.module_artifact_bids)?;
    }
    if let Some(overlay) = module_instance_overlay {
        overlay.serialize_fields(state, &mut output)?;
    } else {
        output.serialize_field("module_instances", &state.module_instances)?;
    }
    if let Some(overlay) = module_release_overlay {
        output.serialize_field(
            "module_release_requests",
            &ReleaseMapProjection {
                base: &state.module_release_requests,
                updates: &overlay.requests,
            },
        )?;
        output.serialize_field(
            "module_release_manifest_mappings",
            &ReleaseMapProjection {
                base: &state.module_release_manifest_mappings,
                updates: &overlay.mappings,
            },
        )?;
    } else {
        output.serialize_field("module_release_requests", &state.module_release_requests)?;
        output.serialize_field(
            "module_release_manifest_mappings",
            &state.module_release_manifest_mappings,
        )?;
    }
    output.serialize_field(
        "next_module_release_request_id",
        &module_release_overlay
            .and_then(|overlay| overlay.next_request_id)
            .unwrap_or(state.next_module_release_request_id),
    )?;
    if let Some(overlay) = module_release_overlay {
        output.serialize_field(
            "module_release_role_bindings",
            &module_release_transition::ReleaseOptionalMapProjection {
                base: &state.module_release_role_bindings,
                updates: &overlay.role_bindings,
            },
        )?;
    } else {
        output.serialize_field(
            "module_release_role_bindings",
            &state.module_release_role_bindings,
        )?;
    }
    if let Some(overlay) = module_instance_overlay {
        overlay.serialize_target_fields(state, &mut output)?;
    } else {
        output.serialize_field("installed_module_targets", &state.installed_module_targets)?;
        output.serialize_field("next_module_instance_id", &state.next_module_instance_id)?;
    }
    if let Some(overlay) = module_marketplace_overlay {
        overlay.serialize_counter_fields(&mut output)?;
    } else {
        output.serialize_field(
            "next_module_market_order_id",
            &state.next_module_market_order_id,
        )?;
        output.serialize_field(
            "next_module_market_sale_id",
            &state.next_module_market_sale_id,
        )?;
    }
    output.serialize_field("main_token_config", &state.main_token_config)?;
    if let Some(overlay) = agent_claim_terminal_overlay {
        overlay.serialize_token(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_economic_overlay {
        overlay.serialize_token(state, &mut output)?;
    } else if let Some(overlay) = starter_oc_claim_overlay {
        overlay.serialize_supply_balances(state, &mut output)?;
    } else if let Some(overlay) = main_token_restricted_claim_overlay {
        overlay.serialize_supply_balances(state, &mut output)?;
    } else if let Some(overlay) = main_token_governance_monetary_overlay {
        overlay.serialize_supply_balances(state, &mut output)?;
    } else if let Some(overlay) = main_token_monetary_overlay {
        overlay.serialize_supply_balances(state, &mut output)?;
    } else if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_main_token_accounts(state, &mut output)?;
    } else {
        output.serialize_field("main_token_supply", &state.main_token_supply)?;
        output.serialize_field("main_token_balances", &state.main_token_balances)?;
    }
    if let Some(overlay) = main_token_restricted_claim_overlay {
        overlay.serialize_grants(state, &mut output)?;
    } else {
        output.serialize_field(
            "restricted_starter_claim_grants",
            &state.restricted_starter_claim_grants,
        )?;
    }
    if let Some(overlay) = main_token_monetary_overlay {
        overlay.serialize_buckets(state, &mut output)?;
        overlay.serialize_issuance(state, &mut output)?;
    } else {
        output.serialize_field(
            "main_token_genesis_buckets",
            &state.main_token_genesis_buckets,
        )?;
        output.serialize_field(
            "main_token_epoch_issuance_records",
            &state.main_token_epoch_issuance_records,
        )?;
    }
    if let Some(overlay) = agent_claim_terminal_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else if let Some(overlay) = agent_claim_economic_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else if let Some(overlay) = starter_oc_claim_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else if let Some(overlay) = main_token_restricted_claim_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else if let Some(overlay) = main_token_governance_monetary_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else if let Some(overlay) = main_token_monetary_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_treasury(state, &mut output)?;
    } else {
        output.serialize_field(
            "main_token_treasury_balances",
            &state.main_token_treasury_balances,
        )?;
    }
    if let Some(overlay) = main_token_monetary_overlay {
        overlay.serialize_claim_nonces(state, &mut output)?;
        overlay.serialize_transfer_nonces(state, &mut output)?;
    } else {
        output.serialize_field("main_token_claim_nonces", &state.main_token_claim_nonces)?;
        output.serialize_field(
            "main_token_transfer_nonces",
            &state.main_token_transfer_nonces,
        )?;
    }
    if let Some(overlay) = main_token_governance_monetary_overlay {
        overlay.serialize_scheduled(state, &mut output)?;
    } else {
        output.serialize_field(
            "main_token_scheduled_policy_updates",
            &state.main_token_scheduled_policy_updates,
        )?;
    }
    if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_bridge(state, &mut output)?;
    } else {
        output.serialize_field(
            "main_token_node_points_bridge_records",
            &state.main_token_node_points_bridge_records,
        )?;
    }
    if let Some(overlay) = main_token_governance_monetary_overlay {
        overlay.serialize_distributions(state, &mut output)?;
    } else {
        output.serialize_field(
            "main_token_treasury_distribution_records",
            &state.main_token_treasury_distribution_records,
        )?;
    }
    if let Some(overlay) = main_token_restricted_claim_overlay {
        overlay.serialize_topups(state, &mut output)?;
    } else {
        output.serialize_field(
            "restricted_starter_claim_liveops_pool_top_up_records",
            &state.restricted_starter_claim_liveops_pool_top_up_records,
        )?;
    }
    output.serialize_field("reward_asset_config", &state.reward_asset_config)?;
    if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_node_balances(state, &mut output)?;
        output.serialize_field("protocol_power_reserve", &state.protocol_power_reserve)?;
    } else if let Some(overlay) = power_redemption_overlay {
        overlay.serialize_node_balances(state, &mut output)?;
        overlay.serialize_reserve(state, &mut output)?;
    } else {
        output.serialize_field("node_asset_balances", &state.node_asset_balances)?;
        output.serialize_field("protocol_power_reserve", &state.protocol_power_reserve)?;
    }
    if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_reward_mints(&mut output)?;
    } else {
        output.serialize_field("reward_mint_records", &state.reward_mint_records)?;
    }
    if let Some(overlay) = power_redemption_overlay {
        overlay.serialize_nonces(state, &mut output)?;
    } else {
        output.serialize_field("node_redeem_nonces", &state.node_redeem_nonces)?;
    }
    if let Some(overlay) = node_points_settlement_overlay {
        overlay.serialize_budgets(state, &mut output)?;
    } else {
        output.serialize_field(
            "system_order_pool_budgets",
            &state.system_order_pool_budgets,
        )?;
    }
    output.serialize_field(
        "node_identity_bindings",
        governance_registry_overlay.map_or(&state.node_identity_bindings, |o| &o.identity_bindings),
    )?;
    output.serialize_field(
        "node_main_token_account_bindings",
        governance_registry_overlay.map_or(&state.node_main_token_account_bindings, |o| {
            &o.account_bindings
        }),
    )?;
    output.serialize_field(
        "governance_finality_signer_registry",
        &state.governance_finality_signer_registry,
    )?;
    output.serialize_field(
        "governance_validator_admissions",
        governance_registry_overlay
            .map_or(&state.governance_validator_admissions, |o| &o.admissions),
    )?;
    output.serialize_field(
        "governance_main_token_controller_registry",
        governance_registry_overlay.map_or(&state.governance_main_token_controller_registry, |o| {
            &o.controller_registry
        }),
    )?;
    output.serialize_field(
        "reward_signature_governance_policy",
        &state.reward_signature_governance_policy,
    )?;
    output.end()
}
