# Viewer 商业化发行缺口收敛 Phase 1：资产管线基础层

## 目标
- 为 `agent_world_viewer` 建立“可渐进替换”的外部资产接入底座，让当前程序化几何可以被 GLTF/GLB mesh 逐步替换。
- 在不破坏现有场景与测试稳定性的前提下，新增统一配置入口与兜底路径（外部资产缺失时仍回退内置基础几何）。
- 为后续商业化迭代（真实模型、动画、VFX、镜头语言）提供可复用的最小资产管线入口。

## 范围

### In Scope
- 在 `Viewer3dConfig` 增加外部 mesh 覆盖配置（按实体类别覆盖）。
- 新增环境变量入口，支持在运行时指定外部 mesh 资源路径。
- 启动场景时优先使用外部 mesh，缺省回退到现有程序化几何。
- 补齐 `agent_world_viewer` 配置解析与回归测试。
- 更新 `doc/viewer-manual.md` 的资产管线使用说明。

### Out of Scope
- 本阶段不引入完整 DCC 资产生产流程（建模、烘焙、贴图自动化导入）。
- 本阶段不接入骨骼动画、状态机动画、特效系统（VFX）。
- 本阶段不修改 Viewer 协议与 world 模拟语义。

## 接口 / 数据
- 新增配置分组：`ViewerAssetExternalMeshConfig`。
- 计划新增环境变量：
  - `AGENT_WORLD_VIEWER_AGENT_MESH_ASSET`
  - `AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET`
  - `AGENT_WORLD_VIEWER_ASSET_MESH_ASSET`
  - `AGENT_WORLD_VIEWER_POWER_PLANT_MESH_ASSET`
  - `AGENT_WORLD_VIEWER_POWER_STORAGE_MESH_ASSET`
- 值语义：`<asset_path>#<sub_asset_label>`（例如 `models/agent.glb#Mesh0/Primitive0`）。
- 运行时策略：
  - 有配置：走 `AssetServer` 加载外部 mesh。
  - 无配置：维持当前 `Sphere/Cuboid/Capsule` 等内置几何。

## 里程碑
- VCR1-0：设计与项目管理文档建档。
- VCR1-1：配置结构与环境变量解析落地。
- VCR1-2：`setup_3d_scene` 接入“外部 mesh 优先 + 内置几何兜底”。
- VCR1-3：测试与手册更新，完成回归。

## 风险
- 资源路径误配导致模型缺失：
  - 缓解：保留回退几何，保证功能与调试能力不受阻断。
- Web 与 native 的资产加载时序差异：
  - 缓解：保持现有渲染链路不变，仅替换 mesh handle 来源。
- 新增配置项导致运维复杂度上升：
  - 缓解：默认全空值，行为与当前版本保持一致；仅在显式配置时启用。
