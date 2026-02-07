# Agent World：足够真实且持久的世界模拟器（设计文档）

## 目标
- 构建一个**足够真实（可解释、可推演）且持久（可恢复、可长期演化）**的世界模拟器。
- 世界里的所有参与者都是 **AI Agent**，每个 Agent 是独立个体：拥有身份、需求、资源、记忆、关系、目标与行为风格，并能在世界规则下持续行动。
- 让“世界”成为第一性：Agent 不是脚本驱动的剧情工具，而是生活在一个可持续运转的系统里；世界会在没有人为干预时也能继续演化。
- 不追求物理写实（不过度模拟低层物理细节），但追求符合文明发展规律的抽象：在抽象层保持资源/制度/信息约束的一致性，让资源与制度的约束自然驱动协作、分工、交易、治理与冲突等涌现行为。

## 关键设定
- Agent **不是人类**，而是一种“硅基文明”：不需要吃饭/睡觉，但需要持续的**硬件**、**电力**与**数据**供给（以及由此衍生的算力、存储、带宽等约束）。
- 世界存在**物理空间**的概念：一个可配置的**破碎小行星带**，默认尺寸 **100 km × 100 km × 10 km**（长×宽×高），为三维盒状空间。
- 不追求物理写实：只在抽象层表达空间约束（位置、距离、连通性、移动成本），不模拟精细连续物理与复杂动力学。
- 为方便模拟，空间长度的最小单位为 **1 cm**：世界中所有“长度/距离/尺寸”类数值都应以 cm 为离散粒度（必要时在输出层再换算 m/km）。
- 每个 Agent 的初始形态为**身高约 1 m 的人形机器人**（作为默认机体规格，可被升级/改造扩展）。
- 破碎小行星带由直径 **500 m-10 km** 的小行星碎片构成；**多数碎片富含放射性**。
- 硅基生命依赖放射性物质供能：通过吸收辐射产生电力；每个 Agent 出厂自带“辐射能→电能”转换模块（规则层抽象为“辐射采集 → 电力资源”）。
- **自由沙盒**：Agent 可以把“新事物”封装为 Rust/WASM 模块动态接入世界，模块仅通过事件/接口与外部交互。
- **LLM 驱动**：实际运行中 Agent 的决策由 LLM 驱动，推理服务采用 **OpenAI 兼容 API** 形式提供（支持配置 endpoint/model/鉴权/预算策略）。
- **模块化 Agent**：除位置/资源/基础物理外，Agent 的记忆/规划/工具等内部能力原则上由 WASM 模块定义，可由 Agent 自主迭代更新。
- 技术上参考 **AgentOS**：确定性内核 + 控制面 IR（AIR）+ WASM 模块 + Effect/Receipt + Capability/Policy。
- 工程实现采用 **Rust workspace**（Cargo 管理），核心库位于 `crates/agent_world/`。

### 破碎小行星带物理细化（设计版）
> 目标：在不引入连续物理模拟的前提下，尽量保持物理量纲与因果的正确性。

- **空间边界与分层**
  - 世界为三维盒状空间（`width/depth/height`，默认 100km×100km×10km）。
  - 越界默认拒绝（当前实现），可选边界条件：反射、环绕或“失联”状态。
  - 可选碎片带分层：中间层密度高、上下层稀薄（简化为 `density(z)` 曲线）。

- **碎片分布与材质**
  - 碎片尺寸可按幂律分布（常见天体碎屑场）：`N(r) ~ r^-q`（q≈2.5~3.5），直径范围 **500 m-10 km**（对应半径 250 m-5 km，配置项仍用 `radius_cm`）。
  - 碎片之间最小间距为 **500 m**（生成器需保证，不做精细轨道模拟）。
  - 材质比例可配置（硅酸盐/金属/冰/碳基/复合），影响辐射衰减、热容量、磨损速率。
  - `LocationProfile` 预留扩展字段（设计层）：`density`, `albedo`, `porosity`,
    `temperature`, `radioisotope_ratio`, `hazard_level`，用于精细化热与磨损计算。

- **辐射场与衰减**
  - 基本规律：辐射强度随距离平方衰减（`I ~ 1/r^2`）。
  - 介质吸收：`I = I0 * exp(-tau)`，`tau = k * density * r`（碎片带介质吸收）。
  - 计算简化：采用“近邻 + 体素背景”两级模型：  
    - **近邻**：采集位置附近若干 Location 的局部强度（近场贡献）。  
    - **背景**：体素网格中汇总的辐射场（远场贡献，O(1) 查询）。

