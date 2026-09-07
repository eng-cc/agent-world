use super::super::events::MainTokenFeeKind;
use super::super::main_token::{
    MAIN_TOKEN_TREASURY_BUCKET_ECOSYSTEM_POOL, MAIN_TOKEN_TREASURY_BUCKET_GAS_FEE,
    MAIN_TOKEN_TREASURY_BUCKET_MODULE_FEE, MAIN_TOKEN_TREASURY_BUCKET_NODE_SERVICE_REWARD,
    MAIN_TOKEN_TREASURY_BUCKET_RESTRICTED_STARTER_CLAIM_LIVEOPS_POOL,
    MAIN_TOKEN_TREASURY_BUCKET_SECURITY_RESERVE, MAIN_TOKEN_TREASURY_BUCKET_SLASH,
    MAIN_TOKEN_TREASURY_BUCKET_STAKING_REWARD, RestrictedStarterClaimGrantState,
    RestrictedStarterClaimGrantStatus, RestrictedStarterClaimLiveopsPoolTopUpRecord,
};
use super::*;

#[path = "apply_domain_event_main_token_economy.rs"]
mod economy;
#[path = "apply_domain_event_main_token_genesis.rs"]
mod genesis;
#[path = "apply_domain_event_main_token_helpers.rs"]
pub(crate) mod helpers;
#[path = "apply_domain_event_main_token_restricted_claims.rs"]
mod restricted_claims;

impl WorldState {
    pub(super) fn apply_domain_event_main_token(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        if matches!(
            event,
            DomainEvent::MainTokenGenesisInitialized { .. }
                | DomainEvent::MainTokenVestingClaimed { .. }
                | DomainEvent::MainTokenTransferred { .. }
                | DomainEvent::MainTokenEpochIssued { .. }
                | DomainEvent::MainTokenFeeSettled { .. }
        ) {
            crate::runtime::world::main_token_monetary_publication::PreparedMainTokenMonetaryEvent::prepare(self, event, now)?.install_infallible(self);
            return Ok(());
        }
        if matches!(
            event,
            DomainEvent::MainTokenPolicyUpdateScheduled { .. }
                | DomainEvent::MainTokenTreasuryDistributed { .. }
        ) {
            crate::runtime::world::main_token_governance_monetary_publication::PreparedMainTokenGovernanceMonetaryEvent::prepare(self, event, now)?.install_infallible(self);
            return Ok(());
        }
        if matches!(
            event,
            DomainEvent::RestrictedStarterClaimLiveopsPoolToppedUp { .. }
                | DomainEvent::RestrictedStarterClaimGrantIssued { .. }
                | DomainEvent::RestrictedStarterClaimGrantExpired { .. }
                | DomainEvent::RestrictedStarterClaimGrantRevoked { .. }
        ) {
            crate::runtime::world::main_token_restricted_claim_publication::PreparedMainTokenRestrictedClaimEvent::prepare(self, event)?.install_infallible(self);
            return Ok(());
        }
        unreachable!("apply_domain_event_main_token received unsupported event variant")
    }
}
