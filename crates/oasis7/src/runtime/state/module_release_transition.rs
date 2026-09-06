//! Owned sparse profile/release mutations; validation never writes canonical state.
use super::*;
use serde::ser::SerializeMap;

#[derive(Debug)]
pub(crate) struct PreparedModuleRelease {
    events: Vec<DomainEvent>,
    agents: BTreeMap<String, AgentCell>,
    pub(crate) products: BTreeMap<String, ProductProfileV1>,
    pub(crate) recipes: BTreeMap<String, RecipeProfileV1>,
    pub(crate) factories: BTreeMap<String, FactoryProfileV1>,
    pub(crate) requests: BTreeMap<u64, ModuleReleaseRequestState>,
    pub(crate) mappings: BTreeMap<u64, ModuleReleaseManifestMappingState>,
    world_materials: BTreeMap<String, i64>,
}

impl PreparedModuleRelease {
    pub(crate) fn new(state: &WorldState, agents: BTreeMap<String, AgentCell>) -> Self {
        Self {
            events: Vec::new(),
            agents,
            products: BTreeMap::new(),
            recipes: BTreeMap::new(),
            factories: BTreeMap::new(),
            requests: BTreeMap::new(),
            mappings: BTreeMap::new(),
            world_materials: state
                .material_ledgers
                .get(&MaterialLedgerId::world())
                .filter(|ledger| !ledger.is_empty())
                .unwrap_or(&state.materials)
                .clone(),
        }
    }

    pub(crate) fn matches_event(&self, event: &DomainEvent) -> bool {
        self.events.as_slice() == std::slice::from_ref(event)
    }

    pub(crate) fn routed_agents(&self) -> BTreeMap<String, AgentCell> {
        let mut agents = self.agents.clone();
        for event in &self.events {
            if let Some(agent_id) = event.agent_id()
                && let Some(cell) = agents.get_mut(agent_id)
            {
                cell.mailbox.push_back(event.clone());
            }
        }
        agents
    }

    pub(crate) fn install_routed(mut self, state: &mut WorldState) {
        self.agents = self.routed_agents();
        self.install_infallible(state);
    }

    pub(crate) fn install_infallible(self, state: &mut WorldState) {
        state.agents.extend(self.agents);
        state.product_profiles.extend(self.products);
        state.recipe_profiles.extend(self.recipes);
        state.factory_profiles.extend(self.factories);
        state.module_release_requests.extend(self.requests);
        state.module_release_manifest_mappings.extend(self.mappings);
        state.materials = self.world_materials.clone();
        state
            .material_ledgers
            .insert(MaterialLedgerId::world(), self.world_materials);
    }

    pub(crate) fn serialize_material_fields<S: serde::ser::SerializeStruct>(
        &self,
        state: &WorldState,
        output: &mut S,
    ) -> Result<(), S::Error> {
        output.serialize_field("materials", &self.world_materials)?;
        let updates = BTreeMap::from([(MaterialLedgerId::world(), &self.world_materials)]);
        let base = state
            .material_ledgers
            .iter()
            .map(|(key, value)| (key.clone(), value))
            .collect();
        output.serialize_field(
            "material_ledgers",
            &ReleaseMapProjection {
                base: &base,
                updates: &updates,
            },
        )
    }