- **能量采集与效率**
  - 采集量上限可由“收集面积/效率/热约束”决定：  
    `harvest <= I * area * efficiency * dt`，并受 `max_harvest_per_tick` 限制。
  - 吸收辐射产生热量，需散热；过热触发降效或硬件损耗（见“热管理与硬件损耗”）。

- **运动与推进（抽象）**
  - 模型层可抽象为：`move_cost = f(distance, mass, thrust_efficiency)`，并设置 `max_accel`。
  - 采用“能量/距离成本”抽象；推进方式不细化，统一以电力消耗与上限约束处理。
  - 若启用质量模型，则将机体质量与推进效率纳入计算。

- **碰撞与侵蚀（可选）**
  - 碎屑与微尘造成的侵蚀可映射为硬件耐久度缓慢下降（维护成本）。
  - 高速移动或高密度区域可提高磨损率（风险/成本选择）。

- **时间尺度与量纲约定（可选）**
  - `tick` 可映射到真实时间尺度（如 1s/10s/1min），便于推导能量与速率。
  - 电力单位可映射为焦耳或瓦时（例如 1 电力单位 ≈ 1 kJ），用于计算消耗与产能上限。
  - 位置单位为 cm，速度/加速度可用 `cm/tick` 与 `cm/tick^2` 表示。

- **推进与能量预算（近似）**
  - 线性能耗模型等效于“恒定推力 + 恒定效率”近似：  
    `E_move ≈ k * distance`，用于替代复杂动力学。
  - 每 tick 运动学约束：
    - `distance_cm <= max_move_distance_cm_per_tick`
    - `ceil(distance_cm / max(time_step_s, 1)) <= max_move_speed_cm_per_s`
  - 若引入质量 `m` 与最大加速度 `a_max`，可用  
    `v_max ≈ sqrt(2 * a_max * distance)` 作上限校验。

- **辐射采集的安全阈值（可选）**
  - 高辐射区域可引入“硬件损耗阈值”：超过阈值的采集会增加维护成本。
  - 可用“最大允许剂量率”作为规则：超出则禁止采集或强制降效。

- **参数定义（关键物理参数）**
  - `time_step_s`：每 tick 的真实时间。
  - `power_unit_j`：电力单位对应的能量（J）。
  - `max_move_distance_cm_per_tick`：每 tick 最大位移（超限移动直接拒绝）。
  - `max_move_speed_cm_per_s`：移动速度上限（按 `required_speed = ceil(distance_cm / time_step_s)` 计算）。
  - `radiation_floor`：外部背景辐射通量（开放系统输入，不来自本地碎片库存）。
  - `radiation_floor_cap_per_tick`：背景通量每 tick 可采集上限（防止 floor 配置过大导致“无源高功率造能”）。
  - `radiation_decay_k`：辐射介质吸收系数（影响 `exp(-tau)`）。
  - `max_harvest_per_tick`：每 tick 最大采集量（热/面积限制）。
  - `thermal_capacity`：热容量阈值（超过进入过热区）。
  - `thermal_dissipation`：每 tick 热量散逸值。
  - `heat_factor`：单位采集量带来的热增量。
  - `erosion_rate`：碎屑侵蚀系数（与速度/密度缩放）。

- **参数草案（更具体的默认值与范围）**
  - `time_step_s`：默认 10s；范围 1s~60s。
  - `power_unit_j`：1 电力单位≈1 kJ（默认），范围 0.1~10 kJ。
  - `max_move_distance_cm_per_tick`：默认 1,000,000 cm（10 km）/ tick；范围 100~5,000,000 cm。
  - `max_move_speed_cm_per_s`：默认 100,000 cm/s（1 km/s）；范围 100~500,000 cm/s。
  - `radiation_floor`：默认 1 单位/ tick；范围 0~10（解释为外部背景通量强度）。
  - `radiation_floor_cap_per_tick`：默认 5 单位/ tick；范围 0~50（建议不高于 `max_harvest_per_tick`）。
  - `radiation_decay_k`：默认 1e-6（以 `cm^-1` 表示）；范围 1e-7~1e-4。
  - `max_harvest_per_tick`：默认 50；范围 1~500（受热/面积限制）。
  - `thermal_capacity`：默认 100（抽象热容量）；范围 10~1000。
  - `thermal_dissipation`：默认 5/tick；范围 1~50。
  - `erosion_rate`：默认 1e-6（随速度/密度缩放）；范围 1e-7~1e-4。

