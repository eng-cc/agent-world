# M4 资源与产品系统：合理性与可玩性一体化设计（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] T0：对齐当前实现边界（资源分层、模块工业链、多账本物流、治理约束）。
- [x] T1：完成设计文档（目标、范围、接口/数据、里程碑、风险）。
- [x] T2：输出实现导向的优先级拆分（P0/P1/P2）与测试口径。
- [x] T3：回写项目状态与当日日志，准备后续立项。

## 依赖
- doc/world-simulator/m4/m4-resource-product-system-playability-2026-02-27.prd.md
- `README.md`（最小内建资源口径）
- `doc/world-simulator/m4/m4-industrial-economy-wasm.prd.md`
- `doc/world-simulator/m4/material-multi-ledger-logistics.prd.md`
- `doc/world-simulator/m4/m4-market-hardware-data-governance-closure-2026-02-26.prd.md`
- `crates/agent_world/src/runtime/*` 当前工业与物流实现

## 状态
- 当前阶段：已完成（T0~T3）。
- 阻塞项：无。
- 下一步：按 P0 单独立项实现（共享中间件竞争 + 运输优先级）。
