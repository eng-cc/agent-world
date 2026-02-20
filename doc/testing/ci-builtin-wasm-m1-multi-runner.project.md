# Agent World: CI 拆分 Builtin Wasm m1 多 Runner 校验（项目管理文档）

## 任务拆解
- [x] T1 建档：设计文档与项目管理文档落地
- [x] T2 实现 runner 摘要与跨 runner 对账脚本
- [x] T3 接入独立 workflow：多 runner 仅执行 `m1 --check` + 汇总对账
- [ ] T4 测试手册同步、回归验证与收口

## 依赖
- `.github/workflows/rust.yml`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/build-builtin-wasm-modules.sh`
- `scripts/build-wasm-module.sh`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.identity.json`
- `testing-manual.md`

## 状态
- 当前阶段：进行中（T4）
- 最近更新：T3 完成，进入文档收口与回归阶段（2026-02-20）
