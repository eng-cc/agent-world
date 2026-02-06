use super::World;
use super::super::{
    ActionOverrideRecord, CausedBy, RuleDecisionRecord, WorldError, WorldEventBody, WorldEventId,
};

impl World {
    // ---------------------------------------------------------------------
    // Rule decision audit helpers
    // ---------------------------------------------------------------------

    pub fn record_rule_decision(
        &mut self,
        record: RuleDecisionRecord,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        self.append_event(WorldEventBody::RuleDecisionRecorded(record), caused_by)
    }

    pub fn record_action_override(
        &mut self,
        record: ActionOverrideRecord,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        self.append_event(WorldEventBody::ActionOverridden(record), caused_by)
    }
}
