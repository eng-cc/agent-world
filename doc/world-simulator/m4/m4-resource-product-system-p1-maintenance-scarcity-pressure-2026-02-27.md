# M4 资源与产品系统 P1：维护压力与本地稀缺供给延迟（2026-02-27）

## 目标
- 在 P0 基础上强化“维护成本压力”，让高负载产线更快折旧，形成持续维护决策。
- 接入“本地稀缺供给延迟”语义：当站点库存不足被迫回退到 world 账本时，配方完工时间增加。
- 保持兼容：不改动作 ABI，不新增必填字段，不破坏旧快照/旧事件回放。

## 范围

### In Scope
- 工厂折旧接入负载系数（按 `active_jobs / recipe_slots` 放大衰减）。
- 配方排产在“本地库存不足且存在关键中间件消耗”时增加供给延迟 tick。
- 补齐 `test_tier_required`：
  - 负载下折旧快于空载。
  - world fallback 触发供给延迟且按期完成。

### Out of Scope
- 不改 `agent_world_wasm_abi::RecipeExecutionPlan` 结构。
- 不引入市场撮合或新治理税种。
- 不改 viewer 结构，仅通过现有事件行为可观测。

## 接口 / 数据

### 1) 负载折旧
- 位置：`runtime/world/economy.rs::process_factory_depreciation`。
- 新逻辑：
  - 基础衰减：`maintenance_per_tick * FACTORY_DEPRECIATION_PPM_PER_MAINTENANCE_UNIT`。
  - 负载放大：`load_factor_bps = 10000 + floor(active_jobs * 10000 / recipe_slots)`，上限 20000。
  - 最终衰减：`base_decay * load_factor_bps / 10000`。

### 2) 本地稀缺供给延迟
- 位置：`runtime/world/event_processing/action_to_event_economy.rs`。
- 触发条件：
  - `preferred_consume_ledger` 不是 `world`。
  - `consume_ledger` 回退为 `world`。
  - 本次配方命中 `bottleneck_tags`（P0 已接线）。
- 延迟规则：
  - 计算本地缺口占比 `deficit / requested`。
  - 缺口 > 0 且 < 70%：`+1 tick`。
  - 缺口 >= 70%：`+2 ticks`。

## 里程碑
- P1-T0：设计文档与项目文档建档。
- P1-T1：代码接线（负载折旧 + 供给延迟）。
- P1-T2：补齐 required 单测并回归。
- P1-T3：回写项目状态与 devlog。

## 风险
- 行为漂移：配方完工时序变化可能影响既有事件顺序断言。
- 参数风险：延迟阈值过高可能导致产线停滞体感。
- 叠加风险：负载折旧与维护成本共同作用可能提高新手失败率。

缓解：
- 阈值采用保守值（1~2 tick），先做可观测，再调参。
- 仅在 world fallback + bottleneck 同时满足时触发延迟。
- 先补 required 测试再跑回归。
