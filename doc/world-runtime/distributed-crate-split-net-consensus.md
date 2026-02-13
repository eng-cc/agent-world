# Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分（设计文档）

## 目标
- 将分布式能力按职责拆为两个 crate：
  - `agent_world_net`：网络与传输相关能力。
  - `agent_world_consensus`：共识与成员同步相关能力。
- 明确将 `distributed_membership_sync` 归类到 `agent_world_consensus`。
- 保持 `agent_world` 对外 API 兼容，避免一次性大范围破坏。

## 范围

### In Scope（本次）
- 新建 crate：
  - `crates/agent_world_net`
  - `crates/agent_world_consensus`
- 将两个 crate 加入 workspace，并形成稳定导出入口（按能力分类导出）。
- `agent_world_consensus` 对外聚合共识与 membership sync 能力（含 `distributed_membership_sync` 相关导出）。
- 补充最小验证（编译/测试）与文档回写。

### In Scope（扩展阶段）
- 在不引入第三个 crate 的前提下，继续推进“实现物理迁移”：
  - 优先将 `distributed_net` 的核心实现（`DistributedNetwork` / `InMemoryNetwork`）下沉到 `agent_world_net`。
  - `agent_world` 保持兼容 API，不改变现有对外行为与语义。
- 通过定向编译/测试验证迁移不回归。

### In Scope（二次扩展阶段）
- 继续推进 `agent_world_net` 的网络基础设施下沉：
  - 将 `distributed_dht` 的核心实现（`DistributedDht` / `InMemoryDht`）下沉到 `agent_world_net`。
  - `MembershipDirectorySnapshot` / `ProviderRecord` 由 `agent_world_proto` 驱动，避免重复协议定义。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（三次扩展阶段）
- 继续推进 `agent_world_net` 的网络访问层下沉：
  - 将 `distributed_client` 核心实现（`DistributedClient` 及其 DHT provider 路由逻辑）下沉到 `agent_world_net`。
  - 在 `agent_world_net` 内补齐 canonical CBOR 序列化 helper，避免依赖 `agent_world::runtime::util` 私有实现。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（四次扩展阶段）
- 继续推进 `agent_world_net` 的网关能力下沉：
  - 将 `distributed_gateway` 核心实现（`ActionGateway` / `NetworkGateway` / `SubmitActionReceipt`）下沉到 `agent_world_net`。
  - 复用 `agent_world_proto::distributed` 的 topic helper 与 `ActionEnvelope` 协议类型，保持 wire 行为一致。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（五次扩展阶段）
- 继续推进 `agent_world_net` 的索引发布能力下沉：
  - 将 `distributed_index` 核心实现（`IndexPublishResult`、head/provider 发布与查询 helper）下沉到 `agent_world_net`。
  - 保持 `ExecutionWriteResult` 输入契约兼容，确保现有执行结果索引路径不回归。
- 保持 `agent_world` 与现有测试行为兼容；迁移过程允许短期共存（runtime 侧保留实现，新 crate 侧提供并行实现）。

### In Scope（六次扩展阶段）
- 在不新增 crate 的前提下，先完成 `agent_world_net` 内部模块化拆分：
  - 将 `src/lib.rs` 按能力拆成 `network` / `dht` / `index` / `client` / `gateway` / `util` 多文件结构。
  - 将 `agent_world_net` 单测拆分到独立 `tests.rs`，保证 `lib.rs` 行数安全余量，避免后续迁移触及 1200 行约束。
- 保持 `agent_world_net` 对外导出 API 不变，确保 `agent_world` 调用方无需改动。

### In Scope（七次扩展阶段）
- 继续推进 `agent_world_net` 的缓存与索引存储能力下沉：
  - 将 `distributed_index_store` 核心实现（`DistributedIndexStore` / `HeadIndexRecord` / `InMemoryIndexStore`）下沉到 `agent_world_net`。
  - 将 `distributed_provider_cache` 核心实现（`ProviderCache` / `ProviderCacheConfig`）下沉到 `agent_world_net`。
  - 将 `distributed_dht_cache` 核心实现（`CachedDht` / `DhtCacheConfig`）下沉到 `agent_world_net`。
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保 `agent_world` 调用方无需改动。

### In Scope（八次扩展阶段）
- 继续推进 `agent_world_net` 的观察者链路能力下沉：
  - 将 `distributed_head_follow` 核心实现（`HeadFollower` / `HeadUpdateDecision`）下沉到 `agent_world_net`。
  - 将 `distributed_observer` 核心实现（`ObserverClient` / `ObserverSubscription` / `HeadSync*` / `HeadFollowReport`）下沉到 `agent_world_net`。
- 复用 `agent_world::runtime` 既有 bootstrap 与 `BlobStore`/`World` 类型契约，保持行为一致。
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保 `agent_world` 调用方无需改动。

### In Scope（九次扩展阶段）
- 继续推进 `agent_world_net` 的观察回放校验能力下沉：
  - 将 `distributed_observer_replay` 核心实现下沉到 `agent_world_net`（head 回放校验、DHT 回放校验、blob hash 校验）。
  - `head_follow` 复用同一回放实现，消除重复逻辑，保持校验语义一致。
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保 `agent_world` 调用方无需改动。

