# 客户端启动器反馈分布式提交迁移（2026-03-02）项目管理文档

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档 + 项目管理文档）。
- [x] T1 (PRD-WORLD_SIMULATOR-001/002)：链运行时实现反馈提交接口（HTTP -> NodeRuntime::submit_feedback）并启用 feedback_p2p。
- [x] T2 (PRD-WORLD_SIMULATOR-002)：启动器反馈提交改造为远端优先、失败回落本地落盘。
- [x] T3 (PRD-WORLD_SIMULATOR-003)：测试回归、文档状态回写、devlog 收口。
- [x] T4 (PRD-WORLD_SIMULATOR-002/003)：补充“连接链状态服务被拒绝（Connection refused）”回归测试，约束回落路径保留远端错误签名。

## 依赖
- doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.prd.md
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/feedback_entry.rs`
- `crates/agent_world_node` 既有 `submit_feedback` 能力
- `crates/agent_world_distfs` 既有 feedback request/signing 数据结构

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（T0~T4）。
- 当前任务：无（项目结项）。
