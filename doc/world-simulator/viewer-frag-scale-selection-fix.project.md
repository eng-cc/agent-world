# Viewer Frag 实际比例与选中显示修复（项目管理文档）

## 任务拆解
- [x] FSF1.1 输出设计文档（`doc/world-simulator/viewer-frag-scale-selection-fix.md`）
- [x] FSF1.2 输出项目管理文档（本文件）
- [ ] FSF2.1 修复 frag 尺度映射（按数据尺寸线性映射）
- [ ] FSF2.2 修复 frag 选中高亮显示异常（选中后恢复原始比例）
- [ ] FSF3.1 补充/更新相关单测（比例与选中恢复）
- [ ] FSF3.2 执行 test_tier_required 回归验证
- [ ] FSF4.1 更新项目文档状态与开发日志

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/location_fragment_render.rs`
- `crates/agent_world_viewer/src/tests_scene_grid.rs`
- `crates/agent_world_viewer/src/tests_scene_entities.rs`

## 状态
- 当前阶段：FSF1 已完成，进入 FSF2。
- 下一步：完成尺度修复与选中显示修复，再补齐测试回归。
- 最近更新：2026-02-15（完成 FSF1 文档落地）。
