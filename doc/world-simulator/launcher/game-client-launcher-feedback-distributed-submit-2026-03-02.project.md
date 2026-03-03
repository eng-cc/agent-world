# 客户端启动器反馈分布式提交迁移（2026-03-02）项目管理文档

## 任务拆解
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档 + 项目管理文档）。
- [x] T1 (PRD-WORLD_SIMULATOR-001/002)：链运行时实现反馈提交接口（HTTP -> NodeRuntime::submit_feedback）并启用 feedback_p2p。
- [x] T2 (PRD-WORLD_SIMULATOR-002)：启动器反馈提交改造为远端优先、失败回落本地落盘。
- [x] T3 (PRD-WORLD_SIMULATOR-003)：测试回归、文档状态回写、devlog 收口。

## 依赖
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/feedback_entry.rs`
- `crates/agent_world_node` 既有 `submit_feedback` 能力
- `crates/agent_world_distfs` 既有 feedback request/signing 数据结构

## 状态
- 当前阶段：已完成（T0~T3）。
- 当前任务：无（项目结项）。
