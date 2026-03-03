# testing PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-TESTING-001 (PRD-TESTING-001): 完成 testing PRD 改写，建立分层测试设计入口。
- [ ] TASK-TESTING-002 (PRD-TESTING-001/002): 对齐 S0~S10 与改动路径触发矩阵。
- [ ] TASK-TESTING-003 (PRD-TESTING-002/003): 建立发布证据包模板（命令、日志、截图、结论）。
- [ ] TASK-TESTING-004 (PRD-TESTING-003): 建立测试质量趋势跟踪（通过率/逃逸率/修复时长）。
- [x] TASK-TESTING-005 (PRD-TESTING-002/003): 建立模块级专题任务映射索引（2026-03-02 批次）。

## 专题任务映射（2026-03-02 批次）
- [x] SUBTASK-TESTING-20260302-001 (PRD-TESTING-002/003): `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260302-002 (PRD-TESTING-002/003): `doc/testing/launcher/launcher-viewer-auth-node-config-autowire-2026-03-02.project.md`
- [x] SUBTASK-TESTING-20260302-003 (PRD-TESTING-002/003): `doc/testing/longrun/p2p-longrun-feedback-event-injection-2026-03-02.project.md`

## 依赖
- `testing-manual.md`
- `doc/testing/manual/web-ui-playwright-closure-manual.md`
- `scripts/ci-tests.sh`
- `.github/workflows/*`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-TESTING-002
- 专题映射状态: 2026-03-02 批次 3/3 已纳入模块项目管理文档。
- 说明: 本文档仅维护 testing 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
