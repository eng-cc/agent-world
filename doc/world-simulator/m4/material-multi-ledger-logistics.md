# M4 材料多账本与物流约束（设计文档）

## 目标
- 将 runtime 现有 world 级共享材料账本升级为多账本模型，支持 owner/site/factory 维度隔离。
- 在不破坏现有 M4 经济闭环与 WASM 模块执行路径的前提下，增加材料物流约束（距离、损耗、在途延迟、吞吐上限）。
- 保持快照/回放兼容：旧状态可自动迁移到新账本模型。

## 范围

### In Scope
- 新增 `MaterialLedgerId` 与 `material_ledgers` 状态。
- 新增多账本材料 API：查询/设置/调整/转移。
- 工厂建造与配方执行改为按账本扣料/入账。
- 新增 `TransferMaterial` 动作与 `MaterialTransferred` / `MaterialTransit*` 事件。
- 新增在途队列与按 tick 结算逻辑。
- 在经济模块请求中补充多账本可用输入（兼容旧字段）。
- 补齐单测、回放兼容测试与场景文档更新。

### Out of Scope
- 完整物流网络寻路（图算法、拥塞路由、路径自动规划）。
- 复杂金融市场机制（期货、信用、税制）。
- Viewer 完整物流可视化前端重构（本轮只保证状态可观测字段与测试可验证）。

## 接口 / 数据

### 账本标识
- `MaterialLedgerId`：
  - `world`
  - `agent:<id>`
  - `site:<id>`
  - `factory:<id>`

### 运行时状态
- `WorldState.material_ledgers: BTreeMap<MaterialLedgerId, BTreeMap<String, i64>>`
- 保留 `WorldState.materials` 作为迁移兼容字段（旧快照读取后迁移到 `world` 账本）。
- `FactoryState` 新增：`input_ledger`、`output_ledger`。
- 新增 `pending_material_transits`（在途物流任务队列）。

### 动作/事件
- Action:
  - `TransferMaterial { from_ledger, to_ledger, kind, amount, distance_km }`
- DomainEvent:
  - `MaterialTransferred { ... }`（即时同账本域转移）
  - `MaterialTransitStarted { ... ready_at ... }`
  - `MaterialTransitCompleted { ... received_amount, loss_amount ... }`

### 约束
- `amount > 0`。
- 距离不得超过 `material_transfer_max_distance_km`。
- 损耗：`loss = amount * distance_km * transfer_loss_bps / 10000`。
- 在途延迟：`ready_at = now + max(1, ceil(distance_km / transfer_speed_km_per_tick))`。
- 吞吐：单 tick 启动中的在途任务总量受 `material_transfer_max_inflight_per_tick` 限制。

### ABI 兼容
- `RecipeExecutionRequest` / `FactoryBuildRequest` 保留 `available_inputs`。
- 新增 `available_inputs_by_ledger`（可选字段），旧模块忽略即可继续运行。

## 里程碑
- T1（模型与迁移）：多账本状态/API、工厂账本映射、旧快照迁移。
- T2（物流约束）：TransferMaterial + 在途队列 + 距离/损耗/延迟/吞吐。
- T3（兼容与收口）：ABI 扩展、测试矩阵、场景与文档收口。

## 风险
- 状态迁移风险：旧快照回放与新状态并存导致行为漂移。
- 兼容风险：旧 WASM 模块仅解析旧请求结构。
- 复杂度风险：物流约束引入后，拒绝路径和事件时序显著增加。
- 性能风险：在途队列增长可能拉长每 tick 处理时延。
