# Agent World: CI 测试分级细化到 Test Case（项目管理）

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-TIER-001): 输出设计文档与项目管理文档。
- [x] T1.1 (PRD-TESTING-CI-TIER-001): 清理 `check-include-warning-baseline` 旧门禁脚本与调用。
- [x] T2 (PRD-TESTING-CI-TIER-001/002): 改造 `scripts/ci-tests.sh`，`required` 执行 test case 级 smoke 筛选。
- [x] T2.1 (PRD-TESTING-CI-TIER-001/002): 移除硬编码 `--test` 清单，改为 `--tests + feature` 自动过滤。
- [x] T3 (PRD-TESTING-CI-TIER-002/003): 文档回写、任务日志更新、验证并提交。
- [x] T4 (PRD-TESTING-CI-TIER-003): 巡检 feature 覆盖并执行 required/full/workspace 回归。
- [x] T5 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并切换命名为 `.prd.md/.prd.project.md`。

## 依赖
- doc/testing/ci/ci-testcase-tiering.prd.md
- `scripts/ci-tests.sh`
- `.github/workflows/rust.yml`
- `scripts/pre-commit.sh`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
- 审计备注（2026-03-05 ROUND-002）：本文件仅保留执行任务记录；标签语义定义以 `ci-testcase-tiering.prd.md` 为准。