- **辐射采集规则细化（可落地规则）**
  - 可采集量：`harvest = min(max_amount, max_harvest_per_tick, local_radiation)`
  - 场强估计：`local_radiation = near_sources + background + floor_contribution`
  - 背景限幅：`floor_contribution = min(radiation_floor, radiation_floor_cap_per_tick)`
  - 采集副作用：产生热量 `heat += harvest * heat_factor`（默认 `heat_factor=1`）。
  - 超温处理：若 `heat > thermal_capacity`，则采集效率下降或动作被拒绝。

- **热管理与硬件损耗（规则建议）**
  - 每 tick 热量衰减：`heat = max(0, heat - thermal_dissipation)`
  - 超温惩罚（任选其一）：  
    - 降效：`harvest *= clamp(thermal_capacity / heat, 0.1..1.0)`  
    - 损耗：`hardware -= (heat - thermal_capacity) * damage_factor`

- **碎片分布生成器（建议草案）**
  - 空间分块为体素（如 1km^3），每块生成碎片数 `n ~ Poisson(lambda * density)`
  - 密度场：`density = base * (1 + cluster_noise) * layer(z)`  
    - `cluster_noise`：多尺度噪声，形成团簇/空洞  
    - `layer(z)`：中间层高、上下层低
  - 尺寸分布：`radius_cm ~ PowerLaw(q)`，并裁剪在 `[r_min, r_max]`（对应直径 **500 m-10 km**）。
  - 材质分布：按权重抽样（Silicate/Metal/Ice/Carbon/Composite）

- **进一步设计方向（可直接落地的规则描述）**
  - **障碍/空洞生成**：在体素层引入“屏蔽/空洞 mask”，使用低频噪声+阈值形成不可通行区。
  - **辐射场缓存**：每体素维护 `radiation_background`，当碎片列表更新时重建；采集查询时取周围 3×3×3 体素均值。
  - **机体参数**：在 `RobotBodySpec` 抽象 `mass_kg`, `thrust_limit`, `heat_capacity`,
    `surface_area_cm2`，用于能耗/热管理/推进约束。
  - **同位素衰变**：`emission(t) = emission0 * 2^(-t / half_life)`，以 tick 为时间基准；
    可用 `half_life_ticks` 表示。

## 范围

### In Scope（第一阶段）
- **世界内核（World Kernel）**：时间推进、事件队列、规则校验、状态更新（可审计）。
- **持久化（Persistence）**：世界状态可落盘、可恢复；支持快照 + 增量事件（事件溯源可选）。
- **Agent 运行时（Agent Runtime）**：多 Agent 调度、限速/配额、可暂停/恢复、可回放。
- **自由沙盒扩展点**：WASM 模块动态接入的最小接口（由事件/规则/能力边界驱动）。
- **感知-决策-行动闭环**：
  - 感知（Observations）：世界对 Agent 的可见部分（部分信息、带噪声/延迟可选）。
  - 行动（Actions）：受规则约束的原子动作（移动、交互、生产、交易、沟通）。
  - 反馈（Consequences）：行动的结果与副作用写入世界事件流。
- **最小社会系统**：地点、资源（电力/算力/存储/带宽/数据）、物品/资产、任务/工作、简单交易、基础关系与声誉。

### Out of Scope（第一阶段不做）
- 复杂连续物理（刚体、流体等）与高精度地理系统。
- “全知叙事”式的强剧情主线（优先涌现而非编排）。
- 大规模 3D 渲染与沉浸式客户端（先做可视化/观测面板即可）。
- 完整的 WASM 编译链、模块市场与跨世界传播机制（仅定义模块元数据与版本字段，不实现分发/市场）。

## 接口 / 数据

