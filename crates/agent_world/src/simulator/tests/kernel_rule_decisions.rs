use super::*;
use std::sync::{Arc, Mutex};

fn register_location_action(location_id: &str) -> Action {
    Action::RegisterLocation {
        location_id: location_id.to_string(),
        name: format!("name-{location_id}"),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    }
}

#[test]
fn merge_kernel_rule_decisions_rejects_conflicting_overrides() {
    let action_id = 7;
    let decisions = vec![
        KernelRuleDecision::modify(action_id, register_location_action("loc-mod-a")),
        KernelRuleDecision::modify(action_id, register_location_action("loc-mod-b")),
    ];

    let err = merge_kernel_rule_decisions(action_id, decisions)
        .expect_err("conflicting modify should fail merge");
    assert!(matches!(
        err,
        KernelRuleDecisionMergeError::ConflictingOverride { action_id: 7 }
    ));
}

#[test]
fn merge_kernel_rule_decisions_rejects_missing_override() {
    let action_id = 9;
    let decisions = vec![KernelRuleDecision {
        action_id,
        verdict: KernelRuleVerdict::Modify,
        override_action: None,
        notes: vec![],
        cost: KernelRuleCost::default(),
    }];

    let err = merge_kernel_rule_decisions(action_id, decisions)
        .expect_err("modify without override should fail merge");
    assert!(matches!(
        err,
        KernelRuleDecisionMergeError::MissingOverride { action_id: 9 }
    ));
}

#[test]
fn kernel_pre_action_rule_deny_rejects_action() {
    let mut kernel = WorldKernel::new();
    kernel.add_pre_action_rule_hook(|action_id, _, _| {
        KernelRuleDecision::deny(action_id, vec!["blocked by test hook".to_string()])
    });

    kernel.submit_action(register_location_action("loc-denied"));
    let event = kernel.step().expect("denied action emits event");

    match event.kind {
        WorldEventKind::ActionRejected { reason } => match reason {
            RejectReason::RuleDenied { notes } => {
                assert!(notes
                    .iter()
                    .any(|note| note.contains("blocked by test hook")));
            }
            other => panic!("unexpected reject reason: {other:?}"),
        },
        other => panic!("unexpected event kind: {other:?}"),
    }
    assert!(
        !kernel.model().locations.contains_key("loc-denied"),
        "deny hook must keep model unchanged"
    );
}

#[test]
fn kernel_pre_action_rule_modify_overrides_action() {
    let mut kernel = WorldKernel::new();
    kernel.add_pre_action_rule_hook(|action_id, _, _| {
        KernelRuleDecision::modify(action_id, register_location_action("loc-overridden"))
    });

    kernel.submit_action(register_location_action("loc-original"));
    let event = kernel.step().expect("modified action emits event");

    match event.kind {
        WorldEventKind::LocationRegistered { location_id, .. } => {
            assert_eq!(location_id, "loc-overridden");
        }
        other => panic!("unexpected event kind: {other:?}"),
    }
    assert!(!kernel.model().locations.contains_key("loc-original"));
    assert!(kernel.model().locations.contains_key("loc-overridden"));
}

#[test]
fn kernel_conflicting_modify_decisions_are_denied() {
    let mut kernel = WorldKernel::new();
    kernel.add_pre_action_rule_hook(|action_id, _, _| {
        KernelRuleDecision::modify(action_id, register_location_action("loc-mod-a"))
    });
    kernel.add_pre_action_rule_hook(|action_id, _, _| {
        KernelRuleDecision::modify(action_id, register_location_action("loc-mod-b"))
    });

    kernel.submit_action(register_location_action("loc-original"));
    let event = kernel.step().expect("conflict should emit reject event");

    match event.kind {
        WorldEventKind::ActionRejected { reason } => match reason {
            RejectReason::RuleDenied { notes } => {
                assert!(
                    notes
                        .iter()
                        .any(|note| note.contains("conflicting override")),
                    "expected conflicting override in notes, got: {notes:?}"
                );
            }
            other => panic!("unexpected reject reason: {other:?}"),
        },
        other => panic!("unexpected event kind: {other:?}"),
    }
}

#[test]
fn post_action_hook_receives_event_after_modify_decision() {
    let mut kernel = WorldKernel::new();
    kernel.add_pre_action_rule_hook(|action_id, _, _| {
        KernelRuleDecision::modify(action_id, register_location_action("loc-post-override"))
    });

    let captured = Arc::new(Mutex::new(None::<WorldEventKind>));
    let captured_hook = Arc::clone(&captured);
    kernel.add_post_action_rule_hook(move |_, _, event| {
        *captured_hook.lock().expect("lock captured") = Some(event.kind.clone());
    });

    kernel.submit_action(register_location_action("loc-original"));
    let emitted = kernel.step().expect("step should emit event");

    let captured = captured.lock().expect("lock captured");
    let captured_kind = captured.clone().expect("captured kind");
    assert_eq!(captured_kind, emitted.kind);
}
