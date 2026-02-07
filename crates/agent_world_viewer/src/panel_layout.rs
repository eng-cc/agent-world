use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::i18n::{
    language_toggle_label, top_controls_label, top_panel_toggle_label, UiI18n, UiLocale,
};

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RightPanelLayoutState {
    pub top_panel_collapsed: bool,
}

#[derive(Component)]
pub(super) struct TopPanelContainer;

#[derive(Component)]
pub(super) struct TopPanelToggleButton;

#[derive(Component)]
pub(super) struct TopPanelToggleLabel;

#[derive(Component)]
pub(super) struct TopPanelTitleLabel;

#[derive(Component)]
pub(super) struct LanguageToggleButton;

#[derive(Component)]
pub(super) struct LanguageToggleLabel;

pub(super) fn spawn_top_panel_toggle(
    parent: &mut ChildSpawnerCommands,
    font: Handle<Font>,
    locale: UiLocale,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(30.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                column_gap: Val::Px(8.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.11, 0.14)),
            BorderColor::all(Color::srgb(0.2, 0.22, 0.27)),
        ))
        .with_children(|row| {
            row.spawn((
                Button,
                Node {
                    min_width: Val::Px(118.0),
                    height: Val::Px(22.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.22, 0.25, 0.32)),
                TopPanelToggleButton,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new(top_panel_toggle_label(false, locale)),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.93, 0.98)),
                    TopPanelToggleLabel,
                ));
            });

            row.spawn((
                Button,
                Node {
                    min_width: Val::Px(136.0),
                    height: Val::Px(22.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.28, 0.22)),
                LanguageToggleButton,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new(language_toggle_label(locale)),
                    TextFont {
                        font: font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.96, 0.91)),
                    LanguageToggleLabel,
                ));
            });

            row.spawn((
                Text::new(top_controls_label(locale)),
                TextFont {
                    font,
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.78, 0.88)),
                TopPanelTitleLabel,
            ));
        });
}

pub(super) fn handle_top_panel_toggle_button(
    mut interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<TopPanelToggleButton>,
        ),
    >,
    mut layout_state: ResMut<RightPanelLayoutState>,
    i18n: Option<Res<UiI18n>>,
    mut top_panel_query: Query<&mut Node, With<TopPanelContainer>>,
    mut label_query: Query<&mut Text, With<TopPanelToggleLabel>>,
) {
    let locale = i18n.map(|i18n| i18n.locale).unwrap_or(UiLocale::EnUs);

    for interaction in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        layout_state.top_panel_collapsed = !layout_state.top_panel_collapsed;
        let collapsed = layout_state.top_panel_collapsed;

        if let Ok(mut top_panel) = top_panel_query.single_mut() {
            top_panel.display = if collapsed {
                Display::None
            } else {
                Display::Flex
            };
        }

        if let Ok(mut label) = label_query.single_mut() {
            label.0 = top_panel_toggle_label(collapsed, locale).to_string();
        }
    }
}

