# Agent World Runtime：DistFS 路径索引接入 execution_storage（项目管理文档）

## 任务拆解
- [x] DPRI-1：设计文档与项目管理文档落地。
- [x] DPRI-2：实现 execution_storage 的路径索引写入与读取接口。
- [ ] DPRI-3：补齐单元测试并完成 `agent_world_net` 回归。
- [ ] DPRI-4：回写状态文档与 devlog。

## 依赖
- `crates/agent_world_net/src/execution_storage.rs`
- `crates/agent_world_net/src/lib.rs`
- `crates/agent_world_distfs/src/lib.rs`
- `doc/world-runtime/distfs-standard-file-io.md`

## 状态
- 当前阶段：DPRI-2 完成（路径索引写入/读取接口已接入）。
- 下一步：DPRI-3（补齐新增接口测试）。
- 最近更新：2026-02-16。
