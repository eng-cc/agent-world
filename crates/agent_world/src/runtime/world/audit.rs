use std::path::Path;

use super::World;
use super::super::{AuditFilter, WorldError, WorldEvent};
use super::super::util::write_json_to_path;

impl World {
    // ---------------------------------------------------------------------
    // Audit
    // ---------------------------------------------------------------------

    pub fn audit_events(&self, filter: &AuditFilter) -> Vec<WorldEvent> {
        self.journal
            .events
            .iter()
            .filter(|event| filter.matches(event))
            .cloned()
            .collect()
    }

    pub fn save_audit_log(
        &self,
        path: impl AsRef<Path>,
        filter: &AuditFilter,
    ) -> Result<(), WorldError> {
        let events = self.audit_events(filter);
        write_json_to_path(&events, path.as_ref())
    }
}
