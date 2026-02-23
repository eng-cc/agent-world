use super::*;

#[test]
fn ratio_per_mille_handles_extreme_values_without_saturation_distortion() {
    let denominator = usize::MAX;
    let numerator = denominator / 2 + 1;
    assert_eq!(ratio_per_mille(numerator, denominator), 500);
}

#[test]
fn exceeds_double_handles_extreme_values_without_overflow() {
    assert!(exceeds_double(usize::MAX, usize::MAX / 2));
    assert!(!exceeds_double(usize::MAX / 2, usize::MAX / 2));
}
