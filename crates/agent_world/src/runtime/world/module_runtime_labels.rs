use super::super::{
    Action, DomainEvent, ModuleKind, ModuleRole, ModuleSubscriptionStage, WorldEventBody,
};

pub(super) fn event_kind_label(body: &WorldEventBody) -> &'static str {
    match body {
        WorldEventBody::Domain(event) => match event {
            DomainEvent::AgentRegistered { .. } => "domain.agent_registered",
            DomainEvent::AgentMoved { .. } => "domain.agent_moved",
            DomainEvent::ActionRejected { .. } => "domain.action_rejected",
            DomainEvent::Observation { .. } => "domain.observation",
            DomainEvent::BodyAttributesUpdated { .. } => "domain.body_attributes_updated",
            DomainEvent::BodyAttributesRejected { .. } => "domain.body_attributes_rejected",
            DomainEvent::BodyInterfaceExpanded { .. } => "domain.body_interface_expanded",
            DomainEvent::BodyInterfaceExpandRejected { .. } => {
                "domain.body_interface_expand_rejected"
            }
            DomainEvent::ModuleArtifactDeployed { .. } => "domain.module_artifact_deployed",
            DomainEvent::ModuleInstalled { .. } => "domain.module_installed",
            DomainEvent::ModuleUpgraded { .. } => "domain.module_upgraded",
            DomainEvent::ModuleArtifactListed { .. } => "domain.module_artifact_listed",
            DomainEvent::ModuleArtifactDelisted { .. } => "domain.module_artifact_delisted",
            DomainEvent::ModuleArtifactDestroyed { .. } => "domain.module_artifact_destroyed",
            DomainEvent::ModuleArtifactBidPlaced { .. } => "domain.module_artifact_bid_placed",
            DomainEvent::ModuleArtifactBidCancelled { .. } => {
                "domain.module_artifact_bid_cancelled"
            }
            DomainEvent::ModuleArtifactSaleCompleted { .. } => "domain.module_artifact_sold",
            DomainEvent::ResourceTransferred { .. } => "domain.resource_transferred",
            DomainEvent::PowerRedeemed { .. } => "domain.power_redeemed",
            DomainEvent::PowerRedeemRejected { .. } => "domain.power_redeem_rejected",
            DomainEvent::NodePointsSettlementApplied { .. } => {
                "domain.reward.node_points_settlement_applied"
            }
            DomainEvent::MaterialTransferred { .. } => "domain.material_transferred",
            DomainEvent::MaterialTransitStarted { .. } => "domain.material_transit_started",
            DomainEvent::MaterialTransitCompleted { .. } => "domain.material_transit_completed",
            DomainEvent::FactoryBuildStarted { .. } => "domain.economy.factory_build_started",
            DomainEvent::FactoryBuilt { .. } => "domain.economy.factory_built",
            DomainEvent::RecipeStarted { .. } => "domain.economy.recipe_started",
            DomainEvent::RecipeCompleted { .. } => "domain.economy.recipe_completed",
            DomainEvent::GameplayPolicyUpdated { .. } => "domain.gameplay.policy_updated",
            DomainEvent::EconomicContractOpened { .. } => {
                "domain.gameplay.economic_contract_opened"
            }
            DomainEvent::EconomicContractAccepted { .. } => {
                "domain.gameplay.economic_contract_accepted"
            }
            DomainEvent::EconomicContractSettled { .. } => {
                "domain.gameplay.economic_contract_settled"
            }
            DomainEvent::EconomicContractExpired { .. } => {
                "domain.gameplay.economic_contract_expired"
            }
            DomainEvent::AllianceFormed { .. } => "domain.gameplay.alliance_formed",
            DomainEvent::WarDeclared { .. } => "domain.gameplay.war_declared",
            DomainEvent::WarConcluded { .. } => "domain.gameplay.war_concluded",
            DomainEvent::GovernanceProposalOpened { .. } => {
                "domain.gameplay.governance_proposal_opened"
            }
            DomainEvent::GovernanceVoteCast { .. } => "domain.gameplay.governance_vote_cast",
            DomainEvent::GovernanceProposalFinalized { .. } => {
                "domain.gameplay.governance_proposal_finalized"
            }
            DomainEvent::CrisisSpawned { .. } => "domain.gameplay.crisis_spawned",
            DomainEvent::CrisisResolved { .. } => "domain.gameplay.crisis_resolved",
            DomainEvent::CrisisTimedOut { .. } => "domain.gameplay.crisis_timed_out",
            DomainEvent::MetaProgressGranted { .. } => "domain.gameplay.meta_progress_granted",
            DomainEvent::ProductValidated { .. } => "domain.economy.product_validated",
        },
        WorldEventBody::EffectQueued(_) => "effect.queued",
        WorldEventBody::ReceiptAppended(_) => "effect.receipt_appended",
        WorldEventBody::PolicyDecisionRecorded(_) => "policy.decision_recorded",
        WorldEventBody::RuleDecisionRecorded(_) => "rule.decision_recorded",
        WorldEventBody::ActionOverridden(_) => "rule.action_overridden",
        WorldEventBody::Governance(_) => "governance",
        WorldEventBody::ModuleEvent(_) => "module.event",
        WorldEventBody::ModuleCallFailed(_) => "module.call_failed",
        WorldEventBody::ModuleEmitted(_) => "module.emitted",
        WorldEventBody::ModuleStateUpdated(_) => "module.state_updated",
        WorldEventBody::ModuleRuntimeCharged(_) => "module.runtime_charged",
        WorldEventBody::SnapshotCreated(_) => "snapshot.created",
        WorldEventBody::ManifestUpdated(_) => "manifest.updated",
        WorldEventBody::RollbackApplied(_) => "rollback.applied",
    }
}