### In Scope（十次扩展阶段）
- 继续推进 `agent_world_net` 的 world bootstrap 能力下沉：
  - 将 `distributed_bootstrap` 核心实现下沉到 `agent_world_net`（`bootstrap_world_from_head` / `bootstrap_world_from_head_with_dht` / `bootstrap_world_from_dht`）。
  - `head_follow` 复用同一 bootstrap 实现，消除本地重复 bootstrap 逻辑，保持行为一致。
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保 `agent_world` 调用方无需改动。

### In Scope（十一次扩展阶段）
- 继续推进 `agent_world_consensus` 的共识主流程能力下沉：
  - 将 `distributed_consensus` 核心实现下沉到 `agent_world_consensus`（`QuorumConsensus`、共识提案/投票 helper、membership 变更与快照读写）。
  - 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十二次扩展阶段）
- 继续推进 `agent_world_consensus` 的协调与聚合能力下沉：
  - 将 `distributed_lease` 核心实现下沉到 `agent_world_consensus`（`LeaseManager` / `LeaseState` / `LeaseDecision`）。
  - 将 `distributed_mempool` 核心实现下沉到 `agent_world_consensus`（`ActionMempool` / `ActionBatchRules` / `ActionMempoolConfig`）。
- 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十三次扩展阶段）
- 继续推进 `agent_world_consensus` 的 membership 同步核心路径下沉（先做低耦合切片）：
  - 将 `distributed_membership_sync` 中 `MembershipSyncClient` 及其直接依赖类型下沉到 `agent_world_consensus`：
    - `MembershipDirectoryAnnounce` / `MembershipKeyRevocationAnnounce`
    - `MembershipDirectorySigner` / `MembershipDirectorySignerKeyring`
    - `MembershipSnapshotRestorePolicy` / `MembershipRevocationSyncPolicy`
    - `MembershipSnapshotAudit*` / `MembershipRestoreAuditReport`
    - `MembershipAuditStore` / `InMemoryMembershipAuditStore` / `FileMembershipAuditStore`
    - `MembershipSyncSubscription` / `MembershipSyncReport` / `MembershipRevocationSyncReport`
  - `recovery` / `reconciliation` 子系统仍暂留 `agent_world::runtime`，由 `agent_world_consensus` 继续桥接导出，避免一次性迁移超大模块。
- 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十四次扩展阶段）
- 继续推进 `agent_world_consensus` 的 membership 吊销对账路径下沉：
  - 将 `distributed_membership_sync/reconciliation.rs` 核心实现下沉到 `agent_world_consensus`：
    - checkpoint 发布/消费与对账
    - reconcile 调度策略、调度状态存储、协调锁
    - anomaly alert 评估、去重、下发
  - `recovery` 子系统仍暂留 `agent_world::runtime`，由 `agent_world_consensus` 继续桥接导出，避免一次性迁移超大模块。
- 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十五次扩展阶段）
- 继续推进 `agent_world_consensus` 的 membership 告警恢复路径下沉（先做低耦合切片）：
  - 将 `distributed_membership_sync/recovery.rs` 中告警恢复核心实现下沉到 `agent_world_consensus`：
    - ack retry：pending alert 状态、`MembershipRevocationAlertAckRetryPolicy`、告警重试/缓冲/丢弃策略
    - dead-letter：dead-letter store（append/list/replace）与 delivery metrics 导出
    - recovery store：pending alert 持久化（in-memory / file）与 legacy 兼容解码
    - coordinator state store：跨节点协调租约状态持久化（in-memory / file）
    - dead-letter replay：从 dead-letter 回放到 pending 的调度 helper（不含 replay policy/governance）
  - 为满足 1200 行约束，在 `agent_world_consensus` 内对 recovery 模块做多文件拆分（`membership_recovery/*`）。
  - `recovery/replay*.rs`（回放策略、自适应调参、治理/归档）仍暂留 `agent_world::runtime`，由 `agent_world_consensus` 继续桥接导出，避免一次性迁移超大模块。
- 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十六次扩展阶段）
- 继续推进 `agent_world_consensus` 的 membership dead-letter replay 策略与持久化下沉（先做 policy 核心切片）：
  - 将 `distributed_membership_sync/recovery/replay.rs` 核心实现下沉到 `agent_world_consensus`：
    - dead-letter replay：`MembershipRevocationDeadLetterReplayPolicy` / `*ScheduleState`
    - replay state store / policy store（in-memory / file）
    - 自适应推荐：metrics 聚合与 policy recommendation / guard / step clamp
    - 回滚保护：`MembershipRevocationDeadLetterReplayRollbackGuard` 与 stable policy 回退
    - 协调运行：coordinated replay schedule（lease coordinator）
  - `replay_audit` / `replay_archive*` 子系统仍暂留 `agent_world::runtime`，由 `agent_world_consensus` 继续桥接导出，避免一次性迁移超大模块。
