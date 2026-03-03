# Agent World: LLM 跳过 Tick 占比指标（设计文档）

## 目标
- 为 LLM 运行链路增加“跳过 LLM 调用的 tick 占比”指标，量化 `execute_until` 等机制带来的调用节省。
- 指标同时可用于：
  - `world_llm_agent_demo` 控制台输出；
  - `report.json` 持久化；
  - `scripts/llm-longrun-stress.sh` summary/聚合输出。
- 指标口径稳定，可被自动脚本和回归测试直接消费。

## 范围

### In Scope
- `crates/agent_world/src/bin/world_llm_agent_demo.rs`
  - 新增跳过计数与占比字段；
  - 在 trace 采样与 finalize 阶段完成统计；
  - 打印新增指标。
- `scripts/llm-longrun-stress.sh`
  - 从 report/log 回填新增指标；
  - 单场景 summary 输出新增指标；
  - 多场景 TSV/聚合 summary/聚合 JSON 同步新增指标。
- `crates/agent_world/src/bin/world_llm_agent_demo/tests.rs`
  - 增加指标统计与占比计算单测。

### Out of Scope
- 不改动 LLM 决策逻辑（仅观测指标，不影响行为）。
- 不新增新的 fail gate（只提供可观测性字段）。
- 不改动 viewer live UI 展示。

## 接口 / 数据
- 新增/扩展字段（`TraceCounts`）：
  - `llm_skipped_ticks: u64`
  - `llm_skipped_tick_ratio_ppm: u64`（百万分比，`1_000_000 == 100%`）
- 指标判定：
  - 当 tick trace 呈现“未发起 LLM 请求”语义时记为 skipped（例如 `llm_input == None` 或 step trace 标记 `skip_llm_with_active_execute_until`）。
- 占比口径：
  - 分子：`llm_skipped_ticks`
  - 分母：`active_ticks`
  - 结果：`llm_skipped_tick_ratio_ppm = llm_skipped_ticks * 1_000_000 / active_ticks`

## 里程碑
- M1：文档与任务建档。
- M2：demo 指标实现与输出。
- M3：stress 脚本指标接入。
- M4：单测与脚本语法校验通过。

## 风险
- 口径风险：不同运行路径可能存在 trace 缺失，导致分母语义偏差；通过固定分母 `active_ticks` 降低歧义。
- 兼容风险：脚本在无 `jq` 场景走 python/log 回退，需要三条路径同步更新避免指标缺失。
- 演进风险：未来若引入新的“跳过 LLM”路径，需要补充判定逻辑与测试。
