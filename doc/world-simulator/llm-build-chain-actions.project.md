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
- [ ] LBA3.1 启动 `world_viewer_live + run-viewer-web`（`llm_bootstrap`）
- [ ] LBA3.2 使用 Playwright 执行 `open/snapshot/console/screenshot`
- [ ] LBA3.3 验证 `console error=0` 且产出 `output/playwright/` 截图
- [ ] LBA3.4 更新文档状态与任务日志

## 依赖
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `doc/world-simulator/viewer-web-closure-testing-policy.md`

## 状态
- 当前阶段：LBA2 完成，LBA3 进行中。
- 下一阶段：执行 Web 闭环验收（Playwright）并回写日志与状态。
- 最近更新：2026-02-15（完成 Prompt schema 与回归测试，进入 Web 闭环验证阶段）。
