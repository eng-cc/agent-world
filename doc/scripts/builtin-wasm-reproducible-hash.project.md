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
- [x] RWH-9 在 `tools/wasm_build_suite` 接入 wasm canonicalize（移除 custom sections 后再计算 hash）
- [x] RWH-10 为 canonicalize 增加单测与模板集成测试覆盖
- [x] RWH-11 重新同步 m1/m4 hash 清单并回归 `sync --check`
- [x] RWH-12 复跑 required tier，确认门禁本地闭环
- [ ] RWH-13 推送并验证 GitHub Actions required-gate 在 Ubuntu 通过
- [x] RWH-14 扩展 `scripts/build-wasm-module.sh`：补充 `RUSTUP_HOME/toolchains/*` 全量 remap，覆盖 toolchain alias 路径

## 依赖
- Rust toolchain `1.92.0`
- target：`wasm32-unknown-unknown`
- 构建入口：`scripts/build-wasm-module.sh`
- 校验入口：`scripts/sync-m1-builtin-wasm-artifacts.sh`、`scripts/sync-m4-builtin-wasm-artifacts.sh`

## 状态
- 当前阶段：执行中（待 RWH-13）
- 最近更新：RWH-1~RWH-12、RWH-14 完成，RWH-13 待验证（2026-02-17）
