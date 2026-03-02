# 客户端启动器 LLM 设置入口（2026-03-02）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 功能实现：设置按钮/窗口 + `config.toml [llm]` 三字段读写 + 单测。
- [ ] T2 回归与收口：测试、文档完成态、devlog、提交结项。

## 依赖
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/tests.rs`
- `config.toml` 小写 TOML 结构约定（`[llm]`）
- `doc/world-simulator/llm-config-toml-style-unification-2026-03-02.md`

## 状态
- 当前阶段：进行中（T0~T1 已完成，执行 T2）。
- 当前任务：T2 回归与收口。
