# LLM 工业采矿闭环与调试补给工具（项目管理文档）

## 任务拆解

### MMD0 文档立项
- [x] MMD0.1 输出设计文档（`doc/world-simulator/llm-industrial-mining-debug-tools.md`）
- [x] MMD0.2 输出项目管理文档（本文件）

### MMD1 机制正确版（采矿 -> 精炼 -> 生产）
- [x] MMD1.1 扩展 `ResourceKind`：新增 `compound`
- [x] MMD1.2 扩展 `Action`：新增 `mine_compound`
- [x] MMD1.3 扩展经济参数：采矿电力成本/单次上限/单 location 上限
- [x] MMD1.4 实现 kernel 采矿执行（fragment 预算扣减 + compound 入账）
- [x] MMD1.5 升级 `refine_compound`：必须消耗 `compound`
- [x] MMD1.6 扩展事件/replay 与相关回归测试
- [x] MMD1.7 调整 `llm_bootstrap` 初始资源口径，验证“不能开局直建厂”

### MMD2 Debug 模式 LLM 补给工具
- [x] MMD2.1 新增配置开关 `AGENT_WORLD_LLM_DEBUG_MODE`（默认关闭）
- [x] MMD2.2 新增 `debug_grant_resource` 决策与 `Action::DebugGrantResource`
- [x] MMD2.3 OpenAI tools 仅在 debug 模式暴露 `agent_debug_grant_resource`
- [x] MMD2.4 非 debug 模式补给决策硬拒绝（解析/守卫）
- [x] MMD2.5 补齐 tool/schema/parser/behavior 单测

### MMD3 闭环验证与收口
- [ ] MMD3.1 跑 `test_tier_required` 相关测试集
- [ ] MMD3.2 运行 `llm_bootstrap` 在线闭环抽样，验证先采矿再生产
- [ ] MMD3.3 回写文档状态与 devlog，提交收口

## 依赖
- `crates/agent_world/src/simulator/types.rs`
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/kernel/replay.rs`
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/openai_payload.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world/scenarios/llm_bootstrap.json`

## 状态
- 当前阶段：MMD2 完成，进入 MMD3。
- 下一阶段：执行 `llm_bootstrap` 在线闭环复跑并收口文档状态。
- 最近更新：2026-02-17（完成 debug 模式 LLM 补给工具与回归）。
