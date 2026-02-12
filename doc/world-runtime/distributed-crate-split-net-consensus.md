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
