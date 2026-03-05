# Agent World Runtime：玩家发布制成品的 WASM 模块与 Profile 治理闭环（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-WORLD_RUNTIME-010 (PRD-WORLD_RUNTIME-010/012) [test_tier_required]: 发布单扩展 `profile_changes`，Apply 时落账 product/recipe profile，并补齐拒绝路径与回放测试。
- [x] TASK-WORLD_RUNTIME-011 (PRD-WORLD_RUNTIME-010/012) [test_tier_required]: 新增 `FactoryProfileV1` + `GovernFactoryProfile` 动作/事件/状态落账与字段白名单校验。
- [x] TASK-WORLD_RUNTIME-012 (PRD-WORLD_RUNTIME-011) [test_tier_full]: 三节点发布链路 SLA 测试（submit->apply p95 <= 60s）与报表。
- [ ] TASK-WORLD_RUNTIME-013 (PRD-WORLD_RUNTIME-012) [test_tier_required]: 角色审批/签名/冲突校验覆盖与回归。

## 依赖
- doc/world-runtime/module/player-published-entities-2026-03-05.prd.md
- doc/world-runtime/wasm/wasm-interface.md
- doc/world-runtime/module/module-lifecycle.md
- doc/world-runtime/module/module-storage.prd.md
- doc/world-runtime/runtime/runtime-integration.md
- testing-manual.md

## 状态
- 更新日期: 2026-03-05
- 当前状态: active
- 下一任务: TASK-WORLD_RUNTIME-013
