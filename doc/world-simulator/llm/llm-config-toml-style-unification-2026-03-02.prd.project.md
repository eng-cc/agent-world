# LLM 配置 TOML 风格统一（2026-03-02）项目管理

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档 + 项目管理文档）。
- [x] T1 (PRD-WORLD_SIMULATOR-001/002)：代码重构（`LlmAgentConfig` 从小写 TOML 结构读取，补充解析测试）。
- [x] T2 (PRD-WORLD_SIMULATOR-002)：配置与文档迁移（更新 `config.toml` / `config.example.toml` 与说明文档）。
- [x] T3 (PRD-WORLD_SIMULATOR-003)：回归与收口（执行回归测试，更新文档完成态与任务日志）。

## 依赖
- doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.prd.md
- `crates/agent_world/src/simulator/llm_agent.rs` 配置加载入口。
- `crates/agent_world/src/simulator/llm_agent/tests_split_part1.rs` 现有配置解析测试。
- `config.toml` 与 `config.example.toml` 配置样例。
- `testing-manual.md` 测试分层规范。

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（T0~T3 已完成）。
- 当前任务：无。
