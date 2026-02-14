# M4 社会经济系统：工业链路与 WASM 模块化（Recipe/Product/Factory）

## 目标

构建一套可演化的 M4 社会经济机制，满足以下约束：
- 从基础资源采集到多级制成品的完整闭环。
- 生产能力不预置，必须通过“先造设备再扩产”的方式逐步建立工业体系。
- 每个**配方**、**制成品**、**工厂**均可由独立 WASM 模块定义，并通过统一接口接入。
- 玩家或 AI 可以提交新模块，经过治理后动态扩展经济系统。

该设计参考《异星工厂》与《工业巨头》的核心体验：
- 资源链分层与瓶颈驱动。
- 产能建设先于利润释放。
- 物流、能耗、设备维护共同决定经济效率。

## 范围

### In Scope（M4-ECO V1）
- 资源分层与制成品分类（基础资源 -> 中间品 -> 终端制成品）。
- 工厂分级建造机制（设施建造依赖上一级产物与能力）。
- 配方执行模型（输入/输出/时长/能耗/副产物/质量）。
- Recipe/Product/Factory 的 WASM 模块接口与数据契约。
- 模块治理、兼容版本、审计与确定性约束。
- 面向后续扩展的事件与动作协议。

### Out of Scope（V1 不做）
- 全量市场金融系统（期货、信用衍生品、复杂税制）。
- 超大规模自动物流寻路优化（仅定义接口，不落地算法细节）。
- 多世界跨服贸易清算。

## 设计原则

- 最小可信内核：内核只负责账本、不变量、审计；经济语义由模块提供。
- 产能先行：制成品产出必须依赖已建成且在线的工厂能力。
- 确定性优先：同输入同种子必须得到同样生产结果，支持回放与审计。
- 插件优先：新增“配方/制成品/工厂”无需改内核代码，仅新增 WASM 模块。
- 渐进复杂度：先最小闭环，再扩展质量、维护、自动化、品牌、合同等机制。

## 机制总览

每个 tick 的经济执行按固定顺序进行：
1. 资源输入确认：检查库存、能量、工厂可用产能。
2. 配方求值：Recipe 模块计算本批次可执行量、损耗与副产物。
3. 工厂约束：Factory 模块裁剪吞吐（槽位、效率、维护状态、功率上限）。
4. 制成品约束：Product 模块校验产物属性（质量、保质期、堆叠规则）。
5. 账本提交：原料扣减、产物入库、副产物处理、事件落盘。

## 资源与制成品层级

| 层级 | 类别 | 示例 | 主要来源 | 主要去向 |
| --- | --- | --- | --- | --- |
| T0 | 基础资源 | 矿石、冰、硅酸盐、碳质块 | 开采/采集 | T1 初加工 |
| T1 | 初级加工品 | 金属锭、纯水、聚合前体、晶圆坯 | 熔炼/提纯/裂解 | T2 部件加工 |
| T2 | 标准部件 | 齿轮、管线、线缆、基板、电池单元 | 机加工/化工线 | T3 功能组件 |
| T3 | 功能组件 | 电机、控制器、传感器、动力包 | 组装车间 | T4 终端制成品 |
| T4 | 终端制成品 | 工业机器人、运输无人机、模块机柜 | 总装线 | 工厂升级/市场交易 |
| T5 | 基础设施件 | 工厂核心、物流节点、能源站套件 | 专项配方 | 新工厂建造与扩建 |

## 工厂渐进建造机制

工业能力必须按阶段解锁，不允许“开局全工厂”。

### 阶段 S0：生存级加工
- 可用设施：便携拆解器、手工装配台。
- 能力：T0 -> 少量 T1/T2。
- 目标：制造第一批固定式矿机与熔炉组件。

### 阶段 S1：基础采炼
- 新工厂：采矿站、熔炼炉、基础仓储。
- 解锁条件：完成 `factory.miner.mk1` 与 `factory.smelter.mk1` 建造配方。
- 能力：稳定产出金属锭、基础构件，形成持续性原料供给。

### 阶段 S2：化工与材料
- 新工厂：化工反应器、精炼站。
- 解锁条件：具备持续电力 + 压力容器 + 控温部件。
- 能力：T1 -> T2（聚合材料、液体化学品、功能介质）。

### 阶段 S3：精密制造
- 新工厂：精密机加工中心、电子装配线。
- 解锁条件：高纯材料、稳定能源、基础自动化控制单元。
- 能力：T2 -> T3（电机、控制器、传感器）。

### 阶段 S4：系统总装
- 新工厂：系统总装厂、质量检测站。
- 解锁条件：多工厂协同能力与供应链稳定度阈值。
- 能力：T3 -> T4/T5（终端制成品与下一阶段工厂核心件）。

## 配方执行机制

每个配方由独立 Recipe 模块定义，最小执行要素：
- 输入：多种原料与最小批量。
- 输出：主产物与副产物。
- 周期：每批次生产时长（tick）。
- 能耗：静态功耗 + 批次功耗。
- 工厂要求：允许执行的工厂标签/等级。
- 环境要求：温度、压力、辐射等可选约束。

标准吞吐计算（建议口径）：

`effective_batches = floor(base_batches * factory_efficiency * power_factor * maintenance_factor * operator_factor)`

质量计算（可选）：

`quality_score = base_quality + factory_bonus + material_bonus + stochastic_term(seed)`

其中 `seed` 必须来源于可回放上下文（world seed + event id），禁止真实随机源。

## 接口 / 数据

### 1) 统一模块分类

- `Recipe` 模块：定义“如何把输入加工成输出”。
- `Product` 模块：定义“产物属性与存储/交易行为”。
- `Factory` 模块：定义“设施建造规则与产线能力边界”。

