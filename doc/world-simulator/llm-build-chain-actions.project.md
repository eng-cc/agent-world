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
- [ ] LBA4.1 Prompt 显式暴露“当前动作集不含 `build_factory/schedule_recipe`”，避免目标与动作能力错配。
- [ ] LBA4.2 Prompt 补充 `refine_compound` 最小有效质量提示（默认 `refine_hardware_yield_ppm=1000` 时，`compound_mass_g < 1000` 不会产出硬件）。
- [ ] LBA4.3 Prompt/解析层增加非法 module 别名纠正或错误提示（例如 `agent_modules_list` -> `agent.modules.list`）。
- [ ] LBA4.4 `world_llm_agent_demo` 报告新增 `reject_reason` 聚合统计，便于回归判断“失败是策略问题还是能力缺失”。

## 依赖
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `doc/world-simulator/viewer-web-closure-testing-policy.md`

## 状态
- 当前阶段：LBA0~LBA3 已完成，LBA4（在线实测优化）待执行。
- 下一阶段：优先推进 LBA4.1~LBA4.4，降低“目标是建工厂/制成品但动作能力不匹配”导致的无效决策回合。
- 最近更新：2026-02-16（完成 `llm_bootstrap` 在线实测，产物：`output/llm_bootstrap/factory_finished/`，并沉淀 LBA4 TODO）。
