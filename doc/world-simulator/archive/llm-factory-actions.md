> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# LLM 工厂动作接入（llm_bootstrap）

## 目标
- 在 `llm_bootstrap` 场景中补齐 LLM 可直接调用的工业动作：`build_factory`、`schedule_recipe`。
- 让 Agent 不再只能“辐射采集/资源精炼”，而是能执行“建厂 -> 排产 -> 产出制成品（最小口径）”。
- 保持现有 LLM 对话闭环机制（含 `execute_until`、trace、report）可复用。

## 范围
- In Scope:
  - 扩展 `simulator::Action` 与 LLM decision 解析，接入 `build_factory`、`schedule_recipe`。
  - 扩展 simulator 世界状态，记录已建工厂。
  - 在 kernel 中实现建厂/排产执行与校验（资源消耗、厂存在性、配方支持、产出写回）。
  - 扩展 prompt schema 与测试，移除“当前动作集不支持建厂”的限制文案。
- Out of Scope:
  - 不切换到 runtime M4 完整模块链路（`BuildFactoryWithModule` / `ScheduleRecipeWithModule` 不在本次范围）。
  - 不重构 viewer 渲染层与面板布局。

## 接口 / 数据

### 1) LLM decision JSON 扩展
- `build_factory`
  - 字段：
    - `owner`: `self | agent:<id> | location:<id>`（缺省 `self`）
    - `location_id`: 工厂站点 location
    - `factory_id`: 工厂唯一 id
    - `factory_kind`: 工厂类型（示例：`factory.assembler.mk1`，缺省该值）
- `schedule_recipe`
  - 字段：
    - `owner`: `self | agent:<id> | location:<id>`（缺省 `self`）
    - `factory_id`: 已建工厂 id
    - `recipe_id`: 配方 id
    - `batches`: 生产批次数（正整数，缺省 1）

### 2) Kernel 执行语义（simulator）
- `BuildFactory`：
  - 校验 owner/site/factory 唯一性与 owner-site 共址约束。
  - 消耗配置化建厂成本（电力+硬件）。
  - 记录工厂到 `WorldModel.factories`，发出 `FactoryBuilt` 事件。
- `ScheduleRecipe`：
  - 校验工厂存在、owner 匹配、共址约束、配方合法性、批次数合法。
  - 按配方批量消耗资源（电力+硬件），产出制成品口径资源（写入 `data`）并发出 `RecipeScheduled` 事件。

### 3) 配置扩展（EconomyConfig）
- 新增：
  - `factory_build_electricity_cost`
  - `factory_build_hardware_cost`
  - `recipe_electricity_cost_per_batch`
  - `recipe_hardware_cost_per_batch`
  - `recipe_data_output_per_batch`

## 里程碑
- LFA0：文档立项与任务拆解。
- LFA1：动作/状态/事件接线完成（含 replay）。
- LFA2：LLM 解析 + prompt + 单测回归完成。
- LFA3：`llm_bootstrap` 在线闭环复跑完成并记录 TODO。

## 风险
- 语义风险：simulator 版建厂/排产是“最小语义”，与 runtime M4 全量链路不完全等价。
- 行为风险：模型可能直接排产但资源不足，需依赖拒绝原因回灌与 prompt 约束引导。
- 兼容风险：新增 action/event 会影响序列化与回放，需补 replay 覆盖测试。
