pub use agent_world_consensus::{LeaseDecision, LeaseManager, LeaseState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release_lease() {
        let mut manager = LeaseManager::new();
        let decision = manager.try_acquire("seq-1", 100, 50);
        assert!(decision.granted);
        let lease_id = decision.lease.as_ref().expect("lease").lease_id.clone();
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
        let lease = first.lease.expect("lease");
        let renewed = manager.renew(&lease.lease_id, 105, 20);
        assert!(renewed.granted);
        assert!(manager.is_active(115));
    }
}
