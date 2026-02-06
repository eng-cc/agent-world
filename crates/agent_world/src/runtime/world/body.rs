use super::World;
use super::super::{CausedBy, DomainEvent, WorldError, WorldEventBody, WorldEventId};
use crate::models::BodyKernelView;

const BODY_MASS_KG_MIN: u64 = 1;
const BODY_MASS_KG_MAX: u64 = 1_000_000_000;
const BODY_RADIUS_CM_MIN: u64 = 1;
const BODY_RADIUS_CM_MAX: u64 = 1_000_000;
const BODY_THRUST_LIMIT_MIN: u64 = 0;
const BODY_THRUST_LIMIT_MAX: u64 = 10_000_000_000;
const BODY_CROSS_SECTION_CM2_MIN: u64 = 1;
const BODY_CHANGE_FACTOR_MAX: u64 = 10;

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
        let current_view = self
            .state
            .agents
            .get(&agent_id)
            .ok_or_else(|| WorldError::AgentNotFound {
                agent_id: agent_id.clone(),
            })?
            .state
            .body_view
            .clone();
        if let Err(reason) = validate_body_kernel_view(&current_view, &view) {
            return self.record_body_attributes_reject(agent_id, reason, caused_by);
        }
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

fn validate_body_kernel_view(
    current: &BodyKernelView,
    candidate: &BodyKernelView,
) -> Result<(), String> {
    ensure_range(
        "mass_kg",
        candidate.mass_kg,
        BODY_MASS_KG_MIN,
        BODY_MASS_KG_MAX,
    )?;
    ensure_range(
        "radius_cm",
        candidate.radius_cm,
        BODY_RADIUS_CM_MIN,
        BODY_RADIUS_CM_MAX,
    )?;
    ensure_range(
        "thrust_limit",
        candidate.thrust_limit,
        BODY_THRUST_LIMIT_MIN,
        BODY_THRUST_LIMIT_MAX,
    )?;
    let cross_section_max = cross_section_max_for_radius(candidate.radius_cm);
    ensure_range(
        "cross_section_cm2",
        candidate.cross_section_cm2,
        BODY_CROSS_SECTION_CM2_MIN,
        cross_section_max,
    )?;
    ensure_rate("mass_kg", current.mass_kg, candidate.mass_kg)?;
    ensure_rate("radius_cm", current.radius_cm, candidate.radius_cm)?;
    ensure_rate("thrust_limit", current.thrust_limit, candidate.thrust_limit)?;
    ensure_rate(
        "cross_section_cm2",
        current.cross_section_cm2,
        candidate.cross_section_cm2,
    )?;
    Ok(())
}

fn ensure_range(field: &str, value: u64, min: u64, max: u64) -> Result<(), String> {
    if value < min || value > max {
        return Err(format!(
            "body guard {field} out of range: {value} not in {min}..={max}"
        ));
    }
    Ok(())
}

fn ensure_rate(field: &str, previous: u64, next: u64) -> Result<(), String> {
    if previous == 0 || BODY_CHANGE_FACTOR_MAX <= 1 {
        return Ok(());
    }
    let max = previous.saturating_mul(BODY_CHANGE_FACTOR_MAX);
    let min = previous / BODY_CHANGE_FACTOR_MAX;
    if next > max || next < min {
        return Err(format!(
            "body guard {field} rate violation: prev={previous} next={next} factor={BODY_CHANGE_FACTOR_MAX}"
        ));
    }
    Ok(())
}

fn cross_section_max_for_radius(radius_cm: u64) -> u64 {
    radius_cm
        .saturating_mul(radius_cm)
        .saturating_mul(4)
        .max(BODY_CROSS_SECTION_CM2_MIN)
}
