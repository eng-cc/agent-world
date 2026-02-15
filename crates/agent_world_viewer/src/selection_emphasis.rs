use bevy::prelude::*;

use super::{Viewer3dScene, ViewerSelection};

const HALO_BASE_RADIUS: f32 = 0.65;
const HALO_RADIUS_MULTIPLIER: f32 = 1.9;
const HALO_HEIGHT_FACTOR: f32 = 0.34;
const HALO_THICKNESS_FACTOR: f32 = 0.16;
const HALO_BOB_SPEED: f32 = 2.4;
const HALO_BOB_AMPLITUDE: f32 = 0.045;

#[derive(Component)]
pub(super) struct SelectionEmphasisHalo;

#[derive(Resource, Default)]
pub(super) struct SelectionEmphasisState {
    halo_entity: Option<Entity>,
}

pub(super) fn update_selection_emphasis(
    mut commands: Commands,
    scene: Res<Viewer3dScene>,
    selection: Res<ViewerSelection>,
    time: Option<Res<Time>>,
    targets: Query<&Transform, Without<SelectionEmphasisHalo>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<SelectionEmphasisState>,
    mut halos: Query<(&mut Transform, &mut Visibility), With<SelectionEmphasisHalo>>,
) {
    let Some(selected) = selection.current.as_ref() else {
        set_halo_visibility(&state, &mut halos, Visibility::Hidden);
        return;
    };

    let Ok(target_transform) = targets.get(selected.entity) else {
        set_halo_visibility(&state, &mut halos, Visibility::Hidden);
        return;
    };
    let halo_transform = halo_transform_for_target(
        target_transform,
        time.as_deref().map(Time::elapsed_secs).unwrap_or(0.0),
    );

    let halo_entity = ensure_halo_entity(
        &mut commands,
        &scene,
        &mut meshes,
        &mut materials,
        state.as_mut(),
        halo_transform,
    );

    if let Ok((mut halo_transform, mut visibility)) = halos.get_mut(halo_entity) {
        *halo_transform = halo_transform_for_target(
            target_transform,
            time.as_deref().map(Time::elapsed_secs).unwrap_or(0.0),
        );
        *visibility = Visibility::Visible;
    }
}

fn ensure_halo_entity(
    commands: &mut Commands,
    scene: &Viewer3dScene,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    state: &mut SelectionEmphasisState,
    initial_transform: Transform,
) -> Entity {
    if let Some(entity) = state.halo_entity {
        return entity;
    }

    let mesh = meshes.add(Sphere::new(1.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.98, 0.82, 0.22, 0.22),
        emissive: Color::srgb(1.0, 0.72, 0.14).into(),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let entity = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            initial_transform,
            Visibility::Visible,
            SelectionEmphasisHalo,
            Name::new("selection:emphasis:halo"),
        ))
        .id();

    if let Some(root) = scene.root_entity {
        commands.entity(root).add_child(entity);
    }

    state.halo_entity = Some(entity);
    entity
}

fn set_halo_visibility(
    state: &SelectionEmphasisState,
    halos: &mut Query<(&mut Transform, &mut Visibility), With<SelectionEmphasisHalo>>,
    visibility: Visibility,
) {
    let Some(entity) = state.halo_entity else {
        return;
    };

    if let Ok((_, mut halo_visibility)) = halos.get_mut(entity) {
        *halo_visibility = visibility;
    }
}

fn halo_transform_for_target(target: &Transform, elapsed_secs: f32) -> Transform {
    let radius = target.scale.max_element().max(HALO_BASE_RADIUS);
    let bob = (elapsed_secs * HALO_BOB_SPEED).sin() * HALO_BOB_AMPLITUDE * radius;
    Transform::from_translation(target.translation + Vec3::Y * (radius * HALO_HEIGHT_FACTOR + bob))
        .with_scale(Vec3::new(
            radius * HALO_RADIUS_MULTIPLIER,
            (radius * HALO_THICKNESS_FACTOR).max(0.08),
            radius * HALO_RADIUS_MULTIPLIER,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SelectionInfo, SelectionKind, ViewerSelection};

    #[test]
    fn halo_transform_scales_and_offsets_from_target() {
        let target = Transform::from_xyz(2.0, 1.0, -3.0).with_scale(Vec3::new(1.2, 2.0, 1.0));
        let halo = halo_transform_for_target(&target, 0.0);

        assert!(halo.scale.x > target.scale.max_element());
        assert!(halo.scale.y < halo.scale.x);
        assert!(halo.translation.y > target.translation.y);
    }

    #[test]
    fn selection_emphasis_hides_halo_when_selection_clears() {
        let mut app = App::new();
        app.add_systems(Update, update_selection_emphasis);
        app.insert_resource(Viewer3dScene::default());
        app.insert_resource(ViewerSelection::default());
        app.insert_resource(SelectionEmphasisState::default());
        app.insert_resource(Assets::<Mesh>::default());
        app.insert_resource(Assets::<StandardMaterial>::default());

        let selected = app
            .world_mut()
            .spawn(Transform::from_xyz(1.0, 2.0, 3.0).with_scale(Vec3::splat(2.0)))
            .id();
        app.world_mut().insert_resource(ViewerSelection {
            current: Some(SelectionInfo {
                entity: selected,
                kind: SelectionKind::Agent,
                id: "agent-1".to_string(),
                name: None,
            }),
        });

        app.update();

        let visible_after_select = {
            let mut query = app
                .world_mut()
                .query::<(&Visibility, &SelectionEmphasisHalo)>();
            query
                .single(app.world_mut())
                .map(|(visibility, _)| *visibility)
                .expect("halo exists")
        };
        assert_eq!(visible_after_select, Visibility::Visible);

        app.world_mut().resource_mut::<ViewerSelection>().clear();
        app.update();

        let visible_after_clear = {
            let mut query = app
                .world_mut()
                .query::<(&Visibility, &SelectionEmphasisHalo)>();
            query
                .single(app.world_mut())
                .map(|(visibility, _)| *visibility)
                .expect("halo exists")
        };
        assert_eq!(visible_after_clear, Visibility::Hidden);
    }
}
