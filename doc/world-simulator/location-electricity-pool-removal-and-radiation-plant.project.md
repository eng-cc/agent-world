# Agent World Simulator：Location 电力池下线与 Agent 辐射电厂（项目管理文档）

## 任务拆解

### R1 文档
- [x] 输出设计文档（`doc/world-simulator/location-electricity-pool-removal-and-radiation-plant.md`）
- [x] 输出项目管理文档（本文件）

### R2 Location 电力池下线
- [x] 初始化清洗 Location `electricity` 库存（场景/初始化统一口径）
- [x] 下线 `DrawPower` / `StorePower` 动作路径
- [x] 限制 `BuyPower` / `SellPower`：Location owner 参与电力交易时拒绝
- [x] 发电入账路径不再写 Location 电力库存

### R3 Agent 辐射电厂建造
- [x] 新增 `factory.power.radiation.mk1` 可建造类型
- [x] `BuildFactory` 对该类型同步注册 `PowerPlant`
- [x] 发电入账到 owner（Agent）资源
- [x] 更新 LLM 提示（`factory_kind` 支持集）

### R4 测试与收口
- [x] 更新 simulator 单元测试（power/kernel/init/llm 相关）
- [x] 运行 required-tier 测试命令并通过
- [x] 更新本项目文档状态
- [x] 追加当日 `doc/devlog/2026-02-17.md`
- [x] 提交 git commit

## 依赖
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/power.rs`
- `crates/agent_world/src/simulator/kernel/replay.rs`
- `crates/agent_world/src/simulator/init.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/tests/*`

## 状态
- 当前阶段：R1-R4 已完成
- 下一阶段：无（等待新需求）
