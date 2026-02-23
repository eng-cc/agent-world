use super::*;

#[test]
fn checked_usize_add_rejects_overflow() {
    let err = checked_usize_add(
        usize::MAX,
        1,
        "membership replay federated aggregate checked add",
    )
    .expect_err("overflow should fail");
    match err {
        WorldError::DistributedValidationFailed { reason } => {
            assert!(
                reason.contains("membership replay federated aggregate checked add overflow"),
                "{reason}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn checked_usize_increment_rejects_overflow() {
    let err = checked_usize_increment(
        usize::MAX,
        "membership replay federated composite cursor checked increment",
    )
    .expect_err("overflow should fail");
    match err {
        WorldError::DistributedValidationFailed { reason } => {
            assert!(
                reason.contains(
                    "membership replay federated composite cursor checked increment overflow"
                ),
                "{reason}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
