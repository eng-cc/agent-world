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
            DomainEvent::ModuleArtifactListed { .. } => "domain.module_artifact_listed",
            DomainEvent::ModuleArtifactDelisted { .. } => "domain.module_artifact_delisted",
            DomainEvent::ModuleArtifactDestroyed { .. } => "domain.module_artifact_destroyed",
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
        Action::InstallModuleFromArtifact { .. } => "action.module.install_from_artifact",
        Action::ListModuleArtifactForSale { .. } => "action.module.list_artifact_for_sale",
        Action::BuyModuleArtifact { .. } => "action.module.buy_artifact",
        Action::DelistModuleArtifact { .. } => "action.module.delist_artifact",
        Action::DestroyModuleArtifact { .. } => "action.module.destroy_artifact",
        Action::TransferResource { .. } => "action.transfer_resource",
        Action::RedeemPower { .. } => "action.redeem_power",
        Action::RedeemPowerSigned { .. } => "action.redeem_power_signed",
        Action::ApplyNodePointsSettlementSigned { .. } => {
            "action.reward.apply_node_points_settlement_signed"
        }
        Action::TransferMaterial { .. } => "action.transfer_material",
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
        ModuleRole::Body => "body",
        ModuleRole::AgentInternal => "agent_internal",
    }
}
