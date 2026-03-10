# Viewer Web 构建体积裁剪 Phase 2（2026-03-02）项目管理

- 对应设计文档: `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档与项目管理文档）
- [x] T1 (PRD-WORLD_SIMULATOR-002)：代码改造（`bevy` feature 精细化并跑编译回归）
- [x] T2 (PRD-WORLD_SIMULATOR-002)：代码改造（字体改为运行时资源加载并跑编译回归）
- [x] T3 (PRD-WORLD_SIMULATOR-003)：体积验证与收口（trunk release 对比 + 文档/日志更新）

## 依赖
- `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.design.md`
- `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.prd.md`
- `crates/agent_world_viewer/Cargo.toml`
- `crates/agent_world_viewer/src/copyable_text.rs`
- `crates/agent_world_viewer/src/main_ui_runtime.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（T0~T3）