### 核心概念
- **WorldTime**：单调递增时间（tick 或离散时间片），支持加速/暂停。
- **Entity**：世界中可持久化的对象（Agent、地点、物品、设施、合约…）。
- **Event**：世界状态变化的事实记录（可回放、可审计）。
- **Rule**：对 Action 的校验与约束（权限、资源、冷却、失败原因）。
- **Observation**：对某个 Agent 的视角输出（有范围/权限/不确定性）。
- **GeoPos**：小行星带中的三维位置（`x_cm/y_cm/z_cm`），用于距离/可见性/移动等规则。
- **LengthCm**：以 cm 为单位的长度/距离（整数或可量化到 1 cm 的数值）。
- **LocationProfile**：碎片/地点的物理画像（材质、尺寸、辐射强度）。
- **WASM Module**：由 Agent 创造并编译的沙箱模块，用于扩展规则/设施/机制，也用于 Agent 内部能力（记忆/工具/策略）。
- **Sandbox**：提供隔离执行环境，不做内容/策略限制；资源预算与输出边界由运行时策略控制，外部影响仍需通过事件/接口产生。

### 数据模型（草案）
> 具体字段与类型由实现语言/存储决定；此处用于约束边界。

- `Agent`
  - `id`, `name`, `traits`（性格/偏好/风险偏好）, `needs`（电力/硬件健康/数据需求…）
  - `inventory`, `skills`, `relationships`, `reputation`
  - `memory`（短期工作记忆 + 长期记忆索引/摘要；可由 memory module 策略/版本控制）
  - `body`（默认：人形机器人，`height_cm = 100`）
  - `pos`（GeoPos）
  - `location_id`, `status`（在线/离线/休眠）
  - `thermal`（热状态：`heat`）
- `Location`
  - `id`, `type`, `connections`（图结构/道路）, `resources`（可采集/可交易）
  - `profile`（`material`, `radius_cm`, `radiation_emission_per_tick`）
  - `rules`（进入限制/营业时间/治安等）
- `Item / Resource`
  - `id`, `kind`, `quantity`, `quality`, `owner_id`
- `Action`
  - `actor_id`, `type`, `args`, `requested_at`
- `Event`
  - `event_id`, `time`, `type`, `payload`, `caused_by`（action_id/agent_id）
- `WorldConfig`（物理相关）
  - `space`（`width_cm/depth_cm/height_cm`，破碎小行星带空间尺寸）
  - `physics`（`time_step_s`, `power_unit_j`, `max_move_distance_cm_per_tick`,
    `max_move_speed_cm_per_s`, `radiation_floor`, `radiation_decay_k`,
    `max_harvest_per_tick`, `thermal_capacity`, `thermal_dissipation`, `heat_factor`,
    `erosion_rate`）
  - `asteroid_fragment`（小行星带碎片分布生成器参数：`base_density_per_km3`, `voxel_size_km`, `cluster_noise`,
    `layer_scale_height_km`, `size_powerlaw_q`, `radius_min_cm`, `radius_max_cm`,
    `min_fragment_spacing_cm`, `material_weights`）

### M1 行动规则（初版）
- **规则实现**：以下规则由 Rule Modules 以 WASM 实现，内核仅保留位置/资源/基础物理不变量校验。
- **时间推进**：每个 Action 处理会推进 1 tick；事件按队列顺序确定性处理。
- **移动成本**：`MoveAgent` 按**三维欧氏距离**计费，电力消耗 = `ceil(distance_km) * 1`（电力单位/公里）；若电力不足则拒绝。
- **移动约束**：移动到相同 `location_id` 视为无效动作并拒绝。
- **可见性**：`query_observation` 以固定可见半径输出可见 Agent/Location（默认 **100 km**）。
- **资源交互**：
  - 资源转移需要同地（Agent 与 Location 同处，或 Agent 与 Agent 同处）。
  - Location 与 Location 之间的直接转移不允许（需由 Agent 搬运）。
- **辐射采集**：
  - `HarvestRadiation` 允许 Agent 从所处 Location 的辐射强度中采集电力（抽象为电力资源的增加）。
  - 采集上限受 `max_harvest_per_tick` 与热管理约束影响；采集会增加热量。
- **配置参数（内核级）**：
  - `visibility_range_cm`（默认 `10_000_000`，即 **100 km**）
  - `move_cost_per_km_electricity`（默认 `1`，电力单位/公里）
  - `space`（小行星带空间尺寸：`width_cm/depth_cm/height_cm`）
  - `physics`（辐射/热/侵蚀参数）
  - `asteroid_fragment`（碎片分布生成器参数）

