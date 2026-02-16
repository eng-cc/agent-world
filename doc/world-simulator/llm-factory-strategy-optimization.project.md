# LLM 工厂闭环策略稳定性优化（项目管理文档）

## 任务拆解

### LFO0 文档立项
- [x] LFO0.1 输出设计文档（`doc/world-simulator/llm-factory-strategy-optimization.md`）
- [x] LFO0.2 输出项目管理文档（本文件）

### LFO1 Prompt 恢复规则（TODO-1）
- [x] LFO1.1 在 prompt 增加“失败原因 -> 下一动作”规则模板
- [x] LFO1.2 透出 `last_action` 结果摘要到 observation（含 reject reason）
- [x] LFO1.3 增加/更新 prompt 断言测试（拒绝原因驱动动作切换）

### LFO2 动作触发率统计（TODO-2）
- [x] LFO2.1 扩展 `world_llm_agent_demo` report 结构（`action_kind_counts` 等）
- [x] LFO2.2 在运行汇总阶段写入 `first_action_tick` 等关键字段
- [x] LFO2.3 增加 report JSON 结构回归测试

### LFO3 固定序列离线回归（TODO-3）
- [ ] LFO3.1 新增 mock 决策序列测试（失败恢复 -> 建厂 -> 排产成功）
- [ ] LFO3.2 新增反循环测试（限制连续无效 harvest）
- [ ] LFO3.3 归并到 `test_tier_required` 并验证稳定通过

### LFO4 在线闭环验证与收口
- [ ] LFO4.1 复跑 `llm_bootstrap --ticks 20` 并记录 `action_kind_counts`
- [ ] LFO4.2 复跑 `llm_bootstrap --ticks 30` 对比策略收敛性
- [ ] LFO4.3 回写文档状态、任务日志并按任务提交

## 依赖
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world/src/bin/world_llm_agent_demo.rs`
- `crates/agent_world/src/simulator/tests/kernel.rs`

## 状态
- 当前阶段：LFO0-LFO2 完成，LFO3 待开始。
- 下一步：执行 LFO3（固定 mock 序列离线回归：成功链路 + 反循环）。
- 最近更新：2026-02-16（完成 LFO2：动作触发率/首达 tick 指标与回归测试）。
