# oasis7 Runtime：分布式能力彻底拆分（Phase 7 项目管理）（项目管理文档）

- 对应设计文档: `doc/p2p/distributed/distributed-hard-split-phase7.design.md`
- 对应需求文档: `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] R7-1 (PRD-P2P-MIG-081)：新建 `oasis7_distfs`，迁移 CAS/分片/组装能力并接入 runtime 基础路径。
- [x] R7-2 (PRD-P2P-MIG-081)：完成分布式彻底拆分，删除 `oasis7` 内分布式实现文件，调用方切到 net/consensus/distfs/proto。
- [x] R7-3 (PRD-P2P-MIG-081)：拆分 `oasis7` 大 facade，收敛导出边界并清理跨层 re-export。
- [x] R7-4 (PRD-P2P-MIG-081)：将 viewer 协议并入 `oasis7_proto`，完成 server/viewer 双端适配。
- [x] R7-5 (PRD-P2P-MIG-081)：收敛 WASM ABI 边界（移除 net 侧重复 `ModuleManifest`，明确运行时缓存归属）。
- [x] R7-6 (PRD-P2P-MIG-081)：拆分所有超 1200 行 Rust 文件并完成回归。

## 依赖
- doc/p2p/distributed/distributed-hard-split-phase7.prd.md
- `crates/oasis7`
- `crates/oasis7_net`
- `crates/oasis7_consensus`
- `crates/oasis7_proto`
- `crates/oasis7_viewer`
- `crates/oasis7_wasm_abi`
- `doc/world-runtime/project.md`

## 状态
- 当前阶段：R7 全部任务已完成（R7-1 ~ R7-6）。
- 下一步：按新的拆分阶段需求立项并进入下一阶段。
- 最近更新：2026-02-14
