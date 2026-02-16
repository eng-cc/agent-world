# LLM 建造链路动作扩展（项目管理文档）

## 任务拆解

### LBA0 文档建模
- [x] LBA0.1 输出设计文档（`doc/world-simulator/llm-build-chain-actions.md`）
- [x] LBA0.2 输出项目管理文档（本文件）
- [x] LBA0.3 在总项目文档挂载任务入口

### LBA1 解析与动作接线
- [x] LBA1.1 扩展 LLM decision 解析：`transfer_resource`
- [x] LBA1.2 扩展 LLM decision 解析：`refine_compound`
- [x] LBA1.3 扩展 `serialize_decision_for_prompt` 对新增动作的回写
- [x] LBA1.4 新增解析单元测试（合法/非法路径）

### LBA2 Prompt 约束与回归
- [x] LBA2.1 更新 `[Decision JSON Schema]`，加入 transfer/refine
- [x] LBA2.2 新增推荐模板与字段约束文案
- [x] LBA2.3 更新 Prompt 相关测试
- [x] LBA2.4 执行 `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent -- --nocapture`

### LBA3 Web 闭环验证
- [x] LBA3.1 启动 `world_viewer_live + run-viewer-web`（`llm_bootstrap`）
- [x] LBA3.2 使用 Playwright 执行 `open/snapshot/console/screenshot`
- [x] LBA3.3 验证 `console error=0` 且产出 `output/playwright/` 截图
- [x] LBA3.4 更新文档状态与任务日志

### LBA4 在线 LLM 实测 TODO（2026-02-16）
- [x] LBA4.1 Prompt 显式暴露“当前动作集不含 `build_factory/schedule_recipe`”，避免目标与动作能力错配。
- [x] LBA4.2 Prompt 补充 `refine_compound` 最小有效质量提示（默认 `refine_hardware_yield_ppm=1000` 时，`compound_mass_g < 1000` 不会产出硬件）。
- [x] LBA4.3 Prompt/解析层增加非法 module 别名纠正或错误提示（例如 `agent_modules_list` -> `agent.modules.list`）。
- [x] LBA4.4 `world_llm_agent_demo` 报告新增 `reject_reason` 聚合统计，便于回归判断“失败是策略问题还是能力缺失”。

### LBA5 在线 LLM 闭环复跑（2026-02-16）
- [x] LBA5.1 使用“建工厂 + 制成品”目标 prompt 复跑 `llm_bootstrap` 20 tick。
- [x] LBA5.2 产出运行证据：
  - `output/llm_bootstrap/factory_finished_rerun_2026-02-16/run.log`
  - `output/llm_bootstrap/factory_finished_rerun_2026-02-16/report.json`
  - 指标：`action_success=19`、`action_failure=1`、`llm_errors=0`、`parse_errors=0`。
- [x] LBA5.3 复跑结论：当前动作集中仍无法直接建厂，但可稳定完成“辐射采集 -> 化合物精炼(1000g) -> 硬件产出”最小制成品闭环。
- [x] LBA5.4 记录产品优化 TODO（待后续排期）：
  - TODO-1：LLM 仍会先发出“移动到当前位置”的无效动作（`agent_already_at_location`），可在 prompt 中进一步强化“distance_cm=0 禁止 move”并在 planner 层增加硬拦截。
  - TODO-2：模型偶发单轮输出多个 JSON（`module_call --- module_call --- decision`）；建议新增“多 JSON 响应拆分与告警统计”，避免依赖隐式容错。
  - TODO-3：`world_llm_agent_demo` 报告未直接暴露“最终硬件库存/增量”，建议补充制成品 KPI 字段，降低人工解读成本。

## 依赖
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `doc/world-simulator/viewer-web-closure-testing-policy.md`

## 状态
- 当前阶段：LBA0~LBA5 全部完成。
- 下一阶段：按 TODO-1~TODO-3 进入产品优化排期（先补多 JSON 解析告警与制成品 KPI，再收敛 move 无效动作）。
- 最近更新：2026-02-16（完成 LBA5 在线闭环复跑与优化 TODO 记录）。
