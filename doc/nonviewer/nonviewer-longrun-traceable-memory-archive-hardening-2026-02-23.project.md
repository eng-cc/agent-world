# Non-Viewer 长稳运行内存安全与可追溯冷归档硬化（项目管理）

## 任务拆解

### T0 建档
- [x] 设计文档：`doc/nonviewer/nonviewer-longrun-traceable-memory-archive-hardening-2026-02-23.md`
- [x] 项目文档：`doc/nonviewer/nonviewer-longrun-traceable-memory-archive-hardening-2026-02-23.project.md`

### T1 Node 内存边界治理（1/2/3）
- [x] `agent_world_node`：引擎 pending actions 有界化（不丢追溯）
- [x] `agent_world_node`：gossip dynamic peer TTL + 容量治理
- [x] `agent_world_node`：committed batches 热窗口有界化
- [x] 补充/更新单测

### T2 Consensus 日志与 dead-letter 冷归档（4/5）
- [x] `agent_world_consensus`：文件型 audit/alert/event 日志热窗口 + CAS 冷归档
- [x] `agent_world_consensus`：dead-letter archive 改 CAS 冷归档
- [x] 补充/更新单测

### T3 Node replication commit 冷归档（6）
- [x] `agent_world_node`：commit message 热窗口 + CAS 冷归档
- [x] `agent_world_node`：按高度读取回退到 cold refs/CAS
- [x] 补充/更新单测

### T4 收口
- [x] required-tier 回归
- [x] 设计/项目文档状态更新
- [x] `doc/devlog/2026-02-23.md` 追加任务日志

## 依赖
- `crates/agent_world_node/src/node_runtime_core.rs`
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/lib_impl_part1.rs`
- `crates/agent_world_node/src/gossip_udp.rs`
- `crates/agent_world_node/src/types.rs`
- `crates/agent_world_node/src/replication.rs`
- `crates/agent_world_consensus/src/membership_reconciliation.rs`
- `crates/agent_world_consensus/src/membership_split_part1.rs`
- `crates/agent_world_consensus/src/membership_recovery/dead_letter.rs`
- `crates/agent_world_consensus/src/membership_recovery/replay_archive_federated.rs`
- `crates/agent_world_distfs/src/lib.rs`（复用）

## 状态
- 当前状态：`已完成`
- 已完成：T0、T1、T2、T3、T4
- 进行中：无
- 未开始：无
- 阻塞项：无
