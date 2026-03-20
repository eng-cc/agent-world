use super::egui_right_panel_player_guide::player_cinematic_intro_alpha;

#[test]
fn player_cinematic_intro_alpha_requires_connected_status() {
    assert_eq!(
        player_cinematic_intro_alpha(&crate::ConnectionStatus::Connecting, 10),
        0.0
    );
    assert_eq!(
        player_cinematic_intro_alpha(
            &crate::ConnectionStatus::Error("transport closed".to_string()),
            10,
        ),
        0.0
    );
}

#[test]
fn player_cinematic_intro_alpha_fades_and_expires_by_tick() {
    let early = player_cinematic_intro_alpha(&crate::ConnectionStatus::Connected, 0);
    let fade_in_end = player_cinematic_intro_alpha(&crate::ConnectionStatus::Connected, 6);
    let hold = player_cinematic_intro_alpha(&crate::ConnectionStatus::Connected, 20);
    let fade_out = player_cinematic_intro_alpha(&crate::ConnectionStatus::Connected, 40);
    let end = player_cinematic_intro_alpha(&crate::ConnectionStatus::Connected, 44);
    let expired = player_cinematic_intro_alpha(&crate::ConnectionStatus::Connected, 45);

    assert!(early > 0.0 && early < 1.0);
    assert!((fade_in_end - 1.0).abs() < f32::EPSILON);
    assert!((hold - 1.0).abs() < f32::EPSILON);
    assert!(fade_out > 0.0 && fade_out < 1.0);
    assert!((end - 0.0).abs() < f32::EPSILON);
    assert_eq!(expired, 0.0);
}
