use super::*;

fn refresh_progress_without_factory(
    state: &WorldState,
    progress: &mut IndustryProgressState,
    now: WorldTime,
    removed: &str,
) {
    let current = progress.stage;
    let active_completed_jobs = state
        .factories
        .iter()
        .filter(|(id, _)| id.as_str() != removed)
        .map(|(_, factory)| factory.production.completed_jobs)
        .sum::<u64>();
    let stable = state
        .factories
        .iter()
        .filter(|(id, _)| id.as_str() != removed)
        .any(|(_, factory)| factory.production.same_recipe_repeat_count >= 3);
    let mut next = if stable {
        IndustryStage::ScaleOut
    } else {
        IndustryStage::Bootstrap
    };
    let governance_enabled =
        state.gameplay_policy.electricity_tax_bps > 0 || state.gameplay_policy.data_tax_bps > 0;
    if next == IndustryStage::ScaleOut
        && governance_enabled
        && (active_completed_jobs >= 6 || progress.completed_material_transits >= 3)
    {
        next = IndustryStage::Governance;
    }
    if next != current {
        progress.stage = next;
        progress.stage_updated_at = now;
    }
}

impl PreparedFactoryLifecycle {
    pub(crate) fn prepare(
        state: &WorldState,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<Self, WorldError> {
        match event {
            DomainEvent::FactoryBuildStarted {
                job_id,
                builder_agent_id,
                site_id,
                spec,
                consume_ledger,
                ready_at,
            } => Self::prepare_started(
                state,
                event,
                now,
                *job_id,
                builder_agent_id,
                site_id,
                spec,
                consume_ledger,
                *ready_at,
            ),
            DomainEvent::FactoryBuilt {
                job_id,
                builder_agent_id,
                site_id,
                spec,
            } => Self::prepare_built(state, event, now, *job_id, builder_agent_id, site_id, spec),
            DomainEvent::FactoryDurabilityChanged {
                factory_id,
                durability_ppm,
                ..
            } => {
                let (materials, world) = normalized_materials(state);
                let factory = state.factories.get(factory_id).cloned().map(|mut value| {
                    value.durability_ppm = (*durability_ppm).clamp(0, 1_000_000);
                    (factory_id.clone(), Some(value))
                });
                Ok(Self {
                    event: event.clone(),
                    pending: None,
                    factory,
                    settled_id: None,
                    retired_insert: None,
                    pending_recipe_deletions: BTreeSet::new(),
                    ledgers: BTreeMap::from([(MaterialLedgerId::world(), world)]),
                    materials,
                    agent: None,
                    progress: None,
                })
            }
            DomainEvent::FactoryMaintained {
                operator_agent_id,
                factory_id,
                consume_ledger,
                consumed_parts,
                durability_ppm,
            } => Self::prepare_maintained(
                state,
                event,
                now,
                operator_agent_id,
                factory_id,
                consume_ledger,
                *consumed_parts,
                *durability_ppm,
            ),
            DomainEvent::FactoryRecycled {
                operator_agent_id,
                factory_id,
                recycle_ledger,
                recovered,
                ..
            } => Self::prepare_recycled(
                state,
                event,
                now,
                operator_agent_id,
                factory_id,
                recycle_ledger,
                recovered,
            ),
            _ => Err(invalid(
                "factory lifecycle preparation requires supported event",
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_started(
        state: &WorldState,
        event: &DomainEvent,
        now: WorldTime,
        job_id: ActionId,
        builder: &str,
        site_id: &str,
        spec: &FactoryModuleSpec,
        ledger_id: &MaterialLedgerId,
        ready_at: WorldTime,
    ) -> Result<Self, WorldError> {
        if state.pending_factory_builds.contains_key(&job_id)
            || state.settled_factory_build_ids.contains(&job_id)
        {
            return Err(invalid(format!(
                "factory build job already settled or pending: job_id={job_id}"
            )));
        }
        if builder.trim().is_empty() || !state.agents.contains_key(builder) {
            return Err(invalid(format!(
                "factory build builder agent is missing or empty: builder_agent_id={builder}"
            )));
        }
        if site_id.trim().is_empty() {
            return Err(invalid("factory build site_id cannot be empty"));
        }
        if spec.factory_id.trim().is_empty() {
            return Err(invalid("factory build factory_id cannot be empty"));
        }
        if ready_at <= now {
            return Err(invalid(format!(
                "factory build ready_at must be in the future: now={now} ready_at={ready_at}"
            )));
        }
        if state.retired_factory_ids.contains(&spec.factory_id) {
            return Err(invalid(format!(
                "factory build started for retired identity: factory_id={}",
                spec.factory_id
            )));
        }
        if state.factories.contains_key(&spec.factory_id)
            || state
                .pending_factory_builds
                .values()
                .any(|job| job.spec.factory_id == spec.factory_id)
        {
            return Err(invalid(format!(
                "factory build identity is already active: factory_id={}",
                spec.factory_id
            )));
        }
        let mut required = BTreeMap::<String, i64>::new();
        for stack in &spec.build_cost {
            if stack.kind.trim().is_empty() || stack.amount <= 0 {
                return Err(invalid(format!(
                    "factory build cost must be positive and named: {}={}",
                    stack.kind, stack.amount
                )));
            }
            let total = required.entry(stack.kind.clone()).or_default();
            *total = total.checked_add(stack.amount).ok_or_else(|| {
                invalid(format!(
                    "factory build cost overflow: kind={} amount={}",
                    stack.kind, stack.amount
                ))
            })?;
        }
        for (kind, amount) in &required {
            let available = state
                .material_ledgers
                .get(ledger_id)
                .and_then(|ledger| ledger.get(kind))
                .copied()
                .unwrap_or(0);
            if available < *amount {
                return Err(invalid(format!(
                    "factory build consume failed: insufficient material {kind}: requested={amount} available={available}"
                )));
            }
        }
        let (mut materials, world) = normalized_materials(state);
        let world_id = MaterialLedgerId::world();
        let mut ledgers = BTreeMap::from([(world_id.clone(), world)]);
        if ledger_id != &world_id {
            ledgers.insert(
                ledger_id.clone(),
                state
                    .material_ledgers
                    .get(ledger_id)
                    .cloned()
                    .unwrap_or_default(),
            );
        }
        for stack in &spec.build_cost {
            remove_material_balance_for_ledger(&mut ledgers, ledger_id, &stack.kind, stack.amount)
                .map_err(|reason| invalid(format!("factory build consume failed: {reason}")))?;
        }
        if ledger_id == &world_id {
            materials = ledgers.get(&world_id).cloned().unwrap_or_default();
        }
        let mut agent = state
            .agents
            .get(builder)
            .cloned()
            .expect("validated builder");
        agent.last_active = now;
        Ok(Self {
            event: event.clone(),
            pending: Some((
                job_id,
                Some(FactoryBuildJobState {
                    job_id,
                    builder_agent_id: builder.into(),
                    site_id: site_id.into(),
                    spec: spec.clone(),
                    consume_ledger: ledger_id.clone(),
                    ready_at,
                }),
            )),
            factory: None,
            settled_id: None,
            retired_insert: None,
            pending_recipe_deletions: BTreeSet::new(),
            ledgers,
            materials,
            agent: Some((builder.into(), agent)),
            progress: None,
        })
    }

    fn prepare_built(
        state: &WorldState,
        event: &DomainEvent,
        now: WorldTime,
        job_id: ActionId,
        builder: &str,
        site_id: &str,
        spec: &FactoryModuleSpec,
    ) -> Result<Self, WorldError> {
        let (materials, world) = normalized_materials(state);
        if state.settled_factory_build_ids.contains(&job_id) {
            if state
                .factories
                .get(&spec.factory_id)
                .is_some_and(|factory| {
                    factory.builder_agent_id == builder
                        && factory.site_id == site_id
                        && factory.spec == *spec
                })
            {
                return Ok(Self::no_op(state, event, builder, materials, world));
            }
            return Err(invalid(format!(
                "settled factory build completion does not match receipt: job_id={job_id}"
            )));
        }
        let pending = state.pending_factory_builds.get(&job_id).ok_or_else(|| {
            invalid(format!(
                "factory build completion has no pending job: job_id={job_id}"
            ))
        })?;
        if state.retired_factory_ids.contains(&spec.factory_id) {
            return Err(invalid(format!(
                "factory built for retired identity: factory_id={}",
                spec.factory_id
            )));
        }
        if pending.builder_agent_id != builder
            || pending.site_id != site_id
            || pending.spec != *spec
        {
            return Err(invalid(format!(
                "factory build completion does not match pending commitment: job_id={job_id}"
            )));
        }
        if now < pending.ready_at {
            return Err(invalid(format!(
                "factory build completion is early: job_id={job_id} ready_at={} now={now}",
                pending.ready_at
            )));
        }
        if state.factories.contains_key(&spec.factory_id) {
            return Err(invalid(format!(
                "factory build completion would overwrite active factory: factory_id={}",
                spec.factory_id
            )));
        }
        let site_ledger = MaterialLedgerId::site(site_id.to_string());
        let factory = FactoryState {
            factory_id: spec.factory_id.clone(),
            site_id: site_id.into(),
            builder_agent_id: builder.into(),
            spec: spec.clone(),
            input_ledger: site_ledger.clone(),
            output_ledger: site_ledger,
            durability_ppm: 1_000_000,
            production: FactoryProductionState::default(),
            built_at: now,
        };
        let agent = state.agents.get(builder).cloned().map(|mut cell| {
            cell.last_active = now;
            (builder.to_string(), cell)
        });
        let mut progress = state.industry_progress.clone();
        material_transit::refresh_progress_stage(state, &mut progress, now);
        Ok(Self {
            event: event.clone(),
            pending: Some((job_id, None)),
            factory: Some((spec.factory_id.clone(), Some(factory))),
            settled_id: Some(job_id),
            retired_insert: None,
            pending_recipe_deletions: BTreeSet::new(),
            ledgers: BTreeMap::from([(MaterialLedgerId::world(), world)]),
            materials,
            agent,
            progress: Some(progress),
        })
    }

    fn no_op(
        state: &WorldState,
        event: &DomainEvent,
        builder: &str,
        materials: BTreeMap<String, i64>,
        world: BTreeMap<String, i64>,
    ) -> Self {
        Self {
            event: event.clone(),
            pending: None,
            factory: None,
            settled_id: None,
            retired_insert: None,
            pending_recipe_deletions: BTreeSet::new(),
            ledgers: BTreeMap::from([(MaterialLedgerId::world(), world)]),
            materials,
            agent: state
                .agents
                .get(builder)
                .cloned()
                .map(|cell| (builder.to_string(), cell)),
            progress: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_maintained(
        state: &WorldState,
        event: &DomainEvent,
        now: WorldTime,
        operator: &str,
        factory_id: &str,
        ledger_id: &MaterialLedgerId,
        parts: i64,
        durability: i64,
    ) -> Result<Self, WorldError> {
        if state.retired_factory_ids.contains(factory_id) {
            return Err(invalid(format!(
                "factory maintenance targets retired identity: factory_id={factory_id}"
            )));
        }
        let mut factory = state.factories.get(factory_id).cloned().ok_or_else(|| {
            invalid(format!(
                "factory maintenance targets unknown factory: factory_id={factory_id}"
            ))
        })?;
        if factory.builder_agent_id != operator {
            return Err(invalid(format!(
                "factory maintenance operator mismatch: factory_id={factory_id} operator={operator} builder={} ",
                factory.builder_agent_id
            )));
        }
        if parts <= 0 {
            return Err(invalid(format!(
                "factory maintenance consumed_parts must be positive: factory_id={factory_id} consumed_parts={parts}"
            )));
        }
        if !(0..=1_000_000).contains(&durability) {
            return Err(invalid(format!(
                "factory maintenance durability is out of range: factory_id={factory_id} durability_ppm={durability}"
            )));
        }
        let available = state
            .material_ledgers
            .get(ledger_id)
            .and_then(|v| v.get("hardware_part"))
            .copied()
            .unwrap_or(0);
        if available < parts {
            return Err(invalid(format!(
                "factory maintenance consume failed: insufficient material hardware_part: requested={parts} available={available}"
            )));
        }
        let (mut materials, world) = normalized_materials(state);
        let world_id = MaterialLedgerId::world();
        let mut ledgers = BTreeMap::from([(world_id.clone(), world)]);
        if ledger_id != &world_id {
            ledgers.insert(
                ledger_id.clone(),
                state
                    .material_ledgers
                    .get(ledger_id)
                    .cloned()
                    .unwrap_or_default(),
            );
        }
        remove_material_balance_for_ledger(&mut ledgers, ledger_id, "hardware_part", parts)
            .map_err(|reason| invalid(format!("factory maintenance consume failed: {reason}")))?;
        if ledger_id == &world_id {
            materials = ledgers[&world_id].clone();
        }
        factory.durability_ppm = durability;
        let agent = state.agents.get(operator).cloned().map(|mut cell| {
            cell.last_active = now;
            (operator.to_string(), cell)
        });
        Ok(Self {
            event: event.clone(),
            pending: None,
            factory: Some((factory_id.to_string(), Some(factory))),
            settled_id: None,
            retired_insert: None,
            pending_recipe_deletions: BTreeSet::new(),
            ledgers,
            materials,
            agent,
            progress: None,
        })
    }

    fn prepare_recycled(
        state: &WorldState,
        event: &DomainEvent,
        now: WorldTime,
        operator: &str,
        factory_id: &str,
        ledger_id: &MaterialLedgerId,
        recovered: &[MaterialStack],
    ) -> Result<Self, WorldError> {
        let (mut materials, world) = normalized_materials(state);
        if state.retired_factory_ids.contains(factory_id) {
            return Ok(Self {
                event: event.clone(),
                pending: None,
                factory: None,
                settled_id: None,
                retired_insert: None,
                pending_recipe_deletions: BTreeSet::new(),
                ledgers: BTreeMap::from([(MaterialLedgerId::world(), world)]),
                materials,
                agent: state
                    .agents
                    .get(operator)
                    .cloned()
                    .map(|cell| (operator.to_string(), cell)),
                progress: None,
            });
        }
        let factory = state.factories.get(factory_id).ok_or_else(|| {
            invalid(format!(
                "factory recycle targets unknown factory: factory_id={factory_id}"
            ))
        })?;
        if factory.builder_agent_id != operator {
            return Err(invalid(format!(
                "factory recycle operator mismatch: factory_id={factory_id} operator={operator} builder={} ",
                factory.builder_agent_id
            )));
        }
        if state
            .pending_recipe_jobs
            .values()
            .any(|job| job.factory_id == factory_id)
        {
            return Err(invalid(format!(
                "factory recycle targets factory with active recipe: factory_id={factory_id}"
            )));
        }
        preflight_material_balance_additions(&state.material_ledgers, ledger_id, recovered)
            .map_err(|reason| {
                invalid(format!(
                    "factory recycle material preflight failed: {reason}"
                ))
            })?;
        let world_id = MaterialLedgerId::world();
        let mut ledgers = BTreeMap::from([(world_id.clone(), world)]);
        if ledger_id != &world_id {
            ledgers.insert(
                ledger_id.clone(),
                state
                    .material_ledgers
                    .get(ledger_id)
                    .cloned()
                    .unwrap_or_default(),
            );
        }
        for stack in recovered {
            add_material_balance_for_ledger(&mut ledgers, ledger_id, &stack.kind, stack.amount)
                .map_err(|reason| {
                    invalid(format!("factory recycle material add failed: {reason}"))
                })?;
        }
        if ledger_id == &world_id {
            materials = ledgers[&world_id].clone();
        }
        let agent = state.agents.get(operator).cloned().map(|mut cell| {
            cell.last_active = now;
            (operator.to_string(), cell)
        });
        let mut progress = state.industry_progress.clone();
        refresh_progress_without_factory(state, &mut progress, now, factory_id);
        let pending_recipe_deletions = state
            .pending_recipe_jobs
            .iter()
            .filter(|(_, job)| job.factory_id == factory_id)
            .map(|(id, _)| *id)
            .collect();
        Ok(Self {
            event: event.clone(),
            pending: None,
            factory: Some((factory_id.to_string(), None)),
            settled_id: None,
            retired_insert: Some(factory_id.to_string()),
            pending_recipe_deletions,
            ledgers,
            materials,
            agent,
            progress: Some(progress),
        })
    }

    pub(crate) fn matches(&self, event: &DomainEvent) -> bool {
        &self.event == event
    }

    pub(crate) fn install(self, state: &mut WorldState) {
        state.material_ledgers.extend(self.ledgers);
        state.materials = self.materials;
        if let Some((id, pending)) = self.pending {
            match pending {
                Some(job) => {
                    state.pending_factory_builds.insert(id, job);
                }
                None => {
                    state.pending_factory_builds.remove(&id);
                }
            }
        }
        if let Some((id, factory)) = self.factory {
            match factory {
                Some(value) => {
                    state.factories.insert(id, value);
                }
                None => {
                    state.factories.remove(&id);
                }
            }
        }
        if let Some(id) = self.settled_id {
            state.settled_factory_build_ids.insert(id);
        }
        if let Some(id) = self.retired_insert {
            state.retired_factory_ids.insert(id);
        }
        for id in self.pending_recipe_deletions {
            state.pending_recipe_jobs.remove(&id);
        }
        if let Some((id, agent)) = self.agent {
            state.agents.insert(id, agent);
        }
        if let Some(progress) = self.progress {
            state.industry_progress = progress;
        }
    }

    pub(super) fn serialize_agents<S: SerializeStruct>(
        &self,
        state: &WorldState,
        out: &mut S,
    ) -> Result<(), S::Error> {
        serialize_agents(&self.event, self.agent.as_ref(), state, out)
    }

    pub(super) fn serialize_materials<S: SerializeStruct>(
        &self,
        state: &WorldState,
        out: &mut S,
    ) -> Result<(), S::Error> {
        out.serialize_field("materials", &self.materials)?;
        out.serialize_field(
            "material_ledgers",
            &SparseOverlay {
                base: &state.material_ledgers,
                updates: &self.ledgers,
            },
        )
    }

    pub(super) fn serialize_factory_fields<S: SerializeStruct>(
        &self,
        state: &WorldState,
        out: &mut S,
    ) -> Result<(), S::Error> {
        let factories = self.factory.clone().into_iter().collect();
        out.serialize_field(
            "factories",
            &SparseOptionalOverlay {
                base: &state.factories,
                changes: &factories,
            },
        )?;
        out.serialize_field(
            "retired_factory_ids",
            &SetInsertOneOverlay {
                base: &state.retired_factory_ids,
                insert: self.retired_insert.as_ref(),
            },
        )?;
        out.serialize_field(
            "settled_factory_build_ids",
            &SetInsertOneOverlay {
                base: &state.settled_factory_build_ids,
                insert: self.settled_id.as_ref(),
            },
        )?;
        let pending = self.pending.clone().into_iter().collect();
        out.serialize_field(
            "pending_factory_builds",
            &SparseOptionalOverlay {
                base: &state.pending_factory_builds,
                changes: &pending,
            },
        )?;
        let recipe_deletions = self
            .pending_recipe_deletions
            .iter()
            .map(|id| (*id, None))
            .collect();
        out.serialize_field(
            "pending_recipe_jobs",
            &SparseOptionalOverlay {
                base: &state.pending_recipe_jobs,
                changes: &recipe_deletions,
            },
        )?;
        out.serialize_field("settled_recipe_job_ids", &state.settled_recipe_job_ids)
    }

    pub(super) fn serialize_transit_and_progress<S: SerializeStruct>(
        &self,
        state: &WorldState,
        out: &mut S,
    ) -> Result<(), S::Error> {
        out.serialize_field(
            "pending_material_transits",
            &state.pending_material_transits,
        )?;
        out.serialize_field(
            "industry_progress",
            self.progress.as_ref().unwrap_or(&state.industry_progress),
        )
    }
}
