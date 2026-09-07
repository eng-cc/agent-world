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
        match event {
            DomainEvent::RestrictedStarterClaimLiveopsPoolToppedUp {
                controller_account_id,
                top_up_id,
                source_treasury_bucket_id,
                target_treasury_bucket_id,
                amount,
                topped_up_at_epoch,
            } => self.apply_restricted_starter_claim_liveops_pool_top_up(
                controller_account_id,
                top_up_id,
                source_treasury_bucket_id,
                target_treasury_bucket_id,
                *amount,
                *topped_up_at_epoch,
            )?,
            DomainEvent::RestrictedStarterClaimGrantIssued {
                issuer_id,
                beneficiary_account_id,
                source_treasury_bucket_id,
                amount,
                issuance_reason,
                spend_scope,
                issued_at_epoch,
                expires_at_epoch,
            } => self.apply_restricted_starter_claim_grant_issued(
                issuer_id,
                beneficiary_account_id,
                source_treasury_bucket_id,
                *amount,
                issuance_reason,
                spend_scope,
                *issued_at_epoch,
                *expires_at_epoch,
            )?,
            DomainEvent::RestrictedStarterClaimGrantExpired {
                beneficiary_account_id,
                issuer_id,
                issuance_reason,
                spend_scope,
                source_treasury_bucket_id,
                issued_amount,
                expired_amount,
                issued_at_epoch,
                expired_at_epoch,
                configured_expires_at_epoch,
            } => self.apply_restricted_starter_claim_grant_expired(
                beneficiary_account_id,
                issuer_id,
                issuance_reason,
                spend_scope,
                source_treasury_bucket_id,
                *issued_amount,
                *expired_amount,
                *issued_at_epoch,
                *expired_at_epoch,
                *configured_expires_at_epoch,
            )?,
            DomainEvent::RestrictedStarterClaimGrantRevoked {
                beneficiary_account_id,
                issuer_id,
                issuance_reason,
                spend_scope,
                source_treasury_bucket_id,
                issued_amount,
                revoked_amount,
                issued_at_epoch,
                revoked_at_epoch,
                configured_expires_at_epoch,
                revoke_reason,
            } => self.apply_restricted_starter_claim_grant_revoked(
                beneficiary_account_id,
                issuer_id,
                issuance_reason,
                spend_scope,
                source_treasury_bucket_id,
                *issued_amount,
                *revoked_amount,
                *issued_at_epoch,
                *revoked_at_epoch,
                *configured_expires_at_epoch,
                revoke_reason,
            )?,
            _ => unreachable!("apply_domain_event_main_token received unsupported event variant"),
        }
        Ok(())
    }
}
