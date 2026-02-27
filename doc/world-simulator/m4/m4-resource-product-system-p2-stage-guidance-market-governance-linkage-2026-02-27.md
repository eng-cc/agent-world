# M4 资源与产品系统 P2：阶段引导与市场-治理联动观测（2026-02-27）

## 目标
- 在不改动作 ABI 的前提下，为工业链补齐“可解释的市场观测”和“阶段化进度”能力。
- 让配方排产在事件层可直接观察到材料侧成本信号，并显式体现治理税率的影响。
- 提供轻量阶段引导状态（`bootstrap/scale_out/governance`），用于后续 UI 与策略模块消费。

## 范围

### In Scope
- 扩展 `RecipeStarted` 事件：追加可选 `market_quotes`，用于输出材料侧行情快照。
- 在 `WorldState` 中新增工业阶段进度状态和观测缓存，并通过产业事件更新。
- 新增 `test_tier_required` 测试：
  - 治理税率变化影响 `market_quotes` 的有效成本指标。
  - 随着产业事件推进，阶段状态按预期从 `bootstrap` 进入 `scale_out/governance`。

### Out of Scope
- 不新增撮合交易动作，不引入复杂订单簿。
- 不改 viewer 交互（仅输出 runtime 可观测状态）。
- 不新增强制阶段闸门（仅观测与提示，不阻断动作执行）。

## 接口 / 数据

### 1) 事件扩展
- 位置：`runtime/events.rs::DomainEvent::RecipeStarted`
- 新增字段：
  - `market_quotes: Vec<MaterialMarketQuote>`（`serde(default)`）

`MaterialMarketQuote` 建议字段：
- `kind: String`
- `requested_amount: i64`
- `local_available_amount: i64`
- `world_available_amount: i64`
- `local_deficit_amount: i64`
- `transit_loss_bps: i64`
- `governance_tax_bps: u16`
- `effective_cost_index_ppm: i64`

### 2) 阶段状态
- 位置：`runtime/state.rs::WorldState`
- 新增状态：`industry_progress: IndustryProgressState`（`serde(default)`）

`IndustryProgressState` 建议字段：
- `stage: IndustryStage`
- `stage_updated_at: WorldTime`
- `completed_recipe_jobs: u64`
- `completed_material_transits: u64`
- `latest_market_quotes: BTreeMap<String, MaterialMarketQuote>`

`IndustryStage`：
- `bootstrap`
- `scale_out`
- `governance`

### 3) 阶段推进规则（首版）
- `bootstrap -> scale_out`：
  - `completed_recipe_jobs >= 3` 且 `factories.len() >= 1`
- `scale_out -> governance`：
  - 已满足 `scale_out`，且 `(electricity_tax_bps > 0 || data_tax_bps > 0)`，且
  - `completed_recipe_jobs >= 6` 或 `completed_material_transits >= 3`
- 推进单向，不自动回退。

## 里程碑
- P2-T0：设计文档 + 项目文档建档。
- P2-T1：事件与状态接线（market quote + industry progress）。
- P2-T2：补齐 required 单测并回归。
- P2-T3：文档与 devlog 收口。

## 风险
- 事件负载增长：`RecipeStarted` 载荷变大，可能影响日志体积。
- 参数偏差：成本指数与阈值设定不当会导致阶段推进过快或过慢。
- 回归风险：新增可选字段可能触发旧测试的事件断言差异。

缓解：
- 首版保持字段精简且 `serde(default)` 兼容。
- 阈值使用保守值，先验证“有信号”再精调。
- 对依赖时序的测试采用“推进至稳定态”断言，避免脆弱断言。
