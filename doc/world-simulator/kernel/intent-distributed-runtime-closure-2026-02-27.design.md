# Intent/分布式/生产物流闭环落地设计（2026-02-27）

- 对应需求文档: `doc/world-simulator/kernel/intent-distributed-runtime-closure-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/kernel/intent-distributed-runtime-closure-2026-02-27.project.md`

## 1. 设计定位
定义 simulator 从“单动作即时执行”迁移到“tick 内 Intent 批次收集、冲突裁决、统一提交”的总体结构，并补齐分布式幂等、生产优先级、物流观测与威胁热图的一致性边界。

## 2. 设计结构
- 批次执行层：同 tick intents 先聚合，再按冲突键做稳定裁决，最终一次性提交事件与状态变更。
- 分布式封装层：在 `ActionEnvelope` 中补齐 `idempotency_key`、`intent_batch_hash`、`zone_id`，保证重复广播不重复执行。
- 运行时调度层：生产队列按保命、供能、扩产、扩张排序，物流与情报能力围绕该执行序列提供观测。
- 区域协调层：zone 维度批次与 lease 负责限制局部自治与全局协调之间的边界。
- 观测层：统一输出 `IntentBatchReport`、物流 SLA 与威胁热图，作为回放与解释依据。

## 3. 关键接口 / 入口
- `WorldKernel::step_intents_batch(...)`
- `IntentBatchReport { tick, batch_hash, accepted, rejected, ... }`
- `agent_world_proto::distributed::ActionEnvelope`
- `ActionMempool` 幂等去重索引
- `World::logistics_sla_metrics()`
- `World::threat_heatmap()`

## 4. 约束与边界
- 同 tick 冲突裁决必须稳定且可复现，不能依赖非确定性顺序。
- `idempotency_key` 去重范围绑定 `actor_id`，避免跨 actor 误伤。
- 批次哈希必须来源于 intent 语义，而不是传输顺序。
- zone 协调仅提供基础接口，不在本阶段承担跨世界事务一致性。
- 威胁热图和物流指标只做观测与调度支撑，不直接改写战争规则本体。

## 5. 设计演进计划
- 先固定批次执行与幂等字段语义，完成最小闭环。
- 再接入生产/物流/情报/zone 协调等 runtime 能力。
- 最后将威胁热图纳入 runtime 刷新链路，并以统一报告支撑回放与排障。
