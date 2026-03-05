use super::super::state::{
    ModuleArtifactBidState, ModuleArtifactListingState, ModuleReleaseRequestStatus,
};
use super::super::{
    Action, ActionEnvelope, ActionId, CausedBy, DomainEvent, ModuleActivation, ModuleChangeSet,
    ModuleProfileChanges, ModuleUpgrade, ProposalDecision, ProposalId, RejectReason, WorldError,
    WorldEventBody,
};
use super::event_processing::action_to_event_economy::ensure_profile_field_whitelist;
use super::World;
use crate::simulator::{ModuleInstallTarget, ResourceKind};
use std::collections::BTreeSet;

const MODULE_DEPLOY_FEE_BYTES_PER_ELECTRICITY: i64 = 2_048;
const MODULE_COMPILE_FEE_BYTES_PER_ELECTRICITY: i64 = 1_024;
const MODULE_LIST_FEE_AMOUNT: i64 = 1;
const MODULE_DELIST_FEE_AMOUNT: i64 = 1;
const MODULE_DESTROY_FEE_AMOUNT: i64 = 1;
const MODULE_RELEASE_DEFAULT_REQUIRED_ROLES: &[&str] = &["security", "economy", "runtime"];

include!("module_actions_impl_part1.rs");
include!("module_actions_impl_part2.rs");
