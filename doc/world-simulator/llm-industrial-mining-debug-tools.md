# LLM 工业采矿闭环与调试补给工具（设计文档）

## 目标
- 将 simulator 的工业链路从“可直接精炼硬件”升级为“先采矿再精炼再建厂/排产”的正确机制，避免开局直接进入建厂生产。
- 在 `llm_bootstrap` 中形成更真实的前置阶段：需要采矿与跨地点移动后，才能稳定进入工厂生产。
- 增加仅供 LLM 调试使用的补给工具：可向背包/位置注入任意资源数量，用于调试闭环与回归场景构造。

## 范围

### In Scope
- simulator 资源类型扩展：新增矿物中间资源（`compound`）。
- simulator 动作扩展：新增采矿动作（`mine_compound`）与调试补给动作（`debug_grant_resource`）。
- `refine_compound` 语义升级：必须消耗 `compound` 与电力，产出硬件。
- kernel/replay/event/LLM parser/prompt/tool schema 全链路接线。
- 仅在 LLM debug 模式暴露补给 tool（默认关闭）。
- 补齐 `test_tier_required` 单测与闭环回归。

### Out of Scope
- 不切换到 runtime M4 模块全链路（本次仍在 simulator 语义层闭环）。
- 不重构 viewer UI 渲染。
- 不引入外部调度服务与训练策略。

## 接口 / 数据

### 1) 资源与动作扩展
- `ResourceKind` 新增：
  - `compound`（单位：克，整数）。
- `Action` 新增：
  - `MineCompound { owner, location_id, compound_mass_g }`
  - `DebugGrantResource { owner, kind, amount }`

### 2) 采矿语义（生产机制）
- 采矿前置条件：
  - `compound_mass_g > 0`。
  - `owner` 与目标 `location_id` 共址。
  - 目标 location 具备 `fragment_budget`。
- 采矿成本与约束（配置化）：
  - `mine_electricity_cost_per_kg`：采矿电力成本。
  - `mine_compound_max_per_action_g`：单次采矿上限。
  - `mine_compound_max_per_location_g`：单 location 累计可采上限（用于迫使迁移采集）。
- 采矿结算：
  - 从 location `fragment_budget` 与 chunk budget 扣减元素质量。
  - 向 owner 增加 `ResourceKind::Compound`。
  - 记录 location 累计已采质量。

### 3) 精炼语义升级
- `RefineCompound` 在原电力成本基础上新增硬约束：
  - owner `compound` 库存必须 `>= compound_mass_g`。
  - 扣减 `compound_mass_g` 后再产出 `hardware`。
- 保持已有电力成本和硬件产率配置（向后兼容）。

### 4) 事件与回放
- `WorldEventKind` 新增：
  - `CompoundMined { owner, location_id, compound_mass_g, electricity_cost, extracted_elements }`
  - `DebugResourceGranted { owner, kind, amount }`
- `kernel/replay` 必须支持新增事件的确定性回放，保证快照/日志一致。

### 5) LLM Debug 工具（仅 debug 模式）
- 新增配置：
  - `AGENT_WORLD_LLM_DEBUG_MODE`（默认 `false`）。
- tool 暴露策略：
  - 默认仅保留现有 query tools + `agent_submit_decision`。
  - 当 `debug_mode=true` 时，追加 `agent_debug_grant_resource` tool。
- tool 参数：
  - `owner`（`self|agent:<id>|location:<id>`）
  - `kind`（资源类型）
  - `amount`（`>=1`）
- 安全约束：
  - 非 debug 模式下，即使模型构造该决策也应拒绝执行（解析/守卫层报错并回退）。

## 里程碑
- MMD0：设计文档与项目管理文档。
- MMD1：采矿与精炼机制落地（含 kernel/replay/tests）。
- MMD2：debug 补给 tool 仅在 debug 模式暴露（含 parser/schema/tests）。
- MMD3：`llm_bootstrap` 闭环复跑、文档/devlog 收口。

## 风险
- 兼容风险：新增 `ResourceKind` 与事件可能影响旧断言与日志消费方。
- 行为风险：location 采矿上限过低会导致策略卡死；过高会退化为“原地采矿”。
- 调试风险：debug tool 暴露边界不严会污染线上策略评估口径。
- 回归风险：LLM guardrail 与新动作并存可能引入未覆盖的决策重写路径。

## MMD5 增量优化（TODO-10 ~ TODO-13）

### 目标
- 提升 `llm_bootstrap` 闭环稳定性，减少 `schedule_recipe`/`move_agent` 高频拒绝，确保“所有工厂 + 所有制成品”目标可持续推进。
- 将“恢复链路”从单步参数猜测改为与经济约束一致的可执行策略，避免无效抖动。
- 在不依赖外部调度器前提下，通过 guardrail 和记忆实现 recipe 覆盖硬切换。

### 范围

