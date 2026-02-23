use super::super::state::{ModuleArtifactBidState, ModuleArtifactListingState};
use super::super::{
    Action, ActionEnvelope, CausedBy, DomainEvent, ModuleActivation, ModuleChangeSet,
    ModuleUpgrade, ProposalDecision, RejectReason, WorldError, WorldEventBody,
};
use super::World;
use crate::simulator::{ModuleInstallTarget, ResourceKind};

const MODULE_DEPLOY_FEE_BYTES_PER_ELECTRICITY: i64 = 2_048;
const MODULE_COMPILE_FEE_BYTES_PER_ELECTRICITY: i64 = 1_024;
const MODULE_LIST_FEE_AMOUNT: i64 = 1;
const MODULE_DELIST_FEE_AMOUNT: i64 = 1;
const MODULE_DESTROY_FEE_AMOUNT: i64 = 1;

include!("module_actions_impl_part1.rs");
include!("module_actions_impl_part2.rs");
