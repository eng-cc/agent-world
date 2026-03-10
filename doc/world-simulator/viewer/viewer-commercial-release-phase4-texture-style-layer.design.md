# Viewer 商业化发行 Phase 4 贴图风格覆盖层设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase4-texture-style-layer.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase4-texture-style-layer.project.md`

## 1. 设计定位
定义在外部 mesh 与材质颜色覆盖基础上引入 `base_color_texture` 覆盖的方案，以更低成本补齐商业化质感，同时保持默认视觉基线不漂移。

## 2. 设计结构
- 贴图配置层：`ViewerExternalTextureConfig` 管理各类实体的 base 贴图路径。
- 环境变量层：通过 `<path>#<label>` 语义声明外部贴图资源。
- 场景接入层：场景初始化时将贴图句柄写入材质 `base_color_texture`。
- 回退层：无配置或空值时保持当前默认材质行为。

## 3. 关键接口 / 入口
- `ViewerExternalTextureConfig`
- `AGENT_WORLD_VIEWER_*_BASE_TEXTURE_ASSET`
- `AssetServer`
- `setup_3d_scene`
- `viewer_3d_config.rs`

## 4. 约束与边界
- 本阶段只覆盖 base 贴图，不引入 normal/ORM/emissive 自动装载。
- 资源路径误配时必须可安全回退，不影响 Viewer 可用性。
- Web/native 对格式支持差异保持在手册口径内，不在本轮做强绑定。
- 默认不启用覆盖，避免 snapshot 基线漂移。

## 5. 设计演进计划
- 先补配置结构与环境变量解析。
- 再在 3D 场景构建中接入贴图覆盖。
- 最后以测试、手册与基线验证收口 Phase 4。
