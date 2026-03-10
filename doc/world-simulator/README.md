# world-simulator 文档索引

审计轮次: 6

## 入口
- PRD: `doc/world-simulator/prd.md`
- 设计总览: `doc/world-simulator/design.md`
- 标准执行入口: `doc/world-simulator/project.md`
- 文件级索引: `doc/world-simulator/prd.index.md`

## 主题目录
- `viewer/`: Viewer 与 Web/交互/可视化相关设计。
- `llm/`: LLM 行为、Prompt、评估与稳定性相关设计。
- `launcher/`: 启动器与链路编排相关设计。
- `scenario/`: 场景定义、初始化与配置相关设计。
- `kernel/`: 内核规则桥接与 WASM 规则执行相关设计。
- `m4/`: M4 专题文档。

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。
- 其余专题文档已迁移到对应主题目录（`viewer/llm/launcher/scenario/kernel/m4`）。

## 专项手册
- Viewer 使用手册：`doc/world-simulator/viewer/viewer-manual.md`

## 根目录 legacy
- `doc/world-simulator.prd.md`
- `doc/world-simulator.project.md`

## 维护约定
- 新文档按主题目录落位，不再默认平铺在模块根目录。
- 模块行为变更需同步更新 `prd.md` 与 `project.md`。

- `doc/world-simulator/kernel/kernel-rule-hook-foundation.design.md`
- `doc/world-simulator/kernel/kernel-rule-wasm-executor-foundation.design.md`
- `doc/world-simulator/kernel/kernel-rule-wasm-module-governance.design.md`
- `doc/world-simulator/kernel/kernel-rule-wasm-sandbox-bridge.design.md`
- `doc/world-simulator/kernel/intent-distributed-runtime-closure-2026-02-27.design.md`
- `doc/world-simulator/kernel/location-electricity-pool-removal-and-radiation-plant.design.md`
- `doc/world-simulator/viewer/viewer-web-closure-testing-policy.design.md`
- `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-panel-2026-03-07.design.md`

- `doc/world-simulator/kernel/kernel-rule-wasm-readiness.design.md`
- `doc/world-simulator/kernel/power-storage-complete-removal-2026-03-06.design.md`
- `doc/world-simulator/kernel/runtime-required-failing-tests-offline-2026-03-09.design.md`
- `doc/world-simulator/kernel/resource-kind-compound-hardware-hard-migration.design.md`

- `doc/world-simulator/kernel/rust-wasm-build-suite.design.md`
- `doc/world-simulator/kernel/social-fact-ledger-declarative-reputation.design.md`
- `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-public-chain-p0-2026-03-07.design.md`
- `doc/world-simulator/launcher/game-client-launcher-availability-ux-hardening-2026-03-08.design.md`

- `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.design.md`
- `doc/world-simulator/launcher/game-client-launcher-web-required-config-gating-2026-03-04.design.md`
- `doc/world-simulator/launcher/game-client-launcher-web-settings-feedback-parity-2026-03-06.design.md`
- `doc/world-simulator/launcher/game-client-launcher-web-transfer-closure-2026-03-06.design.md`

- `doc/world-simulator/launcher/game-client-launcher-feedback-entry-2026-03-02.design.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.design.md`
- `doc/world-simulator/launcher/game-client-launcher-feedback-distributed-submit-2026-03-02.design.md`
- `doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.design.md`

- `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.design.md`
- `doc/world-simulator/launcher/game-client-launcher-native-legacy-cleanup-2026-03-06.design.md`
- `doc/world-simulator/launcher/game-client-launcher-transfer-product-grade-parity-2026-03-06.design.md`
- `doc/world-simulator/launcher/game-client-launcher-self-guided-experience-2026-03-08.design.md`

- `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-ui-ux-optimization-2026-03-08.design.md`
- `doc/world-simulator/launcher/game-client-launcher-chain-runtime-execution-world-dir-output-hardening-2026-03-09.design.md`
- `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.design.md`
- `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.design.md`

- `doc/world-simulator/launcher/game-client-launcher-web-console-gui-agent-interface-2026-03-08.design.md`

- `doc/world-simulator/viewer/viewer-2d-3d-clarity-improvement.design.md`
- `doc/world-simulator/viewer/viewer-2d-visual-polish.design.md`
- `doc/world-simulator/viewer/viewer-3d-commercial-polish.design.md`
- `doc/world-simulator/viewer/viewer-3d-polish-performance.design.md`

- `doc/world-simulator/viewer/viewer-2d-3d-clarity-improvement.design.md`
- `doc/world-simulator/viewer/viewer-2d-visual-polish.design.md`
- `doc/world-simulator/viewer/viewer-3d-commercial-polish.design.md`
- `doc/world-simulator/viewer/viewer-3d-polish-performance.design.md`

