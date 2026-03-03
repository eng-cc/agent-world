# testing PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-TESTING-001 (PRD-TESTING-001): 完成 testing PRD 改写，建立分层测试设计入口。
- [ ] TASK-TESTING-002 (PRD-TESTING-001/002): 对齐 S0~S10 与改动路径触发矩阵。
- [ ] TASK-TESTING-003 (PRD-TESTING-002/003): 建立发布证据包模板（命令、日志、截图、结论）。
- [ ] TASK-TESTING-004 (PRD-TESTING-003): 建立测试质量趋势跟踪（通过率/逃逸率/修复时长）。
- [x] TASK-TESTING-005 (PRD-TESTING-002/003): 建立模块级专题任务映射索引（2026-03-02 批次）。
- [x] TASK-TESTING-006 (PRD-TESTING-001/002/003): 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-TESTING-007 (PRD-TESTING-004): 完成 `ci-wasm32-target-install` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [ ] TASK-TESTING-008 (PRD-TESTING-004): 继续按批次迁移 testing 活跃 legacy 专题文档（优先 `governance/launcher/longrun/performance/manual`）。
- [x] TASK-TESTING-009 (PRD-TESTING-004): 完成 `ci-testcase-tiering` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-010 (PRD-TESTING-004): 完成 `ci-tiered-execution` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-011 (PRD-TESTING-004): 完成 `ci-test-coverage` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-012 (PRD-TESTING-004): 完成 `ci-builtin-wasm-m1-multi-runner` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-013 (PRD-TESTING-004): 完成 `ci-m1-multi-runner-required-check-protection` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-014 (PRD-TESTING-004): 完成 `ci-remove-builtin-wasm-hash-checks-from-base-gate` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-015 (PRD-TESTING-004): 完成 `wasm-build-determinism-guard` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-016 (PRD-TESTING-004): 完成 `release-gate-metric-policy-alignment-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-017 (PRD-TESTING-004): 完成 `llm-skip-tick-ratio-metric` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-018 (PRD-TESTING-004): 完成 `launcher-chain-script-migration-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-019 (PRD-TESTING-004): 完成 `launcher-lifecycle-hardening-2026-03-01` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-020 (PRD-TESTING-004): 完成 `launcher-viewer-auth-node-config-autowire-2026-03-02` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-021 (PRD-TESTING-004): 完成 `chain-runtime-feedback-replication-network-autowire-2026-03-02` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-022 (PRD-TESTING-004): 完成 `chain-runtime-soak-script-reactivation-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-023 (PRD-TESTING-004): 完成 `p2p-longrun-continuous-chaos-injection-2026-02-24` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-024 (PRD-TESTING-004): 完成 `p2p-longrun-endurance-chaos-template-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-025 (PRD-TESTING-004): 完成 `p2p-storage-consensus-longrun-online-stability-2026-02-24` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-026 (PRD-TESTING-004): 完成 `p2p-longrun-feedback-event-injection-2026-03-02` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-027 (PRD-TESTING-004): 完成 `s10-distfs-probe-bootstrap-2026-02-28` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-028 (PRD-TESTING-004): 完成 `s10-five-node-real-game-soak` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-029 (PRD-TESTING-004): 完成 `runtime-performance-observability-foundation-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-030 (PRD-TESTING-004): 完成 `runtime-performance-observability-llm-api-decoupling-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-031 (PRD-TESTING-004): 完成 `viewer-perf-bottleneck-observability-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-032 (PRD-TESTING-004): 完成 `viewer-performance-methodology-closure-2026-02-25` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-033 (PRD-TESTING-004): 完成 `systematic-application-testing-manual` 专题文档逐篇人工迁移到 strict schema，并统一 `.prd` 命名。
- [x] TASK-TESTING-034 (PRD-TESTING-004): 完成 `web-ui-playwright-closure-manual` 专题文档逐篇人工迁移到 strict schema，并补齐 `.prd.project.md` 管理文档。

## 专题任务映射（2026-03-02 批次）
- [x] SUBTASK-TESTING-20260302-001 (PRD-TESTING-002/003): `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.prd.project.md`
- [x] SUBTASK-TESTING-20260302-002 (PRD-TESTING-002/003): `doc/testing/launcher/launcher-viewer-auth-node-config-autowire-2026-03-02.prd.project.md`
- [x] SUBTASK-TESTING-20260302-003 (PRD-TESTING-002/003): `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.prd.project.md`

## 专题任务映射（2026-03-03 批次）
- [x] SUBTASK-TESTING-20260303-001 (PRD-TESTING-004): `doc/testing/ci/ci-wasm32-target-install.prd.project.md`
- [x] SUBTASK-TESTING-20260303-002 (PRD-TESTING-004): `doc/testing/ci/ci-testcase-tiering.prd.project.md`
- [x] SUBTASK-TESTING-20260303-003 (PRD-TESTING-004): `doc/testing/ci/ci-tiered-execution.prd.project.md`
- [x] SUBTASK-TESTING-20260303-004 (PRD-TESTING-004): `doc/testing/ci/ci-test-coverage.prd.project.md`
- [x] SUBTASK-TESTING-20260303-005 (PRD-TESTING-004): `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.prd.project.md`
- [x] SUBTASK-TESTING-20260303-006 (PRD-TESTING-004): `doc/testing/ci/ci-m1-multi-runner-required-check-protection.prd.project.md`
- [x] SUBTASK-TESTING-20260303-007 (PRD-TESTING-004): `doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.prd.project.md`
- [x] SUBTASK-TESTING-20260303-008 (PRD-TESTING-004): `doc/testing/governance/wasm-build-determinism-guard.prd.project.md`
- [x] SUBTASK-TESTING-20260303-009 (PRD-TESTING-004): `doc/testing/governance/release-gate-metric-policy-alignment-2026-02-28.prd.project.md`
- [x] SUBTASK-TESTING-20260303-010 (PRD-TESTING-004): `doc/testing/governance/llm-skip-tick-ratio-metric.prd.project.md`
- [x] SUBTASK-TESTING-20260303-011 (PRD-TESTING-004): `doc/testing/launcher/launcher-chain-script-migration-2026-02-28.prd.project.md`
- [x] SUBTASK-TESTING-20260303-012 (PRD-TESTING-004): `doc/testing/launcher/launcher-lifecycle-hardening-2026-03-01.prd.project.md`
- [x] SUBTASK-TESTING-20260303-013 (PRD-TESTING-004): `doc/testing/launcher/launcher-viewer-auth-node-config-autowire-2026-03-02.prd.project.md`
- [x] SUBTASK-TESTING-20260303-014 (PRD-TESTING-004): `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.prd.project.md`
- [x] SUBTASK-TESTING-20260303-015 (PRD-TESTING-004): `doc/testing/longrun/chain-runtime-soak-script-reactivation-2026-02-28.prd.project.md`
- [x] SUBTASK-TESTING-20260303-016 (PRD-TESTING-004): `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.prd.project.md`
- [x] SUBTASK-TESTING-20260303-017 (PRD-TESTING-004): `doc/testing/longrun/p2p-longrun-endurance-chaos-template-2026-02-25.prd.project.md`
- [x] SUBTASK-TESTING-20260303-018 (PRD-TESTING-004): `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.project.md`
- [x] SUBTASK-TESTING-20260303-019 (PRD-TESTING-004): `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.prd.project.md`
- [x] SUBTASK-TESTING-20260303-020 (PRD-TESTING-004): `doc/testing/longrun/s10-distfs-probe-bootstrap-2026-02-28.prd.project.md`
- [x] SUBTASK-TESTING-20260303-021 (PRD-TESTING-004): `doc/testing/longrun/s10-five-node-real-game-soak.prd.project.md`
- [x] SUBTASK-TESTING-20260303-022 (PRD-TESTING-004): `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.prd.project.md`
- [x] SUBTASK-TESTING-20260303-023 (PRD-TESTING-004): `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.prd.project.md`
- [x] SUBTASK-TESTING-20260303-024 (PRD-TESTING-004): `doc/testing/performance/viewer-perf-bottleneck-observability-2026-02-25.prd.project.md`
- [x] SUBTASK-TESTING-20260303-025 (PRD-TESTING-004): `doc/testing/performance/viewer-performance-methodology-closure-2026-02-25.prd.project.md`
- [x] SUBTASK-TESTING-20260303-026 (PRD-TESTING-004): `doc/testing/manual/systematic-application-testing-manual.prd.project.md`
- [x] SUBTASK-TESTING-20260303-027 (PRD-TESTING-004): `doc/testing/manual/web-ui-playwright-closure-manual.prd.project.md`

## 依赖
- `testing-manual.md`
- `doc/testing/manual/web-ui-playwright-closure-manual.prd.md`
- `scripts/ci-tests.sh`
- `.github/workflows/*`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-TESTING-008
- 专题映射状态: 2026-03-02 批次 3/3 已纳入模块项目管理文档。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 testing 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
