# README 缺口 1/2 收口：Live 共识提交主路径 + LLM/Simulator 模块市场生命周期（项目管理文档）

审计轮次: 4

## 审计备注
- 主项目入口：`doc/readme/gap/readme-gap-distributed-prod-hardening-gap12345.project.md`。
- 本文件仅维护本专题增量任务。

## 任务拆解
- [x] T0：输出设计文档（`doc/readme/gap/readme-gap12-consensus-market-lifecycle-closure.prd.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：Live 共识提交主路径
  - node 已提交动作批次 drain
  - viewer live 提交/回放闭环
  - payload envelope + execution bridge 兼容
  - required tests
- [x] T2：Simulator/LLM 模块市场生命周期入口
  - action/event/model/kernel/replay
  - llm parser/prompt/schema
  - required tests
- [x] T3：回归与收口
  - `env -u RUSTC_WRAPPER cargo check`
  - 定向 required tests
  - 项目文档状态 + devlog 回写

## 依赖
- T1 依赖：
  - `crates/agent_world_node/src/lib.rs`
  - `crates/agent_world/src/viewer/live.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/execution_bridge.rs`
- T2 依赖：
  - `crates/agent_world/src/simulator/types.rs`
  - `crates/agent_world/src/simulator/kernel/*`
  - `crates/agent_world/src/simulator/world_model.rs`
  - `crates/agent_world/src/simulator/llm_agent/*`
- T3 依赖 T1/T2 完成后统一执行。

## 状态
- 当前阶段：T0/T1/T2/T3 全部完成。
- 阻塞项：无。
- 下一步：无（README 缺口 1/2 收口完成）。

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
