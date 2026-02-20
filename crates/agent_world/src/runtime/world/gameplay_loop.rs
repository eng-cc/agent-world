use super::super::{
    CrisisStatus, DomainEvent, GovernanceProposalStatus, WorldError, WorldEvent, WorldEventBody,
};
use super::World;
use std::cmp::Ordering;

const CRISIS_AUTO_INTERVAL_TICKS: u64 = 8;
const CRISIS_DEFAULT_DURATION_TICKS: u64 = 6;
const CRISIS_TIMEOUT_PENALTY_PER_SEVERITY: i64 = 10;
const WAR_SCORE_PER_MEMBER: i64 = 10;

impl World {
    pub(super) fn process_gameplay_cycles(&mut self) -> Result<Vec<WorldEvent>, WorldError> {
        let mut emitted = Vec::new();
        self.finalize_due_governance_proposals(&mut emitted)?;
        self.process_crisis_lifecycle(&mut emitted)?;
        self.process_war_lifecycle(&mut emitted)?;
        Ok(emitted)
    }

    fn finalize_due_governance_proposals(
        &mut self,
        emitted: &mut Vec<WorldEvent>,
    ) -> Result<(), WorldError> {
        let now = self.state.time;
        let mut due_keys: Vec<_> = self
            .state
            .governance_proposals
            .iter()
            .filter(|(_, proposal)| {
                proposal.status == GovernanceProposalStatus::Open && proposal.closes_at <= now
            })
            .map(|(key, _)| key.clone())
            .collect();
        due_keys.sort();

        for proposal_key in due_keys {
            let Some(proposal) = self.state.governance_proposals.get(&proposal_key).cloned() else {
                continue;
            };
            let vote_state = self.state.governance_votes.get(&proposal_key);
            let total_weight = vote_state.map(|value| value.total_weight).unwrap_or(0);
            let (winning_option, winning_weight) = vote_state
                .and_then(|value| {
                    value
                        .tallies
                        .iter()
                        .max_by(|(left_option, left_weight), (right_option, right_weight)| {
                            left_weight
                                .cmp(right_weight)
                                .then_with(|| right_option.cmp(left_option))
                        })
                        .map(|(option, weight)| (Some(option.clone()), *weight))
                })
                .unwrap_or((None, 0));
            let reached_quorum = total_weight >= proposal.quorum_weight;
            let reached_threshold = if total_weight == 0 {
                false
            } else {
                (u128::from(winning_weight) * 10_000_u128)
                    >= (u128::from(total_weight) * u128::from(proposal.pass_threshold_bps))
            };
            let passed = reached_quorum && reached_threshold && winning_option.is_some();
            self.append_gameplay_domain_event(
                DomainEvent::GovernanceProposalFinalized {
                    proposal_key: proposal_key.clone(),
                    winning_option,
                    winning_weight,
                    total_weight,
                    passed,
                },
                emitted,
            )?;
        }
        Ok(())
    }

    fn process_crisis_lifecycle(
        &mut self,
        emitted: &mut Vec<WorldEvent>,
    ) -> Result<(), WorldError> {
        let now = self.state.time;
        let has_active_crisis = self
            .state
            .crises
            .values()
            .any(|crisis| crisis.status == CrisisStatus::Active);
        if !has_active_crisis && now > 0 && now % CRISIS_AUTO_INTERVAL_TICKS == 0 {
            let sequence = now / CRISIS_AUTO_INTERVAL_TICKS;
            let severity = ((sequence % 3) + 1) as u32;
            let kind = match severity {
                1 => "supply_shock",
                2 => "solar_storm",
                _ => "network_outage",
            }
            .to_string();
            let crisis_id = format!("crisis.auto.{now}");
            let expires_at = now
                .saturating_add(CRISIS_DEFAULT_DURATION_TICKS)
                .saturating_add(u64::from(severity));
            self.append_gameplay_domain_event(
                DomainEvent::CrisisSpawned {
                    crisis_id,
                    kind,
                    severity,
                    expires_at,
                },
                emitted,
            )?;
        }

        let mut due_timeouts: Vec<_> = self
            .state
            .crises
            .iter()
            .filter(|(_, crisis)| crisis.status == CrisisStatus::Active && crisis.expires_at <= now)
            .map(|(crisis_id, crisis)| (crisis_id.clone(), crisis.severity.max(1)))
            .collect();
        due_timeouts.sort_by(|left, right| left.0.cmp(&right.0));
        for (crisis_id, severity) in due_timeouts {
            let penalty_impact =
                -i64::from(severity).saturating_mul(CRISIS_TIMEOUT_PENALTY_PER_SEVERITY);
            self.append_gameplay_domain_event(
                DomainEvent::CrisisTimedOut {
                    crisis_id,
                    penalty_impact,
                },
                emitted,
            )?;
        }
        Ok(())
    }

    fn process_war_lifecycle(&mut self, emitted: &mut Vec<WorldEvent>) -> Result<(), WorldError> {
        let now = self.state.time;
        let mut due_wars = self
            .state
            .wars
            .values()
            .filter(|war| {
                war.active
                    && now
                        >= war
                            .declared_at
                            .saturating_add(war.max_duration_ticks.max(1))
            })
            .cloned()
            .collect::<Vec<_>>();
        due_wars.sort_by(|left, right| left.war_id.cmp(&right.war_id));

        for war in due_wars {
            let aggressor_members = self
                .state
                .alliances
                .get(&war.aggressor_alliance_id)
                .map(|alliance| alliance.members.len() as i64)
                .unwrap_or(0);
            let defender_members = self
                .state
                .alliances
                .get(&war.defender_alliance_id)
                .map(|alliance| alliance.members.len() as i64)
                .unwrap_or(0);
            let aggressor_score = aggressor_members
                .saturating_mul(WAR_SCORE_PER_MEMBER)
                .saturating_add(i64::from(war.intensity));
            let defender_score = defender_members.saturating_mul(WAR_SCORE_PER_MEMBER);
            let winner_alliance_id = match aggressor_score.cmp(&defender_score) {
                Ordering::Greater | Ordering::Equal => war.aggressor_alliance_id.clone(),
                Ordering::Less => war.defender_alliance_id.clone(),
            };
            let summary = format!(
                "auto settlement: aggressor_score={} defender_score={}",
                aggressor_score, defender_score
            );
            self.append_gameplay_domain_event(
                DomainEvent::WarConcluded {
                    war_id: war.war_id,
                    winner_alliance_id,
                    aggressor_score,
                    defender_score,
                    summary,
                },
                emitted,
            )?;
        }
        Ok(())
    }

    fn append_gameplay_domain_event(
        &mut self,
        event: DomainEvent,
        emitted: &mut Vec<WorldEvent>,
    ) -> Result<(), WorldError> {
        self.append_event(WorldEventBody::Domain(event), None)?;
        if let Some(event) = self.journal.events.last() {
            emitted.push(event.clone());
        }
        Ok(())
    }
}
