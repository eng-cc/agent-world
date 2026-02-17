# Builtin Wasm Hash 可复现构建（项目管理文档）

## 任务拆解
- [x] RWH-1 输出设计文档（`doc/scripts/builtin-wasm-reproducible-hash.md`）
- [x] RWH-2 输出项目管理文档（本文件）
- [x] RWH-3 在 `scripts/build-wasm-module.sh` 接入 remap-path-prefix
- [x] RWH-4 同步并校验 m1/m4 hash 清单（`sync --check`）
- [x] RWH-5 运行 required tier 回归
- [x] RWH-6 更新 devlog 并提交
- [x] RWH-7 补充 `rustc --print sysroot` remap，消除 host triple 进入 wasm 路径
- [x] RWH-8 复跑 m1/m4 hash 校验与 required tier，确认修复在本地闭环

## 依赖
- Rust toolchain `1.92.0`
- target：`wasm32-unknown-unknown`
- 构建入口：`scripts/build-wasm-module.sh`
- 校验入口：`scripts/sync-m1-builtin-wasm-artifacts.sh`、`scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：已完成
- 最近更新：RWH-1~RWH-8 全部完成（2026-02-17）
