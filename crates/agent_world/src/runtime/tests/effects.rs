use super::super::*;
use serde_json::json;

#[test]
fn effect_pipeline_signs_receipt() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
    world.set_policy(PolicySet::allow_all());
    world.set_receipt_signer(ReceiptSigner::hmac_sha256(b"secret"));

    let intent_id = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com"}),
            "cap_all",
            EffectOrigin::System,
        )
        .unwrap();

    let intent = world.take_next_effect().unwrap();
    assert_eq!(intent.intent_id, intent_id);

    let receipt = EffectReceipt {
        intent_id: intent_id.clone(),
        status: "ok".to_string(),
        payload: json!({"status": 200}),
        cost_cents: Some(5),
        signature: None,
    };

    world.ingest_receipt(receipt).unwrap();

    let event = world.journal().events.last().unwrap();
    match &event.body {
        WorldEventBody::ReceiptAppended(receipt) => {
            assert!(receipt.signature.is_some());
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn policy_denies_effect() {
    let mut world = World::new();
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
    world.set_policy(PolicySet {
        rules: vec![PolicyRule {
            when: PolicyWhen {
                effect_kind: Some("http.request".to_string()),
                origin_kind: None,
                cap_name: None,
            },
            decision: PolicyDecision::Deny {
                reason: "blocked".to_string(),
            },
        }],
    });

    let err = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com"}),
            "cap_all",
            EffectOrigin::System,
        )
        .unwrap_err();

    assert!(matches!(err, WorldError::PolicyDenied { .. }));

    let event = world.journal().events.last().unwrap();
    match &event.body {
        WorldEventBody::PolicyDecisionRecorded(record) => {
            assert!(matches!(record.decision, PolicyDecision::Deny { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn pending_effects_are_bounded_and_track_evictions() {
    let mut world = World::new().with_runtime_memory_limits(WorldRuntimeMemoryLimits {
        max_pending_effects: 1,
        ..WorldRuntimeMemoryLimits::default()
    });
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
    world.set_policy(PolicySet::allow_all());

    let first_intent = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com/a"}),
            "cap_all",
            EffectOrigin::System,
        )
        .expect("emit first");
    let second_intent = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com/b"}),
            "cap_all",
            EffectOrigin::System,
        )
        .expect("emit second");

    assert_eq!(world.pending_effects_len(), 1);
    assert_eq!(
        world.runtime_backpressure_stats().pending_effects_evicted,
        1
    );
    let next = world.take_next_effect().expect("one pending effect");
    assert_ne!(next.intent_id, first_intent);
    assert_eq!(next.intent_id, second_intent);
}

#[test]
fn inflight_effect_dispatch_respects_capacity_limit() {
    let mut world = World::new().with_runtime_memory_limits(WorldRuntimeMemoryLimits {
        max_inflight_effects: 1,
        ..WorldRuntimeMemoryLimits::default()
    });
    world.add_capability(CapabilityGrant::allow_all("cap_all"));
    world.set_policy(PolicySet::allow_all());

    let first_intent = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com/a"}),
            "cap_all",
            EffectOrigin::System,
        )
        .expect("emit first");
    let second_intent = world
        .emit_effect(
            "http.request",
            json!({"url": "https://example.com/b"}),
            "cap_all",
            EffectOrigin::System,
        )
        .expect("emit second");

    let first = world.take_next_effect().expect("first effect");
    assert_eq!(first.intent_id, first_intent);
    assert!(
        world.take_next_effect().is_none(),
        "dispatch should be blocked when inflight reaches limit"
    );
    assert_eq!(
        world
            .runtime_backpressure_stats()
            .inflight_effect_dispatch_blocked,
        1
    );

    world
        .ingest_receipt(EffectReceipt {
            intent_id: first_intent.clone(),
            status: "ok".to_string(),
            payload: json!({"status": 200}),
            cost_cents: Some(1),
            signature: None,
        })
        .expect("receipt");
    let second = world
        .take_next_effect()
        .expect("second effect after receipt");
    assert_eq!(second.intent_id, second_intent);
}
