# Agent World Runtime：分布式能力彻底拆分（Phase 7 项目管理）

## 任务拆解
- [x] R7-1：新建 `agent_world_distfs`，迁移 CAS/分片/组装能力并接入 runtime 基础路径。
- [x] R7-2：完成分布式彻底拆分，删除 `agent_world` 内分布式实现文件，调用方切到 net/consensus/distfs/proto。
- [ ] R7-3：拆分 `agent_world` 大 facade，收敛导出边界并清理跨层 re-export。
- [ ] R7-4：将 viewer 协议并入 `agent_world_proto`，完成 server/viewer 双端适配。
- [ ] R7-5：收敛 WASM ABI 边界（移除 net 侧重复 `ModuleManifest`，明确运行时缓存归属）。
- [ ] R7-6：拆分所有超 1200 行 Rust 文件并完成回归。

## 依赖
- `crates/agent_world`
- `crates/agent_world_net`
- `crates/agent_world_consensus`
- `crates/agent_world_proto`
- `crates/agent_world_viewer`
- `crates/agent_world_wasm_abi`
- `doc/world-runtime.project.md`

## 状态
- 当前阶段：R7-2 已完成，R7-3 待执行。
- 下一步：执行 R7-3，继续拆分 `agent_world` facade 并收敛导出边界。
- 最近更新：2026-02-14
