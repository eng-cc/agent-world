# Agent World: Required Tier 接入 M1 Builtin Wasm Hash 校验（项目管理文档）

## 任务拆解
- [x] T1 设计文档：`doc/testing/ci-required-m1-wasm-hash-check.md`
- [x] T1 项目管理文档：`doc/testing/ci-required-m1-wasm-hash-check.project.md`
- [x] T2 改造 `scripts/ci-tests.sh`：required 前置检查加入 `sync-m1 --check`（CI 强制，local 可手动开启）
- [x] T3 回归验证：`CI_VERBOSE=1 ./scripts/ci-tests.sh required` 通过；`CI=true ./scripts/ci-tests.sh required` 触发 m1 校验分支
- [x] T4 更新 `doc/devlog/2026-02-18.md` 并提交

## 依赖
- 统一测试入口：`scripts/ci-tests.sh`
- m1 校验脚本：`scripts/sync-m1-builtin-wasm-artifacts.sh`
- m1 hash 清单：`crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`

## 状态
- 当前阶段：已完成
