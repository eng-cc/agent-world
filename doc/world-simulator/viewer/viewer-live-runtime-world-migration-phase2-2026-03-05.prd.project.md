# Viewer Live runtime/world 接管 Phase 2（LLM/chat/prompt）（2026-03-05）项目管理文档

- 对应设计文档: doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: `world_viewer_live` 放开 `--runtime-world --llm` 组合并接入 runtime live llm 模式配置。
- [ ] T2 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: 在 `runtime_live.rs` 落地 prompt/chat/auth/llm 决策桥接，消除 phase1 `unsupported` 断裂。
- [ ] T3 (PRD-WORLD_SIMULATOR-017) [test_tier_required]: 执行 `cargo test/check` 回归，更新 viewer 手册、模块项目状态与当日 devlog 收口。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_viewer_live.rs`
- `crates/agent_world/src/viewer/runtime_live.rs`
- `crates/agent_world/src/viewer/auth.rs`
- `crates/agent_world/src/viewer/protocol.rs`
- `crates/agent_world/src/bin/world_llm_agent_demo/runtime_bridge.rs`
- `doc/world-simulator/viewer/viewer-manual.md`

## 状态
- 当前阶段: in_progress
- 当前任务: T1
- 备注: Phase 2 目标为 runtime 模式下打通 `LLM/chat/prompt` 控制链路，优先收敛体验断裂，再迭代动作映射覆盖率。
