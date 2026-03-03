# Agent World Simulator：场景种子化地点生成（项目管理文档）

## 任务拆解
- [x] SL1.1 新增设计文档与项目管理文档
- [x] SL1.2 改造 `WorldScenarioSpec` 数据结构（移除 `locations`，新增 `location_generator`）
- [x] SL1.3 批量更新内置场景 JSON 到新 schema
- [x] SL2.1 实现基于 seed 的地点生成器（确定性）
- [x] SL2.2 接入初始化流程，构造 `LocationSeedConfig` 列表
- [x] SL2.3 改造 Agent 出生逻辑为随机地点选择
- [x] SL2.4 新增错误分支校验（无地点时拒绝）
- [x] SL3.1 更新并新增单元测试（场景解析/地点确定性/出生确定性）
- [x] SL3.2 运行 `env -u RUSTC_WRAPPER cargo test` 相关测试集
- [x] SL3.3 回顾并更新设计文档、项目文档、开发日志

## 依赖
- `crates/agent_world/src/simulator/scenario.rs`
- `crates/agent_world/src/simulator/init.rs`
- `crates/agent_world/src/simulator/tests/init.rs`
- `crates/agent_world/scenarios/*.json`

## 状态
- 当前阶段：SL3.3（完成）
- 下一阶段：按后续需求迭代
