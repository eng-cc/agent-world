# Viewer 商业化发行缺口收敛 Phase 7：工业风资产包与主题批量预览

## 目标
- 补齐“可发行视觉底座”之后的第一批可交付美术资产：提供可直接加载的 mesh + PBR 贴图主题包（industrial v1）。
- 把现有“环境变量 + 手动切换”调参流程升级为“可复跑、可留证据”的主题批量预览流程。
- 保持对当前 Viewer 渲染链路零破坏：未启用主题包时行为与当前版本一致。

## 范围

### In Scope
- 新增 `agent_world_viewer` 资产目录下的工业风主题包（`industrial_v1`）：
  - 五类实体外部 mesh：`agent/location/asset/power_plant/power_storage`。
  - 五类实体 PBR 贴图：`base/normal/metallic_roughness/emissive`。
- 新增主题预设环境变量文件，统一管理外部 mesh/贴图映射。
- 新增批量预览脚本：
  - 一次执行完成多材质变体（`default/matte/glossy`）截图输出；
  - 每次运行生成独立输出目录，便于版本对比与留痕。
- 更新 `doc/viewer-manual.md`、项目状态与 `doc/devlog/YYYY-MM-DD.md`。

### Out of Scope
- 不引入 DCC 全流程（建模软件工程、自动烘焙流水线、骨骼动画导出）。
- 不修改 Viewer 协议和 world 模拟语义。
- 不在本阶段引入运行中贴图热重载 watcher（后续可在 Phase 8 补齐）。

## 接口 / 数据
- 资产目录（计划）：
  - `crates/agent_world_viewer/assets/themes/industrial_v1/meshes/*.gltf|*.bin`
  - `crates/agent_world_viewer/assets/themes/industrial_v1/textures/*.png`
  - `crates/agent_world_viewer/assets/themes/industrial_v1/presets/*.env`
- 主题预设入口（脚本 `source` 后生效）：
  - `AGENT_WORLD_VIEWER_*_MESH_ASSET`
  - `AGENT_WORLD_VIEWER_*_{BASE,NORMAL,METALLIC_ROUGHNESS,EMISSIVE}_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET`
- 批量预览脚本（计划）：
  - `scripts/viewer-theme-pack-preview.sh`
  - 输出目录：`output/theme_preview/<timestamp>/<variant>/`

## 里程碑
- VCR7-0：设计文档与项目管理文档建档。
- VCR7-1：工业风主题包资产落地（mesh + PBR 贴图 + 目录规范）。
- VCR7-2：主题预设与批量预览脚本落地（多变体截图导出）。
- VCR7-3：测试验证、手册更新、状态回写与 devlog 收口。

## 风险
- 资产格式兼容风险（Web/Native）：  
  缓解：首版统一使用 `.gltf + .bin` 与 `.png`，避免平台特定容器差异。
- 新增资产体积导致仓库膨胀：  
  缓解：控制首版分辨率与面数，按主题分目录管理，后续再做高精版本。
- 主题切换带来视觉可读性退化：  
  缓解：默认不启用主题包；批量预览输出作为回归依据。
