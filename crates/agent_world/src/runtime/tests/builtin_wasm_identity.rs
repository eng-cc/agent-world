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
    assert!(identity.matches_unsigned_signature(
        "67ece2dc881cf5136d07a7eb7f4992b3dde8f0d0e59d6cb26caf56624dcb11e1"
    ));
}

#[test]
fn builtin_identity_manifest_resolves_m4_entry() {
    let identity = m4_builtin_module_artifact_identity(
        "m4.factory.miner.mk1",
        "600de767ec4de926415e9641d3e0d75de885e10ad62f4908668d9c81f20acce1",
    )
    .expect("resolve m4 identity");
    assert!(identity.is_complete());
    assert!(identity.matches_unsigned_signature(
        "600de767ec4de926415e9641d3e0d75de885e10ad62f4908668d9c81f20acce1"
    ));
}

#[test]
fn builtin_identity_manifest_resolves_m5_entry() {
    let identity = m5_builtin_module_artifact_identity(
        "m5.gameplay.war.core",
        "86d793ce9bfc2c13bc79805f7a2f1ff6776db0aae96e875fa6aad4a58af05c04",
    )
    .expect("resolve m5 identity");
    assert!(identity.is_complete());
    assert!(identity.matches_unsigned_signature(
        "86d793ce9bfc2c13bc79805f7a2f1ff6776db0aae96e875fa6aad4a58af05c04"
    ));
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
