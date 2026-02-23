# Agent World Runtime：分布式存储自愈控制面（2026-02-23）

## 目标
- 补齐此前 out-of-scope 的控制面空缺：让系统在 provider 异构与节点波动下可持续自动修复副本并做负载重平衡。
- 将“副本修复 / 重平衡”从人工运维动作收敛为可执行、可测试的控制面闭环。
- 不引入“任意单机可完整提供所有数据”的假设，修复与迁移均以多 provider 索引为前提。

## 范围

### In Scope
- `agent_world_net` 新增 `replica_maintenance` 模块，提供：
  - 副本维护策略（目标副本数、每轮最大任务、负载阈值等）。
  - 维护计划生成器（Repair + Rebalance）。
  - 维护计划执行器（执行成功后写回 DHT provider 索引）。
- 对外导出维护计划/报告结构，便于 node/runtime 后续调度接线。
- `agent_world_net` 单测覆盖：
  - 副本不足时能产生修复任务。
  - 高负载倾斜时能产生重平衡任务。
  - 执行成功会发布 target provider，失败不会污染索引。

### Out of Scope
- 跨机真实数据搬运协议（本轮仅定义执行器接口，不绑定具体传输实现）。
- 跨 DC 拓扑感知、地域感知放置策略。
- 纠删码编码/解码与碎片修复协议。

## 接口/数据

### 1) 维护策略
- `ReplicaMaintenancePolicy`：
  - `target_replicas_per_blob: usize`（默认 3）
  - `max_repairs_per_round: usize`（默认 32）
  - `max_rebalances_per_round: usize`（默认 32）
  - `rebalance_source_load_min_per_mille: u16`（默认 850）
  - `rebalance_target_load_max_per_mille: u16`（默认 450）

### 2) 计划与执行
- `plan_replica_maintenance(...) -> ReplicaMaintenancePlan`
  - 输入：`world_id` + 目标 blob 列表 + DHT + 策略
  - 输出：
    - `repair_tasks`
    - `rebalance_tasks`
    - `warnings`（如缺少可用 source/target）
- `execute_replica_maintenance_plan(...) -> ReplicaMaintenanceReport`
  - 通过抽象执行器接口执行任务，成功后 `publish_provider` 到 DHT。

### 3) 任务模型
- `ReplicaTransferTask`
  - `content_hash`
  - `source_provider_id`
  - `target_provider_id`
  - `kind`（`Repair` / `Rebalance`）

## 里程碑
- M0：设计与任务拆解。
- M1：维护计划生成器（Repair + Rebalance）落地。
- M2：计划执行器落地并形成索引回写闭环。
- M3：回归、文档与日志收口。

## 风险
- 风险：DHT provider 信息滞后导致计划不最优。
  - 缓解：计划按轮次短周期运行，单轮任务上限与可回滚错误报告。
- 风险：执行器失败可能造成局部修复停滞。
  - 缓解：失败不写回索引，报告中保留失败项，供下一轮重试或降级处理。

## 当前状态
- 状态：进行中
- 已完成：M0
- 进行中：M1
- 未开始：M2、M3
