# Agent World Simulator：Viewer 默认渲染 Frag 且不渲染 Location（项目管理文档）

## 任务拆解
- [x] FDR1：输出设计文档与项目管理文档
- [ ] FDR2：改造渲染路径（不渲染 location、默认渲染 frag、移除 frag 开关）
- [ ] FDR3：补充 frag 选择与详情（仅显示所属 location）
- [ ] FDR4：补充/更新测试并执行回归
- [ ] FDR5：更新说明文档与 devlog 收口

## 依赖
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/scene_dirty_refresh.rs`
- `crates/agent_world_viewer/src/world_overlay.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/location_fragment_render.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/ui_text.rs`

## 状态
- 当前阶段：进行中（FDR2）
- 最近更新：完成 FDR1（2026-02-15）
