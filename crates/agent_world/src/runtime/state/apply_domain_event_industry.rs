use super::*;

impl WorldState {
    pub(super) fn apply_domain_event_industry(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        match event {
            DomainEvent::MaterialTransferred {
                requester_agent_id,
                from_ledger,
                to_ledger,
                kind,
                amount,
                ..
            } => {
                remove_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    from_ledger,
                    kind.as_str(),
                    *amount,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("material transfer remove failed: {reason}"),
                })?;
                add_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    to_ledger,
                    kind.as_str(),
                    *amount,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("material transfer add failed: {reason}"),
                })?;
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::MaterialTransitStarted {
                job_id,
                requester_agent_id,
                from_ledger,
                to_ledger,
                kind,
                amount,
                distance_km,
                loss_bps,
                priority,
                ready_at,
            } => {
                remove_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    from_ledger,
                    kind.as_str(),
                    *amount,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("material transit reserve failed: {reason}"),
                })?;
                self.pending_material_transits.insert(
                    *job_id,
                    MaterialTransitJobState {
                        job_id: *job_id,
                        requester_agent_id: requester_agent_id.clone(),
                        from_ledger: from_ledger.clone(),
                        to_ledger: to_ledger.clone(),
                        kind: kind.clone(),
                        amount: *amount,
                        distance_km: *distance_km,
                        loss_bps: *loss_bps,
                        priority: *priority,
                        ready_at: *ready_at,
                    },
                );
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::MaterialTransitCompleted {
                job_id,
                requester_agent_id,
                to_ledger,
                kind,
                received_amount,
                ..
            } => {
                self.pending_material_transits.remove(job_id);
                if *received_amount > 0 {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        to_ledger,
                        kind.as_str(),
                        *received_amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("material transit completion failed: {reason}"),
                    })?;
                }
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::FactoryBuildStarted {
                job_id,
                builder_agent_id,
                site_id,
                spec,
                consume_ledger,
                ready_at,
            } => {
                for stack in &spec.build_cost {
                    remove_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        consume_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("factory build consume failed: {reason}"),
                    })?;
                }
                self.pending_factory_builds.insert(
                    *job_id,
                    FactoryBuildJobState {
                        job_id: *job_id,
                        builder_agent_id: builder_agent_id.clone(),
                        site_id: site_id.clone(),
                        spec: spec.clone(),
                        consume_ledger: consume_ledger.clone(),
                        ready_at: *ready_at,
                    },
                );
                if let Some(cell) = self.agents.get_mut(builder_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::FactoryBuilt {
                job_id,
                builder_agent_id,
                site_id,
                spec,
            } => {
                self.pending_factory_builds.remove(job_id);
                let site_ledger = MaterialLedgerId::site(site_id.clone());
                self.factories.insert(
                    spec.factory_id.clone(),
                    FactoryState {
                        factory_id: spec.factory_id.clone(),
                        site_id: site_id.clone(),
                        builder_agent_id: builder_agent_id.clone(),
                        spec: spec.clone(),
                        input_ledger: site_ledger.clone(),
                        output_ledger: site_ledger,
                        durability_ppm: 1_000_000,
                        built_at: now,
                    },
                );
                if let Some(cell) = self.agents.get_mut(builder_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::FactoryDurabilityChanged {
                factory_id,
                durability_ppm,
                ..
            } => {
                if let Some(factory) = self.factories.get_mut(factory_id) {
                    factory.durability_ppm = (*durability_ppm).clamp(0, 1_000_000);
                }
            }
            DomainEvent::FactoryMaintained {
                operator_agent_id,
                factory_id,
                consume_ledger,
                consumed_parts,
                durability_ppm,
            } => {
                remove_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    consume_ledger,
                    "hardware_part",
                    *consumed_parts,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("factory maintenance consume failed: {reason}"),
                })?;
                if let Some(factory) = self.factories.get_mut(factory_id) {
                    factory.durability_ppm = (*durability_ppm).clamp(0, 1_000_000);
                }
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::FactoryRecycled {
                operator_agent_id,
                factory_id,
                recycle_ledger,
                recovered,
                ..
            } => {
                self.factories.remove(factory_id);
                self.pending_recipe_jobs
                    .retain(|_, job| job.factory_id != *factory_id);
                for stack in recovered {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        recycle_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("factory recycle material add failed: {reason}"),
                    })?;
                }
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::RecipeStarted {
                job_id,
                requester_agent_id,
                factory_id,
                recipe_id,
                accepted_batches,
                consume,
                produce,
                byproducts,
                power_required,
                duration_ticks,
                consume_ledger,
                output_ledger,
                bottleneck_tags,
                ready_at,
            } => {
                for stack in consume {
                    remove_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        consume_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("recipe consume failed: {reason}"),
                    })?;
                }
                remove_resource_balance(
                    &mut self.resources,
                    ResourceKind::Electricity,
                    *power_required,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("recipe power consume failed: {reason}"),
                })?;
                self.pending_recipe_jobs.insert(
                    *job_id,
                    RecipeJobState {
                        job_id: *job_id,
                        requester_agent_id: requester_agent_id.clone(),
                        factory_id: factory_id.clone(),
                        recipe_id: recipe_id.clone(),
                        accepted_batches: *accepted_batches,
                        consume: consume.clone(),
                        produce: produce.clone(),
                        byproducts: byproducts.clone(),
                        power_required: *power_required,
                        duration_ticks: *duration_ticks,
                        consume_ledger: consume_ledger.clone(),
                        output_ledger: output_ledger.clone(),
                        bottleneck_tags: bottleneck_tags.clone(),
                        ready_at: *ready_at,
                    },
                );
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::RecipeCompleted {
                job_id,
                requester_agent_id,
                produce,
                byproducts,
                output_ledger,
                ..
            } => {
                self.pending_recipe_jobs.remove(job_id);
                for stack in produce {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        output_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("recipe produce failed: {reason}"),
                    })?;
                }
                for stack in byproducts {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        output_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("recipe byproduct failed: {reason}"),
                    })?;
                }
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            _ => unreachable!("apply_domain_event_industry received unsupported event variant"),
        }
        Ok(())
    }
}
