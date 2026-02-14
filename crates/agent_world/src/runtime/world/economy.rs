use super::super::{DomainEvent, WorldError, WorldEvent, WorldEventBody};
use super::World;

impl World {
    // ---------------------------------------------------------------------
    // Economy runtime helpers
    // ---------------------------------------------------------------------

    pub fn pending_factory_builds_len(&self) -> usize {
        self.state.pending_factory_builds.len()
    }

    pub fn pending_recipe_jobs_len(&self) -> usize {
        self.state.pending_recipe_jobs.len()
    }

    pub fn has_factory(&self, factory_id: &str) -> bool {
        self.state.factories.contains_key(factory_id)
    }

    pub(super) fn process_due_economy_jobs(&mut self) -> Result<Vec<WorldEvent>, WorldError> {
        let now = self.state.time;
        let mut emitted = Vec::new();

        let mut due_builds: Vec<_> = self
            .state
            .pending_factory_builds
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_builds.sort_by_key(|job| (job.ready_at, job.job_id));

        for job in due_builds {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::FactoryBuilt {
                    job_id: job.job_id,
                    builder_agent_id: job.builder_agent_id,
                    site_id: job.site_id,
                    spec: job.spec,
                }),
                None,
            )?;
            if let Some(event) = self.journal.events.last() {
                emitted.push(event.clone());
            }
        }

        let mut due_recipes: Vec<_> = self
            .state
            .pending_recipe_jobs
            .values()
            .filter(|job| job.ready_at <= now)
            .cloned()
            .collect();
        due_recipes.sort_by_key(|job| (job.ready_at, job.job_id));

        for job in due_recipes {
            self.append_event(
                WorldEventBody::Domain(DomainEvent::RecipeCompleted {
                    job_id: job.job_id,
                    requester_agent_id: job.requester_agent_id,
                    factory_id: job.factory_id,
                    recipe_id: job.recipe_id,
                    accepted_batches: job.accepted_batches,
                    produce: job.produce,
                    byproducts: job.byproducts,
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
