# M4 材料多账本与物流约束（项目管理文档）

## 任务拆解

### 任务1：多账本基础与迁移（对应原 1/2/3/4/11）
- [x] 输出设计文档与项目管理文档
- [x] 新增 `MaterialLedgerId` 与 `WorldState.material_ledgers`
- [x] 保留 `materials` 兼容字段并实现旧快照迁移到 `world` ledger
- [x] 新增多账本材料 API（balance/set/adjust/transfer）
- [x] `FactoryState` 增加输入/输出账本映射
- [x] `BuildFactory`/`ScheduleRecipe` 从账本读取可用材料并扣料
- [x] 回归测试：旧闭环不退化、迁移后守恒成立
- [x] 任务日志与 commit

### 任务2：物流动作与约束（对应原 5/6/7/8/9）
- [x] 新增 `TransferMaterial` 动作与拒绝原因
- [x] 新增 `MaterialTransferred`/`MaterialTransitStarted`/`MaterialTransitCompleted` 事件
- [x] 新增 `MaterialTransitJob` 在途队列与 step 结算
- [x] 落地距离/损耗/延迟/吞吐约束
- [x] 回归测试：即时转移、跨站延迟、超距拒绝、吞吐拒绝
- [x] 任务日志与 commit

### 任务3：ABI 兼容、场景与收口（对应原 10/12 + 回归）
- [x] ABI 请求新增 `available_inputs_by_ledger`（兼容旧字段）
- [x] runtime 模块求值请求接入多账本视图
- [x] 回放一致性/快照兼容测试补齐
- [x] 场景与文档更新（增加物流瓶颈验证口径）
- [x] 任务日志与 commit

## 依赖
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/runtime/world/event_processing.rs`
- `crates/agent_world/src/runtime/world/economy.rs`
- `crates/agent_world_wasm_abi/src/economy.rs`
- `doc/world-simulator/scenario-files.md`

## 状态
- 当前阶段：任务1/2/3 全部完成
- 下一阶段：等待下一轮多账本物流扩展需求
- 最近更新：完成任务3（ABI 多账本视图兼容、模块请求透传、回归测试与场景口径文档，2026-02-14）
