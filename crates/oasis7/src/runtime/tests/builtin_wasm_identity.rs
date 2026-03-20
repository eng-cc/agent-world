use super::super::{
    m1_builtin_module_artifact_identity, m4_builtin_module_artifact_identity,
    m5_builtin_module_artifact_identity, WorldError,
};

#[test]
fn builtin_identity_manifest_resolves_m1_entry() {
    let identity = m1_builtin_module_artifact_identity(
        "m1.rule.move",
        "a56595b357bbfb34533c571c2651e2c81cd5a9fcce839ffd712e1655d82179bf",
    )
    .expect("resolve m1 identity");
    assert!(identity.is_complete());
    assert_eq!(identity.signer_node_id, "builtin.module.release.signer");
    match identity.signature_scheme.as_str() {
        "ed25519" => {
            assert!(
                identity
                    .artifact_signature
                    .starts_with("modsig:ed25519:v1:"),
                "unexpected signature: {}",
                identity.artifact_signature
            );
        }
        "identity_hash_v1" => {
            assert!(
                identity.artifact_signature.starts_with("idhash:"),
                "unexpected signature: {}",
                identity.artifact_signature
            );
        }
        other => panic!("unexpected signature scheme: {other}"),
    }
}

#[test]
fn builtin_identity_manifest_resolves_m4_entry() {
    let identity = m4_builtin_module_artifact_identity(
        "m4.factory.miner.mk1",
        "83207ca136c01c7ee1b2eda9262332b2e73fcfd72fb035a149bfbee9b4c656a4",
    )
    .expect("resolve m4 identity");
    assert!(identity.is_complete());
    assert_eq!(identity.signer_node_id, "builtin.module.release.signer");
    match identity.signature_scheme.as_str() {
        "ed25519" => {
            assert!(
                identity
                    .artifact_signature
                    .starts_with("modsig:ed25519:v1:"),
                "unexpected signature: {}",
                identity.artifact_signature
            );
        }
        "identity_hash_v1" => {
            assert!(
                identity.artifact_signature.starts_with("idhash:"),
                "unexpected signature: {}",
                identity.artifact_signature
            );
        }
        other => panic!("unexpected signature scheme: {other}"),
    }
}

#[test]
fn builtin_identity_manifest_resolves_m5_entry() {
    let identity = m5_builtin_module_artifact_identity(
        "m5.gameplay.war.core",
        "464de0c01d34102e0835f4c3ccd8aaaadea9bed3741b5a2d3a5c368ac758b8b2",
    )
    .expect("resolve m5 identity");
    assert!(identity.is_complete());
    assert_eq!(identity.signer_node_id, "builtin.module.release.signer");
    match identity.signature_scheme.as_str() {
        "ed25519" => {
            assert!(
                identity
                    .artifact_signature
                    .starts_with("modsig:ed25519:v1:"),
                "unexpected signature: {}",
                identity.artifact_signature
            );
        }
        "identity_hash_v1" => {
            assert!(
                identity.artifact_signature.starts_with("idhash:"),
                "unexpected signature: {}",
                identity.artifact_signature
            );
        }
        other => panic!("unexpected signature scheme: {other}"),
    }
}

#[test]
fn builtin_identity_manifest_rejects_missing_module() {
    let err = m1_builtin_module_artifact_identity(
        "m1.rule.missing",
        "a56595b357bbfb34533c571c2651e2c81cd5a9fcce839ffd712e1655d82179bf",
    )
    .expect_err("missing module should fail");

    match err {
        WorldError::ModuleChangeInvalid { reason } => {
            assert!(
                reason.contains("missing module_id=m1.rule.missing"),
                "unexpected reason: {reason}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
