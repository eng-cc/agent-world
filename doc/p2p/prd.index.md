# p2p PRD 文件级索引

审计轮次: 5

更新时间：2026-03-07

## 入口
- 模块 PRD：`doc/p2p/prd.md`
- 模块项目管理：`doc/p2p/prd.project.md`

## 覆盖规则（ROUND-005 统一）
- 纳入规则：纳入 `doc/p2p/**` 下所有 `*.prd.md` 与同名 `*.prd.project.md`。
- 排除规则：不纳入 `doc/devlog/**` 与非 PRD 配对文档（如 `*.release.md` 补充材料）。
- 历史入口：根目录历史入口文件（`p2p.prd.md` / `p2p.prd.project.md`）仅保留兼容跳转语义，不作为主索引分母。
- 兼容跳转：历史路径命中时统一跳转到本目录 `prd.md` / `prd.project.md` 主入口。

| 专题 PRD | 专题项目文档 |
| --- | --- |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.prd.project.md` |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.prd.project.md` |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.prd.project.md` |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.prd.project.md` |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.prd.project.md` |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.prd.project.md` |
| `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase8.prd.md` | `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase8.prd.project.md` |
| `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.prd.md` | `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.prd.project.md` |
| `doc/p2p/blockchain/production-grade-blockchain-p2pfs-phaseb-consensus-execution.prd.md` | `doc/p2p/blockchain/production-grade-blockchain-p2pfs-phaseb-consensus-execution.prd.project.md` |
| `doc/p2p/blockchain/production-grade-blockchain-p2pfs-phasec-distfs-proof-network.prd.md` | `doc/p2p/blockchain/production-grade-blockchain-p2pfs-phasec-distfs-proof-network.prd.project.md` |
| `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md` | `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.project.md` |
| `doc/p2p/consensus/builtin-wasm-identity-consensus.prd.md` | `doc/p2p/consensus/builtin-wasm-identity-consensus.prd.project.md` |
| `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.prd.md` | `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.prd.project.md` |
| `doc/p2p/distfs/distfs-builtin-wasm-api-closure.prd.md` | `doc/p2p/distfs/distfs-builtin-wasm-api-closure.prd.project.md` |
| `doc/p2p/distfs/distfs-builtin-wasm-storage.prd.md` | `doc/p2p/distfs/distfs-builtin-wasm-storage.prd.project.md` |
| `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.prd.md` | `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.prd.project.md` |
| `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.prd.md` | `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.prd.project.md` |
| `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.prd.md` | `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.prd.project.md` |
| `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.prd.md` | `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.prd.project.md` |
| `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.prd.md` | `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.prd.project.md` |
| `doc/p2p/distfs/distfs-path-index-observer-bootstrap.prd.md` | `doc/p2p/distfs/distfs-path-index-observer-bootstrap.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase1.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase1.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase2.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase2.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase3.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase3.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase4.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase4.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase5.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase5.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase6.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase6.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase7.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase7.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase8.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase8.prd.project.md` |
| `doc/p2p/distfs/distfs-production-hardening-phase9.prd.md` | `doc/p2p/distfs/distfs-production-hardening-phase9.prd.project.md` |
| `doc/p2p/distfs/distfs-runtime-path-index.prd.md` | `doc/p2p/distfs/distfs-runtime-path-index.prd.project.md` |
| `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.prd.md` | `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.prd.project.md` |
| `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.prd.md` | `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.prd.project.md` |
| `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.prd.md` | `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.prd.project.md` |
| `doc/p2p/distfs/distfs-standard-file-io.prd.md` | `doc/p2p/distfs/distfs-standard-file-io.prd.project.md` |
| `doc/p2p/distributed/distributed-hard-split-phase7.prd.md` | `doc/p2p/distributed/distributed-hard-split-phase7.prd.project.md` |
| `doc/p2p/distributed/distributed-pos-consensus.prd.md` | `doc/p2p/distributed/distributed-pos-consensus.prd.project.md` |
| `doc/p2p/distributed/distributed-runtime.prd.md` | `doc/p2p/distributed/distributed-runtime.prd.project.md` |
| `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.prd.md` | `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.prd.project.md` |
| `doc/p2p/network/net-runtime-bridge-closure.prd.md` | `doc/p2p/network/net-runtime-bridge-closure.prd.project.md` |
| `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md` | `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.project.md` |
| `doc/p2p/network/readme-p1-network-production-hardening.prd.md` | `doc/p2p/network/readme-p1-network-production-hardening.prd.project.md` |
| `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.md` | `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.project.md` |
| `doc/p2p/node/node-consensus-signer-binding-replication-hardening.prd.md` | `doc/p2p/node/node-consensus-signer-binding-replication-hardening.prd.project.md` |
| `doc/p2p/node/node-contribution-points-multi-node-closure-test.prd.md` | `doc/p2p/node/node-contribution-points-multi-node-closure-test.prd.project.md` |
| `doc/p2p/node/node-contribution-points-runtime-closure.prd.md` | `doc/p2p/node/node-contribution-points-runtime-closure.prd.project.md` |
| `doc/p2p/node/node-contribution-points.prd.md` | `doc/p2p/node/node-contribution-points.prd.project.md` |
| `doc/p2p/node/node-distfs-replication-network-closure.prd.md` | `doc/p2p/node/node-distfs-replication-network-closure.prd.project.md` |
| `doc/p2p/node/node-execution-reward-consensus-bridge.prd.md` | `doc/p2p/node/node-execution-reward-consensus-bridge.prd.project.md` |
| `doc/p2p/node/node-execution-verification-reward-leader-failover-hardening.prd.md` | `doc/p2p/node/node-execution-verification-reward-leader-failover-hardening.prd.project.md` |
| `doc/p2p/node/node-keypair-config-bootstrap.prd.md` | `doc/p2p/node/node-keypair-config-bootstrap.prd.project.md` |
| `doc/p2p/node/node-redeemable-power-asset-audit-hardening.prd.md` | `doc/p2p/node/node-redeemable-power-asset-audit-hardening.prd.project.md` |
| `doc/p2p/node/node-redeemable-power-asset-signature-governance-phase3.prd.md` | `doc/p2p/node/node-redeemable-power-asset-signature-governance-phase3.prd.project.md` |
| `doc/p2p/node/node-redeemable-power-asset.prd.md` | `doc/p2p/node/node-redeemable-power-asset.prd.project.md` |
| `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md` | `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.project.md` |
| `doc/p2p/node/node-replication-libp2p-migration.prd.md` | `doc/p2p/node/node-replication-libp2p-migration.prd.project.md` |
| `doc/p2p/node/node-reward-runtime-production-hardening-phase1.prd.md` | `doc/p2p/node/node-reward-runtime-production-hardening-phase1.prd.project.md` |
| `doc/p2p/node/node-reward-settlement-native-transaction.prd.md` | `doc/p2p/node/node-reward-settlement-native-transaction.prd.project.md` |
| `doc/p2p/node/node-storage-system-reward-pool.prd.md` | `doc/p2p/node/node-storage-system-reward-pool.prd.project.md` |
| `doc/p2p/node/node-uptime-base-reward.prd.md` | `doc/p2p/node/node-uptime-base-reward.prd.project.md` |
| `doc/p2p/node/node-wasm32-libp2p-compile-guard.prd.md` | `doc/p2p/node/node-wasm32-libp2p-compile-guard.prd.project.md` |
| `doc/p2p/node/node-net-stack-unification-readme.prd.md` | `doc/p2p/node/node-net-stack-unification-readme.prd.project.md` |
| `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.prd.md` | `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.prd.project.md` |
| `doc/p2p/observer/observer-sync-mode-observability.prd.md` | `doc/p2p/observer/observer-sync-mode-observability.prd.project.md` |
| `doc/p2p/observer/observer-sync-mode-runtime-metrics.prd.md` | `doc/p2p/observer/observer-sync-mode-runtime-metrics.prd.project.md` |
| `doc/p2p/observer/observer-sync-source-dht-mode.prd.md` | `doc/p2p/observer/observer-sync-source-dht-mode.prd.project.md` |
| `doc/p2p/observer/observer-sync-source-mode.prd.md` | `doc/p2p/observer/observer-sync-source-mode.prd.project.md` |
| `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md` | `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.project.md` |
| `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md` | `doc/p2p/token/mainchain-token-allocation-mechanism.prd.project.md` |
| `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.prd.md` | `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.prd.project.md` |
| `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.prd.md` | `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.prd.project.md` |
| `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.prd.md` | `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.prd.project.md` |