- 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十七次扩展阶段）
- 继续推进 `agent_world_consensus` 的 membership dead-letter replay 审计/告警/治理下沉（先做 `replay_audit` 核心切片）：
  - 将 `distributed_membership_sync/recovery/replay_audit.rs` 核心实现下沉到 `agent_world_consensus`：
    - replay policy adoption audit record 与 audit store（in-memory / file）
    - rollback alert policy/state 与 state store（in-memory / file）
    - rollback governance level/policy/state 与 state store（in-memory / file）
    - governance audit record/store 与 recovery drill report helper
    - 带 audit/alert/governance 的 replay schedule helper（coordinated + persisted policy）
  - `replay_archive*`（归档/联邦 event bus 等）仍暂留 `agent_world::runtime`，由 `agent_world_consensus` 继续桥接导出，避免一次性迁移超大模块。
- 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十八次扩展阶段）
- 继续推进 `agent_world_consensus` 的 membership dead-letter replay 归档/联邦 event bus 下沉（完成 `replay_archive*` 切片）：
  - 将 `distributed_membership_sync/recovery/replay_archive.rs` / `replay_archive_tiered.rs` / `replay_archive_federated.rs` 核心实现下沉到 `agent_world_consensus`：
    - governance audit retention store（in-memory / file）与 prune helper
    - recovery drill schedule policy/state 与 state store（in-memory / file）
    - tiered offload policy/report 与 drill alert state store（in-memory / file）
    - federated recovery drill alert event bus（in-memory / file）与 aggregate query helper
    - composite sequence cursor state/store（in-memory / file）
  - 保持 `agent_world_consensus` 对外导出名与调用语义不变，确保调用方无需改动。
  - 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（十九次扩展阶段）
- 继续推进 `agent_world_net` 的分布式存储/校验计算下沉（执行结果写入与 head 验证）：
  - 将 `distributed_storage.rs` 核心实现下沉到 `agent_world_net`：
    - `store_execution_result`（block/snapshot/journal refs 写入）
    - `ExecutionWriteConfig` / `ExecutionWriteResult`（保持类型与语义兼容）
  - 将 `distributed_validation.rs` 核心实现下沉到 `agent_world_net`：
    - `validate_head_update` / `assemble_snapshot` / `assemble_journal`
    - `HeadValidationResult`（保持类型与语义兼容）
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（二十次扩展阶段）
- 继续推进 `agent_world_net` 的 p2p 网络实现下沉（`libp2p` adapter）：
  - 将 `libp2p_net.rs` 核心实现下沉到 `agent_world_net`（保持 feature name 为 `libp2p`）：
    - `Libp2pNetwork` / `Libp2pNetworkConfig`
    - gossipsub publish/subscribe 与 request/response
    - Kademlia DHT provider/head/membership record CRUD
  - `agent_world_net --features libp2p` 可独立编译通过（并尽量覆盖最小单测）。
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保调用方无需改动。
- 保持 `agent_world` runtime 兼容导出不变，迁移过程允许短期并行实现共存。

### In Scope（二十一次扩展阶段）
- 针对 `agent_world_net` 的 `libp2p` adapter 做最小闭环增强，降低“仅能编译但不可用”的风险：
  - 增加最小可观测性（用于测试与调试）：
    - 可查询 listen addrs（首帧监听就绪）
    - 可查询已连接 peers（用于等待连接建立）
  - 新增跨节点 smoke test：
    - request/response 可跨 peer 收发
    - gossipsub publish 可在 peer 间传播
- 保持 `agent_world_net` 对外导出名与调用语义不变，确保调用方无需改动。

### In Scope（二十二次扩展阶段）
- 进一步消除 `agent_world::runtime` 与新 crate 的重复实现，先完成“同源实现复用”切片：
  - `agent_world::runtime` 的以下模块改为直接复用新 crate 源实现（`include!`）：
    - `distributed_net.rs` <- `agent_world_net/src/network.rs`
    - `distributed_dht.rs` <- `agent_world_net/src/dht.rs`
    - `distributed_gateway.rs` <- `agent_world_net/src/gateway.rs`
    - `distributed_lease.rs` <- `agent_world_consensus/src/lease.rs`
    - `distributed_mempool.rs` <- `agent_world_consensus/src/mempool.rs`
  - 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
  - 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十三次扩展阶段）
- 继续推进 runtime 与 `agent_world_net` 的同源实现复用，覆盖网络缓存与索引链路核心：
  - `agent_world::runtime` 的以下模块改为直接复用 `agent_world_net` 源实现（`include!`）：
    - `distributed_client.rs` <- `agent_world_net/src/client.rs`
    - `distributed_index.rs` <- `agent_world_net/src/index.rs`
    - `distributed_index_store.rs` <- `agent_world_net/src/index_store.rs`
    - `distributed_provider_cache.rs` <- `agent_world_net/src/provider_cache.rs`
    - `distributed_dht_cache.rs` <- `agent_world_net/src/dht_cache.rs`
  - 在 `agent_world_net` 增加兼容导出命名层（`distributed_*` alias），保证同源文件在两侧 crate 均可编译。
  - 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
  - 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十四次扩展阶段）
