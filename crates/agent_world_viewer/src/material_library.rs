use agent_world::simulator::FragmentElementKind;
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub(super) struct FragmentElementMaterialHandles {
    by_element: BTreeMap<FragmentElementKind, Handle<StandardMaterial>>,
}

impl FragmentElementMaterialHandles {
    pub(super) fn handle_for(&self, element: FragmentElementKind) -> Handle<StandardMaterial> {
        self.by_element
            .get(&element)
            .cloned()
            .unwrap_or_else(Handle::default)
    }
}

const ALL_FRAGMENT_ELEMENTS: [FragmentElementKind; 20] = [
    FragmentElementKind::Oxygen,
    FragmentElementKind::Silicon,
    FragmentElementKind::Magnesium,
    FragmentElementKind::Aluminum,
    FragmentElementKind::Calcium,
    FragmentElementKind::Iron,
    FragmentElementKind::Nickel,
    FragmentElementKind::Cobalt,
    FragmentElementKind::Titanium,
    FragmentElementKind::Chromium,
    FragmentElementKind::Hydrogen,
    FragmentElementKind::Carbon,
    FragmentElementKind::Nitrogen,
    FragmentElementKind::Sulfur,
    FragmentElementKind::Copper,
    FragmentElementKind::Zinc,
    FragmentElementKind::Lithium,
    FragmentElementKind::Neodymium,
    FragmentElementKind::Uranium,
    FragmentElementKind::Thorium,
];

pub(super) fn build_fragment_element_material_handles(
    materials: &mut Assets<StandardMaterial>,
) -> FragmentElementMaterialHandles {
    let mut by_element = BTreeMap::new();
    for element in ALL_FRAGMENT_ELEMENTS {
        by_element.insert(
            element,
            materials.add(fragment_element_standard_material(element)),
        );
    }
    FragmentElementMaterialHandles { by_element }
}

fn fragment_element_standard_material(element: FragmentElementKind) -> StandardMaterial {
    let (r, g, b) = fragment_element_base_tint(element);
    StandardMaterial {
        base_color: Color::srgba(r, g, b, 0.92),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

fn fragment_element_base_tint(element: FragmentElementKind) -> (f32, f32, f32) {
    match element {
        FragmentElementKind::Oxygen => (0.35, 0.76, 0.98),
        FragmentElementKind::Silicon => (0.82, 0.73, 0.57),
        FragmentElementKind::Magnesium => (0.56, 0.88, 0.64),
        FragmentElementKind::Aluminum => (0.78, 0.82, 0.90),
        FragmentElementKind::Calcium => (0.92, 0.82, 0.67),
        FragmentElementKind::Iron => (0.86, 0.48, 0.33),
        FragmentElementKind::Nickel => (0.64, 0.77, 0.82),
        FragmentElementKind::Cobalt => (0.33, 0.54, 0.88),
        FragmentElementKind::Titanium => (0.74, 0.78, 0.84),
        FragmentElementKind::Chromium => (0.44, 0.80, 0.88),
        FragmentElementKind::Hydrogen => (0.64, 0.96, 0.96),
        FragmentElementKind::Carbon => (0.30, 0.30, 0.36),
        FragmentElementKind::Nitrogen => (0.56, 0.52, 0.92),
        FragmentElementKind::Sulfur => (0.95, 0.88, 0.27),
        FragmentElementKind::Copper => (0.90, 0.56, 0.36),
        FragmentElementKind::Zinc => (0.66, 0.72, 0.90),
        FragmentElementKind::Lithium => (0.76, 0.58, 0.90),
        FragmentElementKind::Neodymium => (0.70, 0.40, 0.88),
        FragmentElementKind::Uranium => (0.44, 0.90, 0.42),
        FragmentElementKind::Thorium => (0.84, 0.62, 0.35),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_fragment_element_material_handles_covers_all_elements() {
        let mut materials = Assets::<StandardMaterial>::default();
        let handles = build_fragment_element_material_handles(&mut materials);
        assert_eq!(handles.by_element.len(), ALL_FRAGMENT_ELEMENTS.len());

        for element in ALL_FRAGMENT_ELEMENTS {
            let handle = handles.handle_for(element);
            assert!(materials.get(&handle).is_some());
        }
    }

    #[test]
    fn fragment_element_palette_keeps_key_elements_distinct() {
        let oxygen = fragment_element_base_tint(FragmentElementKind::Oxygen);
        let iron = fragment_element_base_tint(FragmentElementKind::Iron);
        let sulfur = fragment_element_base_tint(FragmentElementKind::Sulfur);
        let uranium = fragment_element_base_tint(FragmentElementKind::Uranium);

        assert_ne!(oxygen, iron);
        assert_ne!(sulfur, uranium);
        assert!(sulfur.0 > iron.0);
        assert!(uranium.1 > iron.1);
    }
}
