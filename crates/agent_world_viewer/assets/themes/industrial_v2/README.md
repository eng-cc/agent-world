# industrial_v2

Second-pass industrial theme pack oriented for release polish.

## Content
- `meshes/`:
  - `agent_industrial_v2.gltf(.bin)`
  - `location_industrial_v2.gltf(.bin)`
  - `asset_industrial_v2.gltf(.bin)`
  - `power_plant_industrial_v2.gltf(.bin)`
  - `power_storage_industrial_v2.gltf(.bin)`
- `textures/`:
  - `*_base.png`
  - `*_normal.png`
  - `*_metallic_roughness.png`
  - `*_emissive.png`
- `presets/`:
  - `industrial_v2_default.env`
  - `industrial_v2_matte.env`
  - `industrial_v2_glossy.env`

## Regenerate
```bash
python3 scripts/generate-viewer-industrial-theme-assets.py --quality v2 --out-dir crates/agent_world_viewer/assets/themes/industrial_v2
```
