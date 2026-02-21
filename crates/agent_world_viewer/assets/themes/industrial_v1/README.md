# industrial_v1

Viewer industrial-style starter pack for commercial visual polish.

## Content
- `meshes/`:
  - `agent_industrial.gltf(.bin)`
  - `location_industrial.gltf(.bin)`
  - `asset_industrial.gltf(.bin)`
  - `power_plant_industrial.gltf(.bin)`
  - `power_storage_industrial.gltf(.bin)`
- `textures/`:
  - `*_base.png`
  - `*_normal.png`
  - `*_metallic_roughness.png`
  - `*_emissive.png`
- `presets/`:
  - `industrial_default.env`
  - `industrial_matte.env`
  - `industrial_glossy.env`

## Regenerate
```bash
python3 scripts/generate-viewer-industrial-theme-assets.py
```

## Quick Apply
```bash
source crates/agent_world_viewer/assets/themes/industrial_v1/presets/industrial_default.env
env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5023
```
