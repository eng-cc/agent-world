# Agent World Runtime：`agent_world_net` runtime_bridge 可编译闭环（项目管理文档）

## 任务拆解
- [x] RB1：设计文档与项目管理文档落地。
- [ ] RB2：runtime_bridge 模块导入与依赖收敛（切到 `agent_world_distfs` / `agent_world::runtime` / `agent_world_proto` 稳定路径）。
- [ ] RB3：修复 `HeadValidationResult` 导出路径并完成编译回归。
- [ ] RB4：回写总文档状态与当日 devlog。

## 依赖
- `doc/world-runtime/distributed-crate-split-net-consensus.project.md`
- `crates/agent_world_net`
- `crates/agent_world_distfs`
- `crates/agent_world`
- `crates/agent_world_proto`

## 状态
- 当前阶段：RB1 完成（文档已建立）。
- 下一步：RB2（代码收敛与编译闭环）。
- 最近更新：2026-02-16。