- 继续推进 runtime 与 `agent_world_net` 的同源实现复用，覆盖 bootstrap/observer 与校验存储链路：
  - `agent_world::runtime` 的以下模块改为直接复用 `agent_world_net` 源实现（`include!`）：
    - `distributed_bootstrap.rs` <- `agent_world_net/src/bootstrap.rs`
    - `distributed_head_follow.rs` <- `agent_world_net/src/head_follow.rs`
    - `distributed_observer.rs` <- `agent_world_net/src/observer.rs`
    - `distributed_observer_replay.rs` <- `agent_world_net/src/observer_replay.rs`
    - `distributed_storage.rs` <- `agent_world_net/src/execution_storage.rs`（保留 runtime 侧 `ExecutionWrite*` 类型定义）
    - `distributed_validation.rs` <- `agent_world_net/src/head_validation.rs`（保留 runtime 侧 `HeadValidationResult` 类型定义）
  - 在 `agent_world_net` 增加补充兼容导出命名层（`distributed_*` + `blob_store/world/events/segmenter/snapshot/types/world_event` alias），保证同源文件在两侧 crate 均可编译。
  - 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
  - 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十五次扩展阶段）
- 继续推进 runtime 与 `agent_world_consensus` 的同源实现复用，覆盖 quorum 共识主流程：
  - `agent_world::runtime` 的以下模块改为直接复用 `agent_world_consensus` 源实现（`include!`）：
    - `distributed_consensus.rs` <- `agent_world_consensus/src/quorum.rs`
  - 在 `agent_world_consensus` 增加兼容导出命名层（`distributed/distributed_dht/distributed_lease/error/util` alias），保证同源文件在两侧 crate 均可编译。
  - 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
  - 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十六次扩展阶段）
- 继续推进 runtime 与 `agent_world_consensus` 的同源实现复用，覆盖 membership sync 主体逻辑：
  - `agent_world::runtime` 的 `distributed_membership_sync.rs` 改为直接复用 `agent_world_consensus/src/membership.rs`（`include!`），保留 runtime 本地 `reconciliation/recovery` 子模块导出不变。
  - 在 runtime `distributed_membership_sync` 增加 `shared` 包装与兼容 alias 层（`distributed/distributed_consensus/distributed_dht/distributed_net/error/util/membership_logic`），确保同源文件在两侧 crate 均可编译。
  - 在 `agent_world_consensus` 增补 membership 复用所需 alias 导出（`distributed_consensus`、`distributed_net`、`distributed_dht::MembershipDirectorySnapshot`、`util::to_canonical_cbor`）。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十七次扩展阶段）
- 继续推进 runtime 与 `agent_world_consensus` 的同源实现复用，覆盖 membership reconcile 子模块：
  - `agent_world::runtime::distributed_membership_sync::reconciliation.rs` 改为直接复用 `agent_world_consensus/src/membership_reconciliation.rs`（`include!`）。
  - `agent_world_consensus/src/membership_reconciliation.rs` 调整为 `super::*` 兼容导入（`distributed/error/membership/membership_logic`），保证同源文件在两侧 crate 均可编译。
  - 在 runtime `distributed_membership_sync` 增加 `membership` alias 层以对齐共享文件路径解析。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十八次扩展阶段）
- 继续推进 runtime 与 `agent_world_consensus` 的同源实现复用，覆盖 membership recovery 子模块：
  - `agent_world::runtime::distributed_membership_sync::recovery.rs` 改为路径模块复用
    `agent_world_consensus/src/membership_recovery/mod.rs`（`#[path] mod + pub use`）。
  - `agent_world_consensus/src/membership_recovery/*` 调整为 `super::*` 相对导入（替换 `crate::*`），保证 recovery 共享源码在两侧 crate 均可编译。
  - 在 runtime `distributed_membership_sync::recovery` 增加 `membership/membership_logic/membership_reconciliation`
    alias 层，确保共享 recovery 模块解析路径稳定。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（二十九次扩展阶段）
- 继续推进 runtime 与 `agent_world_consensus` 的重复实现清理，完成 recovery 旧文件物理下线：
  - 删除 `agent_world::runtime/distributed_membership_sync/recovery/*` 下已不再编译的旧实现文件：
    - `dead_letter.rs` / `replay.rs` / `replay_audit.rs`
    - `replay_archive.rs` / `replay_archive_tiered.rs` / `replay_archive_federated.rs`
  - 运行时 recovery 能力保持由 `recovery.rs -> agent_world_consensus/src/membership_recovery/mod.rs` 同源复用提供，导出 API 不变。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过 workspace 级回归验证收口，确保 CI 路径可直接覆盖该切片。

### In Scope（三十次扩展阶段）
- 继续推进 runtime 与 `agent_world_consensus` 的同源复用维护性收敛，整理 membership recovery 导出层：
  - 将 `agent_world::runtime::distributed_membership_sync.rs` 中超长 `recovery` re-export 清单拆分到独立模块文件 `recovery_exports.rs`，按能力归档导出，降低主文件维护成本。
  - 将仅测试依赖的 composite sequence cursor 类型导出改为 `#[cfg(test)]` 门控，避免常规 `cargo check` 命中无效导入 warning。
  - 在 runtime `recovery.rs` 共享模块包装层补充 `#[allow(unused_imports)]`，消除双上下文 include 时由上游公共导出引入的噪音 warning。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过定向 `cargo check` + federated recovery replay 测试验证收口。

