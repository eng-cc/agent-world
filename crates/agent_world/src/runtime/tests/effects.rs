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
