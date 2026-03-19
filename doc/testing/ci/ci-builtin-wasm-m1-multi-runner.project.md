# oasis7: Builtin Wasm 确定性 Gate（m1 历史文件名归档提示 / 项目管理）

- 对应设计文档: `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.design.md`
- 对应需求文档: `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.prd.md`
- 当前活跃设计文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-m1.design.md`
- 当前活跃需求文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-m1.prd.md`

> 状态更新（2026-03-17）:
> - 该旧项目文件名仅保留为历史追溯入口。
> - 当前活跃项目文档见 `doc/testing/ci/ci-builtin-wasm-determinism-gate-m1.project.md`。

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-M1RUNNER-001): 历史专题保留并对齐 strict schema。
- [x] T2 (PRD-TESTING-CI-M1RUNNER-002): 使用现行 summary / receipt evidence schema 回写专题口径。
- [x] T3 (PRD-TESTING-CI-M1RUNNER-001/002/003): 现行 gate 已统一收敛到 `.github/workflows/wasm-determinism-gate.yml` + `scripts/wasm-release-evidence-report.sh`。
- [x] T4 (PRD-TESTING-CI-M1RUNNER-003): testing-manual / project / devlog 同步。

## 依赖
- `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.prd.md`
- `.github/workflows/wasm-determinism-gate.yml`
- `scripts/ci-m1-wasm-summary.sh`
- `scripts/ci-verify-m1-wasm-summaries.py`
- `scripts/wasm-release-evidence-report.sh`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.identity.json`
- `testing-manual.md`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-17
- 当前阶段：已完成（历史主题已并入 `wasm-determinism-gate`）
- 阻塞项：无
- 下一步：无（当前专题已收口；外部跨宿主 full-tier 证据由 `wasm-determinism-gate` 补充）