## 发布说明文档（release，补充材料）
| 发布说明 | 对应专题 |
| --- | --- |
| `doc/p2p/node/node-redeemable-power-asset.release.md` | `doc/p2p/node/node-redeemable-power-asset.prd.md` |
| `doc/p2p/node/node-redeemable-power-asset-audit-hardening.release.md` | `doc/p2p/node/node-redeemable-power-asset-audit-hardening.prd.md` |
| `doc/p2p/token/mainchain-token-allocation-mechanism.release.md` | `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md` |

## 说明
- 本索引用于保证模块专题文档在根入口文档树中可达。
- 文档配对规则：`*.prd.md` 与同名 `*.prd.project.md`。
- `*.release.md` 为发布补充材料，不参与 PRD 任务配对规则。
- ROUND-002 主从口径（observer）：`observer-sync-source-mode` 为主文档；`observer-sync-source-dht-mode` 为 DHT 增量子文档。
- ROUND-002 主从口径（observer）：`observer-sync-mode-runtime-metrics` 为主文档；`metrics-runtime-bridge` 与 `observability` 为增量子文档。
- ROUND-002 主从口径（node）：`node-contribution-points` 为主文档；`runtime-closure` 与 `multi-node-closure-test` 为增量子文档。
- ROUND-002 主从口径（node）：`node-redeemable-power-asset` 为主文档；`audit-hardening` 与 `signature-governance-phase3` 为增量子文档。
- ROUND-002 主从口径（distfs）：`distfs-self-healing-control-plane-2026-02-23` 为主文档；`polling-loop` 与 `runtime-polling-wiring` 为增量子文档。
- ROUND-002 主从口径（distfs）：`distfs-production-hardening-phase1` 为主文档；`phase2~phase9` 为增量子文档。
