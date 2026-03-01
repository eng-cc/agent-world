# industrial_v3

Third-pass industrial theme pack prepared for release-facing visual quality.

## Content
- `meshes/`:
  - `agent_industrial_v3.gltf(.bin)`
  - `location_industrial_v3.gltf(.bin)`
  - `asset_industrial_v3.gltf(.bin)`
  - `power_plant_industrial_v3.gltf(.bin)`
  - `power_storage_industrial_v3.gltf(.bin)`
- `textures/`:
  - `*_base.png`
  - `*_normal.png`
  - `*_metallic_roughness.png`
  - `*_emissive.png`
- `presets/`:
  - `industrial_v3_default.env`
  - `industrial_v3_matte.env`
  - `industrial_v3_glossy.env`

## Regenerate
```bash
python3 scripts/generate-viewer-industrial-theme-assets.py --quality v3 --out-dir crates/agent_world_viewer/assets/themes/industrial_v3
```
