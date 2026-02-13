// Lease-based single-writer coordination for sequencer roles.

use super::util::blake3_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseState {
    pub holder_id: String,
    pub lease_id: String,
    pub acquired_at_ms: i64,
    pub expires_at_ms: i64,
    pub term: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseDecision {
    pub granted: bool,
    pub lease: Option<LeaseState>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LeaseManager {
    lease: Option<LeaseState>,
    next_term: u64,
}

impl Default for LeaseManager {
    fn default() -> Self {
        Self {
            lease: None,
            next_term: 1,
        }
    }
}

impl LeaseManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> Option<&LeaseState> {
        self.lease.as_ref()
    }

    pub fn is_active(&self, now_ms: i64) -> bool {
        self.lease
            .as_ref()
            .map(|lease| lease.expires_at_ms > now_ms)
            .unwrap_or(false)
    }

    pub fn try_acquire(&mut self, holder_id: &str, now_ms: i64, ttl_ms: i64) -> LeaseDecision {
        if ttl_ms <= 0 {
            return LeaseDecision {
                granted: false,
                lease: self.lease.clone(),
                reason: Some("ttl must be positive".to_string()),
            };
        }

        if let Some(lease) = &self.lease {
            if lease.expires_at_ms > now_ms {
                return LeaseDecision {
                    granted: false,
                    lease: self.lease.clone(),
                    reason: Some("lease already held".to_string()),
                };
            }
        }

        let term = self.next_term;
        self.next_term = self.next_term.saturating_add(1);
        let lease_id = lease_id_for(holder_id, now_ms, term);
        let lease = LeaseState {
            holder_id: holder_id.to_string(),
            lease_id,
            acquired_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(ttl_ms),
            term,
        };
        self.lease = Some(lease.clone());

        LeaseDecision {
            granted: true,
            lease: Some(lease),
            reason: None,
        }
    }

    pub fn renew(&mut self, lease_id: &str, now_ms: i64, ttl_ms: i64) -> LeaseDecision {
        let Some(lease) = &mut self.lease else {
            return LeaseDecision {
                granted: false,
                lease: None,
                reason: Some("no active lease".to_string()),
            };
        };
        if lease.lease_id != lease_id {
            return LeaseDecision {
                granted: false,
                lease: Some(lease.clone()),
                reason: Some("lease_id mismatch".to_string()),
            };
        }
        if lease.expires_at_ms <= now_ms {
            return LeaseDecision {
                granted: false,
                lease: Some(lease.clone()),
                reason: Some("lease expired".to_string()),
            };
        }
        if ttl_ms <= 0 {
            return LeaseDecision {
                granted: false,
                lease: Some(lease.clone()),
                reason: Some("ttl must be positive".to_string()),
            };
        }

        lease.expires_at_ms = now_ms.saturating_add(ttl_ms);
        LeaseDecision {
            granted: true,
            lease: Some(lease.clone()),
            reason: None,
        }
    }

    pub fn release(&mut self, lease_id: &str) -> bool {
        if let Some(lease) = &self.lease {
            if lease.lease_id == lease_id {
                self.lease = None;
                return true;
            }
        }
        false
    }

    pub fn expire_if_needed(&mut self, now_ms: i64) -> Option<LeaseState> {
        if let Some(lease) = &self.lease {
            if lease.expires_at_ms <= now_ms {
                return self.lease.take();
            }
        }
        None
    }
}

fn lease_id_for(holder_id: &str, now_ms: i64, term: u64) -> String {
    let payload = format!("{holder_id}:{now_ms}:{term}");
    blake3_hex(payload.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release_lease() {
        let mut manager = LeaseManager::new();
        let decision = manager.try_acquire("seq-1", 100, 50);
        assert!(decision.granted);
        let lease_id = decision.lease.as_ref().unwrap().lease_id.clone();
        assert!(manager.release(&lease_id));
        assert!(manager.current().is_none());
    }

    #[test]
    fn deny_acquire_when_active() {
        let mut manager = LeaseManager::new();
        let first = manager.try_acquire("seq-1", 100, 50);
        assert!(first.granted);
        let second = manager.try_acquire("seq-2", 120, 50);
        assert!(!second.granted);
    }

    #[test]
    fn allow_acquire_after_expiry() {
        let mut manager = LeaseManager::new();
        let first = manager.try_acquire("seq-1", 100, 10);
        assert!(first.granted);
        manager.expire_if_needed(200);
        let second = manager.try_acquire("seq-2", 200, 10);
        assert!(second.granted);
    }

    #[test]
    fn renew_extends_lease() {
        let mut manager = LeaseManager::new();
        let first = manager.try_acquire("seq-1", 100, 10);
        let lease = first.lease.unwrap();
        let renewed = manager.renew(&lease.lease_id, 105, 20);
        assert!(renewed.granted);
        assert!(manager.is_active(115));
    }
}
