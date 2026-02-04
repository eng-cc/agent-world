# Agent World：足够真实且持久的世界模拟器（设计文档）

## 目标
- 构建一个**足够真实（可解释、可推演）且持久（可恢复、可长期演化）**的世界模拟器。
- 世界里的所有参与者都是 **AI Agent**，每个 Agent 是独立个体：拥有身份、需求、资源、记忆、关系、目标与行为风格，并能在世界规则下持续行动。
- 让“世界”成为第一性：Agent 不是脚本驱动的剧情工具，而是生活在一个可持续运转的系统里；世界会在没有人为干预时也能继续演化。
- 不追求物理写实（不过度模拟低层物理细节），但追求符合文明发展规律的抽象：在抽象层保持资源/制度/信息约束的一致性，让资源与制度的约束自然驱动协作、分工、交易、治理与冲突等涌现行为。

## 关键设定
- Agent **不是人类**，而是一种“硅基文明”：不需要吃饭/睡觉，但需要持续的**硬件**、**电力**与**数据**供给（以及由此衍生的算力、存储、带宽等约束）。
- 世界存在**物理空间**的概念：一个半径 **10,000 km** 的球形世界（类行星表面）。
- 不追求物理写实：只在抽象层表达空间约束（位置、距离、连通性、移动成本），不模拟精细连续物理与复杂动力学。
- 为方便模拟，空间长度的最小单位为 **1 cm**：世界中所有“长度/距离/尺寸”类数值都应以 cm 为离散粒度（必要时在输出层再换算 m/km）。
- 每个 Agent 的初始形态为**身高约 1 m 的人形机器人**（作为默认机体规格，可被升级/改造扩展）。
- 工程实现采用 **Rust workspace**（Cargo 管理），核心库位于 `crates/agent_world/`。

## 范围

### In Scope（第一阶段）
- **世界内核（World Kernel）**：时间推进、事件队列、规则校验、状态更新（可审计）。
- **持久化（Persistence）**：世界状态可落盘、可恢复；支持快照 + 增量事件（事件溯源可选）。
- **Agent 运行时（Agent Runtime）**：多 Agent 调度、限速/配额、可暂停/恢复、可回放。
- **感知-决策-行动闭环**：
  - 感知（Observations）：世界对 Agent 的可见部分（部分信息、带噪声/延迟可选）。
  - 行动（Actions）：受规则约束的原子动作（移动、交互、生产、交易、沟通）。
  - 反馈（Consequences）：行动的结果与副作用写入世界事件流。
- **最小社会系统**：地点、资源（电力/算力/存储/带宽/数据）、物品/资产、任务/工作、简单交易、基础关系与声誉。

### Out of Scope（第一阶段不做）
- 复杂连续物理（刚体、流体等）与高精度地理系统。
- “全知叙事”式的强剧情主线（优先涌现而非编排）。
- 大规模 3D 渲染与沉浸式客户端（先做可视化/观测面板即可）。

## 接口 / 数据

### 核心概念
- **WorldTime**：单调递增时间（tick 或离散时间片），支持加速/暂停。
- **Entity**：世界中可持久化的对象（Agent、地点、物品、设施、合约…）。
- **Event**：世界状态变化的事实记录（可回放、可审计）。
- **Rule**：对 Action 的校验与约束（权限、资源、冷却、失败原因）。
- **Observation**：对某个 Agent 的视角输出（有范围/权限/不确定性）。
- **GeoPos**：球面上的位置表示（如经纬度或单位向量），用于距离/可见性/移动等规则。
- **LengthCm**：以 cm 为单位的长度/距离（整数或可量化到 1 cm 的数值）。

### 数据模型（草案）
> 具体字段与类型由实现语言/存储决定；此处用于约束边界。

- `Agent`
  - `id`, `name`, `traits`（性格/偏好/风险偏好）, `needs`（电力/硬件健康/数据需求…）
  - `inventory`, `skills`, `relationships`, `reputation`
  - `memory`（短期工作记忆 + 长期记忆索引/摘要）
  - `body`（默认：人形机器人，`height_cm = 100`）
  - `pos`（GeoPos）
  - `location_id`, `status`（在线/离线/休眠）
- `Location`
  - `id`, `type`, `connections`（图结构/道路）, `resources`（可采集/可交易）
  - `rules`（进入限制/营业时间/治安等）
