# M4 资源与产品系统 P0：共享中间件竞争 + 运输优先级（项目管理文档）

## 任务拆解
- [x] T0：输出 P0 设计文档与项目管理文档。
- [x] T1：代码接线（bottleneck tags + material transit priority + 排序与指标）。
- [x] T2：补齐 `test_tier_required` 单测并执行回归。
- [x] T3：回写文档状态与 devlog，完成收口。

## 依赖
- `doc/world-simulator/m4/m4-resource-product-system-playability-2026-02-27.md`
- `crates/agent_world/src/runtime/world/economy.rs`
- `crates/agent_world/src/runtime/world/logistics.rs`
- `crates/agent_world/src/runtime/events.rs`
- `crates/agent_world/src/runtime/state.rs`

## 状态
- 当前阶段：已完成（T0~T3）。
- 阻塞项：无。
- 下一步：无（P0 范围内事项已收口）。
