# Viewer Frag 实际比例与选中显示修复（项目管理文档）

## 任务拆解
- [x] FSF1.1 输出设计文档（`doc/world-simulator/viewer-frag-scale-selection-fix.md`）
- [x] FSF1.2 输出项目管理文档（本文件）
- [x] FSF2.1 修复 frag 尺度映射（按数据尺寸线性映射）
- [x] FSF2.2 修复 frag 选中高亮显示异常（选中后恢复原始比例）
- [x] FSF3.1 补充/更新相关单测（比例与选中恢复）
- [x] FSF3.2 执行 test_tier_required 回归验证
- [x] FSF4.1 更新项目文档状态与开发日志
- [x] FSF5.1 修复 3D 点击误选不可见 location 锚点（仅可拾取带 Mesh 的 location）
- [x] FSF5.2 frag 选中禁用缩放高亮，保持真实比例显示
- [x] FSF5.3 补充二次回归测试并执行全量 viewer 回归

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/location_fragment_render.rs`
- `crates/agent_world_viewer/src/tests_scene_grid.rs`
- `crates/agent_world_viewer/src/tests_scene_entities.rs`

## 状态
- 当前阶段：FSF1~FSF5 已完成。
- 下一步：若仍有“比例可读性”反馈，可评估按缩放层级补充非破坏性视觉辅助（不改真实尺度）。
- 最近更新：2026-02-15（完成 FSF5 二次修复与全量回归）。
