# Viewer 商业化发行 Phase 1 资产管线设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase1-asset-pipeline.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase1-asset-pipeline.project.md`

## 1. 设计定位
定义 Viewer 面向商业化发行的最小外部资产接入底座：允许按实体类别用外部 mesh 渐进替换程序化几何，同时保留内置几何兜底和既有测试稳定性。

## 2. 设计结构
- 配置入口层：`ViewerAssetExternalMeshConfig` 统一管理外部 mesh 覆盖配置。
- 环境变量层：通过实体类别环境变量声明外部资源路径与子资产标签。
- 渲染兜底层：有配置优先走 `AssetServer`，无配置回退当前程序化几何。
- 手册同步层：在 `viewer-manual` 中沉淀资产接入最小使用说明。

## 3. 关键接口 / 入口
- `ViewerAssetExternalMeshConfig`
- `OASIS7_VIEWER_*_MESH_ASSET`
- `<asset_path>#<sub_asset_label>`
- `setup_3d_scene`
- `AssetServer`

## 4. 约束与边界
- 不引入完整 DCC 资产生产流程和骨骼动画。
- 资源路径误配时必须回退内置几何，不阻断 Viewer 使用。
- Web/native 资产加载差异不能改变主体渲染逻辑，仅改变 mesh 来源。
- 本阶段不修改 Viewer 协议与 world 语义。

## 5. 设计演进计划
- 先接入配置结构与环境变量解析。
- 再在场景初始化中实现“外部 mesh 优先 + 内置兜底”。
- 最后通过回归测试和手册完成 Phase 1 资产管线收口。
