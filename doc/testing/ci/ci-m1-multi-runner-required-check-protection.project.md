# Agent World: m1 多 Runner CI Required Check 保护（项目管理）

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-REQUIRED-001): 建档（设计文档与项目管理文档落地）。
- [x] T2 (PRD-TESTING-CI-REQUIRED-001/002): 实现 required check 自动化脚本。
- [x] T3 (PRD-TESTING-CI-REQUIRED-001/002/003): 应用到仓库并验证生效。
- [x] T4 (PRD-TESTING-CI-REQUIRED-003): 测试手册同步、回归验证与收口。
- [x] T5 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并切换命名为 `.prd.md/.project.md`。

## 依赖
- doc/testing/ci/ci-m1-multi-runner-required-check-protection.prd.md
- `.github/workflows/builtin-wasm-m1-multi-runner.yml`
- `scripts/ci-m1-wasm-summary.sh`
- `scripts/ci-verify-m1-wasm-summaries.py`
- `scripts/ci-ensure-required-checks.sh`
- `testing-manual.md`
- GitHub REST API（`gh api`）
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成（T1~T5）
- 阻塞项：无
- 下一步：无（当前专题已收口）
