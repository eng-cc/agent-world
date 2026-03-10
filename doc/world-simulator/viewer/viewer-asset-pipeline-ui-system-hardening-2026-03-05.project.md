# Viewer 资产管线与 UI 体系硬化（2026-03-05）（项目管理文档）

- 对应设计文档: `doc/world-simulator/viewer/viewer-asset-pipeline-ui-system-hardening-2026-03-05.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-asset-pipeline-ui-system-hardening-2026-03-05.prd.md`

审计轮次: 5


## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_SIMULATOR-040 (PRD-WORLD_SIMULATOR-018 / PRD-VAPUI-001/002/003) [test_tier_required]: 输出专题 PRD 与项目管理文档，完成三步改造拆解与依赖梳理。
- [x] TASK-WORLD_SIMULATOR-041 (PRD-WORLD_SIMULATOR-018 / PRD-VAPUI-001) [test_tier_required]: 为 `validate-viewer-theme-pack.py` 增加 profile 级资产预算上限门禁（纹理尺寸、纹理总字节、网格顶点上限）并完成 v1/v2/v3 校验回归。
- [x] TASK-WORLD_SIMULATOR-042 (PRD-WORLD_SIMULATOR-018 / PRD-VAPUI-002) [test_tier_required]: 收敛 `setup_3d_scene` 与 `apply_theme_to_assets_and_scene` 的重复逻辑，并统一 external config 解析入口，补齐定向测试。
- [x] TASK-WORLD_SIMULATOR-043 (PRD-WORLD_SIMULATOR-018 / PRD-VAPUI-003) [test_tier_required + test_tier_full]: 拆分超长 UI/Shell 文件（右侧面板与纹理检查脚本）并完成语法、单测、quick full coverage 回归。
- [x] TASK-WORLD_SIMULATOR-044 (PRD-WORLD_SIMULATOR-018 / PRD-VAPUI-004) [test_tier_required + test_tier_full]: 移除 legacy Bevy UI 路径并迁移 RightPanelLayoutState 到 egui 单一路径，完成 CI required 与 Web 端闭环截图验证。
- [x] TASK-WORLD_SIMULATOR-046 (PRD-WORLD_SIMULATOR-018 / PRD-VAPUI-005) [test_tier_required]: 移除 legacy Bevy UI 测试路径，改为文本函数直测并清理遗留模块引用。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `scripts/validate-viewer-theme-pack.py`
- `scripts/viewer-texture-inspector.sh`
- `scripts/viewer-release-full-coverage.sh`
- `crates/agent_world_viewer/src/main_ui_runtime.rs`
- `crates/agent_world_viewer/src/theme_runtime.rs`
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `testing-manual.md`

## 状态
- 当前阶段: TASK-WORLD_SIMULATOR-046 已完成。
- 当前任务: 无。
- 阻塞项: 无。
- 并行待办: 无。
- 最近更新: 2026-03-05 23:56 CST。