pub(super) fn action_kind_label(action: &Action) -> &'static str {
    match action {
        Action::RegisterAgent { .. } => "action.register_agent",
        Action::MoveAgent { .. } => "action.move_agent",
        Action::QueryObservation { .. } => "action.query_observation",
        Action::EmitObservation { .. } => "action.emit_observation",
        Action::BodyAction { .. } => "action.body_action",
        Action::EmitBodyAttributes { .. } => "action.emit_body_attributes",
        Action::ExpandBodyInterface { .. } => "action.expand_body_interface",
        Action::DeployModuleArtifact { .. } => "action.module.deploy_artifact",
        Action::CompileModuleArtifactFromSource { .. } => {
            "action.module.compile_artifact_from_source"
        }
        Action::InstallModuleFromArtifact { .. } => "action.module.install_from_artifact",
        Action::InstallModuleToTargetFromArtifact { .. } => {
            "action.module.install_to_target_from_artifact"
        }
        Action::UpgradeModuleFromArtifact { .. } => "action.module.upgrade_from_artifact",
        Action::ListModuleArtifactForSale { .. } => "action.module.list_artifact_for_sale",
        Action::BuyModuleArtifact { .. } => "action.module.buy_artifact",
        Action::DelistModuleArtifact { .. } => "action.module.delist_artifact",
        Action::DestroyModuleArtifact { .. } => "action.module.destroy_artifact",
        Action::PlaceModuleArtifactBid { .. } => "action.module.place_artifact_bid",
        Action::CancelModuleArtifactBid { .. } => "action.module.cancel_artifact_bid",
        Action::TransferResource { .. } => "action.transfer_resource",
        Action::RedeemPower { .. } => "action.redeem_power",
        Action::RedeemPowerSigned { .. } => "action.redeem_power_signed",
        Action::ApplyNodePointsSettlementSigned { .. } => {
            "action.reward.apply_node_points_settlement_signed"
        }
        Action::TransferMaterial { .. } => "action.transfer_material",
        Action::FormAlliance { .. } => "action.gameplay.form_alliance",
        Action::DeclareWar { .. } => "action.gameplay.declare_war",
        Action::OpenGovernanceProposal { .. } => "action.gameplay.open_governance_proposal",
        Action::CastGovernanceVote { .. } => "action.gameplay.cast_governance_vote",
        Action::ResolveCrisis { .. } => "action.gameplay.resolve_crisis",
        Action::GrantMetaProgress { .. } => "action.gameplay.grant_meta_progress",
        Action::UpdateGameplayPolicy { .. } => "action.gameplay.update_policy",
        Action::OpenEconomicContract { .. } => "action.gameplay.open_economic_contract",
        Action::AcceptEconomicContract { .. } => "action.gameplay.accept_economic_contract",
        Action::SettleEconomicContract { .. } => "action.gameplay.settle_economic_contract",
        Action::EmitResourceTransfer { .. } => "action.emit_resource_transfer",
        Action::BuildFactory { .. } => "action.economy.build_factory",
        Action::BuildFactoryWithModule { .. } => "action.economy.build_factory_with_module",
        Action::ScheduleRecipe { .. } => "action.economy.schedule_recipe",
        Action::ScheduleRecipeWithModule { .. } => "action.economy.schedule_recipe_with_module",
        Action::ValidateProduct { .. } => "action.economy.validate_product",
        Action::ValidateProductWithModule { .. } => "action.economy.validate_product_with_module",
    }
}

pub(super) fn subscription_stage_label(stage: ModuleSubscriptionStage) -> &'static str {
    match stage {
        ModuleSubscriptionStage::PreAction => "pre_action",
        ModuleSubscriptionStage::PostAction => "post_action",
        ModuleSubscriptionStage::PostEvent => "post_event",
        ModuleSubscriptionStage::Tick => "tick",
    }
}

pub(super) fn module_kind_label(kind: &ModuleKind) -> &'static str {
    match kind {
        ModuleKind::Reducer => "reducer",
        ModuleKind::Pure => "pure",
    }
}

pub(super) fn module_role_label(role: &ModuleRole) -> &'static str {
    match role {
        ModuleRole::Rule => "rule",
        ModuleRole::Domain => "domain",
        ModuleRole::Gameplay => "gameplay",
        ModuleRole::Body => "body",
        ModuleRole::AgentInternal => "agent_internal",
    }
}
