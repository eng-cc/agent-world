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
- [x] FSF6.1 去掉 frag 选中黄色高亮范围（selection halo）
- [x] FSF6.2 Agent 几何尺度改为按 `effective_cm_to_unit` 量纲映射
- [x] FSF6.3 更新相关测试并执行 test_tier_required 回归

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/location_fragment_render.rs`
- `crates/agent_world_viewer/src/tests_scene_grid.rs`
- `crates/agent_world_viewer/src/tests_scene_entities.rs`
- `crates/agent_world_viewer/src/selection_emphasis.rs`

## 状态
- 当前阶段：FSF1~FSF6 已完成。
- 下一步：若仍有“真实比例下可见性/可点选性”反馈，可单开“真实尺度上的辅助视觉层”任务（不改物理量纲）。
- 最近更新：2026-02-15（完成 FSF6：移除 frag halo + Agent 尺度量纲修正 + 回归通过）。