- `doc/world-simulator/viewer/viewer-agent-module-rendering.design.md`
- `doc/world-simulator/viewer/viewer-agent-quick-locate.design.md`
- `doc/world-simulator/viewer/viewer-agent-size-inspection.design.md`
- `doc/world-simulator/viewer/viewer-asset-pipeline-ui-system-hardening-2026-03-05.design.md`

- `doc/world-simulator/viewer/viewer-auto-focus-capture.design.md`
- `doc/world-simulator/viewer/viewer-auto-select-capture.design.md`
- `doc/world-simulator/viewer/viewer-bevy-web-runtime.design.md`
- `doc/world-simulator/viewer/viewer-chat-agent-prompt-default-values-prefill.design.md`

- `doc/world-simulator/viewer/viewer-chat-dedicated-right-panel.design.md`
- `doc/world-simulator/viewer/viewer-chat-enter-send.design.md`
- `doc/world-simulator/viewer/viewer-chat-ime-cn-input.design.md`
- `doc/world-simulator/viewer/viewer-chat-ime-egui-bridge.design.md`

- `doc/world-simulator/viewer/viewer-chat-prompt-presets.design.md`
- `doc/world-simulator/viewer/viewer-chat-prompt-presets-profile-editing.design.md`
- `doc/world-simulator/viewer/viewer-chat-prompt-presets-scroll.design.md`
- `doc/world-simulator/viewer/viewer-chat-right-panel-polish.design.md`

- `doc/world-simulator/viewer/viewer-chat-web-deadlock-resolution.design.md`
- `doc/world-simulator/viewer/viewer-commercial-release-phase1-asset-pipeline.design.md`
- `doc/world-simulator/viewer/viewer-commercial-release-phase2-visual-quality-gate.design.md`
- `doc/world-simulator/viewer/viewer-commercial-release-phase3-material-style-layer.design.md`

- `doc/world-simulator/viewer/viewer-commercial-release-phase4-texture-style-layer.design.md`
- `doc/world-simulator/viewer/viewer-commercial-release-phase5-advanced-texture-maps.design.md`
- `doc/world-simulator/viewer/viewer-commercial-release-phase6-material-variant-preview.design.md`
- `doc/world-simulator/viewer/viewer-commercial-release-phase7-theme-pack-batch-preview.design.md`

- `doc/world-simulator/viewer/viewer-commercial-release-phase8-runtime-theme-hot-reload-and-asset-v2.design.md`
- `doc/world-simulator/viewer/viewer-control-advanced-debug-folding.design.md`
- `doc/world-simulator/viewer/viewer-control-feedback-iteration-checklist-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-control-feedback-step-recovery-p0-2026-02-27.design.md`

- `doc/world-simulator/viewer/viewer-control-plane-split-live-playback-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-control-predictability-tasklist-2026-02-28.design.md`
- `doc/world-simulator/viewer/viewer-copyable-text.design.md`
- `doc/world-simulator/viewer/viewer-dual-view-2d-3d.design.md`

