> [!WARNING]
> 该文档已归档，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# P2P 文档迁移映射清单

## 迁移信息
- 执行日期：2026-02-16
- 首轮迁移（world-runtime -> p2p）总数：132
- 二轮核查后活跃文档：53
- 二轮核查后归档文档：79

## 过期判定标准
- 文档明确为历史分卷归档（例如 `*.archive-*`）。
- 文档被后续阶段文档明确替代。
- 文档依赖的核心代码路径在当前仓库已删除（本次重点：`crates/agent_world/src/runtime/distributed*` 与 `distributed_membership_sync*`）。

## 二轮核查方法（文档-代码比对）
- 扫描 `doc/p2p`（不含 `archive`）内反引号路径引用。
- 将引用路径与当前工作区文件存在性进行比对。
- 对指向已删除运行时路径且已被 split crate 替代的文档，判定为过期并归档。

## 映射清单

### 最终归档映射
- `doc/world-runtime/agent-world-proto.md` -> `doc/p2p/archive/agent-world-proto.md`
- `doc/world-runtime/agent-world-proto.project.md` -> `doc/p2p/archive/agent-world-proto.project.md`
- `doc/world-runtime/blockchain-p2pfs-foundation-closure.md` -> `doc/p2p/archive/blockchain-p2pfs-foundation-closure.md`
- `doc/world-runtime/blockchain-p2pfs-foundation-closure.project.md` -> `doc/p2p/archive/blockchain-p2pfs-foundation-closure.project.md`
- `doc/world-runtime/blockchain-p2pfs-hardening-phase1.md` -> `doc/p2p/archive/blockchain-p2pfs-hardening-phase1.md`
- `doc/world-runtime/blockchain-p2pfs-hardening-phase1.project.md` -> `doc/p2p/archive/blockchain-p2pfs-hardening-phase1.project.md`
- `doc/world-runtime/distributed-consensus-membership-audit-revocation.md` -> `doc/p2p/archive/distributed-consensus-membership-audit-revocation.md`
- `doc/world-runtime/distributed-consensus-membership-audit-revocation.project.md` -> `doc/p2p/archive/distributed-consensus-membership-audit-revocation.project.md`
- `doc/world-runtime/distributed-consensus-membership-auth.md` -> `doc/p2p/archive/distributed-consensus-membership-auth.md`
- `doc/world-runtime/distributed-consensus-membership-auth.project.md` -> `doc/p2p/archive/distributed-consensus-membership-auth.project.md`
- `doc/world-runtime/distributed-consensus-membership-dht.md` -> `doc/p2p/archive/distributed-consensus-membership-dht.md`
- `doc/world-runtime/distributed-consensus-membership-dht.project.md` -> `doc/p2p/archive/distributed-consensus-membership-dht.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-alert-dedup-coordination.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-alert-dedup-coordination.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-alert-dedup-coordination.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-alert-dedup-coordination.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-alert-delivery-state-store.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-alert-delivery-state-store.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-alert-delivery-state-store.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-alert-delivery-state-store.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-alerting-scheduler.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-alerting-scheduler.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-alerting-scheduler.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-alerting-scheduler.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-auth-archive.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-auth-archive.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-auth-archive.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-auth-archive.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-coordinator-state-alert-recovery.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-metrics.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-metrics.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-metrics.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-metrics.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-priority-coordination.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-priority-coordination.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-priority-coordination.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-priority-coordination.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-metrics-export.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-observability-adaptive-policy.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-adoption-audit-rollback-alert.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-adoption-audit-rollback-alert.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-adoption-audit-rollback-alert.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-adoption-audit-rollback-alert.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-audit-state-governance.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-cooldown-drift-guard.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-aggregate-query-drill-alert-event-bus.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-archive-drill.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-retention-drill-schedule.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-audit-tiered-offload-drill-alert-linkage.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-aggregate-pull-pagination.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-cursor.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-cursor.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-cursor.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-cursor.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store-monotonic-guard.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store-monotonic-guard.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store-monotonic-guard.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store-monotonic-guard.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor-store.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-composite-sequence-cursor.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-advance.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-governance-recovery-drill-alert-event-bus-incremental-watermark-summary.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-policy-persistence-rollback.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-dead-letter-replay-state-fair-scheduling.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-governance-reconcile.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-governance-reconcile.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-governance-reconcile.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-governance-reconcile.project.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-recovery-queue-ack-retry.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-recovery-queue-ack-retry.md`
- `doc/world-runtime/distributed-consensus-membership-revocation-recovery-queue-ack-retry.project.md` -> `doc/p2p/archive/distributed-consensus-membership-revocation-recovery-queue-ack-retry.project.md`
- `doc/world-runtime/distributed-consensus-membership-rotation-audit.md` -> `doc/p2p/archive/distributed-consensus-membership-rotation-audit.md`
- `doc/world-runtime/distributed-consensus-membership-rotation-audit.project.md` -> `doc/p2p/archive/distributed-consensus-membership-rotation-audit.project.md`
- `doc/world-runtime/distributed-consensus-membership.md` -> `doc/p2p/archive/distributed-consensus-membership.md`
- `doc/world-runtime/distributed-consensus-membership.project.md` -> `doc/p2p/archive/distributed-consensus-membership.project.md`
- `doc/world-runtime/distributed-consensus-persistence.md` -> `doc/p2p/archive/distributed-consensus-persistence.md`
- `doc/world-runtime/distributed-consensus-persistence.project.md` -> `doc/p2p/archive/distributed-consensus-persistence.project.md`
- `doc/world-runtime/distributed-consensus-sync.md` -> `doc/p2p/archive/distributed-consensus-sync.md`
- `doc/world-runtime/distributed-consensus-sync.project.md` -> `doc/p2p/archive/distributed-consensus-sync.project.md`
- `doc/world-runtime/distributed-consensus.md` -> `doc/p2p/archive/distributed-consensus.md`
- `doc/world-runtime/distributed-consensus.project.md` -> `doc/p2p/archive/distributed-consensus.project.md`
- `doc/world-runtime/distributed-crate-split-net-consensus.md` -> `doc/p2p/archive/distributed-crate-split-net-consensus.md`
- `doc/world-runtime/distributed-crate-split-net-consensus.project.md` -> `doc/p2p/archive/distributed-crate-split-net-consensus.project.md`
- `doc/world-runtime/distributed-runtime.project.archive-0-41.md` -> `doc/p2p/archive/distributed-runtime.project.archive-0-41.md`

