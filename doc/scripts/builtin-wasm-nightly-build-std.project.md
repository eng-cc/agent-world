# Builtin Wasm Nightly build-std 可复现构建（项目管理文档）

## 任务拆解
- [x] NBS-1 设计文档：`doc/scripts/builtin-wasm-nightly-build-std.md`
- [x] NBS-2 项目管理文档：本文件
- [x] NBS-3 `scripts/build-wasm-module.sh` 固化 nightly toolchain + rust-src/wasm target 准备
- [x] NBS-4 `tools/wasm_build_suite` 接入 `-Z build-std` / `-Z build-std-features`
- [ ] NBS-5 CI workflow 固化 nightly build-std 环境变量与组件安装
- [ ] NBS-6 重新同步 m1/m4 hash 清单并通过 `sync --check`
- [ ] NBS-7 required tier 回归通过（`CI_VERBOSE=1 ./scripts/ci-tests.sh required`）
- [ ] NBS-8 更新 devlog、收口文档并提交

## 依赖
- pinned nightly：`nightly-2025-12-11`
- target：`wasm32-unknown-unknown`
- rustup component：`rust-src`

## 状态
- 当前阶段：执行中
- 最近更新：NBS-1~NBS-4 完成（2026-02-17）
