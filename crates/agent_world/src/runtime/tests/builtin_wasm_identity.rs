use super::super::{
    m1_builtin_module_artifact_identity, m4_builtin_module_artifact_identity,
    m5_builtin_module_artifact_identity, WorldError,
};

#[test]
fn builtin_identity_manifest_resolves_m1_entry() {
    let identity = m1_builtin_module_artifact_identity(
        "m1.rule.move",
        "67ece2dc881cf5136d07a7eb7f4992b3dde8f0d0e59d6cb26caf56624dcb11e1",
    )
    .expect("resolve m1 identity");
    assert!(identity.is_complete());
    assert_eq!(identity.signer_node_id, "builtin.module.release.signer");
    assert_eq!(identity.signature_scheme, "ed25519");
    assert!(
        identity
            .artifact_signature
            .starts_with("modsig:ed25519:v1:"),
        "unexpected signature: {}",
        identity.artifact_signature
    );
}

#[test]
fn builtin_identity_manifest_resolves_m4_entry() {
    let identity = m4_builtin_module_artifact_identity(
        "m4.factory.miner.mk1",
        "6e4573f87d6c723c7a472ad1857746c3f3ea0cdaf0944851e22f9a2d7fbb28ef",
    )
    .expect("resolve m4 identity");
    assert!(identity.is_complete());
    assert_eq!(identity.signer_node_id, "builtin.module.release.signer");
    assert_eq!(identity.signature_scheme, "ed25519");
    assert!(
        identity
            .artifact_signature
            .starts_with("modsig:ed25519:v1:"),
        "unexpected signature: {}",
        identity.artifact_signature
    );
}

#[test]
fn builtin_identity_manifest_resolves_m5_entry() {
    let identity = m5_builtin_module_artifact_identity(
        "m5.gameplay.war.core",
        "9c19a832b28ddb8ee4db1dcc73dd41fe34b34c077a0f9386599e45bf466f8c15",
    )
    .expect("resolve m5 identity");
    assert!(identity.is_complete());
    assert_eq!(identity.signer_node_id, "builtin.module.release.signer");
    assert_eq!(identity.signature_scheme, "ed25519");
    assert!(
        identity
            .artifact_signature
            .starts_with("modsig:ed25519:v1:"),
        "unexpected signature: {}",
        identity.artifact_signature
    );
}

#[test]
fn builtin_identity_manifest_rejects_missing_module() {
    let err = m1_builtin_module_artifact_identity(
        "m1.rule.missing",
        "67ece2dc881cf5136d07a7eb7f4992b3dde8f0d0e59d6cb26caf56624dcb11e1",
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