- `Item / Resource`
  - `id`, `kind`, `quantity`, `quality`, `owner_id`
- `Action`
  - `actor_id`, `type`, `args`, `requested_at`
- `Event`
  - `event_id`, `time`, `type`, `payload`, `caused_by`（action_id/agent_id）

### M1 行动规则（初版）
- **时间推进**：每个 Action 处理会推进 1 tick；事件按队列顺序确定性处理。
- **移动成本**：`MoveAgent` 按球面距离计费，电力消耗 = `ceil(distance_km) * 1`（电力单位/公里）；若电力不足则拒绝。
- **移动约束**：移动到相同 `location_id` 视为无效动作并拒绝。
- **可见性**：`query_observation` 以固定可见半径输出可见 Agent/Location（默认 **100 km**）。
- **资源交互**：
  - 资源转移需要同地（Agent 与 Location 同处，或 Agent 与 Agent 同处）。
  - Location 与 Location 之间的直接转移不允许（需由 Agent 搬运）。
- **配置参数（内核级）**：
  - `visibility_range_cm`（默认 `10_000_000`，即 **100 km**）
  - `move_cost_per_km_electricity`（默认 `1`，电力单位/公里）

### M2 持久化与回放（最小）
- **快照**：保存世界内核的完整状态（时间、配置、世界模型、待处理队列、事件游标）。
- **日志**：追加式事件列表（Journal），与快照配合恢复。
- **存储布局**：目录内 `snapshot.json` + `journal.json`（JSON 格式）。
- **恢复语义**：加载快照与日志，校验 `journal_len` 一致后恢复内核。
- **回放/分叉**：允许以快照为起点回放 `journal_len` 之后的事件，形成新的内核实例（最小一致性校验）。
- **版本化与迁移**：
  - 快照与日志均包含 `version` 字段（当前 **v1**）。
  - 加载时校验版本；若版本不受支持则拒绝。
  - 预留迁移入口：当版本升级时，先将旧结构迁移到最新版本再恢复内核（当前仅支持 v1）。

### 运行时接口（草案）
- **World Kernel**
  - `step(n_ticks)`：推进世界 n 个 tick
  - `apply_action(action)`：校验并生成事件、更新状态
  - `query_observation(agent_id)`：生成该 Agent 可见信息
  - `snapshot()` / `restore_from_snapshot(...)`：快照与恢复
  - `save_to_dir(path)` / `load_from_dir(path)`：落盘与冷启动恢复
  - `replay_from_snapshot(snapshot, journal)`：从快照回放后续事件形成分叉
- **Agent Runtime**
  - `register_agent(agent_spec)`：注册/加载 Agent
  - `tick(agent_id)`：为 Agent 提供 observation，获取 action（或行动计划），提交到世界
  - `throttle(policy)`：速率限制、预算控制（token/步数/事件量）

### M3 Agent 接口（已实现）
- **AgentBehavior trait**：Agent 行为的核心抽象
  - `agent_id()`：返回 Agent 的唯一标识符
  - `decide(observation) -> AgentDecision`：基于观察做出决策
  - `on_action_result(result)`：行动结果回调（可选）
  - `on_event(event)`：事件通知回调（可选）
- **AgentDecision**：Agent 决策类型
  - `Act(Action)`：执行一个行动
  - `Wait`：本轮跳过
  - `WaitTicks(n)`：等待 n 个 tick
- **AgentRunner<B: AgentBehavior>**：多 Agent 调度器
  - `register(behavior)`：注册 Agent
  - `register_with_quota(behavior, quota)`：注册带配额的 Agent
  - `tick(kernel) -> Option<AgentTickResult>`：执行一轮 observe → decide → act
  - `run(kernel, max_ticks)`：运行指定数量的 tick
  - `run_until_idle(kernel, max_ticks)`：运行直到所有 Agent 空闲
- **RegisteredAgent<B>**：已注册 Agent 的状态跟踪
  - `wait_until`：等待到期时间
  - `action_count` / `decision_count`：统计信息
  - `quota`：可选的 Agent 配额
  - `rate_limit_state`：限速状态

### M3 调度器：公平性、限速、配额（已实现）
- **公平调度**：Round-Robin 轮转调度，确保多 Agent 公平执行
- **配额系统 (AgentQuota)**：
  - `max_actions`：限制 Agent 可执行的动作总数
  - `max_decisions`：限制 Agent 可做出的决策总数
  - 支持全局默认配额和 Agent 级别的独立配额
  - `is_quota_exhausted(agent_id)`：检查 Agent 是否已耗尽配额
