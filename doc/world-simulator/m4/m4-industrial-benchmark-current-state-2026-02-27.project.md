# M4 产业链借鉴点（结合当前实现现状，项目管理文档）

## 任务拆解
- [x] T0：读取近期开发日志与 M4/runtime 现状文档，确认当前实现边界。
- [x] T1：梳理“资源、工业链、物流、市场治理”四类现状基线。
- [x] T2：输出可借鉴点清单（按 P0/P1/P2 优先级）并给出最小接口增量建议。
- [x] T3：完成设计文档收口，补充里程碑、风险与推荐验收口径。

## 依赖
- `doc/world-simulator/m4/m4-industrial-economy-wasm.md`
- `doc/world-simulator/material-multi-ledger-logistics.md`
- `doc/world-simulator/m4/m4-market-hardware-data-governance-closure-2026-02-26.md`
- `README.md`（资源分层口径）
- `crates/agent_world/src/runtime/*` 与 `crates/agent_world/src/simulator/*` 当前实现

## 状态
- 当前阶段：已完成（T0~T3）。
- 阻塞项：无。
- 下一步：按本文档 P0 借鉴项单独立项（建议优先“共享中间件竞争 + 物流时效分层”）。