模块命名建议：
- `m4.recipe.<chain>.<name>`
- `m4.product.<category>.<name>`
- `m4.factory.<tier>.<name>`

### 2) 核心结构（接口草案）

```rust
pub enum EconomyModuleKind {
    Recipe,
    Product,
    Factory,
}

pub struct MaterialStack {
    pub kind: String,
    pub amount: i64,
}

pub struct RecipeModuleSpec {
    pub recipe_id: String,
    pub display_name: String,
    pub inputs: Vec<MaterialStack>,
    pub outputs: Vec<MaterialStack>,
    pub byproducts: Vec<MaterialStack>,
    pub cycle_ticks: u32,
    pub power_per_cycle: i64,
    pub allowed_factory_tags: Vec<String>,
    pub min_factory_tier: u8,
}

pub struct ProductModuleSpec {
    pub product_id: String,
    pub display_name: String,
    pub category: String,
    pub stack_limit: u32,
    pub decay_per_tick_bps: u32,
    pub quality_levels: Vec<String>,
    pub tradable: bool,
}

pub struct FactoryModuleSpec {
    pub factory_id: String,
    pub display_name: String,
    pub tier: u8,
    pub tags: Vec<String>,
    pub build_cost: Vec<MaterialStack>,
    pub build_time_ticks: u32,
    pub base_power_draw: i64,
    pub recipe_slots: u16,
    pub throughput_bps: u32,
    pub maintenance_per_tick: i64,
}
```

### 3) 运行时请求/响应（接口草案）

```rust
pub struct RecipeExecutionRequest {
    pub recipe_id: String,
    pub factory_id: String,
    pub desired_batches: u32,
    pub available_inputs: Vec<MaterialStack>,
    pub available_power: i64,
    pub deterministic_seed: u64,
}

pub struct RecipeExecutionPlan {
    pub accepted_batches: u32,
    pub consume: Vec<MaterialStack>,
    pub produce: Vec<MaterialStack>,
    pub byproducts: Vec<MaterialStack>,
    pub power_required: i64,
    pub duration_ticks: u32,
    pub reject_reason: Option<String>,
}

pub struct FactoryBuildRequest {
    pub factory_id: String,
    pub site_id: String,
    pub builder: String,
    pub available_inputs: Vec<MaterialStack>,
    pub available_power: i64,
}

pub struct FactoryBuildDecision {
    pub accepted: bool,
    pub consume: Vec<MaterialStack>,
    pub duration_ticks: u32,
    pub reject_reason: Option<String>,
}
```

### 4) Rust Trait 约定（模块作者侧）

```rust
pub trait RecipeModuleApi {
    fn describe_recipe(&self) -> RecipeModuleSpec;
    fn evaluate_recipe(&self, req: RecipeExecutionRequest) -> RecipeExecutionPlan;
}

pub trait ProductModuleApi {
    fn describe_product(&self) -> ProductModuleSpec;
    fn validate_product_state(&self, current: &MaterialStack) -> Result<(), String>;
}

pub trait FactoryModuleApi {
    fn describe_factory(&self) -> FactoryModuleSpec;
    fn evaluate_build(&self, req: FactoryBuildRequest) -> FactoryBuildDecision;
}
```

## 动作与事件协议（草案）

### 动作
- `action.economy.build_factory`
- `action.economy.schedule_recipe`
- `action.economy.transfer_inventory`
- `action.economy.maintain_factory`

### 事件
- `domain.economy.factory_built`
- `domain.economy.recipe_started`
- `domain.economy.recipe_completed`
- `domain.economy.product_quality_assessed`
- `domain.economy.factory_degraded`

所有事件要求可回放，且由模块输出到统一审计链路。

## 模块治理与兼容

- 模块提交：玩家/AI 提交 `wasm_hash + manifest + spec`。
- 治理流程：`propose -> shadow -> approve -> apply`。
- 兼容策略：
  - ABI 版本：`wasm-1`（底层）+ `economy-v1`（领域层）。
  - 向后兼容字段仅新增不破坏；破坏性变更通过新版本模块 ID 发布。
- 安全约束：
  - 禁止模块直接写账本；只能输出意图，由内核做最终提交。
  - 禁止真实时间和系统随机数；使用上下文种子。
  - 输出大小、effect/emits 数量受 `ModuleLimits` 限制。

## 测试与验收

V1 需要覆盖以下测试组：
- 接口契约测试：Recipe/Product/Factory 结构体序列化反序列化稳定。
- 决策一致性测试：同输入与同 seed 的执行计划一致。
- 账本守恒测试：输入扣减与输出增加严格平衡（含副产物）。
- 工厂门槛测试：未达工厂等级或标签不匹配时配方必须拒绝。
- 构建链路测试：S0 -> S4 解锁顺序可重复通过。

## 里程碑

- M4-E1：完成机制与接口设计（本文件 + 项目文档）。
- M4-E2：落地 ABI 数据结构与基础测试。
- M4-E3：接入 runtime 动作/事件最小闭环（build_factory/schedule_recipe）。
- M4-E4：完成首批内置示例模块（最少 6 配方、4 制成品、3 工厂）。
- M4-E5：开放玩家/AI 自定义模块接入与治理模板。

## 风险

- 复杂度激增：配方树与工厂约束会迅速膨胀，需要阶段化范围控制。
- 模块质量参差：外部模块可能性能差或语义冲突，需要 shadow 校验与评分机制。
- 平衡性风险：高阶配方收益过高会导致经济塌缩，需要参数治理。
- 性能风险：模块数量增加会拉长 tick 时延，需要缓存与调用预算。
- 可解释性风险：若缺少标准事件与诊断字段，难以定位产能瓶颈。