### 最终活跃映射
- `doc/world-runtime/blockchain-p2pfs-hardening-phase2.md` -> `doc/p2p/blockchain-p2pfs-hardening-phase2.md`
- `doc/world-runtime/blockchain-p2pfs-hardening-phase2.project.md` -> `doc/p2p/blockchain-p2pfs-hardening-phase2.project.md`
- `doc/world-runtime/builtin-wasm-distfs-api-closure.md` -> `doc/p2p/builtin-wasm-distfs-api-closure.md`
- `doc/world-runtime/builtin-wasm-distfs-api-closure.project.md` -> `doc/p2p/builtin-wasm-distfs-api-closure.project.md`
- `doc/world-runtime/builtin-wasm-distfs-storage.md` -> `doc/p2p/builtin-wasm-distfs-storage.md`
- `doc/world-runtime/builtin-wasm-distfs-storage.project.md` -> `doc/p2p/builtin-wasm-distfs-storage.project.md`
- `doc/world-runtime/builtin-wasm-fetch-fallback-compile.md` -> `doc/p2p/builtin-wasm-fetch-fallback-compile.md`
- `doc/world-runtime/builtin-wasm-fetch-fallback-compile.project.md` -> `doc/p2p/builtin-wasm-fetch-fallback-compile.project.md`
- `doc/world-runtime/distfs-path-index-observer-bootstrap.md` -> `doc/p2p/distfs-path-index-observer-bootstrap.md`
- `doc/world-runtime/distfs-path-index-observer-bootstrap.project.md` -> `doc/p2p/distfs-path-index-observer-bootstrap.project.md`
- `doc/world-runtime/distfs-runtime-path-index.md` -> `doc/p2p/distfs-runtime-path-index.md`
- `doc/world-runtime/distfs-runtime-path-index.project.md` -> `doc/p2p/distfs-runtime-path-index.project.md`
- `doc/world-runtime/distfs-standard-file-io.md` -> `doc/p2p/distfs-standard-file-io.md`
- `doc/world-runtime/distfs-standard-file-io.project.md` -> `doc/p2p/distfs-standard-file-io.project.md`
- `doc/world-runtime/distributed-hard-split-phase7.md` -> `doc/p2p/distributed-hard-split-phase7.md`
- `doc/world-runtime/distributed-hard-split-phase7.project.md` -> `doc/p2p/distributed-hard-split-phase7.project.md`
- `doc/world-runtime/distributed-node-mainloop.md` -> `doc/p2p/archive/distributed-node-mainloop.md`
- `doc/world-runtime/distributed-node-mainloop.project.md` -> `doc/p2p/archive/distributed-node-mainloop.project.md`
- `doc/world-runtime/distributed-node-pos-gossip.md` -> `doc/p2p/archive/distributed-node-pos-gossip.md`
- `doc/world-runtime/distributed-node-pos-gossip.project.md` -> `doc/p2p/archive/distributed-node-pos-gossip.project.md`
- `doc/world-runtime/distributed-node-pos-mainloop.md` -> `doc/p2p/archive/distributed-node-pos-mainloop.md`
- `doc/world-runtime/distributed-node-pos-mainloop.project.md` -> `doc/p2p/archive/distributed-node-pos-mainloop.project.md`
- `doc/world-runtime/distributed-pos-consensus.md` -> `doc/p2p/distributed-pos-consensus.md`
- `doc/world-runtime/distributed-pos-consensus.project.md` -> `doc/p2p/distributed-pos-consensus.project.md`
- `doc/world-runtime/distributed-runtime.md` -> `doc/p2p/distributed-runtime.md`
- `doc/world-runtime/distributed-runtime.project.md` -> `doc/p2p/distributed-runtime.project.md`
- `doc/world-runtime/net-runtime-bridge-closure.md` -> `doc/p2p/net-runtime-bridge-closure.md`
- `doc/world-runtime/net-runtime-bridge-closure.project.md` -> `doc/p2p/net-runtime-bridge-closure.project.md`
- `doc/world-runtime/node-contribution-points-multi-node-closure-test.md` -> `doc/p2p/node-contribution-points-multi-node-closure-test.md`
- `doc/world-runtime/node-contribution-points-multi-node-closure-test.project.md` -> `doc/p2p/node-contribution-points-multi-node-closure-test.project.md`
- `doc/world-runtime/node-contribution-points-runtime-closure.project.md` -> `doc/p2p/node-contribution-points-runtime-closure.project.md`
- `doc/world-runtime/node-contribution-points.md` -> `doc/p2p/node-contribution-points.md`
- `doc/world-runtime/node-contribution-points.project.md` -> `doc/p2p/node-contribution-points.project.md`
- `doc/world-runtime/node-distfs-replication-network-closure.md` -> `doc/p2p/node-distfs-replication-network-closure.md`
- `doc/world-runtime/node-distfs-replication-network-closure.project.md` -> `doc/p2p/node-distfs-replication-network-closure.project.md`
- `doc/world-runtime/node-keypair-config-bootstrap.md` -> `doc/p2p/node-keypair-config-bootstrap.md`
- `doc/world-runtime/node-keypair-config-bootstrap.project.md` -> `doc/p2p/node-keypair-config-bootstrap.project.md`
- `doc/world-runtime/node-replication-libp2p-migration.md` -> `doc/p2p/node-replication-libp2p-migration.md`
- `doc/world-runtime/node-replication-libp2p-migration.project.md` -> `doc/p2p/node-replication-libp2p-migration.project.md`
- `doc/world-runtime/node-wasm32-libp2p-compile-guard.md` -> `doc/p2p/node-wasm32-libp2p-compile-guard.md`
- `doc/world-runtime/node-wasm32-libp2p-compile-guard.project.md` -> `doc/p2p/node-wasm32-libp2p-compile-guard.project.md`
- `doc/world-runtime/observer-sync-mode-metrics-runtime-bridge.md` -> `doc/p2p/observer-sync-mode-metrics-runtime-bridge.md`
- `doc/world-runtime/observer-sync-mode-metrics-runtime-bridge.project.md` -> `doc/p2p/observer-sync-mode-metrics-runtime-bridge.project.md`
- `doc/world-runtime/observer-sync-mode-observability.md` -> `doc/p2p/observer-sync-mode-observability.md`
- `doc/world-runtime/observer-sync-mode-observability.project.md` -> `doc/p2p/observer-sync-mode-observability.project.md`
- `doc/world-runtime/observer-sync-mode-runtime-metrics.md` -> `doc/p2p/observer-sync-mode-runtime-metrics.md`
- `doc/world-runtime/observer-sync-mode-runtime-metrics.project.md` -> `doc/p2p/observer-sync-mode-runtime-metrics.project.md`
- `doc/world-runtime/observer-sync-source-dht-mode.md` -> `doc/p2p/observer-sync-source-dht-mode.md`
- `doc/world-runtime/observer-sync-source-dht-mode.project.md` -> `doc/p2p/observer-sync-source-dht-mode.project.md`
- `doc/world-runtime/observer-sync-source-mode.md` -> `doc/p2p/observer-sync-source-mode.md`
- `doc/world-runtime/observer-sync-source-mode.project.md` -> `doc/p2p/observer-sync-source-mode.project.md`
- `doc/world-runtime/wasm-artifact-identity-reproducibility.md` -> `doc/p2p/wasm-artifact-identity-reproducibility.md`
- `doc/world-runtime/wasm-artifact-identity-reproducibility.project.md` -> `doc/p2p/wasm-artifact-identity-reproducibility.project.md`