- `doc/world-simulator/viewer/viewer-egui-right-panel.design.md`
- `doc/world-simulator/viewer/viewer-first-session-goal-clarity-hardening-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-first-session-goal-control-feedback-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-frag-default-rendering.design.md`
- `doc/world-simulator/viewer/viewer-frag-scale-selection-stability.design.md`
- `doc/world-simulator/viewer/viewer-fragment-element-rendering.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-experience-overhaul.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase2.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase3.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase4.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase5.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase6.design.md`
- `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase7.design.md`
- `doc/world-simulator/viewer/viewer-generic-focus-targets.design.md`
- `doc/world-simulator/viewer/viewer-i18n.design.md`
- `doc/world-simulator/viewer/viewer-industrial-visual-closure.design.md`
- `doc/world-simulator/viewer/viewer-industry-graph-layered-symbolic-zoom-2026-02-28.design.md`
- `doc/world-simulator/viewer/viewer-live-disable-seek-p2p-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-live-llm-event-driven-trigger-2026-02-26.design.md`
- `doc/world-simulator/viewer/viewer-live-logical-time-interface-phase11-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.design.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase2-2026-03-05.design.md`
- `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.design.md`
- `doc/world-simulator/viewer/viewer-live-step-control-progress-stability-2026-02-28.design.md`
- `doc/world-simulator/viewer/viewer-live-tick-driven-doc-archive-2026-02-27.design.md`
- `doc/world-simulator/viewer/viewer-location-depletion-visualization.design.md`
- `doc/world-simulator/viewer/viewer-location-fine-grained-rendering.design.md`
- `doc/world-simulator/viewer/viewer-minimal-system.design.md`
- `doc/world-simulator/viewer/viewer-module-visual-entities.design.md`
- `doc/world-simulator/viewer/viewer-node-hard-decouple-2026-02-28.design.md`
- `doc/world-simulator/viewer/viewer-observability-visual-optimization.design.md`
- `doc/world-simulator/viewer/viewer-overview-map-zoom.design.md`
- `doc/world-simulator/viewer/viewer-player-ui-declutter-2026-02-24.design.md`
- `doc/world-simulator/viewer/viewer-release-full-coverage-gate.design.md`
- `doc/world-simulator/viewer/viewer-release-qa-iteration-loop.design.md`
- `doc/world-simulator/viewer/viewer-rendering-physical-accuracy.design.md`
- `doc/world-simulator/viewer/viewer-right-panel-module-visibility.design.md`
- `doc/world-simulator/viewer/viewer-selection-details.design.md`
- `doc/world-simulator/viewer/viewer-step-completion-ack-2026-02-28.design.md`
- `doc/world-simulator/viewer/viewer-texture-inspector.design.md`
- `doc/world-simulator/viewer/viewer-visual-release-readiness-hardening-2026-03-01.design.md`
- `doc/world-simulator/viewer/viewer-visual-upgrade.design.md`
- `doc/world-simulator/viewer/viewer-visualization.design.md`
- `doc/world-simulator/viewer/viewer-visualization-3d.design.md`
- `doc/world-simulator/viewer/viewer-wasd-camera-navigation.design.md`
- `doc/world-simulator/viewer/viewer-web-fullscreen-panel-toggle.design.md`
- `doc/world-simulator/viewer/viewer-web-playability-unblock-2026-02-26.design.md`
- `doc/world-simulator/viewer/viewer-web-test-api-step-control-2026-02-24.design.md`
- `doc/world-simulator/viewer/viewer-web-usability-hardening-2026-02-22.design.md`
- `doc/world-simulator/viewer/viewer-webgl-deferred-compat-2026-02-24.design.md`
- `doc/world-simulator/scenario/agent-frag-initial-spawn-position.design.md`
- `doc/world-simulator/scenario/asteroid-fragment-renaming.design.md`
- `doc/world-simulator/scenario/chunked-fragment-generation.design.md`
- `doc/world-simulator/scenario/frag-resource-balance-onboarding.design.md`
- `doc/world-simulator/scenario/fragment-spacing.design.md`
- `doc/world-simulator/scenario/scenario-asteroid-fragment-overrides.design.md`
- `doc/world-simulator/scenario/scenario-files.design.md`
- `doc/world-simulator/scenario/scenario-power-facility-baseline.design.md`
- `doc/world-simulator/scenario/scenario-seed-locations.design.md`
- `doc/world-simulator/scenario/world-initialization.design.md`
- `doc/world-simulator/llm/indirect-control-tick-lifecycle-long-term-memory.design.md`
- `doc/world-simulator/llm/llm-agent-behavior.design.md`
- `doc/world-simulator/llm/llm-async-openai-responses.design.md`
- `doc/world-simulator/llm/llm-chat-user-message-tool-visualization.design.md`
- `doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.design.md`
- `doc/world-simulator/llm/llm-dialogue-chat-loop.design.md`
- `doc/world-simulator/llm/llm-factory-strategy-optimization.design.md`
- `doc/world-simulator/llm/llm-industrial-mining-debug-tools.design.md`
- `doc/world-simulator/llm/llm-lmso29-stability.design.md`
- `doc/world-simulator/llm/llm-multi-scenario-evaluation.design.md`
- `doc/world-simulator/llm/llm-prompt-effect-receipt.design.md`
- `doc/world-simulator/llm/llm-prompt-multi-step-orchestration.design.md`
- `doc/world-simulator/llm/llm-prompt-system.design.md`
- `doc/world-simulator/m4/m4-builtin-wasm-maintainability-2026-02-26.design.md`
- `doc/world-simulator/m4/m4-industrial-benchmark-current-state-2026-02-27.design.md`
- `doc/world-simulator/m4/m4-industrial-economy-wasm.design.md`
- `doc/world-simulator/m4/m4-market-hardware-data-governance-closure-2026-02-26.design.md`
- `doc/world-simulator/m4/m4-power-system.design.md`
- `doc/world-simulator/m4/m4-resource-product-system-p0-shared-bottleneck-logistics-priority-2026-02-27.design.md`
- `doc/world-simulator/m4/m4-resource-product-system-p1-maintenance-scarcity-pressure-2026-02-27.design.md`
- `doc/world-simulator/m4/m4-resource-product-system-p2-stage-guidance-market-governance-linkage-2026-02-27.design.md`
- `doc/world-simulator/m4/m4-resource-product-system-p3-layer-profile-chain-expansion-2026-02-27.design.md`
- `doc/world-simulator/m4/m4-resource-product-system-playability-2026-02-27.design.md`
- `doc/world-simulator/m4/m4-resource-product-system-playability-priority-hardening-2026-02-28.design.md`
- `doc/world-simulator/m4/material-multi-ledger-logistics.design.md`
