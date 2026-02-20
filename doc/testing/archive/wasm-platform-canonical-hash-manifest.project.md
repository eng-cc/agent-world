# Agent World: Builtin Wasm 平台 Canonical Hash 清单（项目管理文档）

> 归档说明（2026-02-20）：该任务已由 `doc/p2p/builtin-wasm-identity-consensus.md` / `.project.md` 覆盖并替代。

## 任务拆解
- [x] T1 设计文档：`doc/testing/wasm-platform-canonical-hash-manifest.md`
- [x] T1 项目管理文档：`doc/testing/wasm-platform-canonical-hash-manifest.project.md`
- [x] T2 `sync-m1` 脚本改造：平台 canonical 校验与更新
- [x] T3 runtime/hydrator/测试解析支持 `platform=hash`（兼容 legacy）
- [x] T4 迁移 m1 manifest 数据并完成回归测试
- [x] T5 更新 `doc/devlog/2026-02-18.md` 并提交

## 依赖
- m1 清单校验脚本：`scripts/sync-m1-builtin-wasm-artifacts.sh`
- runtime builtin loader：`crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- DistFS hydration：`crates/agent_world_distfs/src/bin/hydrate_builtin_wasm.rs`
- required 入口：`scripts/ci-tests.sh`

## 状态
- 当前阶段：已完成