#### In Scope
- TODO-10：新增 recipe 覆盖进度记忆（已完成/待完成），并在 `schedule_recipe` 守卫中对已覆盖配方执行硬切换。
- TODO-11：`schedule_recipe` 增加“工厂地点可达性”前置检查，已知工厂地点不共址时优先改写为移动动作。
- TODO-12：`move_agent` 增加分段规划守卫，单步距离超限时自动降级为中继点移动。
- TODO-13：`schedule_recipe -> refine_compound` 恢复质量与 `mine_compound` 单次上限对齐，必要时优先回退到 `mine_compound`。
- 补齐 `test_tier_required` 下的 LLM agent 单测与 prompt 断言。

#### Out of Scope
- 不改 kernel 原子动作语义（不调整真实经济参数与拒绝条件）。
- 不引入全局地图寻路模块（仅在 LLM guardrail 层做局部分段）。
- 不改 viewer 交互面板与可视化样式。

### 接口 / 数据
- `LlmAgentBehavior` 新增轻量运行态：
  - recipe 覆盖状态（completed/missing）。
  - 已知工厂位置映射（factory_id -> location_id）。
- `schedule_recipe` guardrail 扩展：
  - owner=self 且工厂已知不共址时，改写为 `move_agent`（必要时分段）。
  - 硬件不足恢复时按 `mine_compound_max_per_action_g` 限制恢复质量。
- `move_agent` guardrail 扩展：
  - 目标地点可见且距离超过 `max_move_distance_cm_per_tick` 时，改写到可达中继 location。
- prompt 观察上下文新增 recipe 覆盖摘要，用于提示模型主动切换未覆盖配方。

### 里程碑
- MMD5.1：文档更新与任务拆解。
- MMD5.2：TODO-11/12/13 守卫与测试落地。
- MMD5.3：TODO-10 覆盖记忆 + prompt 约束落地。
- MMD5.4：`llm_bootstrap` 在线抽样回归与文档/devlog 收口。

### 风险
- 守卫强改写风险：过度改写可能压制模型探索，导致局部最优。
- 位置映射风险：工厂位置缓存若不及时更新，可能导致错误回退。
- 分段移动风险：可见地点稀疏时中继选择失败，仍可能出现距离超限拒绝。
- 覆盖策略风险：硬切换可能降低单一配方产量，应限定为“未覆盖优先”而非“永久禁止重复”。

## MMD6 增量优化（TODO-14 ~ TODO-16）

### 目标
- 降低 `facility_not_found` 与 `facility_already_exists` 抖动，避免因 `factory_id` 歧义导致的重复建厂/排产失败。
- 在目标 location 不可见或上一轮移动已超距时，提供可执行分段回退，进一步压降 `move_distance_exceeded`。
- 为采矿动作补充“矿点可采量记忆”，在矿点耗尽后主动迁移或降级，降低重复 `mine_compound` 空耗。

### 范围

#### In Scope
- TODO-14：`factory_id` 归一化与去重 guardrail。
  - `schedule_recipe` 支持把“factory_kind 误填为 factory_id”映射到已知工厂 ID。
  - `build_factory` 在已知同 ID 工厂存在时改写为生产推进动作，避免重复建造拒绝。
- TODO-15：`move_agent` 超距回退强化。
  - 记录 `move_distance_exceeded` 目标；
  - 对“已知超距目标 + 不可见目标”应用探索式中继移动。
- TODO-16：采矿耗尽感知。
  - 记录 location 侧 `compound` 可用量（来自拒绝回执）；
  - `mine_compound` 质量按已知可用量裁剪；
  - 当目标点已知耗尽时改写为迁移或电力恢复动作。
- 补齐 `test_tier_required` 单测，并进行 `llm_bootstrap` 在线复核。

#### Out of Scope
- 不修改 kernel 真实拒绝规则与地图生成逻辑。
- 不引入全局寻路器或多步规划器。
- 不调整 recipe 配方数值平衡。

### 接口 / 数据
- `LlmAgentBehavior` 新增运行态记忆：
  - 工厂类型到已知工厂 ID 的映射（用于 `factory_id` 归一化）。
  - 超距移动目标集合（用于后续回退）。
  - 位置可采 `compound` 可用量提示（来自 `InsufficientResource` 回执）。
- guardrail 扩展：
  - `build_factory` 去重改写；
  - `schedule_recipe.factory_id` 归一化；
  - `move_agent` 对已超距目标启用探索中继；
  - `mine_compound` 位置前置检查、质量裁剪与耗尽回退。

### 里程碑
- MMD6.1：文档增量设计与任务拆解。
- MMD6.2：`factory_id` 归一化与建厂去重 guardrail + 单测。
- MMD6.3：移动超距回退增强 + 单测。
- MMD6.4：采矿耗尽感知 guardrail + 单测。
- MMD6.5：`test_tier_required` + 在线闭环抽样复核与文档收口。

### 风险
- 归一化风险：错误映射 factory_id 可能把动作导向非目标工厂，需要优先使用“明确已知映射”。
- 回退风险：探索式中继可能出现局部往返，需要保持“已超距目标”触发条件而非全局强制。
- 采矿记忆风险：可采量提示是时点信息，若 chunk 补给后未清理缓存，可能误判耗尽。
