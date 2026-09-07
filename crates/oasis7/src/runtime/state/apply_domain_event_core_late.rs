use super::*;

impl WorldState {
    pub(super) fn apply_domain_event_core_late(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        match event {
            DomainEvent::ModuleArtifactListed { .. }
            | DomainEvent::ModuleArtifactDelisted { .. }
            | DomainEvent::ModuleArtifactDestroyed { .. }
            | DomainEvent::ModuleArtifactBidPlaced { .. }
            | DomainEvent::ModuleArtifactBidCancelled { .. }
            | DomainEvent::ModuleArtifactSaleCompleted { .. } => {
                self.prepare_module_marketplace_event(event, now)?
                    .install_infallible(self);
            }
            event @ (DomainEvent::ResourceTransferred { .. }
            | DomainEvent::DataCollected { .. }
            | DomainEvent::DataCollectedAuthenticated { .. }
            | DomainEvent::DataAccessGranted { .. }
            | DomainEvent::DataAccessRevoked { .. }) => {
                crate::runtime::world::economy_data_publication::PreparedEconomyDataEvent::prepare(
                    self, event, now,
                )?
                .install_infallible(self);
            }
            event @ (DomainEvent::PowerRedeemed { .. }
            | DomainEvent::PowerRedeemRejected { .. }) => {
                crate::runtime::world::power_redemption_publication::PreparedPowerRedemptionEvent::prepare(
                    self, event, now,
                )?
                .install_infallible(self);
            }
            event @ DomainEvent::NodePointsSettlementApplied { .. } => {
                crate::runtime::world::node_points_settlement_publication::PreparedNodePointsSettlement::prepare(self, event)?
                    .install_infallible(self);
            }
            event @ DomainEvent::MainTokenGenesisInitialized { .. }
            | event @ DomainEvent::MainTokenVestingClaimed { .. }
            | event @ DomainEvent::MainTokenTransferred { .. }
            | event @ DomainEvent::MainTokenEpochIssued { .. }
            | event @ DomainEvent::MainTokenFeeSettled { .. }
            | event @ DomainEvent::MainTokenPolicyUpdateScheduled { .. }
            | event @ DomainEvent::MainTokenTreasuryDistributed { .. }
            | event @ DomainEvent::RestrictedStarterClaimLiveopsPoolToppedUp { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantIssued { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantExpired { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantRevoked { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::LogisticsRouteRegistered { .. }
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
            | event @ DomainEvent::FactoryProductionPaused { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            DomainEvent::MaterialProfileGoverned {
                operator_agent_id,
                proposal_id,
                profile,
            } => {
                if !self.agents.contains_key(operator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
                if *proposal_id == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "material profile governed proposal_id must be > 0".to_string(),
                    });
                }
                if profile.kind.trim().is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "material profile kind cannot be empty".to_string(),
                    });
                }
                if profile.tier == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("material profile tier must be >= 1: {}", profile.kind),
                    });
                }
                if profile.category.trim().is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "material profile category cannot be empty: {}",
                            profile.kind
                        ),
                    });
                }
                if profile.stack_limit <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "material profile stack_limit must be > 0: {}",
                            profile.kind
                        ),
                    });
                }
                self.material_profiles
                    .insert(profile.kind.clone(), profile.clone());
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ProductProfileGoverned { .. }
            | DomainEvent::RecipeProfileGoverned { .. }
            | DomainEvent::FactoryProfileGoverned { .. } => {
                self.prepare_module_release_event(event, now)?
                    .install_infallible(self);
            }
            _ => unreachable!("apply_domain_event_core_late received unsupported event"),
        }
        Ok(())
    }
}