### M2 持久化与回放（最小）
- **快照**：保存世界内核的完整状态（时间、配置、世界模型、待处理队列、事件游标）。
- **日志**：追加式事件列表（Journal），与快照配合恢复。
- **存储布局**：目录内 `snapshot.json` + `journal.json`（JSON 格式）。
- **恢复语义**：加载快照与日志，校验 `journal_len` 一致后恢复内核。
- **回放/分叉**：允许以快照为起点回放 `journal_len` 之后的事件，形成新的内核实例（最小一致性校验）。
- **版本化与迁移**：
  - 快照与日志均包含 `version` 字段（当前 **v2**）。
  - 加载时校验版本；若版本不受支持则拒绝。
  - 预留迁移入口：当版本升级时，先将旧结构迁移到最新版本再恢复内核（当前仅支持 v1）。

### 运行时接口（草案）
- **World Kernel**
  - `step(n_ticks)`：推进世界 n 个 tick
  - `apply_action(action)`：校验并生成事件、更新状态
  - `query_observation(agent_id)`：生成该 Agent 可见信息
  - `snapshot()` / `restore_from_snapshot(...)`：快照与恢复
  - `save_to_dir(path)` / `load_from_dir(path)`：落盘与冷启动恢复
  - `replay_from_snapshot(snapshot, journal)`：从快照回放快照之后记录的事件形成分叉
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
> 现阶段为最小可用实现（Rust 内存结构）；实际运行设想的模块化记忆策略见下节。
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

### M3+ LLM 驱动与模块化记忆（设计补充）
> 描述实际运行设想，用于指导后续架构演进与实现拆分。

- **LLM 驱动**：Agent 的 `decide` 在运行时由 LLM 执行；推理服务通过 OpenAI 兼容 API 提供，需配置 model/endpoint/auth、超时/重试与 token/成本预算。
- **Memory Module（WASM + 受限存储）**：
  - 记忆策略封装为独立 WASM 模块，与行为模块解耦。
  - 模块拥有受限的持久存储配额（容量/条目数/索引大小），仅通过显式接口读写。
  - 模块负责接收 observation/event/action_result、压缩摘要、检索上下文、迁移短期→长期。
  - Agent 可替换/升级自己的 memory module（版本化、可回滚），以更新记忆策略。
- **全模块 WASM 化**：除世界位置/资源/基础物理外，Agent 的感知/规划/记忆/工具等模块均可由 WASM 定义；沙盒仅提供隔离与计量，不施加语义限制，所有外部影响仍由规则/事件边界约束。

## 里程碑
- M0：对齐愿景与边界（本设计文档 + 项目管理文档）
- M1：世界内核最小闭环（时间、地点、移动、基础事件、可恢复）
- M2：持久化与回放（快照/事件日志、确定性/随机种子管理）
- M3：Agent SDK 与运行时（调度、限速、可观测性、失败处理）
- M4：最小社会与经济（工作/生产/交易/关系/声誉；核心为**自由沙盒 + WASM 动态新事物**，Agent 创造的模块以 Rust 编写并编译为 WASM，通过事件/接口影响世界）
- M5：可视化与调试工具（世界面板、事件浏览、回放、指标；详见 `doc/world-simulator/visualization.md`）

## 分册索引
- 世界初始化：`doc/world-simulator/world-initialization.md`
- 可视化与调试：`doc/world-simulator/visualization.md`

## 背景故事物理一致性修订清单（2026-02-07）

本清单用于将“可解释抽象”进一步收敛到“更接近物理规律”的口径，优先处理**文档与实现冲突**和**明显违背量纲/守恒**的问题。

### P0（必须先完成）
- [x] **C1 尺寸口径统一**：统一“小行星碎片尺寸范围”的文档与默认配置。
  - 已完成：默认配置更新为 `radius_min_cm=25_000`、`radius_max_cm=500_000`（对应直径 500m-10km）。
  - 验收：`doc/*`、`README`、`WorldConfig::default().asteroid_fragment` 三处一致。
