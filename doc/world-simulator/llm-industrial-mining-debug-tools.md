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