### In Scope（三十一次扩展阶段）
- 继续推进 runtime 同源复用上下文的 warning 分层治理，聚焦 `distributed_membership_sync`：
  - 删除 `distributed_membership_sync` 内 `logic.rs` 的重复模块编译路径（移除 `mod logic;`，统一复用 `membership_logic`），降低重复 dead_code 噪音。
  - runtime recovery 共享包装层增加 `dead_code` + `unused_imports` 局部门控，避免 consensus recovery 共享源码在 runtime 非测试上下文产生无效 warning。
  - 保持 recovery 与 membership sync 对外导出 API 不变，不改业务语义。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过定向 `cargo check` 与 membership/recovery federated 回归测试验证收口。

### In Scope（三十二次扩展阶段）
- 继续推进 runtime 同源复用上下文的 warning 分层治理，聚焦 `distributed_observer_replay`：
  - 在 runtime `distributed_observer_replay` 包装层增加局部 `dead_code` 门控，抑制共享 `observer_replay` 在 runtime 侧未导出 helper（`replay_validate_head_with_dht`）带来的孤立 warning。
  - 保持 `agent_world_net` 共享实现与 runtime 对外 API 不变，不改动验证流程语义。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过定向 `cargo check` + distributed observer replay 相关集成测试验证收口。

### In Scope（三十三次扩展阶段）
- 对 runtime 与 `agent_world_net` 剩余 `include!` 同源复用模块做 warning 基线评估：
  - 扫描 `runtime` 侧全部 `include!` 入口，确认评估范围完整。
  - 执行 `agent_world` 在 `--all-targets` / `--all-targets --features wasmtime` 的编译检查，验证无新增上下文特异 warning。
  - 保持代码行为与导出 API 不变，不做额外门控扩散。
- 保持 `agent_world` 对外 API 命名与行为语义兼容（`runtime` 导出名不变）。
- 通过编译检查与定向集成测试确认“无新增 warning”状态稳定。

### In Scope（三十四次扩展阶段）
- 把 include 复用 warning 基线检查固化为可复用脚本并接入 CI 统一入口：
  - 新增脚本封装 `runtime include!` 入口扫描与双维度编译检查（`--all-targets` / `--all-targets --features wasmtime`）。
  - 在 `scripts/ci-tests.sh` 中接入该脚本，确保 pre-commit 与 CI 复用同一基线门禁。
  - 失败时输出 warning 命中日志，便于定位回归来源。
- 保持代码行为与导出 API 不变，不改业务语义。
- 通过脚本自检 + 分布式定向集成测试验证收口。

### In Scope（三十五次扩展阶段）
- 推进 `agent_world_net` 脱离对 `agent_world` 的硬依赖，并启动 runtime 对 net 的直接依赖切换：
  - `agent_world_net` 移除 `agent_world` 依赖，保留纯网络能力导出（network/dht/client/index/cache/gateway）。
  - 为 net crate 增加本地 `WorldError` 与最小 `modules/distributed_storage` 数据面，保证 `agent_world_net --features libp2p` 可独立编译和测试。
  - `agent_world` 新增对 `agent_world_net` 的依赖，先将 `distributed_net/dht/client/gateway/index_store/provider_cache/dht_cache` 切到直接 re-export。
  - 对 `distributed_index` 保持 runtime 本地实现，避免 `ExecutionWriteResult` 在 net/runtime 两侧类型尚未统一时产生签名断裂。
- 保持对外 API 语义兼容，优先保障多节点与一致性回归通过。

### In Scope（三十六次扩展阶段）
- 将 net/runtime 共享错误类型下沉到 `agent_world_proto`：
  - 新增 `agent_world_proto::world_error::WorldError`，承载 net 链路所需错误语义（network request/validation/io/serde）。
  - `agent_world_net` 改为直接复用 proto `WorldError`，去除本地重复定义。
  - 保持 `agent_world` 通过 `From<agent_world_net::WorldError>` 桥接，避免 runtime 侧错误语义回归。
- 保持 `agent_world_net` 基础化目标不变，并为后续 `ExecutionWriteResult` 公共数据面下沉做准备。

### In Scope（三十七次扩展阶段）
- 将执行产物索引相关公共数据面下沉到 `agent_world_proto` 并收敛 runtime/net 双类型：
  - 新增 `agent_world_proto::distributed_storage`：
    - `SegmentConfig` / `JournalSegmentRef`
    - `ExecutionWriteConfig` / `ExecutionWriteResult`
  - `agent_world_net::distributed_storage` 改为直接复用 proto 类型，去除 net 本地重复定义。
  - `agent_world::runtime::distributed_storage` 改为复用 net/proto 统一类型，保持 `store_execution_result` 行为不变。
  - `agent_world::runtime::segmenter` 复用 proto 的 `SegmentConfig` / `JournalSegmentRef`，避免并行结构体漂移。
  - `agent_world::runtime::distributed_index` 改为调用 `agent_world_net` 索引实现并做错误桥接，消除 runtime 本地重复逻辑。