    pub(crate) fn apply_event(
        &mut self,
        state: &WorldState,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        let (operator, profile_kind, proposal_id) = match event {
            DomainEvent::ProductProfileGoverned {
                operator_agent_id,
                proposal_id,
                ..
            } => (operator_agent_id, "product", Some(*proposal_id)),
            DomainEvent::RecipeProfileGoverned {
                operator_agent_id,
                proposal_id,
                ..
            } => (operator_agent_id, "recipe", Some(*proposal_id)),
            DomainEvent::FactoryProfileGoverned {
                operator_agent_id,
                proposal_id,
                ..
            } => (operator_agent_id, "factory", Some(*proposal_id)),
            DomainEvent::ModuleReleaseApplied {
                operator_agent_id, ..
            } => (operator_agent_id, "", None),
            DomainEvent::ModuleReleaseShadowed {
                operator_agent_id, ..
            } => (operator_agent_id, "", None),
            DomainEvent::ModuleReleaseRoleApproved {
                approver_agent_id, ..
            } => (approver_agent_id, "", None),
            DomainEvent::ModuleReleaseRejected {
                rejector_agent_id, ..
            } => (rejector_agent_id, "", None),
            _ => unreachable!("release preparation requires a review, profile or completion event"),
        };
        if let Some(proposal_id) = proposal_id {
            if !self.agents.contains_key(operator) && !state.agents.contains_key(operator) {
                return Err(WorldError::AgentNotFound {
                    agent_id: operator.clone(),
                });
            }
            if proposal_id == 0 {
                return Err(WorldError::ResourceBalanceInvalid {
                    reason: format!("{profile_kind} profile governed proposal_id must be > 0"),
                });
            }
        }
        match event {
            DomainEvent::ModuleReleaseShadowed {
                request_id,
                manifest_hash,
                ..
            } => {
                let mut request = self
                    .requests
                    .get(request_id)
                    .or_else(|| state.module_release_requests.get(request_id))
                    .ok_or_else(|| {
                        invalid(format!(
                            "module release shadow rejected: request not found ({request_id})"
                        ))
                    })?
                    .clone();
                if !matches!(request.status, ModuleReleaseRequestStatus::Requested) {
                    return Err(invalid(format!(
                        "module release shadow invalid status for request {}: {:?}",
                        request_id, request.status
                    )));
                }
                let mut mapping = self
                    .mappings
                    .get(request_id)
                    .or_else(|| state.module_release_manifest_mappings.get(request_id))
                    .ok_or_else(|| {
                        invalid(format!(
                            "module release mapping missing for shadow request_id={request_id}"
                        ))
                    })?
                    .clone();
                request.status = ModuleReleaseRequestStatus::Shadowed;
                request.shadow_manifest_hash = Some(manifest_hash.clone());
                request.updated_at = now;
                mapping.status = ModuleReleaseRequestStatus::Shadowed;
                mapping.shadow_manifest_hash = Some(manifest_hash.clone());
                mapping.updated_at = now;
                self.requests.insert(*request_id, request);
                self.mappings.insert(*request_id, mapping);
            }
            DomainEvent::ModuleReleaseRoleApproved {
                request_id,
                approver_agent_id,
                role,
            } => {
                let mut request = self
                    .requests
                    .get(request_id)
                    .or_else(|| state.module_release_requests.get(request_id))
                    .ok_or_else(|| {
                        invalid(format!(
                            "module release approve_role rejected: request not found ({request_id})"
                        ))
                    })?
                    .clone();
                if !matches!(
                    request.status,
                    ModuleReleaseRequestStatus::Shadowed
                        | ModuleReleaseRequestStatus::PartiallyApproved
                        | ModuleReleaseRequestStatus::Approved
                ) {
                    return Err(invalid(format!(
                        "module release approve_role invalid status for request {}: {:?}",
                        request_id, request.status
                    )));
                }
                let normalized_role = role.trim().to_ascii_lowercase();
                if normalized_role.is_empty() {
                    return Err(invalid(format!(
                        "module release approve_role role cannot be empty (request_id={request_id})"
                    )));
                }
                if !request
                    .required_roles
                    .iter()
                    .any(|item| item == &normalized_role)
                {
                    return Err(invalid(format!(
                        "module release approve_role role not required: request_id={} role={}",
                        request_id, normalized_role
                    )));
                }
                if let Some(existing) = request.role_approvals.get(&normalized_role) {
                    if existing != approver_agent_id {
                        return Err(invalid(format!(
                            "module release approve_role approver mismatch: request_id={} role={} existing={} incoming={}",
                            request_id, normalized_role, existing, approver_agent_id
                        )));
                    }
                } else {
                    request
                        .role_approvals
                        .insert(normalized_role, approver_agent_id.clone());
                }
                request.status = if request
                    .required_roles
                    .iter()
                    .all(|required| request.role_approvals.contains_key(required))
                {
                    ModuleReleaseRequestStatus::Approved
                } else {
                    ModuleReleaseRequestStatus::PartiallyApproved
                };
                request.updated_at = now;
                if let Some(mut mapping) = self
                    .mappings
                    .get(request_id)
                    .or_else(|| state.module_release_manifest_mappings.get(request_id))
                    .cloned()
                {
                    mapping.status = request.status;
                    mapping.updated_at = now;
                    self.mappings.insert(*request_id, mapping);
                }
                self.requests.insert(*request_id, request);
            }
            DomainEvent::ModuleReleaseRejected {
                request_id, reason, ..
            } => {
                let mut request = self
                    .requests
                    .get(request_id)
                    .or_else(|| state.module_release_requests.get(request_id))
                    .ok_or_else(|| {
                        invalid(format!(
                            "module release reject rejected: request not found ({request_id})"
                        ))
                    })?
                    .clone();
                if matches!(
                    request.status,
                    ModuleReleaseRequestStatus::Applied | ModuleReleaseRequestStatus::Rejected
                ) {
                    return Err(invalid(format!(
                        "module release reject invalid status for request {}: {:?}",
                        request_id, request.status
                    )));
                }
                if reason.trim().is_empty() {
                    return Err(invalid(format!(
                        "module release reject reason cannot be empty (request_id={request_id})"
                    )));
                }
                request.status = ModuleReleaseRequestStatus::Rejected;
                request.rejected_reason = Some(reason.clone());
                request.updated_at = now;
                if let Some(mut mapping) = self
                    .mappings
                    .get(request_id)
                    .or_else(|| state.module_release_manifest_mappings.get(request_id))
                    .cloned()
                {
                    mapping.status = ModuleReleaseRequestStatus::Rejected;
                    mapping.updated_at = now;
                    self.mappings.insert(*request_id, mapping);
                }
                self.requests.insert(*request_id, request);
            }
            DomainEvent::ProductProfileGoverned { profile, .. } => {
                if profile.product_id.trim().is_empty() {
                    return Err(invalid("product profile product_id cannot be empty"));
                }
                if profile.role_tag.trim().is_empty() {
                    return Err(invalid(format!(
                        "product profile role_tag cannot be empty: {}",
                        profile.product_id
                    )));
                }
                self.products
                    .insert(profile.product_id.clone(), profile.clone());
            }
            DomainEvent::RecipeProfileGoverned { profile, .. } => {
                if profile.recipe_id.trim().is_empty() {
                    return Err(invalid("recipe profile recipe_id cannot be empty"));
                }
                self.recipes
                    .insert(profile.recipe_id.clone(), profile.clone());
            }
            DomainEvent::FactoryProfileGoverned { profile, .. } => {
                if profile.factory_id.trim().is_empty() {
                    return Err(invalid("factory profile factory_id cannot be empty"));
                }
                if profile.tier == 0 {
                    return Err(invalid(format!(
                        "factory profile tier must be >= 1: {}",
                        profile.factory_id
                    )));
                }
                if profile.recipe_slots == 0 {
                    return Err(invalid(format!(
                        "factory profile recipe_slots must be > 0: {}",
                        profile.factory_id
                    )));
                }
                self.factories
                    .insert(profile.factory_id.clone(), profile.clone());
            }
            DomainEvent::ModuleReleaseApplied {
                request_id,
                manifest_hash,
                proposal_id,
                ..
            } => {
                let mut request = self
                    .requests
                    .get(request_id)
                    .or_else(|| state.module_release_requests.get(request_id))
                    .ok_or_else(|| {
                        invalid(format!(
                            "module release apply rejected: request not found ({request_id})"
                        ))
                    })?
                    .clone();
                if !matches!(request.status, ModuleReleaseRequestStatus::Approved) {
                    return Err(invalid(format!(
                        "module release apply invalid status for request {}: {:?}",
                        request_id, request.status
                    )));
                }
                request.status = ModuleReleaseRequestStatus::Applied;
                request.applied_manifest_hash = Some(manifest_hash.clone());
                request.applied_proposal_id = (*proposal_id != 0).then_some(*proposal_id);
                request.updated_at = now;
                let mut mapping = self
                    .mappings
                    .get(request_id)
                    .or_else(|| state.module_release_manifest_mappings.get(request_id))
                    .ok_or_else(|| {
                        invalid(format!(
                            "module release mapping missing for apply request_id={request_id}"
                        ))
                    })?
                    .clone();
                mapping.status = ModuleReleaseRequestStatus::Applied;
                mapping.applied_manifest_hash = Some(manifest_hash.clone());
                mapping.applied_proposal_id = (*proposal_id != 0).then_some(*proposal_id);
                mapping.updated_at = now;
                self.requests.insert(*request_id, request);
                self.mappings.insert(*request_id, mapping);
            }
            _ => unreachable!(),
        }
        // Resolve against the staged fee-debited installer first when roles coincide.
        if !self.agents.contains_key(operator)
            && let Some(cell) = state.agents.get(operator)
        {
            self.agents.insert(operator.clone(), cell.clone());
        }
        if let Some(cell) = self.agents.get_mut(operator) {
            cell.last_active = now;
        }
        self.events.push(event.clone());
        Ok(())
    }
}

