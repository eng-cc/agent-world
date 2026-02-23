use super::*;

#[test]
fn default_camera_mode_is_2d() {
    assert_eq!(ViewerCameraMode::default(), ViewerCameraMode::TwoD);
}

#[test]
fn default_panel_mode_is_observe() {
    assert_eq!(ViewerPanelMode::default(), ViewerPanelMode::Observe);
}

#[test]
fn default_experience_mode_is_player() {
    assert_eq!(
        ViewerExperienceMode::default(),
        ViewerExperienceMode::Player
    );
}