- 保持 `agent_world` 对外 API 命名与行为语义兼容，确保分布式一致性/多节点回归无变化。

### In Scope（三十八次扩展阶段）
- 清理 `distributed_storage` / `distributed_validation` 的 runtime include 包装层：
  - `agent_world::runtime::distributed_storage.rs` 移除 `include!`，改为 runtime 本地直接实现（继续复用 net/proto 统一类型）。
  - `agent_world::runtime::distributed_validation.rs` 移除 `include!`，改为 runtime 本地直接实现（`HeadValidationResult`/校验流程语义不变）。
  - include warning 基线脚本继续收口，确认去掉两处 include 后无新增 warning。
- 保持 `agent_world` 对外 API 命名与行为语义兼容，不改变执行产物写入与 head 校验结果。

### In Scope（三十九次扩展阶段）
- 继续清理 runtime include 包装层，优先收敛低耦合 bootstrap 入口：
  - `agent_world::runtime::distributed_bootstrap.rs` 移除 `include!`，改为 runtime 本地直接实现。
  - 保持 `bootstrap_world_from_head` / `bootstrap_world_from_head_with_dht` / `bootstrap_world_from_dht` 对外签名与行为不变。
  - 通过 bootstrap 定向测试 + include warning 基线脚本验证收口。

### In Scope（四十次扩展阶段）
- 继续推进 T78，优先收敛低耦合 head follow 入口：
  - `agent_world::runtime::distributed_head_follow.rs` 移除 `include!`，改为 runtime 本地直接实现。
  - 保持 `HeadFollower` / `HeadUpdateDecision` 对外签名与行为不变，继续复用 runtime 本地 bootstrap/client/dht 链路。
  - 通过 head_follow + observer_sync 定向测试与 include warning 基线脚本验证无回归。

### Out of Scope（本次不做）
- 不在本轮强制把 `agent_world` 现有 runtime 实现文件全部物理迁移到新 crate。
- 不做协议层额外重构（协议仍以 `agent_world_proto` 为主）。
- 不改动业务语义与现有行为策略。

## 接口 / 数据

### `agent_world_net`（边界）
- 负责导出：
  - 网络抽象与 in-memory 网络实现。
  - 分布式网络客户端/网关/观察者等网络链路能力。
  - 可选 `libp2p` 能力导出（feature 透传）。

### `agent_world_consensus`（边界）
- 负责导出：
  - 共识主流程（提案/投票/提交）相关能力。
  - 成员目录同步与恢复能力。
  - 吊销对账、调度、告警、恢复、审计相关能力。
- `distributed_membership_sync` 及其子模块归属此 crate 的能力面。

