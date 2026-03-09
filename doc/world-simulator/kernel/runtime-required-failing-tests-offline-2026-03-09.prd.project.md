# runtime required 失败用例临时下线（2026-03-09）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/kernel/runtime-required-failing-tests-offline-2026-03-09.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-032) [test_tier_required]: 完成“runtime required 失败用例临时下线”PRD 建模与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-032) [test_tier_required]: 对 10 个已知失败用例执行精确白名单下线（`#[ignore]`），并完成 required 回归验证与文档/日志收口。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/runtime/tests/agent_default_modules.rs`
- `crates/agent_world/src/runtime/tests/power_bootstrap.rs`

## 状态
- 最近更新：2026-03-09
- 当前阶段: completed
- 当前任务: 无
- 备注: `T0/T1` 已完成；required 套件恢复可执行，10 项临时下线保留恢复追踪。
