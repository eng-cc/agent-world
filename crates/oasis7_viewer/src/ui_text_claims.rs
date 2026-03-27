use oasis7::runtime::{
    agent_claim_cap_for_tier, agent_claim_quote, agent_claim_reputation_tier, AgentClaimState,
};
use oasis7::simulator::WorldSnapshot;

pub(super) fn extend_agent_details_with_claim_lines(
    agent_id: &str,
    snapshot: &WorldSnapshot,
    lines: &mut Vec<String>,
) {
    let Some(runtime_snapshot) = snapshot.runtime_snapshot.as_ref() else {
        return;
    };

    lines.push("".to_string());
    lines.push("Agent Claim:".to_string());
    let current_epoch = agent_claim_epoch(
        runtime_snapshot.state.time,
        runtime_snapshot.governance_execution_policy.epoch_length_ticks,
    );

    if let Some(claim) = runtime_snapshot.state.agent_claims.get(agent_id) {
        lines.push(format!("- Owner: {}", claim.claim_owner_id));
        lines.push(format!("- Status: {}", claim_status(claim, current_epoch)));
        lines.push(format!(
            "- Bond Locked: {} | Upkeep/Epoch: {}",
            claim.locked_bond_amount, claim.upkeep_per_epoch
        ));
        if let Some(remaining) = claim
            .release_ready_at_epoch
            .map(|epoch| epoch.saturating_sub(current_epoch))
        {
            lines.push(format!("- Release Ready In Epochs: {remaining}"));
        }
        if let Some(remaining) = claim
            .grace_deadline_epoch
            .map(|epoch| epoch.saturating_sub(current_epoch))
        {
            lines.push(format!("- Grace Remaining Epochs: {remaining}"));
        }
        let last_control_epoch = runtime_snapshot
            .state
            .agents
            .get(agent_id)
            .map(|cell| {
                agent_claim_epoch(
                    cell.last_active,
                    runtime_snapshot.governance_execution_policy.epoch_length_ticks,
                )
            })
            .unwrap_or(current_epoch);
        let forced_reclaim_in_epochs = last_control_epoch
            .saturating_add(claim.forced_idle_reclaim_epochs)
            .saturating_sub(current_epoch);
        lines.push(format!(
            "- Forced Reclaim In Epochs: {forced_reclaim_in_epochs}"
        ));
        return;
    }

    lines.push("- Status: unclaimed".to_string());
    let Some(primary_agent_id) = snapshot.model.agents.keys().next() else {
        lines.push("- Quote: unavailable (no primary agent)".to_string());
        return;
    };
    let owned_claim_count = runtime_snapshot
        .state
        .agent_claims
        .values()
        .filter(|claim| claim.claim_owner_id == *primary_agent_id)
        .count();
    let reputation_score = runtime_snapshot
        .state
        .reputation_scores
        .get(primary_agent_id)
        .copied()
        .unwrap_or(0);
    let reputation_tier = agent_claim_reputation_tier(reputation_score);
    let claim_cap = agent_claim_cap_for_tier(reputation_tier);
    let liquid_main_token_balance = runtime_snapshot
        .state
        .main_token_balances
        .get(primary_agent_id)
        .map(|balance| balance.liquid_balance)
        .unwrap_or(0);

    match agent_claim_quote(reputation_score, owned_claim_count) {
        Ok(quote) => {
            let upfront = quote
                .activation_fee_amount
                .saturating_add(quote.claim_bond_amount)
                .saturating_add(quote.upkeep_per_epoch);
            lines.push(format!(
                "- Quote For {}: slot={} tier={} cap={} owned={} upfront={} upkeep={}",
                primary_agent_id,
                quote.slot_index,
                quote.reputation_tier,
                quote.claim_cap,
                owned_claim_count,
                upfront,
                quote.upkeep_per_epoch
            ));
            if liquid_main_token_balance < upfront {
                lines.push(format!(
                    "- Claim Blocker: insufficient_liquid_main_token balance={} required={}",
                    liquid_main_token_balance, upfront
                ));
            }
        }
        Err(reason) => {
            lines.push(format!(
                "- Quote For {}: tier={} cap={} owned={}",
                primary_agent_id, reputation_tier, claim_cap, owned_claim_count
            ));
            lines.push(format!("- Claim Blocker: {reason}"));
        }
    }
}

fn claim_status(claim: &AgentClaimState, current_epoch: u64) -> &'static str {
    if claim.grace_deadline_epoch.is_some() {
        "upkeep_grace"
    } else if let Some(ready_at_epoch) = claim.release_ready_at_epoch {
        if current_epoch >= ready_at_epoch {
            "release_ready"
        } else {
            "release_cooldown"
        }
    } else if claim.idle_warning_emitted_at_epoch.is_some() {
        "idle_reclaim_candidate"
    } else {
        "claimed_active"
    }
}

fn agent_claim_epoch(time: u64, epoch_length_ticks: u64) -> u64 {
    time / epoch_length_ticks.max(1)
}
