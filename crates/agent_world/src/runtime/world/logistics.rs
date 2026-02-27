use super::super::{DomainEvent, WorldError, WorldEvent, WorldEventBody};
use super::World;

pub(super) const MATERIAL_TRANSFER_MAX_DISTANCE_KM: i64 = 10_000;
pub(super) const MATERIAL_TRANSFER_LOSS_PER_KM_BPS: i64 = 5;
pub(super) const MATERIAL_TRANSFER_SPEED_KM_PER_TICK: i64 = 100;
pub(super) const MATERIAL_TRANSFER_MAX_INFLIGHT: usize = 2;

impl World {
    pub fn pending_material_transits_len(&self) -> usize {
        self.state.pending_material_transits.len()
    }

    pub(super) fn process_due_material_transits(&mut self) -> Result<Vec<WorldEvent>, WorldError> {
        let now = self.state.time;
        let mut emitted = Vec::new();

        let mut due_jobs: Vec<_> = self
            .state
            .pending_material_transits
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_jobs.sort_by_key(|job| (job.ready_at, job.priority, job.job_id));

        for job in due_jobs {
            let loss_amount = ((job.amount as i128)
                .saturating_mul(job.distance_km as i128)
                .saturating_mul(job.loss_bps as i128)
                / 10_000)
                .clamp(0, job.amount as i128) as i64;
            let received_amount = job.amount.saturating_sub(loss_amount);
            self.record_logistics_sla_completion(job.ready_at, now, job.priority);

            self.append_event(
                WorldEventBody::Domain(DomainEvent::MaterialTransitCompleted {
                    job_id: job.job_id,
                    requester_agent_id: job.requester_agent_id,
                    from_ledger: job.from_ledger,
                    to_ledger: job.to_ledger,
                    kind: job.kind,
                    sent_amount: job.amount,
                    received_amount,
                    loss_amount,
                    distance_km: job.distance_km,
                    priority: job.priority,
                }),
                None,
            )?;
            if let Some(event) = self.journal.events.last() {
                emitted.push(event.clone());
            }
        }

        Ok(emitted)
    }
}
