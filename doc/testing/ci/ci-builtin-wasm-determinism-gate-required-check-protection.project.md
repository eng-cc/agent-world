# Agent World: Builtin Wasm Determinism Gate Required Check 保护（项目管理）

- 对应设计文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.design.md`
- 对应需求文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.prd.md`

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-REQUIRED-001): 建档（设计文档与项目管理文档落地）。
- [x] T2 (PRD-TESTING-CI-REQUIRED-001/002): 实现 required check 自动化脚本。
- [x] T3 (PRD-TESTING-CI-REQUIRED-001/002/003): 默认上下文切换为 `Wasm Determinism Gate` 的 m1/m4/m5 verify job。
- [x] T4 (PRD-TESTING-CI-REQUIRED-003): 测试手册同步、回归验证与收口。

## 依赖
- `doc/testing/ci/ci-builtin-wasm-determinism-gate-required-check-protection.prd.md`
- `.github/workflows/wasm-determinism-gate.yml`
- `scripts/ci-m1-wasm-summary.sh`
- `scripts/ci-verify-m1-wasm-summaries.py`
- `scripts/wasm-release-evidence-report.sh`
- `scripts/ci-ensure-required-checks.py`
- `testing-manual.md`
- GitHub REST API（`gh api`）
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-17
- 当前阶段：已完成（默认上下文已与现行 workflow 对齐）
- 阻塞项：无
- 下一步：无（当前专题已收口）
