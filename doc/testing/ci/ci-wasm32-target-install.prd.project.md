# Agent World: CI 安装 wasm32-unknown-unknown target（项目管理）

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] WASMCI-1 (PRD-TESTING-CI-WASM-001/003): 设计文档与项目管理文档落地。
- [x] WASMCI-2 (PRD-TESTING-CI-WASM-001/002): 更新 CI workflow，`required-gate/full-regression` 均显式安装 wasm target。
- [x] WASMCI-3 (PRD-TESTING-CI-WASM-002/003): 执行最小回归验证并完成任务日志收口。
- [x] WASMCI-4 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并切换命名为 `.prd.md/.prd.project.md`。

## 依赖
- doc/testing/ci/ci-wasm32-target-install.prd.md
- GitHub Actions workflow：`.github/workflows/rust.yml`
- 统一 CI 脚本：`scripts/ci-tests.sh`
- builtin wasm 工件校验脚本：
  - `scripts/sync-m1-builtin-wasm-artifacts.sh`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh`
- 模块主追踪文档：
  - `doc/testing/prd.md`
  - `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