fn invalid(reason: impl Into<String>) -> WorldError {
    WorldError::ResourceBalanceInvalid {
        reason: reason.into(),
    }
}

impl WorldState {
    pub(crate) fn prepare_module_release_event(
        &self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<PreparedModuleRelease, WorldError> {
        let mut prepared = PreparedModuleRelease::new(self, BTreeMap::new());
        prepared.apply_event(self, event, now)?;
        Ok(prepared)
    }
}

pub(crate) struct ReleaseMapProjection<'a, K, V> {
    pub(crate) base: &'a BTreeMap<K, V>,
    pub(crate) updates: &'a BTreeMap<K, V>,
}

impl<K: Ord + Serialize, V: Serialize> Serialize for ReleaseMapProjection<'_, K, V> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let added = self
            .updates
            .keys()
            .filter(|key| !self.base.contains_key(*key))
            .count();
        let mut map = serializer.serialize_map(Some(self.base.len() + added))?;
        let mut updates = self.updates.iter().peekable();
        for (key, value) in self.base {
            while updates.peek().is_some_and(|(next, _)| *next < key) {
                let (next, replacement) = updates.next().unwrap();
                map.serialize_entry(next, replacement)?;
            }
            if updates.peek().is_some_and(|(next, _)| *next == key) {
                let (_, replacement) = updates.next().unwrap();
                map.serialize_entry(key, replacement)?;
            } else {
                map.serialize_entry(key, value)?;
            }
        }
        for (key, value) in updates {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}
