# Viewer 商业化发行缺口收敛 Phase 5：高级贴图通道覆盖层

## 目标
- 在 Phase 4 基础贴图覆盖基础上，补齐商业化质感所需的高级贴图通道覆盖能力。
- 支持按实体类别覆盖 `normal`、`metallic_roughness`、`emissive` 三类贴图，降低“只有 albedo”造成的塑料感与层次不足。
- 保持向后兼容：未配置高级贴图时沿用当前材质参数，不改变既有视觉基线。

## 范围

### In Scope
- 扩展外部贴图覆盖配置结构，支持五类实体（agent/location/asset/power_plant/power_storage）的高级贴图路径。
- 新增环境变量入口，解析并注入 `ViewerExternalTextureConfig`。
- `setup_3d_scene` 接入高级贴图通道装载：
  - `normal_map_texture`
  - `metallic_roughness_texture`
  - `emissive_texture`
- location 专用 core/halo 材质分支支持高级贴图覆盖。
- 补配置解析测试与关键辅助逻辑测试。
- 更新 `doc/viewer-manual.md` 高级贴图覆盖说明。

### Out of Scope
- 本阶段不引入 AO/height/parallax/clearcoat 通道。
- 本阶段不实现运行中热重载 UI 或主题包切换面板。
- 本阶段不调整 world 协议与模拟数据结构。

## 接口 / 数据
- 扩展配置：`ViewerExternalTextureSlotConfig`
  - `base_texture_asset`（已有）
  - `normal_texture_asset`（新增）
  - `metallic_roughness_texture_asset`（新增）
  - `emissive_texture_asset`（新增）
- 新增环境变量（每类实体各 3 个，共 15 个）：
  - `AGENT_WORLD_VIEWER_<ENTITY>_NORMAL_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_<ENTITY>_METALLIC_ROUGHNESS_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_<ENTITY>_EMISSIVE_TEXTURE_ASSET`
- `<ENTITY>` 取值：`AGENT`、`LOCATION`、`ASSET`、`POWER_PLANT`、`POWER_STORAGE`。
- 值语义：`<asset_path>#<sub_asset_label>` 或运行时可识别路径（如 `png`、`ktx2`）。

## 里程碑
- VCR5-0：设计文档与项目管理文档建档。
- VCR5-1：高级贴图配置结构与环境变量解析落地。
- VCR5-2：`setup_3d_scene` 接入高级贴图通道并补测试。
- VCR5-3：手册、项目状态与 devlog 收口。

## 风险
- 贴图通道错配导致画面异常（法线反向、发光过曝）：
  - 缓解：所有覆盖均为可选；默认回退现有材质配置。
- Web 与 native 对贴图格式支持差异：
  - 缓解：手册明确按 runtime 能力选格式；保持路径覆盖机制中立。
- location 与 chunk/world 材质复用路径的样式串扰：
  - 缓解：location 继续使用独立材质分支，避免覆盖污染公共材质。
