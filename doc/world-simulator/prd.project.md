# world-simulator PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_SIMULATOR-001 (PRD-WORLD_SIMULATOR-001): 完成 world-simulator PRD 改写，建立模拟层设计主入口。
- [x] TASK-WORLD_SIMULATOR-002 (PRD-WORLD_SIMULATOR-001/002): 对齐场景系统、Viewer、启动器的统一验收清单。
- [x] TASK-WORLD_SIMULATOR-003 (PRD-WORLD_SIMULATOR-002/003): 固化 Web-first 闭环与 LLM 链路的测试证据模板。
- [x] TASK-WORLD_SIMULATOR-004 (PRD-WORLD_SIMULATOR-003): 建立 simulator 体验质量趋势跟踪。
- [x] TASK-WORLD_SIMULATOR-005 (PRD-WORLD_SIMULATOR-004/005): 完成“启动器链上转账”PRD 条款建模与验收标准冻结（接口、安全、测试口径）。
- [x] TASK-WORLD_SIMULATOR-006 (PRD-WORLD_SIMULATOR-004): `world_chain_runtime` 新增转账提交接口（含请求校验、结构化响应、单元测试）。
- [x] TASK-WORLD_SIMULATOR-007 (PRD-WORLD_SIMULATOR-005): runtime 新增主 token 账户转账动作/事件/状态更新（含 nonce anti-replay、余额约束、回归测试）。
- [x] TASK-WORLD_SIMULATOR-008 (PRD-WORLD_SIMULATOR-004): `agent_world_client_launcher` 新增转账 UI 与提交流程（含输入校验、状态提示、错误展示）。
- [x] TASK-WORLD_SIMULATOR-009 (PRD-WORLD_SIMULATOR-004/005): 完成启动器-链运行时转账闭环测试（`test_tier_required`）与测试证据沉淀。
- [x] TASK-WORLD_SIMULATOR-010 (PRD-WORLD_SIMULATOR-001/002/003): 建立模块级专题任务映射索引（2026-03-02 批次）。

## 专题任务映射（2026-03-02 批次）
- [x] SUBTASK-WORLD_SIMULATOR-20260302-001 (PRD-WORLD_SIMULATOR-001/002/003): `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-002 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-003 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-004 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-005 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-006 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-007 (PRD-WORLD_SIMULATOR-001/002/003): `doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-008 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/viewer/viewer-web-build-pruning-2026-03-02.project.md`
- [x] SUBTASK-WORLD_SIMULATOR-20260302-009 (PRD-WORLD_SIMULATOR-002/003): `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.project.md`

## 依赖
- `doc/world-simulator/scenario/scenario-files.md`
- `doc/world-simulator/viewer/viewer-web-closure-testing-policy.md`
- `doc/world-simulator/launcher/game-unified-launcher-2026-02-27.md`
- `doc/world-simulator/launcher/launcher-chain-runtime-decouple-2026-02-28.md`
- `doc/world-simulator/prd/acceptance/unified-checklist.md`
- `doc/world-simulator/prd/acceptance/web-llm-evidence-template.md`
- `doc/world-simulator/prd/quality/experience-trend-tracking.md`
- `doc/world-simulator/prd/launcher/blockchain-transfer.md`
- `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.md`
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world/src/runtime/world/event_processing/action_to_event_core.rs`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active（当前任务清单已完成，等待新增需求）
- 当前优先任务: 无（待新增任务）
- 并行待办: 无
- 专题映射状态: 2026-03-02 批次 9/9 已纳入模块项目管理文档。
- 说明: 本文档仅维护 world-simulator 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
