# Agent World Runtime：分布式计算与存储（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-runtime/distributed-runtime.md`）
- [x] 输出项目管理文档（本文件）
- [x] 与 runtime 分册术语对齐（事件/快照/模块存储）

### 1. 协议与数据结构冻结
- [x] 冻结 gossipsub topic 与 rr 协议命名
- [x] 固化消息封装（ActionEnvelope/WorldHead/BlockAnnounce）
- [x] 定义错误码与重试语义映射
- [x] 选择并固化 wire encoding（CBOR/二进制）

### 2. 内容寻址存储与分片
- [x] 抽象 BlobStore 接口（put/get/has）
- [x] 本地 CAS 参考实现（LocalCasStore）
- [x] 快照/日志分片与 manifest 生成
- [x] 本地 pin/evict 策略（pins.json + prune_unpinned）
- [x] provider 发布与索引（DHT provider）
- [x] 远端缓存策略（provider cache/republish）
  - [x] provider cache 与 TTL 规则
  - [x] provider republish 机制
  - [x] provider cache 测试

### 3. 执行节点协同
- [x] 执行节点拉取 wasm/state/journal 的网络客户端（基于 DistributedNetwork）
- [x] 执行结果写入 storage（block/journal/snapshot）
- [x] 头指针更新与回放校验流程

### 4. 网络适配器原型（P1）
- [x] 定义 DistributedNetwork 抽象接口
- [x] 提供 InMemoryNetwork 参考实现（测试用）
- [x] Libp2pNetwork 骨架（peer_id/keypair，占位实现）
- [x] DHT 适配器抽象 + InMemoryDht 参考实现
- [x] libp2p Swarm 事件循环（gossipsub + rr 基线）
- [x] libp2p 实现（gossipsub + rr + dht）

### 5. Sequencer 与批处理
- [x] action mempool 聚合与去重
- [x] 批次生成与排序规则
- [x] 租约式单写者切换与超时恢复

### 6. 索引与发现
- [x] world head 发布到 DHT（libp2p/kad 适配）
- [x] 内容 provider 索引与查询（libp2p/kad 适配）
- [x] 轻量 index store 接口（可选）

### 7. Gateway/Observer
- [x] 提交 action 的网关 API
- [x] 事件订阅与 head 追踪
- [x] 观察者回放验证工具

### 8. 测试与验证
- [x] 本地多节点集成测试（3 节点：exec+storage+gateway）
- [x] 恢复测试（重启后继续同步 head）
- [x] 数据一致性测试（state_root 校验）

### 9. P2.1 索引缓存封装
- [x] CachedDht 封装（provider/head 缓存）
- [x] TTL 与 provider 截断策略
- [x] CachedDht 单元测试

### 10. P2.2 观察者 DHT 引导
- [x] 观察者回放支持 DHT head 引导
- [x] DHT 引导单元测试

### 11. P2.3 ProviderCache 执行接入
- [x] 执行结果发布接入 ProviderCache
- [x] ProviderCache 发布单元测试

### 12. P2.4 Provider-aware 拉取
- [x] DistributedNetwork 支持 provider 列表请求
- [x] Observer 回放拉取使用 provider 列表
- [x] provider-aware 客户端单元测试

### 13. P2.5 Provider-aware 拉取封装
- [x] DistributedClient 提供 DHT 拉取封装
- [x] Observer 回放复用 DHT 拉取封装
- [x] DHT 拉取封装单元测试

### 14. P2.6 模块拉取封装
- [x] 模块 manifest 拉取走 DHT provider-aware 路径
- [x] 模块 artifact 拉取走 DHT provider-aware 路径
- [x] 模块拉取封装单元测试

### 15. P2.7 libp2p provider 重发
- [x] libp2p provider 重发配置与定时触发
- [x] provider 重发间隔单元测试

### 16. P2.8 模块加载补全
- [x] World 模块加载支持 DHT 拉取补全
- [x] 模块加载补全单元测试

### 17. P2.9 模块预热加载
- [x] World 预热活跃模块（DHT 拉取补全）
- [x] 模块预热加载单元测试

### 18. P3.1 治理模块补全
- [x] shadow/apply 支持 DHT 拉取缺失工件
- [x] 治理拉取补全单元测试

### 19. P3.2 执行节点启动引导
- [x] bootstrap_world_from_dht 启动引导
- [x] 启动引导单元测试

### 20. P3.3 Head 跟随与同步
- [x] head 选择策略（height/timestamp/block_hash）
- [x] HeadFollower 同步 helper（忽略重复/陈旧）
- [x] HeadFollower 单元测试

### 21. P3.4 Observer head 同步
- [x] ObserverClient 同步 helper（sync_heads/sync_heads_with_dht）
- [x] Observer head 同步单元测试

