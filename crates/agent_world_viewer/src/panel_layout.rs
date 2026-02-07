use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

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

pub(super) fn spawn_top_panel_toggle(parent: &mut ChildSpawnerCommands, font: Handle<Font>) {
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
                    Text::new(top_panel_toggle_label(false)),
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
                Text::new("Top Controls"),
                TextFont {
                    font,
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.78, 0.88)),
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
    mut top_panel_query: Query<&mut Node, With<TopPanelContainer>>,
    mut label_query: Query<&mut Text, With<TopPanelToggleLabel>>,
) {
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
            label.0 = top_panel_toggle_label(collapsed).to_string();
        }
    }
}

fn top_panel_toggle_label(collapsed: bool) -> &'static str {
    if collapsed {
        "Show Top"
    } else {
        "Hide Top"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_label_reflects_state() {
        assert_eq!(top_panel_toggle_label(false), "Hide Top");
        assert_eq!(top_panel_toggle_label(true), "Show Top");
    }

    #[test]
    fn toggle_button_flips_layout_and_updates_label() {
        let mut app = App::new();
        app.add_systems(Update, handle_top_panel_toggle_button);
        app.world_mut()
            .insert_resource(RightPanelLayoutState::default());

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
}
