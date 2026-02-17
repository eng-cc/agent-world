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
- [x] MMD3.1 跑 `test_tier_required` 相关测试集
- [x] MMD3.2 运行 `llm_bootstrap` 在线闭环抽样，验证先采矿再生产
- [x] MMD3.3 回写文档状态与 devlog，提交收口

### MMD4 用户指令闭环复跑（2026-02-17）
- [x] MMD4.1 按用户目标执行在线闭环抽样（“所有工厂 + 所有制成品”）并保存 `run.log/report.json`
- [x] MMD4.2 基于首轮结果迭代 prompt 再抽样，验证是否提升 recipe 覆盖
- [x] MMD4.3 从两轮日志提取 recipe 级成功计数，回写 TODO 与项目状态

### MMD5 TODO-10~13 优化迭代（2026-02-17）
- [x] MMD5.1 文档增量设计：补充 TODO-10/11/12/13 优化目标、接口与风险
- [x] MMD5.2 守卫优化 A：`schedule_recipe` 位置前置检查 + `move_agent` 分段规划
- [x] MMD5.3 守卫优化 B：恢复链路与 `mine_compound` 单次上限对齐
- [ ] MMD5.4 覆盖优化：recipe 覆盖记忆与硬切换策略
- [ ] MMD5.5 回归验证：`test_tier_required` + `llm_bootstrap` 在线抽样复核
- [ ] MMD5.6 文档收口：状态更新、TODO 回填、devlog 记录

### MMD4 结果摘要（2026-02-17）
- 运行 #1（100 tick，基线强化 prompt）：
  - 产物：
    - `output/llm_bootstrap/user_all_factories_all_products_2026-02-17_142739/run.log`
    - `output/llm_bootstrap/user_all_factories_all_products_2026-02-17_142739/report.json`
  - 指标：`action_success=69`、`action_failure=31`、`parse_errors=0`、`llm_errors=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.control_chip=3`
    - `recipe.assembler.motor_mk1=2`
    - `recipe.assembler.logistics_drone=0`
- 运行 #2（120 tick，先 `logistics_drone` 的顺序约束 prompt）：
  - 产物：
    - `output/llm_bootstrap/user_all_factories_all_products_retry_2026-02-17_143342/run.log`
    - `output/llm_bootstrap/user_all_factories_all_products_retry_2026-02-17_143342/report.json`
  - 指标：`action_success=79`、`action_failure=41`、`parse_errors=0`、`llm_errors=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.logistics_drone=1`
    - `recipe.assembler.motor_mk1=0`
    - `recipe.assembler.control_chip=0`
- 结论：
  - 两轮都未达成“三配方全覆盖”，但分别覆盖了不同配方，验证了“prompt 可影响配方偏置，但仍会被路径/守卫问题打断”。

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
- 当前阶段：MMD5 进行中（TODO-10/TODO-11/TODO-12/TODO-13 优化迭代）。
- 下一阶段：完成守卫与覆盖策略改造后进行在线闭环复核。
- 最近更新：2026-02-17（已完成 MMD5.3：恢复链路按硬件缺口计算并与 `mine_compound` 单次上限对齐）。

## 遗留 TODO（产品优化）
- TODO-10：配方覆盖进度记忆与硬切换仍不足，模型会在单一配方上长时间循环。
- TODO-11：`schedule_recipe` 缺少“工厂位置可达性”前置约束，导致大量 `agent_not_at_location` 拒绝后再回退。
- TODO-12：缺少分段路径规划，长距离回厂出现高频 `move_distance_exceeded`（单步 > 1000000cm）。
- TODO-13：`schedule_recipe -> refine_compound(8000)` 的恢复链路与 `mine_compound` 单次上限存在错配，易诱发无效恢复或参数抖动。