## 里程碑
- P1：设计文档与项目管理文档完成。
- P2：新 crate 脚手架与 workspace 接线完成。
- P3：导出能力面落地（net/consensus 分类稳定）。
- P4：编译与定向测试回归通过，项目文档收口。
- P5：`distributed_net` 核心实现下沉到 `agent_world_net`。
- P6：扩展阶段回归验证与文档收口。
- P7：`distributed_dht` 核心实现下沉到 `agent_world_net`。
- P8：二次扩展阶段回归验证与文档收口。
- P9：`distributed_client` 核心实现下沉到 `agent_world_net`。
- P10：三次扩展阶段回归验证与文档收口。
- P11：`distributed_gateway` 核心实现下沉到 `agent_world_net`。
- P12：四次扩展阶段回归验证与文档收口。
- P13：`distributed_index` 核心实现下沉到 `agent_world_net`。
- P14：五次扩展阶段回归验证与文档收口。
- P15：完成 `agent_world_net` 内部模块化拆分（多文件 + 单测迁移）。
- P16：六次扩展阶段回归验证与文档收口。
- P17：`distributed_index_store` / `distributed_provider_cache` / `distributed_dht_cache` 核心实现下沉到 `agent_world_net`。
- P18：七次扩展阶段回归验证与文档收口。
- P19：`distributed_head_follow` / `distributed_observer` 核心实现下沉到 `agent_world_net`。
- P20：八次扩展阶段回归验证与文档收口。
- P21：`distributed_observer_replay` 核心实现下沉到 `agent_world_net`，并复用到 `head_follow`。
- P22：九次扩展阶段回归验证与文档收口。
- P23：`distributed_bootstrap` 核心实现下沉到 `agent_world_net`，并复用到 `head_follow`。
- P24：十次扩展阶段回归验证与文档收口。
- P25：`distributed_consensus` 核心实现下沉到 `agent_world_consensus`。
- P26：十一次扩展阶段回归验证与文档收口。
- P27：`distributed_lease` / `distributed_mempool` 核心实现下沉到 `agent_world_consensus`。
- P28：十二次扩展阶段回归验证与文档收口。
- P29：`distributed_membership_sync` 的签名/审计/同步核心（不含 recovery/reconciliation）下沉到 `agent_world_consensus`。
- P30：十三次扩展阶段回归验证与文档收口。
- P31：`distributed_membership_sync/reconciliation.rs` 核心实现下沉到 `agent_world_consensus`。
- P32：十四次扩展阶段回归验证与文档收口。
- P33：`distributed_membership_sync/recovery.rs` 告警恢复核心实现下沉到 `agent_world_consensus`。
- P34：十五次扩展阶段回归验证与文档收口。
- P35：`distributed_membership_sync/recovery/replay.rs` 核心实现下沉到 `agent_world_consensus`。
- P36：十六次扩展阶段回归验证与文档收口。
- P37：`distributed_membership_sync/recovery/replay_audit.rs` 核心实现下沉到 `agent_world_consensus`。
- P38：十七次扩展阶段回归验证与文档收口。
- P39：`distributed_membership_sync/recovery/replay_archive*.rs` 核心实现下沉到 `agent_world_consensus`。
- P40：十八次扩展阶段回归验证与文档收口。
- P41：`distributed_storage.rs` 核心实现下沉到 `agent_world_net`。
- P42：`distributed_validation.rs` 核心实现下沉到 `agent_world_net`。
- P43：十九次扩展阶段回归验证与文档收口。
- P44：`libp2p_net.rs` 核心实现下沉到 `agent_world_net`。
- P45：二十次扩展阶段回归验证与文档收口。
- P46：`libp2p` adapter 最小可观测性补齐 + 跨节点 smoke test。
- P47：二十一次扩展阶段回归验证与文档收口。
- P48：完成 runtime 与新 crate 的同源实现复用切片（net+dht+gateway+lease+mempool）。
- P49：二十二次扩展阶段回归验证与文档收口。
- P50：完成 runtime 与 `agent_world_net` 同源实现复用切片（client+index+cache+index_store）。
- P51：二十三次扩展阶段回归验证与文档收口。
- P52：完成 runtime 与 `agent_world_net` 同源实现复用切片（bootstrap+head_follow+observer+observer_replay+storage+validation）。
- P53：二十四次扩展阶段回归验证与文档收口。
- P54：完成 runtime 与 `agent_world_consensus` 同源实现复用切片（distributed_consensus）。
- P55：二十五次扩展阶段回归验证与文档收口。
- P56：完成 runtime 与 `agent_world_consensus` 同源实现复用切片（distributed_membership_sync 主体）。
- P57：二十六次扩展阶段回归验证与文档收口。
- P58：完成 runtime 与 `agent_world_consensus` 同源实现复用切片（distributed_membership_sync/reconciliation）。
- P59：二十七次扩展阶段回归验证与文档收口。
- P60：完成 runtime 与 `agent_world_consensus` 同源实现复用切片（distributed_membership_sync/recovery）。
- P61：二十八次扩展阶段回归验证与文档收口。
- P62：完成 runtime `distributed_membership_sync/recovery/*` 旧实现文件清理。
- P63：二十九次扩展阶段回归验证与文档收口。
- P64：完成 runtime `distributed_membership_sync` recovery 导出清单分组与测试导出门控收敛。
- P65：三十次扩展阶段回归验证与文档收口。
- P66：完成 runtime `distributed_membership_sync` 同源复用 dead_code warning 分层治理。
- P67：三十一次扩展阶段回归验证与文档收口。
- P68：完成 runtime `distributed_observer_replay` 孤立 dead_code warning 分层治理。
- P69：三十二次扩展阶段回归验证与文档收口。
- P70：完成 runtime 与 `agent_world_net` 剩余 include 模块 warning 基线评估。
- P71：三十三次扩展阶段回归验证与文档收口。
- P72：完成 include warning 基线脚本化与 CI 接入收口。
- P73：完成 `agent_world_net` 去除 `agent_world` 依赖与 runtime 首批直接依赖切换收口。
- P74：完成 `WorldError` 下沉到 `agent_world_proto` 并在 net crate 收敛复用。
- P75：完成执行产物索引数据面（`ExecutionWrite*` / `Segment*`）下沉到 `agent_world_proto` 并收敛 runtime/net 双类型。
- P76：完成 runtime `distributed_storage` / `distributed_validation` 去 include 包装层收口。
- P77：完成 runtime `distributed_bootstrap` 去 include 包装层收口。
- P78：完成 runtime `distributed_head_follow` 去 include 包装层收口（T78 子步骤）。

