use super::World;
use super::super::{CausedBy, DomainEvent, WorldError, WorldEventBody, WorldEventId};
use crate::models::BodyKernelView;

impl World {
    // ---------------------------------------------------------------------
    // Body module helpers
    // ---------------------------------------------------------------------

    pub fn record_body_attributes_update(
        &mut self,
        agent_id: impl Into<String>,
        view: BodyKernelView,
        reason: impl Into<String>,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        let agent_id = agent_id.into();
        let reason = reason.into();
        self.append_event(
            WorldEventBody::Domain(DomainEvent::BodyAttributesUpdated {
                agent_id,
                view,
                reason,
            }),
            caused_by,
        )
    }

    pub fn record_body_attributes_reject(
        &mut self,
        agent_id: impl Into<String>,
        reason: impl Into<String>,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        let agent_id = agent_id.into();
        let reason = reason.into();
        self.append_event(
            WorldEventBody::Domain(DomainEvent::BodyAttributesRejected { agent_id, reason }),
            caused_by,
        )
    }
}
