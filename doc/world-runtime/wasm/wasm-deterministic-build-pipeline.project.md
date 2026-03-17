# Agent World Runtime：WASM Docker 确定性构建与工件治理管线（项目管理）

- 对应设计文档: `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.design.md`
- 对应需求文档: `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.prd.md`

审计轮次: 2

## 任务拆解（含 PRD-ID 映射）
- [x] WDBP-0 (PRD-WORLD_RUNTIME-020/021/022) [test_tier_required]: 将专题目标从“host deterministic guard + keyed 平台 hash 对账”修正为“Docker-first canonical builder”，并回写 root PRD / project / README / devlog。
- [ ] WDBP-1 (PRD-WORLD_RUNTIME-020/021) [test_tier_required]: 新增 pinned WASM builder image（`docker/wasm-builder/Dockerfile`）与 host wrapper，固定 `linux-x86_64` container platform 作为 canonical publish build 平台。
- [ ] WDBP-2 (PRD-WORLD_RUNTIME-020/021) [test_tier_required]: 将现有 `tools/wasm_build_suite` 收敛到容器内执行，输出 build receipt，并把 manifest 从多宿主 keyed token 迁移为单 canonical token `linux-x86_64=<sha256>`。
- [ ] WDBP-3 (PRD-WORLD_RUNTIME-021/022) [test_tier_required + test_tier_full]: 将 identity / release evidence / CI summary / release gate 全面切换为 Docker canonical hash，对 macOS/Linux 只比较容器输出，不再比较 host-native 输出。
- [ ] WDBP-4 (PRD-WORLD_RUNTIME-022) [test_tier_required]: 把 `compile_module_artifact_from_source` 的生产路径外移到 external Docker builder 或 production 默认禁用，runtime 只消费 binary + build receipt。

## 依赖
- `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.prd.md`
- `scripts/build-wasm-module.sh`
- `tools/wasm_build_suite/src/lib.rs`
- `crates/agent_world/src/runtime/module_source_compiler.rs`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`
- `scripts/sync-m5-builtin-wasm-artifacts.sh`
- `crates/agent_world_distfs/src/bin/sync_builtin_wasm_identity.rs`
- `scripts/ci-m1-wasm-summary.sh`
- `scripts/ci-verify-m1-wasm-summaries.py`
- `crates/agent_world/src/runtime/builtin_wasm_materializer.rs`
- `crates/agent_world/src/runtime/world/release_manifest.rs`

## 状态
- 更新日期: 2026-03-17
- 当前阶段: WDBP-1 待执行
- owner role: `wasm_platform_engineer`
- 联审角色: `producer_system_designer`、`runtime_engineer`
- 验证角色: `qa_engineer`
- 阻塞项: 当前仓库尚无 Docker builder image 或容器构建入口
- 实施备注:
  - 当前仓库没有 `Dockerfile`；现有 host deterministic guard 只能算过渡方案，不能满足“不同宿主同一 canonical publish hash”目标。
  - 现有 keyed manifest / multi-runner workflow 需要保留读路径兼容，但写路径目标应切换为单 canonical token。
  - `compile_module_artifact_from_source` 是 Docker-first 迁移中的最大结构性变更点，因为 production runtime 不应默认持有 Docker daemon 权限。
