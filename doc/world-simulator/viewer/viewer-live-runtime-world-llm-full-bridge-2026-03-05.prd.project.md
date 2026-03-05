# Viewer Live runtime/world 真 LLM 全量接管（LLM 决策 + 100% 事件/快照 + hard-fail）（2026-03-05）项目管理文档

- 对应设计文档: doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 移除启发式 sidecar，落地真实 LLM driver + shadow WorldKernel，并接入硬失败语义。
- [ ] T2 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 补齐 runtime 事件/快照 100% 映射、扩展 viewer 协议并输出 DecisionTrace。
- [ ] T3 (PRD-WORLD_SIMULATOR-019) [test_tier_required]: 执行 required 回归、更新 viewer 手册与模块项目状态收口。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/viewer/runtime_live.rs`
- `crates/agent_world/src/viewer/runtime_live/control_plane.rs`
- `crates/agent_world/src/viewer/protocol.rs`
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/runner.rs`
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/runtime/world/domain.rs`
- `doc/world-simulator/viewer/viewer-manual.md`

## 状态
- 当前阶段: in_progress
- 当前任务: T2
- 备注: 阶段目标为 true LLM 取代 sidecar + 100% 事件/快照覆盖。
