# LLM 配置 TOML 风格统一（2026-03-02）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 代码重构：`LlmAgentConfig` 从小写 TOML 结构读取，补充解析测试。
- [x] T2 配置与文档迁移：更新 `config.toml` / `config.example.toml` 与说明文档。
- [ ] T3 回归与收口：执行回归测试，更新文档完成态与任务日志。

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs` 配置加载入口。
- `crates/agent_world/src/simulator/llm_agent/tests_split_part1.rs` 现有配置解析测试。
- `config.toml` 与 `config.example.toml` 配置样例。
- `testing-manual.md` 测试分层规范。

## 状态
- 当前阶段：进行中（T0~T2 已完成，执行 T3）。
- 当前任务：T3 回归与收口。