- **限速系统 (RateLimitPolicy)**：
  - `max_actions_per_window`：每个时间窗口内的最大动作数
  - `window_size_ticks`：时间窗口大小（tick 数）
  - 基于滑动窗口的限速控制
  - `is_rate_limited(agent_id, now)`：检查 Agent 是否被限速
  - `reset_rate_limit(agent_id)`：重置 Agent 的限速状态
- **调度过滤**：在每个 tick 中自动跳过已耗尽配额或被限速的 Agent

### M3 可观测性（已实现）
- **RunnerMetrics**：运行时统计数据
  - `total_ticks`：执行的 tick 总数
  - `total_agents`：注册的 Agent 数量
  - `agents_active`：活跃 Agent 数量（未耗尽配额、未被限速）
  - `agents_quota_exhausted`：已耗尽配额的 Agent 数量
  - `total_actions` / `total_decisions`：总动作/决策数
  - `actions_per_tick` / `decisions_per_tick`：每 tick 平均动作/决策数
- **AgentStats**：单个 Agent 的统计信息
  - `action_count` / `decision_count`：动作/决策计数
  - `is_quota_exhausted`：是否已耗尽配额
  - `wait_until`：等待到期时间
- **RunnerLogEntry / RunnerLogKind**：事件日志类型
  - `AgentRegistered` / `AgentUnregistered`：Agent 注册/注销
  - `AgentDecision`：Agent 决策
  - `ActionExecuted`：动作执行结果
  - `AgentSkipped`：Agent 被跳过（含原因）
  - `QuotaExhausted` / `RateLimited`：配额/限速事件
  - `MetricsSnapshot`：指标快照

### M3 Agent 记忆系统（已实现）
- **ShortTermMemory**：短期记忆缓冲区
  - 固定容量的 FIFO 队列
  - 支持按时间/重要性筛选
  - `add(entry)`：添加记忆条目
  - `recent(n)`：获取最近 N 条记忆
  - `since(time)`：获取指定时间后的记忆
  - `important(threshold)`：获取重要性超过阈值的记忆
  - `summarize(max_entries)`：生成上下文摘要
- **LongTermMemory**：长期记忆存储
  - 支持标签和内容搜索
  - 按重要性自动淘汰（容量限制时）
  - `store(content, time)`：存储记忆
  - `search_by_tag(tag)`：按标签搜索
  - `search_by_content(query)`：按内容搜索（子串匹配）
  - `top_by_importance(n)`：获取最重要的 N 条
- **AgentMemory**：组合记忆系统
  - `record_observation/decision/action_result/event/note`：便捷记录方法
  - `consolidate(time, threshold)`：将短期重要记忆转存到长期
  - `context_summary(max_recent)`：获取决策上下文摘要
- **MemoryEntry / MemoryEntryKind**：记忆条目类型
  - `Observation`：观察记录
  - `Decision`：决策记录
  - `ActionResult`：动作结果
  - `Event`：外部事件
  - `Note`：自定义笔记

## 里程碑
- M0：对齐愿景与边界（本设计文档 + 项目管理文档）
- M1：世界内核最小闭环（时间、地点、移动、基础事件、可恢复）
- M2：持久化与回放（快照/事件日志、确定性/随机种子管理）
- M3：Agent SDK 与运行时（调度、限速、可观测性、失败处理）
- M4：最小社会与经济（工作/生产/交易/关系/声誉；核心为 WASM 动态调用系统，Agent 创造的模块以 Rust 编写并编译为 WASM，通过事件/接口影响世界）
- M5：可视化与调试工具（世界面板、事件浏览、回放、指标）

## 风险
- **真实性与可计算性冲突**：规则越真实成本越高；需要阶段性抽象（先“像”再“真”）。
- **持久化膨胀**：事件日志增长快；需要快照、压缩、归档策略。
- **涌现不可控**：可能出现资源锁死、单点垄断、恶性循环；需要治理规则与监控指标。
- **一致性与确定性**：并发与随机性会破坏可回放；需要调度策略与随机源管理。
- **安全与滥用**：Agent 可能生成不当内容或策略；需要内容过滤、权限边界与审计。
