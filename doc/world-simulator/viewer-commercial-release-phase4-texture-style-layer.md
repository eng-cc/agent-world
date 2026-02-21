# Viewer 商业化发行缺口收敛 Phase 4：贴图风格覆盖层

## 目标
- 在 Phase 1（外部 mesh）与 Phase 3（外部颜色）基础上，补齐基础贴图覆盖能力，减少“纯色材质”导致的质感不足。
- 允许按实体类别覆盖 `base_color_texture`，支持快速切换发行主题材质包并保留默认回退行为。
- 保持兼容性：未配置贴图时延续当前视觉基线，不影响既有测试与渲染路径。

## 范围

### In Scope
- 新增外部贴图覆盖配置资源（agent/location/asset/power_plant/power_storage）。
- 新增环境变量入口，支持传入贴图资源路径（`<path#label>`）。
- `setup_3d_scene` 接入 `base_color_texture` 覆盖逻辑（有配置加载贴图，无配置保持默认）。
- 补齐配置解析测试与关键辅助逻辑测试。
- 更新 `doc/viewer-manual.md` 的贴图覆盖说明。

### Out of Scope
- 本阶段不引入 normal/ORM/emissive 贴图自动装载。
- 本阶段不引入材质实例热切换 UI 与运行中动态重载。
- 本阶段不改动 world 数据协议与模拟语义。

## 接口 / 数据
- 新增配置资源：`ViewerExternalTextureConfig`。
- 新增环境变量（首批）：
  - `AGENT_WORLD_VIEWER_AGENT_BASE_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_ASSET_BASE_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_BASE_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_TEXTURE_ASSET`
- 值语义：`<asset_path>#<sub_asset_label>`（例如 `textures/agent/albedo.ktx2#Image0`）。
- 运行时策略：
  - 有合法配置：通过 `AssetServer` 加载贴图并写入对应材质 `base_color_texture`。
  - 无配置或空值：保持当前材质行为。

## 里程碑
- VCR4-0：设计文档与项目管理文档建档。
- VCR4-1：贴图覆盖配置结构与环境变量解析落地。
- VCR4-2：`setup_3d_scene` 接入贴图覆盖并补测试。
- VCR4-3：手册、项目状态与 devlog 收口。

## 风险
- 贴图路径误配导致材质加载失败：
  - 缓解：配置仅为可选覆盖；缺省不改变当前表现。
- Web 与 native 资源格式支持差异（如 `ktx2/png`）：
  - 缓解：本阶段只提供路径接入，不做格式强绑定；手册示例注明按 runtime 能力选择格式。
- 贴图引入后视觉基线漂移：
  - 缓解：默认不启用覆盖，沿用现有 snapshot 与 CI gate 做回归。
