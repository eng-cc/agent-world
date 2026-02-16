# Agent World Runtime：节点贡献积分运行时闭环（项目管理文档）

## 任务拆解
- [x] NCPR-1：完成设计文档与项目管理文档。
- [x] NCPR-2：实现运行时采样器（snapshot/storage -> epoch settlement）。
- [x] NCPR-3：新增多节点运行时闭环集成测试并验证。
- [ ] NCPR-4：执行 test_tier_required 回归，回写文档与 devlog 收口。

## 依赖
- `crates/agent_world/src/runtime/node_points.rs`
- `crates/agent_world/src/runtime/mod.rs`
- `crates/agent_world_node/src/lib.rs`（`NodeSnapshot`）
- `doc/world-runtime/node-contribution-points.md`

## 状态
- 当前阶段：NCPR-1~NCPR-3 已完成，NCPR-4 待执行。
- 最近更新：2026-02-16。
