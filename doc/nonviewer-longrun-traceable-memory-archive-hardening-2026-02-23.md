# Non-Viewer 长稳运行内存安全与可追溯冷归档硬化（2026-02-23）

## 目标
- 针对 non-viewer 代码路径的 6 个长稳风险点完成治理：
  - 1) `PosNodeEngine.pending_consensus_actions` 长期增长风险。
  - 2) Gossip 动态 peer 集合无界增长与广播放大风险。
  - 3) `NodeRuntime.committed_action_batches` 无界增长风险。
  - 4) 多类审计/告警日志 append-only + 全量读取风险。
  - 5) dead-letter archive 文件长期膨胀风险。
  - 6) replication commit message 本地热文件长期膨胀风险。
- 实现原则：
  - 运行内存必须有硬边界。
  - 历史信息不可丢失，必须可追溯。
  - 冷数据优先以分布式内容寻址存储（CAS）承载，单机仅保留热窗口与索引引用。

## 范围

### In Scope
- `crates/agent_world_node`：
  - 共识动作队列与引擎缓存双层有界化。
  - gossip 动态 peer TTL + 容量治理。
  - committed batch 内存窗口治理。
  - replication commit message 热窗口 + CAS 冷归档。
- `crates/agent_world_consensus`：
  - 文件型 audit/alert/event 日志热窗口治理。
  - dead-letter archive 从本地无限 append 改为 CAS 分段冷归档。
- `crates/agent_world_distfs`（复用）：
  - 使用 `LocalCasStore` 作为内容寻址冷归档介质。
- 文档与测试：
  - 文档更新、devlog、tier-required 测试补齐。

### Out of Scope
- viewer 代码与交互。
- 共识协议语义改造。
- 业务规则参数调优。

## 接口/数据
- NodeConfig 新增内存边界参数：
  - `max_engine_pending_consensus_actions`
  - `max_committed_action_batches`
  - `max_dynamic_gossip_peers`
  - `dynamic_gossip_peer_ttl_ms`
- GossipEndpoint 内部 peer 存储由无界集合改为带 TTL 与容量的 peer book。
- 冷归档统一采用：
  - 热数据文件（有界）
  - CAS 分段 blob（内容哈希）
  - 本地 refs 索引（小文件）
- replication commit message：
  - 新增热窗口保留策略（按高度数量）。
  - 冷数据 commit 以 CAS blob 存储，并通过 height->hash refs 可回放读取。

## 里程碑
- M0：建档与任务拆解。
- M1：Node 内存边界治理（1/2/3）+ 测试。
- M2：Consensus 日志热窗口与 dead-letter CAS 归档（4/5）+ 测试。
- M3：Replication commit message 热冷分层与 CAS 归档（6）+ 测试。
- M4：required-tier 回归 + 文档收口。

## 风险
- 有界策略会引入“拒绝/跳过/淘汰”行为。
  - 缓解：保留计数、错误原因、冷归档 refs，确保可追溯。
- 冷归档读取链路更长。
  - 缓解：保留热窗口；冷数据仅按需读取。
- CAS 归档后若本机被清理，需依赖分布式副本。
  - 缓解：保留 refs 与内容哈希，支持跨节点回放；由分布式复制策略保障可得性。

## 当前状态
- 状态：进行中
- 进行中：M4
- 已完成：M0、M1、M2、M3
- 未开始：无
