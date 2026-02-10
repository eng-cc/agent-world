use agent_world::simulator::MaterialKind;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MaterialRenderPreset {
    pub density_kg_m3: f32,
    pub albedo: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub emissivity: f32,
}

#[derive(Clone, Debug)]
pub(super) struct LocationMaterialHandles {
    pub silicate: Handle<StandardMaterial>,
    pub metal: Handle<StandardMaterial>,
    pub ice: Handle<StandardMaterial>,
    pub carbon: Handle<StandardMaterial>,
    pub composite: Handle<StandardMaterial>,
}

impl Default for LocationMaterialHandles {
    fn default() -> Self {
        Self {
            silicate: Handle::default(),
            metal: Handle::default(),
            ice: Handle::default(),
            carbon: Handle::default(),
            composite: Handle::default(),
        }
    }
}

impl LocationMaterialHandles {
    pub(super) fn handle_for(&self, material: MaterialKind) -> Handle<StandardMaterial> {
        match material {
            MaterialKind::Silicate => self.silicate.clone(),
            MaterialKind::Metal => self.metal.clone(),
            MaterialKind::Ice => self.ice.clone(),
            MaterialKind::Carbon => self.carbon.clone(),
            MaterialKind::Composite => self.composite.clone(),
        }
    }
}

pub(super) fn build_location_material_handles(
    materials: &mut Assets<StandardMaterial>,
) -> LocationMaterialHandles {
    LocationMaterialHandles {
        silicate: materials.add(location_standard_material(MaterialKind::Silicate)),
        metal: materials.add(location_standard_material(MaterialKind::Metal)),
        ice: materials.add(location_standard_material(MaterialKind::Ice)),
        carbon: materials.add(location_standard_material(MaterialKind::Carbon)),
        composite: materials.add(location_standard_material(MaterialKind::Composite)),
    }
}

pub(super) fn material_render_preset(material: MaterialKind) -> MaterialRenderPreset {
    match material {
        MaterialKind::Silicate => MaterialRenderPreset {
            density_kg_m3: 2800.0,
            albedo: 0.12,
            roughness: 0.82,
            metallic: 0.02,
            emissivity: 0.92,
        },
        MaterialKind::Metal => MaterialRenderPreset {
            density_kg_m3: 7800.0,
            albedo: 0.55,
            roughness: 0.35,
            metallic: 0.95,
            emissivity: 0.18,
        },
        MaterialKind::Ice => MaterialRenderPreset {
            density_kg_m3: 920.0,
            albedo: 0.65,
            roughness: 0.20,
            metallic: 0.0,
            emissivity: 0.97,
        },
        MaterialKind::Carbon => MaterialRenderPreset {
            density_kg_m3: 1800.0,
            albedo: 0.06,
            roughness: 0.88,
            metallic: 0.05,
            emissivity: 0.85,
        },
        MaterialKind::Composite => MaterialRenderPreset {
            density_kg_m3: 3200.0,
            albedo: 0.25,
            roughness: 0.60,
            metallic: 0.35,
            emissivity: 0.70,
        },
    }
}

fn location_standard_material(material: MaterialKind) -> StandardMaterial {
    let preset = material_render_preset(material);
    let (tint_r, tint_g, tint_b) = material_base_tint(material);
    let intensity = (0.34 + preset.albedo * 1.05).clamp(0.08, 1.0);
    StandardMaterial {
        base_color: Color::srgb(
            (tint_r * intensity).min(1.0),
            (tint_g * intensity).min(1.0),
            (tint_b * intensity).min(1.0),
        ),
        perceptual_roughness: preset.roughness,
        metallic: preset.metallic,
        reflectance: (1.0 - preset.emissivity).clamp(0.02, 0.9),
        ..default()
    }
}

fn material_base_tint(material: MaterialKind) -> (f32, f32, f32) {
    match material {
        MaterialKind::Silicate => (0.82, 0.79, 0.74),
        MaterialKind::Metal => (0.77, 0.84, 0.93),
        MaterialKind::Ice => (0.62, 0.81, 1.0),
        MaterialKind::Carbon => (0.48, 0.44, 0.41),
        MaterialKind::Composite => (0.92, 0.75, 0.55),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_match_design_baseline() {
        let silicate = material_render_preset(MaterialKind::Silicate);
        let metal = material_render_preset(MaterialKind::Metal);
        let ice = material_render_preset(MaterialKind::Ice);
        let carbon = material_render_preset(MaterialKind::Carbon);
        let composite = material_render_preset(MaterialKind::Composite);

        assert_eq!(silicate.density_kg_m3 as i32, 2800);
        assert_eq!(metal.density_kg_m3 as i32, 7800);
        assert_eq!(ice.density_kg_m3 as i32, 920);
        assert_eq!(carbon.density_kg_m3 as i32, 1800);
        assert_eq!(composite.density_kg_m3 as i32, 3200);

        assert!(metal.metallic > silicate.metallic);
        assert!(ice.albedo > carbon.albedo);
        assert!(silicate.roughness > metal.roughness);
    }

    #[test]
    fn build_material_handles_covers_all_material_kinds() {
        let mut materials = Assets::<StandardMaterial>::default();
        let handles = build_location_material_handles(&mut materials);
        assert_ne!(handles.silicate, handles.metal);
        assert_ne!(handles.ice, handles.carbon);
        assert_ne!(handles.composite, handles.silicate);

        let mapped = [
            handles.handle_for(MaterialKind::Silicate),
            handles.handle_for(MaterialKind::Metal),
            handles.handle_for(MaterialKind::Ice),
            handles.handle_for(MaterialKind::Carbon),
            handles.handle_for(MaterialKind::Composite),
        ];
        assert!(mapped.iter().all(|handle| materials.get(handle).is_some()));
    }

    #[test]
    fn tint_palette_keeps_materials_visually_distinct() {
        let silicate = material_base_tint(MaterialKind::Silicate);
        let metal = material_base_tint(MaterialKind::Metal);
        let ice = material_base_tint(MaterialKind::Ice);
        let carbon = material_base_tint(MaterialKind::Carbon);
        let composite = material_base_tint(MaterialKind::Composite);

        assert_ne!(silicate, metal);
        assert_ne!(ice, carbon);
        assert_ne!(composite, silicate);
        assert!(ice.2 > carbon.2);
    }
}
