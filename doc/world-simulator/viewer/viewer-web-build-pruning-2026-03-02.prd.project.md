# Viewer Web 构建体积裁剪（2026-03-02）项目管理

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档与项目管理文档）
- [x] T1 (PRD-WORLD_SIMULATOR-002)：代码裁剪（wasm 路径剥离 non-web 模块与 native-only 依赖）
- [x] T2 (PRD-WORLD_SIMULATOR-003)：回归验证（wasm check + trunk release 构建 + 体积对比）
- [x] T3 (PRD-WORLD_SIMULATOR-003)：收口（更新文档状态与当日日志）

## 依赖
- `doc/world-simulator/viewer/viewer-web-build-pruning-2026-03-02.prd.md`
- `crates/agent_world/src/lib.rs`
- `crates/agent_world/src/viewer/mod.rs`
- `crates/agent_world/src/simulator/mod.rs`
- `crates/agent_world/src/simulator/kernel/module_lifecycle.rs`
- `crates/agent_world/src/simulator/llm_defaults.rs`（新增）
- `crates/agent_world/Cargo.toml`

## 状态
- 当前阶段：已完成（T0~T3）
