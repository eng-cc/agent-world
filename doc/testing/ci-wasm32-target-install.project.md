# Agent World: CI 安装 wasm32-unknown-unknown target（项目管理文档）

## 任务拆解
- [x] WASMCI-1 设计文档落地（`doc/testing/ci-wasm32-target-install.md`）
- [x] WASMCI-1 项目管理文档落地（本文件）
- [x] WASMCI-2 更新 CI workflow：`required-gate/full-regression` 均安装 wasm target
- [x] WASMCI-3 执行最小回归验证并记录结果
- [x] WASMCI-3 更新任务日志并提交

## 依赖
- GitHub Actions workflow：`.github/workflows/rust.yml`
- 统一 CI 脚本：`scripts/ci-tests.sh`
- builtin wasm 工件校验脚本：`scripts/sync-m1-builtin-wasm-artifacts.sh`、`scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：已完成
