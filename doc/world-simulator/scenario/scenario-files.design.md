# Agent World Simulator：场景文件化（设计文档）设计

- 对应需求文档: `doc/world-simulator/scenario/scenario-files.prd.md`
- 对应项目管理文档: `doc/world-simulator/scenario/scenario-files.project.md`

## 1. 设计定位
定义场景文件化设计，把内置 `WorldScenario` 迁移为 JSON 场景文件并作为单一权威来源。

## 2. 设计结构
- 文件模型层：用场景 JSON 描述 seed、地点生成与可选碎片配置。
- 加载接线层：通过 `include_str!` 与调试入口统一场景加载。
- 生成表达层：用 `location_generator`、origin 等字段驱动初始化。
- 测试矩阵层：维护场景到测试目标的映射与稳定性校验。

## 3. 关键接口 / 入口
- `crates/agent_world/scenarios/*.json`
- `WorldInitConfig::from_scenario`
- `WorldScenario::parse`
- `world_init_demo --scenario-file`

## 4. 约束与边界
- 内置场景文件需保持单一来源。
- 同场景配置在相同 seed 下必须可复现。
- 不在本专题扩展复杂 DSL 与版本迁移工具。

## 5. 设计演进计划
- 先迁移内置场景为 JSON。
- 再接入加载逻辑与调试入口。
- 最后维护场景测试覆盖矩阵。
