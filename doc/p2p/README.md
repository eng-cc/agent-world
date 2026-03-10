# p2p 文档索引

审计轮次: 6

## 入口
- PRD: `doc/p2p/prd.md`
- 设计总览: `doc/p2p/design.md`
- 标准执行入口: `doc/p2p/project.md`
- 文件级索引: `doc/p2p/prd.index.md`

## 主题目录
- `distfs/`: DistFS 设计与稳定性加固。
- `node/`: 节点能力、奖励、身份与复制链路。
- `observer/`: 观察者同步模式与可观测性。
- `blockchain/`: 区块链与 P2PFS 硬化阶段。
- `token/`: 主链 token 分配与治理分发。
- `viewer-live/`: viewer live 发行与开关策略。
- `consensus/`: 共识相关专题。
- `distributed/`: 分布式运行时专题。
- `network/`: 网络桥接专题。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。

## 维护约定
- 新文档按主题目录落位，不再默认平铺在模块根目录。
- 模块行为变更需同步更新 `prd.md` 与 `project.md`。

- `doc/p2p/blockchain/production-grade-blockchain-p2pfs-phaseb-consensus-execution.design.md`
- `doc/p2p/blockchain/production-grade-blockchain-p2pfs-phasec-distfs-proof-network.design.md`
- `doc/p2p/consensus/builtin-wasm-identity-consensus.design.md`
- `doc/p2p/distfs/distfs-builtin-wasm-api-closure.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase3.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase4.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase5.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase6.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.design.md`
- `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase8.design.md`
- `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.design.md`
- `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.design.md`
- `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.design.md`
- `doc/p2p/distfs/distfs-builtin-wasm-storage.design.md`
- `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.design.md`
- `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.design.md`
- `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.design.md`
- `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.design.md`
- `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase1.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase2.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase3.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase4.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase5.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase6.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase7.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase8.design.md`
- `doc/p2p/distfs/distfs-production-hardening-phase9.design.md`
- `doc/p2p/distfs/distfs-runtime-path-index.design.md`
- `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.design.md`
- `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.design.md`
- `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.design.md`
- `doc/p2p/distfs/distfs-standard-file-io.design.md`
- `doc/p2p/distributed/distributed-hard-split-phase7.design.md`
- `doc/p2p/distributed/distributed-pos-consensus.design.md`
- `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.design.md`
- `doc/p2p/distributed/distributed-runtime.design.md`
- `doc/p2p/network/net-runtime-bridge-closure.design.md`
- `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.design.md`
- `doc/p2p/network/readme-p1-network-production-hardening.design.md`
- `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.design.md`
- `doc/p2p/node/node-consensus-signer-binding-replication-hardening.design.md`
- `doc/p2p/node/node-contribution-points-multi-node-closure-test.design.md`
- `doc/p2p/node/node-contribution-points-runtime-closure.design.md`
- `doc/p2p/node/node-contribution-points.design.md`
- `doc/p2p/node/node-distfs-replication-network-closure.design.md`
- `doc/p2p/node/node-execution-reward-consensus-bridge.design.md`
- `doc/p2p/node/node-execution-verification-reward-leader-failover-hardening.design.md`
- `doc/p2p/node/node-keypair-config-bootstrap.design.md`
- `doc/p2p/node/node-net-stack-unification-readme.design.md`
- `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.design.md`
- `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.design.md`
- `doc/p2p/node/node-pos-time-anchor-control-plane-alignment-2026-03-07.design.md`
- `doc/p2p/node/node-redeemable-power-asset-audit-hardening.design.md`
- `doc/p2p/node/node-redeemable-power-asset-signature-governance-phase3.design.md`
- `doc/p2p/node/node-redeemable-power-asset.design.md`
- `doc/p2p/node/node-replication-libp2p-migration.design.md`
- `doc/p2p/node/node-reward-runtime-production-hardening-phase1.design.md`
- `doc/p2p/node/node-reward-settlement-native-transaction.design.md`
- `doc/p2p/node/node-storage-system-reward-pool.design.md`
- `doc/p2p/node/node-uptime-base-reward.design.md`
- `doc/p2p/node/node-wasm32-libp2p-compile-guard.design.md`
- `doc/p2p/distfs/distfs-path-index-observer-bootstrap.design.md`
- `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.design.md`
- `doc/p2p/observer/observer-sync-mode-observability.design.md`
- `doc/p2p/observer/observer-sync-mode-runtime-metrics.design.md`
- `doc/p2p/observer/observer-sync-source-dht-mode.design.md`
- `doc/p2p/observer/observer-sync-source-mode.design.md`
- `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.design.md`
- `doc/p2p/token/mainchain-token-allocation-mechanism.design.md`
- `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.design.md`
- `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.design.md`
- `doc/p2p/viewer-live/world-viewer-live-release-locked-launch-2026-02-23.design.md`
