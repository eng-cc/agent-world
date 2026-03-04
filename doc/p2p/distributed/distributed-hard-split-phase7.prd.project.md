# Agent World Runtime：分布式能力彻底拆分（Phase 7 项目管理）（项目管理文档）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] R7-1 (PRD-P2P-MIG-081)：新建 `agent_world_distfs`，迁移 CAS/分片/组装能力并接入 runtime 基础路径。
- [x] R7-2 (PRD-P2P-MIG-081)：完成分布式彻底拆分，删除 `agent_world` 内分布式实现文件，调用方切到 net/consensus/distfs/proto。
- [x] R7-3 (PRD-P2P-MIG-081)：拆分 `agent_world` 大 facade，收敛导出边界并清理跨层 re-export。
- [x] R7-4 (PRD-P2P-MIG-081)：将 viewer 协议并入 `agent_world_proto`，完成 server/viewer 双端适配。
- [x] R7-5 (PRD-P2P-MIG-081)：收敛 WASM ABI 边界（移除 net 侧重复 `ModuleManifest`，明确运行时缓存归属）。
- [x] R7-6 (PRD-P2P-MIG-081)：拆分所有超 1200 行 Rust 文件并完成回归。

## 依赖
- doc/p2p/distributed/distributed-hard-split-phase7.prd.md
- `crates/agent_world`
- `crates/agent_world_net`
- `crates/agent_world_consensus`
- `crates/agent_world_proto`
- `crates/agent_world_viewer`
- `crates/agent_world_wasm_abi`
- `doc/world-runtime/prd.project.md`

## 状态
- 当前阶段：R7 全部任务已完成（R7-1 ~ R7-6）。
- 下一步：按新的拆分阶段需求立项并进入下一阶段。
- 最近更新：2026-02-14