## 风险
- 仅做边界导出时，可能出现“新 crate 已存在但实现仍在 `agent_world`”的过渡期认知偏差。
- 导出项过多时，维护成本上升；需要后续按使用频率做分组收敛。
- feature 透传（尤其 `libp2p`）若配置不完整，可能导致构建行为与预期不一致。
- 迁移期间若同名类型在 runtime 与新 crate 并行存在，可能引入调用方误用；需要通过文档和导出边界降低歧义。
- `distributed_client` 涉及协议编解码与错误映射，若 canonical CBOR 行为偏差可能导致跨节点请求兼容性回归，需要定向测试覆盖。
- `distributed_gateway` 涉及 action 发布 topic 与序列化，若 topic 生成或 payload 编码偏差，可能导致上游节点收不到动作，需要保留端到端发布测试。
- `distributed_index` 涉及执行产物 hash 聚合与 provider 发布，若 hash 收集集合不一致会导致拉取路径缺 provider，需要保留执行产物全量索引测试。
- `agent_world_net` 模块化拆分若处理不当可能引入循环依赖或可见性收窄，需保持导出面稳定并保留全量 `agent_world_net` 单测回归。
- cache/index store 下沉涉及 TTL、provider 截断与 head 缓存刷新路径，若时间窗口判定偏差会导致命中过期数据或频繁回源，需保留缓存命中/过期刷新测试。
- 观察者链路下沉涉及 head 选择与同步回放入口，若 `world_id` 校验或冲突判定偏差会导致错误 world 被应用，需要保留订阅/同步与 head 冲突路径测试。
- 观察回放校验下沉涉及 block/snapshot/journal 三段数据一致性验证，若 hash 校验或装配顺序偏差会导致误判，需要保留回放 round-trip 与 DHT 路径测试。
- world bootstrap 下沉涉及 head 获取与回放结果装配，若 fallback 路径偏差会导致启动失败，需要保留 head 直连与 DHT 启动路径测试。
- 共识主流程下沉涉及 quorum 阈值判定、提案/投票终态与快照恢复，若状态机迁移偏差会导致错误提交或恢复失败，需要保留提案冲突、否决终态与快照 round-trip 测试。
- lease/mempool 下沉涉及主写租约时序与 action 批次切片，若租约续期/过期判定或 payload 约束偏差会导致 leader 抖动与动作丢弃，需要保留租约续期、过期接管与 batch 限流测试。
- membership sync 核心路径下沉涉及签名验证策略、DHT 恢复与审计落盘，若策略对象或签名载荷行为偏差会导致合法快照被拒或非法快照被接受，需要保留 keyring 签名/验签、restore 审计与 key revocation 策略测试。
- membership reconcile 下沉涉及跨节点 revoked set 对账、调度持久化与协调锁，若 checkpoint hash、时间窗口或锁租约判定偏差会导致重复告警或漏合并，需要保留 reconcile merge、schedule due 与 coordinated lock 相关测试。
- membership recovery 下沉涉及 pending alert buffer/ack-retry/dead-letter 与落盘格式兼容，若 retry/backoff/capacity 判定或 legacy 解码偏差会导致漏告警或重复告警，需要保留 ack-retry 缓冲/丢弃、dead-letter 回放与持久化 round-trip 测试。
- runtime 通过 `shared` 包装 include 共用 membership 文件时，若 alias 层与测试编译路径不一致，可能导致仅 `cargo check` 通过但 `cargo test` 失败，需要以 workspace 级测试作为收口门禁。
- reconcile 子模块同源复用后，`membership_logic` 与 `to_canonical_cbor` 的可见性边界若调整不当，可能造成跨模块编译失败或测试回归，需要持续保持 `super::*` alias 的双上下文一致性。
- recovery 子模块切换为 `#[path]` 复用后，若相对路径或父级 alias 发生漂移，可能导致仅单 crate 通过而 workspace 失败，需要维持 `membership_recovery/*` 的相对导入一致性并用 workspace 测试收口。
- recovery 旧文件物理删除后，若未来误将 runtime 本地子模块重新声明为编译入口，可能触发路径找不到错误；需要以 `recovery.rs` 单入口同源复用为准并在项目文档持续约束。
- recovery 导出分组后若 runtime 与 consensus 端的测试门控不一致，可能出现“`cargo check` 干净但联调测试缺类型”回归，需要保留 federated replay 相关定向测试作为回归门禁。
- warning 分层治理若范围过大（全局 allow）可能掩盖真实回归；需要保持局部门控并持续用定向测试覆盖 membership/recovery 关键路径。
- observer replay warning 治理若误扩散到共享源码层，可能影响 `agent_world_net` 端可见性语义；需将门控限定在 runtime 包装层并保留跨节点集成测试回归。
- warning 基线评估若缺少 feature/target 维度，可能出现“当前无 warning、切换 feature 后回归”的盲区；需固定 `--all-targets` 与 `--features wasmtime` 双路径校验。
- warning 基线脚本若未和 CI/本地入口统一，可能出现“手工通过但流水线遗漏”的门禁漂移；需统一复用 `scripts/ci-tests.sh` 作为执行入口。
- `agent_world` 切换到 `agent_world_net` 直接依赖后，`WorldError` 与 `ExecutionWriteResult` 若双定义并存，容易出现隐式签名不兼容；需通过显式 `From` 桥接与 runtime 本地 wrapper 稳定过渡。
- `ExecutionWriteConfig` 下沉到 proto 后，若未来 runtime/net 对分段参数解释不一致，可能导致跨节点数据组装差异；需以统一类型 + 分布式回归测试持续约束。
- `distributed_storage` / `distributed_validation` 改为 runtime 本地实现后，若后续 net 侧同名逻辑继续演进可能出现实现漂移；需通过定向测试与阶段性比对保持语义一致。
- `distributed_bootstrap` 去 include 后，若 observer replay 或 client/DHT 依赖链后续变更未同步，可能导致 bootstrap 路径漂移；需保留 bootstrap round-trip 定向测试作为门禁。
