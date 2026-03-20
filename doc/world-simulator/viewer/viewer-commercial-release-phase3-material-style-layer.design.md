# Viewer 商业化发行 Phase 3 材质风格覆盖层设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase3-material-style-layer.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase3-material-style-layer.project.md`

## 1. 设计定位
定义在已有外部 mesh 能力之上的材质风格覆盖层：允许按实体类别用 `#RRGGBB` 覆盖基础色和自发光色，以低成本实现多发行主题的快速调优。

## 2. 设计结构
- 配置资源层：`ViewerExternalMaterialConfig` 收敛各类实体材质覆盖。
- 环境变量层：通过统一前缀的颜色配置注入主题色。
- 构建逻辑层：场景初始化时优先应用合法覆盖，否则回退默认色。
- 回归保护层：snapshot 基线继续保护默认主题不漂移。

## 3. 关键接口 / 入口
- `ViewerExternalMaterialConfig`
- `OASIS7_VIEWER_*_BASE_COLOR`
- `OASIS7_VIEWER_*_EMISSIVE_COLOR`
- `viewer_3d_config.rs`
- `setup_3d_scene`

## 4. 约束与边界
- 仅接受严格 `#RRGGBB`，非法值自动回退默认色。
- 默认不启用覆盖，避免无意漂移基线。
- 本阶段不引入纹理贴图、后处理 LUT 或风格化 shader。
- 不改动 world 数据协议与运行时语义。

## 5. 设计演进计划
- 先补配置资源与环境变量解析。
- 再把材质覆盖接入 3D 场景构建逻辑。
- 最后用测试、手册和基线验证收口 Phase 3。
