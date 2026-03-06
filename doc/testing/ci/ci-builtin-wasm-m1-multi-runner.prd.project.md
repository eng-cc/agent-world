# Agent World: CI 拆分 Builtin Wasm m1 多 Runner 校验（项目管理）

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-M1RUNNER-001): 建档（设计文档与项目管理文档落地）。
- [x] T2 (PRD-TESTING-CI-M1RUNNER-002): 实现 runner 摘要与跨 runner 对账脚本。
- [x] T3 (PRD-TESTING-CI-M1RUNNER-001/002/003): 接入独立 workflow，多 runner 仅执行 `m1 --check` + 汇总对账。
- [x] T4 (PRD-TESTING-CI-M1RUNNER-003): 测试手册同步、回归验证与收口。
- [x] T5 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并切换命名为 `.prd.md/.prd.project.md`。

## 依赖
- doc/testing/ci/ci-builtin-wasm-m1-multi-runner.prd.md
- `.github/workflows/rust.yml`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/build-builtin-wasm-modules.sh`
- `scripts/build-wasm-module.sh`
- `scripts/ci-m1-wasm-summary.sh`
- `scripts/ci-verify-m1-wasm-summaries.py`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.identity.json`
- `testing-manual.md`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成（T1~T5）
- 阻塞项：无
- 下一步：无（当前专题已收口）
