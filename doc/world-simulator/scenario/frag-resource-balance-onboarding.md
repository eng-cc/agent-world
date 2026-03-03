# Agent World Simulator：Frag 资源平衡与新手友好生成（设计文档）

本分册定义一版可落地的 frag 资源生成改造，目标是同时满足两点：
1) 资源产出不出现明显“前期断供/后期通胀”；
2) 新手开局在默认场景下更稳定地获得可采集 frag。

## 目标
- 降低 chunk 级生成方差：避免默认参数下出现“部分 chunk 几乎无 frag”的冷启动断供。
- 增强新手可达性：让 chunk 中心区域更容易出现 frag，减少开局盲目搜索成本。
- 补齐运行期恢复节奏：当 chunk frag 数量低于上限时，按固定周期小步补种，避免一次性暴涨。
- 补齐资源类型空间分布：支持按区域偏置材质分布，形成可解释的“中心/边缘”资源差异。
- 保持确定性：同 `seed + config + chunk` 生成结果一致，便于回放与调试。
- 保持现有预算体系兼容：不改动 `total/remaining` 账本守恒规则。

## 范围

### In Scope
- 扩展 `AsteroidFragmentConfig`：
  - `min_fragments_per_chunk`：每 chunk 最低 frag 数量保底。
  - `starter_core_radius_ratio`：中心引导区半径占比（0~1）。
  - `starter_core_density_multiplier`：中心区密度倍率（>=1）。
  - `replenish_interval_ticks`：运行期补种周期（tick）。
  - `replenish_percent_ppm`：每周期补种比例（ppm，默认 1%）。
  - `material_distribution_strategy`：资源类型分布策略（均匀/分区偏置）。
- 改造 `generate_fragments`：
  - 在体素泊松采样基础上，叠加中心区密度倍率；
  - 采样结束后执行确定性保底补种，尽量达到 `min_fragments_per_chunk`；
  - 材质选择支持按空间分区策略调整权重。
- 改造运行期补种：
  - `WorldKernel::step` 每经过 `replenish_interval_ticks` 检查可生成 chunk；
  - 对低于 `max_fragments_per_chunk` 的 chunk，按 `replenish_percent_ppm` 补入新 frag；
  - 补种结果写入独立事件，保障 replay 一致性。
- 测试补充：覆盖“保底数量生效”“中心分布偏置生效”“配置 sanitize 边界”。

### Out of Scope
- 按玩家行为实时动态调参（在线经济反馈回路）。
- 新资源品类、加工链条或 UI 引导流程重构。
- 跨 chunk 的新保底策略（当前仅保证单 chunk 生成器语义）。
- 在线经济反馈闭环（玩家行为驱动的实时调参）。

## 接口 / 数据

### 配置扩展
在 `AsteroidFragmentConfig` 新增字段：
- `min_fragments_per_chunk: i64`
  - 默认值：`6`
  - 约束：`0 <= min_fragments_per_chunk <= max_fragments_per_chunk`
- `starter_core_radius_ratio: f64`
  - 默认值：`0.35`
  - 约束：`0.0..=1.0`
- `starter_core_density_multiplier: f64`
  - 默认值：`1.6`
  - 约束：`>= 1.0`
- `replenish_interval_ticks: i64`
  - 默认值：`100`
  - 约束：`>= 0`（`0` 表示关闭运行期补种）
- `replenish_percent_ppm: i64`
  - 默认值：`10_000`（1%）
  - 约束：`0..=1_000_000`
- `material_distribution_strategy: MaterialDistributionStrategy`
  - 默认值：`uniform`
  - 可选：
    - `uniform`：沿用单一 `material_weights`
    - `core_metal_rim_volatile`：中心提升 metal/composite，边缘提升 ice/carbon

### 生成公式（增量）
- 体素中心平面距离归一化：
  - `r = planar_distance_to_chunk_center / max_planar_distance`
- 中心区判定：
  - `r <= starter_core_radius_ratio` 视为中心区。
- 密度倍率：
  - 中心区 `density *= starter_core_density_multiplier`
  - 非中心区 `density` 不变。

### 保底补种策略
- 正常体素采样完成后，若 `current_count < min_fragments_per_chunk`，触发补种。
- 补种在 chunk 空间内进行随机候选投放，沿用已有半径/材质/间距判定。
- 补种为有界循环（固定最大尝试次数），不可达时允许低于保底值，但必须可终止。

### 运行期补种策略（FRB4）
- 触发条件：`time % replenish_interval_ticks == 0` 且 `current_fragments < max_fragments_per_chunk`。
- 目标补种量：
  - `target = ceil(max_fragments_per_chunk * replenish_percent_ppm / 1_000_000)`；
  - 每次至少 `1`（仅在存在缺口时）。
- 结果落账：
  - 新 frag 写入 `WorldModel.locations`；
  - 新 frag 的 `fragment_budget` 计入对应 `chunk_resource_budgets`；
  - 追加 `FragmentsReplenished` 事件（包含新增 frag 明细），用于 replay。

### 资源类型分布策略（FRB4）
- `uniform`：所有区域使用同一 `material_weights`。
- `core_metal_rim_volatile`：
  - 中心区（`r <= starter_core_radius_ratio`）：提高 `metal/composite` 权重；
  - 外缘区：提高 `ice/carbon` 权重；
  - 中间区：权重平滑回落，避免阶跃突变。

### 兼容性
- 与 `max_fragments_per_chunk` 三档预算兼容：
  - 生成器侧保底不应突破上层 chunk 预算裁剪语义。
- 与回放兼容：
  - 运行期补种通过事件重放，不依赖“隐式重算”。
- 与场景覆盖兼容：
  - 场景未配置新字段时使用默认值；
  - 已有 `min_fragment_spacing_cm` 覆盖机制保持不变。

## 里程碑
- **FRB1**：输出本设计文档与项目管理文档。
- **FRB2**：完成配置与生成器实现，补齐 `test_tier_required` 覆盖。
- **FRB3**：回写总项目文档与当日日志，完成回归与收口。
- **FRB4**：完成运行期补种与资源类型分布策略（含 replay 与测试）。

## 风险
- 中心区倍率过高可能形成“中心资源过热”，抬高后期通胀风险。
- 保底补种若尝试上限过高，可能放大极端参数下的生成耗时。
- 生成数量提高会间接增加预算体量，需要持续观察采集与加工节奏指标。
- 运行期补种若参数过大，可能导致中后期资源膨胀与事件日志增长过快。
