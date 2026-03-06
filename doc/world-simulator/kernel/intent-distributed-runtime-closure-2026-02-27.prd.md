# Intent/分布式/生产物流闭环落地（2026-02-27）

审计轮次: 3

- 对应项目管理文档: doc/world-simulator/kernel/intent-distributed-runtime-closure-2026-02-27.prd.project.md

## 1. Executive Summary
- 将 simulator 从“动作提交即执行”升级为“tick 内 Intent 收集、冲突裁决、统一提交”，保证同 tick 冲突可复现、可解释、可回放。
- 在分布式链路为 Action/Intent 增加批次哈希与幂等键语义，确保重复广播不重复执行，跨节点回放一致。
- 在 runtime 侧补齐生产队列优先级、物流 SLA 可观测、情报 TTL、区域自治协调与威胁热图，形成可持续扩张的自动化控制基础。

## 2. User Experience & Functionality
### In Scope
- Simulator: Intent 批次执行入口、冲突键裁决、统一提交事件。
- Consensus: mempool 幂等去重键、批次哈希扩展、区域批次/租约协调基础能力。
- Runtime: 生产队列优先级调度、物流履约统计、威胁热图计算与查询。
- Observation: 情报缓存 TTL 与过期回源机制。

### Out of Scope
- 完整经济平衡调参与 AI 策略重训练。
- 多世界跨分区事务一致性协议。
- 战争/危机规则本体重写。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- `WorldKernel::step_intents_batch(...)`：接收同 tick intents，按冲突键做稳定裁决，统一提交并返回批次报告。
- `IntentBatchReport`：包含 `tick`, `batch_hash`, `accepted`, `rejected`、冲突解释。
- `agent_world_proto::distributed::ActionEnvelope`：新增 `idempotency_key`、`intent_batch_hash`、`zone_id`（默认空）。
- `ActionMempool`：新增 `(actor_id,idempotency_key)` 去重索引，批次哈希纳入 intent 维度字段。
- Runtime 观测接口：
  - `World::logistics_sla_metrics()`：履约率、违约率、平均延迟。
  - `World::threat_heatmap()`：战区风险指数。

## 5. Risks & Roadmap
- M1 (P0): Intent 批次执行 + 分布式幂等去重 + 生产队列优先级。
- M2 (P1): 物流 SLA 统计 + 情报 TTL + 区域自治/租约协调基础接口。
- M3 (P2): 威胁热图接入 runtime 循环并提供可查询观测。

### Technical Risks
- 既有测试对动作顺序高度敏感，批次化后可能触发回归。
- 分布式 envelope 字段扩展会影响签名/测试样例，需要统一更新。
- 生产/物流优先级策略若过于激进，可能导致低优先任务长期滞留。

## 完成态（2026-02-27）
- M1：已完成。落地 intent 批次执行与冲突裁决报告、分布式幂等键与批次哈希增强、生产队列优先级排序。
- M2：已完成。落地物流 SLA 指标（履约率/违约率/延迟）与情报 TTL 缓存，以及 zone 批次/作用域 lease 能力。
- M3：已完成。落地威胁热图并接入 runtime step 刷新与查询接口。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