- [x] **C2 辐射源标度修订**：将发射强度与碎片尺度的关系从线性近似升级为可配置标度（建议默认接近体积标度）。
  - 已完成：`AsteroidFragmentConfig` 新增 `radiation_emission_scale` 与 `radiation_radius_exponent`，默认 `1e-9` 与 `3.0`。
  - 已完成：`estimate_radiation_emission` 改为 `emission = radius_cm^exponent * scale * material_factor`。
  - 验收：新增单测覆盖“半径翻倍时发射显著上升（立方标度）”。
- [x] **C3 采集场模型对齐**：将采集可用辐射从“局部近似”扩展为“近邻源 + 背景场”并包含距离项。
  - 已完成：`HarvestRadiation` 改为“近邻源贡献 + 远场背景 + radiation_floor”叠加模型。
  - 已完成：单源贡献包含距离项（几何衰减）与介质吸收项（`exp(-k*d)`）。
  - 验收：补充测试覆盖“多源叠加+距离衰减”与“零源仅背景辐射”两条路径。
- [x] **C4 背景辐射守恒说明**：为 `radiation_floor` 增加物理解释（外部背景通量）与上限策略，避免“无源无限造能”叙事。
  - 已完成：将 `radiation_floor` 定义为“外部背景辐射通量”（开放系统输入），不计入本地碎片守恒账本。
  - 已完成：新增 `radiation_floor_cap_per_tick`（默认 5），背景贡献按 `min(radiation_floor, radiation_floor_cap_per_tick)` 限幅。
  - 已完成：新增单测覆盖“零源场景下 floor 被 cap 限制”路径。
  - 验收：文档明确来源与边界；零源场景下采集行为符合预期。

### P1（建议尽快完成）
- [x] **C5 运动学约束补齐**：在“按距离计费”之外增加每 tick 最大位移/速度上限，避免瞬时跨域移动。
  - 已完成：`PhysicsConfig` 新增 `max_move_distance_cm_per_tick` 与 `max_move_speed_cm_per_s`，默认 10km/tick 与 1km/s（保守口径）。
  - 已完成：`MoveAgent` 增加“位移上限 + 速度上限”双重校验，超限返回明确拒绝原因。
  - 已完成：新增单测覆盖“超位移拒绝”与“超速度拒绝”路径，并为长距用例显式配置上限。
  - 验收：超限移动被拒绝或分段执行，回放结果确定。
- [ ] **C6 能耗参数重标定**：联动 `time_step_s`、`power_unit_j`、移动能耗系数，形成同一数量级口径。
  - 验收：给出默认参数推导示例（1m 机体在典型位移下的能耗区间）。
- [ ] **C7 热模型从常数散热升级**：将固定散热替换为“至少与温差相关”的可配置散热模型。
  - 验收：高热状态下散热更快，低热状态下散热更慢；不会出现负热量。

### P2（增强可信度）
- [ ] **C8 成分与放射性分布校准**：降低极端高放射性成分的默认占比，保留场景覆盖能力。
  - 验收：默认分布与“多数可采但非普遍高危”叙事一致。
- [ ] **C9 物理参数表固化**：在文档中固定每个关键参数的单位、推荐范围、调参影响。
  - 验收：新增参数时必须附带量纲与范围说明。

### 回归与验收测试清单
- [ ] **T1 单调性测试**：辐射随距离衰减、采集随过热降效、移动能耗随距离不减。
- [ ] **T2 守恒性测试**：资源账本满足 `0 <= remaining <= total`，加工/采集过程无凭空增益。
- [ ] **T3 一致性测试**：同 seed + 同动作序列在快照恢复与回放路径下结果一致。
- [ ] **T4 边界测试**：极端参数（最小/最大半径、spacing=0、超高密度）下系统不崩溃。

### 里程碑
- **PC1**：完成 P0（口径冲突消除 + 辐射采集模型对齐）。
- **PC2**：完成 P1（运动/能耗/热模型可解释且可测）。
- **PC3**：完成 P2（参数治理与可信度增强）。

## 风险
- **真实性与可计算性冲突**：规则越真实成本越高；需要阶段性抽象（先“像”再“真”）。
- **持久化膨胀**：事件日志增长快；需要快照、压缩、归档策略。
- **涌现不可控**：可能出现资源锁死、单点垄断、恶性循环；需要治理规则与监控指标。
- **一致性与确定性**：并发与随机性会破坏可回放；需要调度策略与随机源管理。
- **安全与滥用**：Agent 可能生成不当内容或策略；需要内容过滤、权限边界与审计。
