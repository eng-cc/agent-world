# Agent World Runtime：节点贡献积分激励（项目管理文档）

## 任务拆解
- [x] NCP-1：完成设计文档与项目管理文档。
- [x] NCP-2：实现节点积分结算引擎（额外计算/存储/在线/惩罚 + 积分台账）。
- [ ] NCP-3：补齐单元测试并在 runtime 模块导出接口，执行 test_tier_required 回归。
- [ ] NCP-4：回写项目状态与 devlog，完成收口。

## 依赖
- `crates/agent_world/src/runtime/mod.rs`
- `crates/agent_world/src/runtime`（新增节点积分模块）
- `doc/world-runtime/distributed-runtime.md`
- `doc/world-runtime/blockchain-p2pfs-hardening-phase2.md`

## 状态
- 当前阶段：NCP-1~NCP-2 已完成，NCP-3~NCP-4 待执行。
- 最近更新：2026-02-16。