pub(super) fn handle_language_toggle_button(
    mut interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<LanguageToggleButton>,
        ),
    >,
    i18n: Option<ResMut<UiI18n>>,
    layout_state: Res<RightPanelLayoutState>,
    mut labels: ParamSet<(
        Query<&mut Text, With<TopPanelToggleLabel>>,
        Query<&mut Text, With<LanguageToggleLabel>>,
        Query<&mut Text, With<TopPanelTitleLabel>>,
    )>,
) {
    let Some(mut i18n) = i18n else {
        return;
    };

    let mut locale_changed = false;
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            i18n.locale = i18n.locale.toggled();
            locale_changed = true;
        }
    }

    if !locale_changed {
        return;
    }

    let locale = i18n.locale;
    if let Ok(mut label) = labels.p0().single_mut() {
        label.0 = top_panel_toggle_label(layout_state.top_panel_collapsed, locale).to_string();
    }
    if let Ok(mut label) = labels.p1().single_mut() {
        label.0 = language_toggle_label(locale).to_string();
    }
    if let Ok(mut title) = labels.p2().single_mut() {
        title.0 = top_controls_label(locale).to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{UiI18n, UiLocale};

    #[test]
    fn toggle_label_reflects_state() {
        assert_eq!(top_panel_toggle_label(false, UiLocale::EnUs), "Hide Top");
        assert_eq!(top_panel_toggle_label(true, UiLocale::EnUs), "Show Top");
        assert_eq!(top_panel_toggle_label(false, UiLocale::ZhCn), "隐藏顶部");
        assert_eq!(top_panel_toggle_label(true, UiLocale::ZhCn), "显示顶部");
    }

    #[test]
    fn toggle_button_flips_layout_and_updates_label() {
        let mut app = App::new();
        app.add_systems(Update, handle_top_panel_toggle_button);
        app.world_mut()
            .insert_resource(RightPanelLayoutState::default());
        app.world_mut().insert_resource(UiI18n {
            locale: UiLocale::EnUs,
        });

        let panel = app
            .world_mut()
            .spawn((
                Node {
                    display: Display::Flex,
                    ..default()
                },
                TopPanelContainer,
            ))
            .id();
        let label = app
            .world_mut()
            .spawn((Text::new("Hide Top"), TopPanelToggleLabel))
            .id();
        let button = app
            .world_mut()
            .spawn((Button, Interaction::Pressed, TopPanelToggleButton))
            .id();

        app.update();

        let state = app.world().resource::<RightPanelLayoutState>();
        assert!(state.top_panel_collapsed);

        let panel_node = app.world().entity(panel).get::<Node>().expect("panel node");
        assert_eq!(panel_node.display, Display::None);

        let label_text = app.world().entity(label).get::<Text>().expect("label text");
        assert_eq!(label_text.0, "Show Top");

        app.world_mut().entity_mut(button).insert(Interaction::None);
        app.update();
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        let state = app.world().resource::<RightPanelLayoutState>();
        assert!(!state.top_panel_collapsed);
        let panel_node = app.world().entity(panel).get::<Node>().expect("panel node");
        assert_eq!(panel_node.display, Display::Flex);
        let label_text = app.world().entity(label).get::<Text>().expect("label text");
        assert_eq!(label_text.0, "Hide Top");
    }

    #[test]
    fn language_toggle_switches_locale_and_labels() {
        let mut app = App::new();
        app.add_systems(Update, handle_language_toggle_button);
        app.world_mut().insert_resource(RightPanelLayoutState {
            top_panel_collapsed: false,
        });
        app.world_mut().insert_resource(UiI18n {
            locale: UiLocale::ZhCn,
        });

        let toggle_label = app
            .world_mut()
            .spawn((Text::new("隐藏顶部"), TopPanelToggleLabel))
            .id();
        let language_label = app
            .world_mut()
            .spawn((Text::new("语言：中文"), LanguageToggleLabel))
            .id();
        let title_label = app
            .world_mut()
            .spawn((Text::new("顶部控制区"), TopPanelTitleLabel))
            .id();

        app.world_mut()
            .spawn((Button, Interaction::Pressed, LanguageToggleButton));

        app.update();

        let i18n = app.world().resource::<UiI18n>();
        assert_eq!(i18n.locale, UiLocale::EnUs);

        let toggle_text = app
            .world()
            .entity(toggle_label)
            .get::<Text>()
            .expect("toggle label text");
        assert_eq!(toggle_text.0, "Hide Top");

        let language_text = app
            .world()
            .entity(language_label)
            .get::<Text>()
            .expect("language label text");
        assert_eq!(language_text.0, "Language: English");

        let title_text = app
            .world()
            .entity(title_label)
            .get::<Text>()
            .expect("title label text");
        assert_eq!(title_text.0, "Top Controls");
    }
}
