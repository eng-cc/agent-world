# Viewer 商业化发行 Phase 5 高级贴图通道覆盖设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase5-advanced-texture-maps.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase5-advanced-texture-maps.project.md`

## 1. 设计定位
定义在基础贴图覆盖之上继续引入 `normal`、`metallic_roughness`、`emissive` 三类高级贴图通道，以减少“只有 albedo”带来的塑料感并提升商业化质感层次。

## 2. 设计结构
- 扩展配置层：在 `ViewerExternalTextureSlotConfig` 中补齐三类高级贴图字段。
- 环境变量层：统一前缀注入高级贴图路径。
- 材质接入层：在 `setup_3d_scene` 中为标准实体和 location core/halo 分支注入通道贴图。
- 回退层：未配置时继续沿用当前材质参数与表现。

## 3. 关键接口 / 入口
- `normal_map_texture`
- `metallic_roughness_texture`
- `emissive_texture`
- `ViewerExternalTextureSlotConfig`
- `setup_3d_scene`

## 4. 约束与边界
- 本阶段不引入 AO/height/parallax/clearcoat 通道。
- 不扩展运行中热重载 UI。
- 高级贴图仅作为可选覆盖，默认基线不改。
- 不修改 world 协议与模拟数据结构。

## 5. 设计演进计划
- 先扩展配置结构与解析入口。
- 再把高级贴图注入到 3D 材质构建路径。
- 最后通过测试与手册收口 Phase 5 质感提升能力。