### 22. P3.5 Observer 同步结果回传
- [x] ObserverClient 同步结果 helper（sync_heads_with_result/sync_heads_with_dht_result）
- [x] Observer 同步结果单元测试

### 23. P3.6 Observer 同步报告
- [x] ObserverClient 同步报告 helper（sync_heads_report/sync_heads_with_dht_report）
- [x] Observer 同步报告单元测试

### 24. P3.7 Observer 循环跟随
- [x] ObserverClient 循环跟随 helper（follow_heads/follow_heads_with_dht）
- [x] Observer 循环跟随单元测试


### 25. P3.8 Head 共识层
- [x] 新增 QuorumConsensus 与投票状态机
- [x] 共识提交门控 DHT head 发布
- [x] 共识单元测试与导出接口


### 26. P3.9 Head 共识持久化
- [x] 共识快照文件结构与版本定义
- [x] QuorumConsensus 快照保存/加载能力
- [x] 共识快照单元测试与分布式回归


### 27. P3.10 成员治理与租约联动
- [x] 共识成员变更接口（add/remove/replace）
- [x] 租约授权校验（holder + 时效）
- [x] lease holder 自动补齐 validator helper
- [x] 单元测试与分布式回归


### 28. P3.11 成员目录同步与变更广播
- [x] 新增 membership topic 与消息结构
- [x] MembershipSyncClient 发布/订阅/同步能力
- [x] 幂等同步处理与单元测试
- [x] 分布式回归测试

### 29. P3.12 成员目录 DHT 快照与恢复策略
- [x] 扩展 DistributedDht 成员目录快照 put/get 接口
- [x] MembershipSyncClient 发布联动 DHT 快照写入
- [x] MembershipSyncClient DHT 快照恢复接口
- [x] 单元测试与分布式回归

### 30. P3.13 成员目录快照签名与来源校验
- [x] 成员目录快照/广播增加可选 signature 字段
- [x] MembershipDirectorySigner 签名与验签能力
- [x] DHT 恢复入口来源校验（trusted requester）
- [x] 校验策略与单元测试、分布式回归

### 31. P3.14 成员目录快照密钥轮换与审计
- [x] 成员目录快照/广播增加可选 signature_key_id 字段
- [x] MembershipDirectorySignerKeyring 多密钥签名与验签
- [x] DHT 恢复 key_id 策略（require/allow list）
- [x] 恢复审计报告结构与单元测试、分布式回归

### 32. P3.15 成员目录审计持久化与吊销传播
- [x] 新增 MembershipAuditStore 抽象与 InMemoryMembershipAuditStore
- [x] 新增 restore_membership_from_dht_verified_with_audit_store 持久化入口
- [x] 新增 membership.revoke topic 与发布/订阅/同步能力
- [x] MembershipDirectorySignerKeyring 增加 revoked key 管理与验签拦截
- [x] 恢复策略增加 revoked_signature_key_ids，拒绝吊销 key_id
- [x] 单元测试与分布式回归

### 33. P3.16 成员目录吊销来源鉴权与审计落盘归档
- [x] 吊销广播结构扩展 signature/signature_key_id（向后兼容）
- [x] signer/keyring 支持吊销消息签名与验签
- [x] 新增吊销同步策略 `MembershipRevocationSyncPolicy`
- [x] 新增吊销同步报告 `MembershipRevocationSyncReport`
- [x] 新增 `sync_key_revocations_with_policy` 来源校验入口
- [x] 新增 `FileMembershipAuditStore` JSONL 落盘实现
- [x] 单元测试与分布式回归

### 34. P3.17 成员目录吊销授权治理与跨节点对账
- [x] 扩展 `MembershipRevocationSyncPolicy.authorized_requesters`
- [x] 吊销同步增加 authorized requester 校验
- [x] 新增 `membership.reconcile` topic 与 checkpoint 消息
- [x] 新增 `MembershipRevocationReconcilePolicy/Report`
- [x] 新增 `publish_revocation_checkpoint/drain_revocation_checkpoints/reconcile_revocations_with_policy`
- [x] 对账支持可选自动收敛（auto revoke missing keys）
- [x] 单元测试与分布式回归

### 35. P3.18 成员目录吊销异常告警与对账调度自动化
- [x] 新增 `MembershipRevocationAlertPolicy/Severity/AnomalyAlert`
- [x] 新增 `evaluate_revocation_reconcile_alerts`
- [x] 新增 `MembershipRevocationReconcileSchedulePolicy/State`
- [x] 新增 `MembershipRevocationScheduledRunReport`
- [x] 新增 `run_revocation_reconcile_schedule` 调度入口
- [x] 调度策略 interval 参数校验（必须为正）
- [x] 单元测试与分布式回归

