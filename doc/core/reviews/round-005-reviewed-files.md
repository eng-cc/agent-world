# ROUND-005 已审读文档清单（S_round005）

审计轮次: 5

- 生成时间: 2026-03-06 17:58:03 CST（第二批整改后刷新）
- 统计口径: `doc/world-simulator/**/*.md` + `doc/p2p/**/*.md` + `doc/site/**/*.md` + `doc/playability_test_result/**/*.md`
- 生成规则: `rg -l "^审计轮次:\s*5$" doc/world-simulator doc/p2p doc/site doc/playability_test_result --glob '*.md' | sort`
- 当前目标范围文档数: 516
- 当前已审读文档数: 78

## 范围拆分
| 范围 | 文档数 |
| --- | --- |
| `doc/world-simulator/**` | 314 |
| `doc/p2p/**` | 147 |
| `doc/site/**` | 42 |
| `doc/playability_test_result/**` | 13 |
| 合计 | 516 |

## 已审文件列表
- `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.prd.md`
- `doc/p2p/distfs/distfs-heterogeneous-node-optimal-stability-2026-02-23.prd.project.md`
- `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.prd.md`
- `doc/p2p/distfs/distfs-no-single-full-node-assumption-2026-02-23.prd.project.md`
- `doc/p2p/distfs/distfs-production-hardening-phase6.prd.md`
- `doc/p2p/distfs/distfs-production-hardening-phase9.prd.md`
- `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.prd.md`
- `doc/p2p/distfs/distfs-self-healing-control-plane-2026-02-23.prd.project.md`
- `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.prd.md`
- `doc/p2p/distfs/distfs-self-healing-polling-loop-2026-02-23.prd.project.md`
- `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.prd.md`
- `doc/p2p/distfs/distfs-self-healing-runtime-polling-wiring-2026-02-23.prd.project.md`
- `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.prd.project.md`
- `doc/p2p/network/readme-p1-network-production-hardening.prd.project.md`
- `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.project.md`
- `doc/p2p/node/node-consensus-signer-binding-replication-hardening.prd.project.md`
- `doc/p2p/node/node-net-stack-unification-readme.prd.project.md`
- `doc/p2p/node/node-reward-settlement-native-transaction.prd.md`
- `doc/p2p/prd.index.md`
- `doc/world-simulator/kernel/intent-distributed-runtime-closure-2026-02-27.prd.project.md`
- `doc/world-simulator/kernel/location-electricity-pool-removal-and-radiation-plant.prd.project.md`
- `doc/world-simulator/kernel/resource-kind-compound-hardware-hard-migration.prd.project.md`
- `doc/world-simulator/kernel/social-fact-ledger-declarative-reputation.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-chain-runtime-decouple-2026-02-28.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-llm-settings-panel-2026-03-02.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-web-wasm-time-compat-2026-03-04.prd.project.md`
- `doc/world-simulator/llm/indirect-control-tick-lifecycle-long-term-memory.prd.project.md`
- `doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.prd.project.md`
- `doc/world-simulator/m4/m4-builtin-wasm-maintainability-2026-02-26.prd.project.md`
- `doc/world-simulator/m4/m4-industrial-benchmark-current-state-2026-02-27.prd.project.md`
- `doc/world-simulator/m4/m4-market-hardware-data-governance-closure-2026-02-26.prd.project.md`
- `doc/world-simulator/m4/m4-power-system.prd.project.md`
- `doc/world-simulator/m4/m4-resource-product-system-p0-shared-bottleneck-logistics-priority-2026-02-27.prd.project.md`
- `doc/world-simulator/m4/m4-resource-product-system-p1-maintenance-scarcity-pressure-2026-02-27.prd.project.md`
- `doc/world-simulator/m4/m4-resource-product-system-p2-stage-guidance-market-governance-linkage-2026-02-27.prd.project.md`
- `doc/world-simulator/m4/m4-resource-product-system-p3-layer-profile-chain-expansion-2026-02-27.prd.project.md`
- `doc/world-simulator/m4/m4-resource-product-system-playability-2026-02-27.prd.project.md`
- `doc/world-simulator/m4/m4-resource-product-system-playability-priority-hardening-2026-02-28.prd.project.md`
- `doc/world-simulator/prd.index.md`
- `doc/world-simulator/scenario/agent-frag-initial-spawn-position.prd.project.md`
- `doc/world-simulator/scenario/asteroid-fragment-renaming.prd.project.md`
- `doc/world-simulator/scenario/chunked-fragment-generation.prd.project.md`
- `doc/world-simulator/scenario/frag-resource-balance-onboarding.prd.project.md`
- `doc/world-simulator/scenario/fragment-spacing.prd.project.md`
- `doc/world-simulator/scenario/scenario-asteroid-fragment-overrides.prd.project.md`
- `doc/world-simulator/scenario/scenario-seed-locations.prd.project.md`
- `doc/world-simulator/scenario/world-initialization.prd.project.md`
- `doc/world-simulator/viewer/viewer-control-feedback-iteration-checklist-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-control-feedback-step-recovery-p0-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-control-plane-split-live-playback-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-first-session-goal-clarity-hardening-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-disable-seek-p2p-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-llm-event-driven-trigger-2026-02-26.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-logical-time-interface-phase11-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-llm-full-bridge-2026-03-05.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-step-control-progress-stability-2026-02-28.prd.project.md`
- `doc/world-simulator/viewer/viewer-live-tick-driven-doc-archive-2026-02-27.prd.project.md`
- `doc/world-simulator/viewer/viewer-minimal-system.prd.project.md`
- `doc/world-simulator/viewer/viewer-node-hard-decouple-2026-02-28.prd.project.md`
- `doc/world-simulator/viewer/viewer-step-completion-ack-2026-02-28.prd.project.md`
- `doc/world-simulator/viewer/viewer-visual-upgrade.prd.project.md`
- `doc/world-simulator/viewer/viewer-web-build-pruning-2026-03-02.prd.project.md`
- `doc/world-simulator/viewer/viewer-web-build-pruning-phase2-2026-03-02.prd.project.md`
- `doc/world-simulator/viewer/viewer-web-semantic-test-api.prd.md`
