# Viewer 商业化发行 Phase 8 运行中主题切换与热重载设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase8-runtime-theme-hot-reload-and-asset-v2.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase8-runtime-theme-hot-reload-and-asset-v2.project.md`

## 1. 设计定位
定义商业化发行链路从“脚本批量预览”迈向“运行中可切换主题”的方案：在 Viewer 内直接应用 preset、支持热重载，并交付 `industrial_v2` 资产包与主题校验脚本。

## 2. 设计结构
- 运行时状态层：`ThemeRuntimeState` 管理当前 preset、路径、应用状态和热重载开关。
- 控制 UI 层：右侧面板提供 preset 选择、自定义路径、Apply 与 Hot Reload 控件。
- 资源应用层：主题切换走资源重算与场景全量重建，避免半刷新串扰。
- 资产治理层：`industrial_v2` 资产包与校验脚本共同保证结构和质量门槛。

## 3. 关键接口 / 入口
- `ThemeRuntimeState`
- 右侧主题控制区
- `AGENT_WORLD_VIEWER_*_MESH_ASSET`
- `AGENT_WORLD_VIEWER_*_{BASE,NORMAL,METALLIC_ROUGHNESS,EMISSIVE}_TEXTURE_ASSET`
- `validate-viewer-theme-pack.py`

## 4. 约束与边界
- 运行中切换必须能明确回显成功/失败，并保留上一次成功配置。
- 热重载失败不能破坏当前场景可用状态。
- 本阶段不引入动画/VFX 和 DCC 自动导入链路。
- 资产体积增长要通过校验脚本和阈值守门。

## 5. 设计演进计划
- 先补 ThemeRuntimeState 与 preset 解析。
- 再接 UI 控制区与场景重建应用路径。
- 最后落地 `industrial_v2` 与资产校验，完成 Phase 8 收口。
