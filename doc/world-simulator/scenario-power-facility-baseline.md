# Agent World Simulator：默认电力设施语义下沉为场景配置

## 目标

- 避免 simulator 与 runtime 出现“双轨默认供能语义”：
  - runtime 已通过 `m1.power.radiation_harvest` + `m1.power.storage` 提供生命出厂基础供能；
  - simulator 不再在常用默认场景里额外预置发电/储能设施。
- 保留电力设施能力，但改为“显式场景选择”：只有在场景文件中显式声明 `power_plants` / `power_storages` 时才注入设施。

## 范围

### In Scope
- 调整内置场景 JSON：将非电力专项场景中的默认设施移除。
- 保留 `power_bootstrap` 作为“设施专项”场景，用于设施闭环验证与回归。
- 更新 simulator 场景相关测试与文档矩阵，确保“设施是否存在”与场景定义一致。

### Out of Scope
- 移除 simulator 的设施数据结构或 Action（`RegisterPowerPlant`/`RegisterPowerStorage` 等仍保留）。
- runtime 模块 ABI 或治理流程变更。
- viewer UI 面板字段删除（即使场景无设施，面板能力仍保留）。

## 接口 / 数据

- 初始化数据源仍然是 `WorldInitConfig`：
  - `power_plants: Vec<PowerPlantSeedConfig>`
  - `power_storages: Vec<PowerStorageSeedConfig>`
- 语义调整为：
  - 默认内置场景（`llm_bootstrap`、`twin_region_bootstrap`、`triad_region_bootstrap`、`asteroid_fragment_*`）不再自带设施；
  - 设施只在显式场景文件中出现（例如 `power_bootstrap`）。
- `build_world_model` 初始化逻辑不变，仍按 `init.power_plants / init.power_storages` 注入；本次仅调整默认场景配置与测试口径。

## 里程碑

- **S1**：新增设计文档与项目管理文档。
- **S2**：更新内置场景 JSON（移除非专项默认设施）。
- **S3**：更新 `simulator/tests/init.rs` 场景断言与稳定性矩阵断言。
- **S4**：更新场景文档矩阵并完成测试回归。

## 风险

- 现有依赖“场景内设施存在”的 demo/测试可能失效，需要同步调整断言。
- 默认场景设施移除后，某些展示（如设施统计）将变为 0，需在文档中明确这是预期行为。
- 若后续又在场景层回填大量默认设施，会再次引入语义漂移，需要保持矩阵与测试一致。
