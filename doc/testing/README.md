# testing 文档索引

审计轮次: 6

## 入口
- PRD: `doc/testing/prd.md`
- 设计总览: `doc/testing/design.md`
- 标准执行入口: `doc/testing/project.md`
- 兼容执行入口: `doc/testing/project.md`
- 文件级索引: doc/testing/prd.index.md

## 关键文档
- 系统测试手册：`testing-manual.md`
- 模块化测试细则：`doc/testing/manual/`
- CI 与门禁专题：`doc/testing/ci/`
- 启动器链路测试：`doc/testing/launcher/`
- 长稳与压力测试：`doc/testing/longrun/`、`doc/testing/performance/`
- 门禁策略与治理：`doc/testing/governance/`、`doc/testing/chaos-plans/`

## 根目录收口
- 模块根目录仅保留：`README.md`、`prd.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `ci/launcher/longrun/performance/manual/governance`。

## 维护约定
- 测试门禁变更需同步 required/full 分层口径与对应脚本。
- `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.design.md`
- `doc/testing/ci/ci-builtin-wasm-m4-m5-hash-drift-hardening.design.md`
- `doc/testing/ci/ci-m1-multi-runner-required-check-protection.design.md`
- `doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.design.md`
- `doc/testing/ci/ci-test-coverage.design.md`
- `doc/testing/ci/ci-testcase-tiering.design.md`
- `doc/testing/ci/ci-tiered-execution.design.md`
- `doc/testing/ci/ci-wasm32-target-install.design.md`
- `doc/testing/governance/llm-skip-tick-ratio-metric.design.md`
- `doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.design.md`
- `doc/testing/governance/wasm-build-determinism-guard.design.md`
- `doc/testing/launcher/launcher-chain-script-migration-2026-02-28.design.md`
- `doc/testing/launcher/launcher-full-usability-closure-audit-2026-03-08.design.md`
- `doc/testing/launcher/launcher-lifecycle-hardening-2026-03-01.design.md`
- `doc/testing/launcher/launcher-viewer-auth-node-config-autowire-2026-03-02.design.md`
- `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.design.md`
- `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.design.md`
- `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.design.md`
- `doc/testing/longrun/p2p-longrun-endurance-chaos-template-2026-02-25.design.md`
- `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.design.md`
- `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.design.md`
- `doc/testing/longrun/s10-distfs-probe-bootstrap-2026-02-28.design.md`
- `doc/testing/longrun/s10-five-node-real-game-soak.design.md`
- `doc/testing/manual/systematic-application-testing-manual.design.md`
- `doc/testing/manual/web-ui-playwright-closure-manual.design.md`
- `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.design.md`
- `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.design.md`
- `doc/testing/performance/viewer-perf-bottleneck-observability-2026-02-25.design.md`
- `doc/testing/performance/viewer-performance-methodology-closure-2026-02-25.design.md`
