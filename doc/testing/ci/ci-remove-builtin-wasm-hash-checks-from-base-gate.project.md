# oasis7: 基础 CI 门禁移除 Builtin Wasm Hash 校验（项目管理）

- 对应设计文档: `doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.design.md`
- 对应需求文档: `doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-HASHBASE-001): 建档（设计文档与项目管理文档落地）。
- [x] T2 (PRD-TESTING-CI-HASHBASE-001/002): 修改基础门禁脚本，移除 m1/m4/m5 hash 校验。
- [x] T3 (PRD-TESTING-CI-HASHBASE-002/003): 同步测试手册口径。
- [x] T4 (PRD-TESTING-CI-HASHBASE-003): 验证与收口。
- [x] T5 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并切换命名为 `.prd.md/.project.md`。

## 依赖
- doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.prd.md
- `scripts/ci-tests.sh`
- `testing-manual.md`
- `.github/workflows/rust.yml`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成（T1~T5）
- 阻塞项：无
- 下一步：无（当前专题已收口）