### 36. P3.19 成员目录吊销告警上报与调度状态持久化
- [x] 新增 `MembershipRevocationAlertSink` 与内存/文件实现
- [x] 新增 `MembershipRevocationScheduleStateStore` 与内存/文件实现
- [x] 新增 `emit_revocation_reconcile_alerts`
- [x] 新增 `run_revocation_reconcile_schedule_with_store_and_alerts`
- [x] 文件落盘按 world/node 维度分片
- [x] 单元测试与分布式回归

### 37. P3.20 成员目录吊销告警抑制去重与调度多节点协同
- [x] 新增 `MembershipRevocationAlertDedupPolicy/State`
- [x] 新增 `deduplicate_revocation_alerts` 去重入口
- [x] 新增 `MembershipRevocationScheduleCoordinator` 与内存实现
- [x] 新增 `MembershipRevocationCoordinatedRunReport`
- [x] 新增 `run_revocation_reconcile_coordinated`
- [x] 协同锁支持 lease 过期与显式 release
- [x] 单元测试与分布式回归

### 38. P3.21 成员目录吊销协同状态外部存储与告警恢复机制
- [x] 新增 `MembershipRevocationCoordinatorStateStore` 与内存/文件实现
- [x] 新增 `StoreBackedMembershipRevocationScheduleCoordinator`
- [x] 新增 `MembershipRevocationAlertRecoveryStore` 与内存/文件实现
- [x] 新增 `emit_revocation_reconcile_alerts_with_recovery`
- [x] 新增 `MembershipRevocationCoordinatedRecoveryRunReport`
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery`
- [x] 单元测试与分布式回归

### 39. P3.22 成员目录吊销恢复队列容量治理与告警 ACK 重试
- [x] 新增 `MembershipRevocationPendingAlert` 结构与持久化元数据
- [x] 新增 `MembershipRevocationAlertAckRetryPolicy`
- [x] 新增 `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry`
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry`
- [x] recovery store 兼容旧格式 pending 队列读取
- [x] 恢复报告增加 deferred/capacity/retry_limit 统计
- [x] 单元测试与分布式回归

### 40. P3.23 成员目录吊销告警恢复死信归档与投递指标
- [x] 新增 `MembershipRevocationAlertDeadLetterStore` 与内存/文件实现
- [x] 新增 `MembershipRevocationAlertDeadLetterRecord/Reason`
- [x] 新增 `MembershipRevocationAlertDeliveryMetrics`
- [x] 新增 `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter`
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter`
- [x] 恢复与协同报告增加投递指标字段
- [x] 单元测试与分布式回归

### 41. P3.24 成员目录吊销死信回放调度与指标导出
- [x] 扩展 `MembershipRevocationAlertDeadLetterStore`（list/replace/metrics）
- [x] 新增 `replay_revocation_dead_letters`
- [x] 新增 `run_revocation_dead_letter_replay_schedule`
- [x] 新增 `export_revocation_alert_delivery_metrics`
- [x] 新增 `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter_and_metrics_export`
- [x] 内存/文件 dead-letter store 支持 delivery metrics 导出查询
- [x] 单元测试与分布式回归

### 42. P3.25 成员目录吊销死信优先级回放与跨节点回放协同
- [x] `replay_revocation_dead_letters` 按 reason/attempt/dropped_at 优先级回放
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated`
- [x] 回放协同 lease key 按 `world_id + target_node_id` 维度隔离
- [x] 新增优先级回放与跨节点协同单元测试
- [x] 单元测试与分布式回归

### 43. P3.26 成员目录吊销死信回放状态持久化与公平调度
- [x] 新增 `MembershipRevocationDeadLetterReplayStateStore` 与内存/文件实现
- [x] 新增 `MembershipRevocationDeadLetterReplayScheduleState`
- [x] 新增 `MembershipRevocationDeadLetterReplayPolicy`（公平调度参数）
- [x] 新增 `replay_revocation_dead_letters_with_policy`
- [x] 新增 `run_revocation_dead_letter_replay_schedule_with_state_store`
- [x] 新增 `run_revocation_dead_letter_replay_schedule_coordinated_with_state_store`
- [x] 新增状态持久化与公平调度单元测试
- [x] 单元测试与分布式回归

## 依赖
- `doc/world-runtime.md`
- `doc/world-runtime/runtime-integration.md`
- `doc/world-runtime/module-storage.md`
- libp2p 协议栈与实现

## 状态
- 当前阶段：P3.26 完成（成员目录吊销死信回放状态持久化与公平调度）
- 下一步：P3.27（成员目录吊销死信回放状态观测聚合与策略自适应）
- 最近更新：成员目录吊销死信回放状态持久化与公平调度（2026-02-11）
