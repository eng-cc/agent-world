# Agent World Runtime：WASM Docker 确定性构建与工件治理管线（项目管理）

- 对应设计文档: `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.design.md`
- 对应需求文档: `doc/world-runtime/wasm/wasm-deterministic-build-pipeline.prd.md`

审计轮次: 2

## 任务拆解（含 PRD-ID 映射）
- [x] WDBP-0 (PRD-WORLD_RUNTIME-020/021/022) [test_tier_required]: 将专题目标从“host deterministic guard + keyed 平台 hash 对账”修正为“Docker-first canonical builder”，并回写 root PRD / project / README / devlog。
- [x] WDBP-1 (PRD-WORLD_RUNTIME-020/021) [test_tier_required]: 新增 pinned WASM builder image（`docker/wasm-builder/Dockerfile`）与 host wrapper，固定 `linux-x86_64` container platform 作为 canonical publish build 平台。
- [x] WDBP-2 (PRD-WORLD_RUNTIME-020/021) [test_tier_required]: 将现有 `tools/wasm_build_suite` 收敛到容器内执行，输出 build receipt，并把 manifest 从多宿主 keyed token 迁移为单 canonical token `linux-x86_64=<sha256>`。
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
- 当前阶段: WDBP-3 待执行
- owner role: `wasm_platform_engineer`
- 联审角色: `producer_system_designer`、`runtime_engineer`
- 验证角色: `qa_engineer`
- 阻塞项: release evidence / release gate 仍未完全切换到 receipt 驱动；`compile_module_artifact_from_source` 生产路径也尚未外移或默认禁用
- 实施备注:
  - `docker/wasm-builder/Dockerfile` 与 `scripts/build-wasm-module.sh` 已落地，当前 canonical build 已收敛为 Docker-only path，不再提供 host-native fallback。
  - `tools/wasm_build_suite` 已新增 `build receipt`、`source_hash`、`build_manifest_hash`、`builder_image_digest` 与 `container_platform` 输出；builtin `m1/m4/m5` hash manifest 已全部改写为单 canonical token `linux-x86_64=<sha256>`。
  - `crates/agent_world_distfs/src/bin/sync_builtin_wasm_identity.rs` 已切换为 receipt 驱动 identity 生成；写路径只输出 canonical token，读路径仍兼容 legacy multi-token manifest。
  - `scripts/ci-m1-wasm-summary.sh` 与 `scripts/ci-verify-m1-wasm-summaries.py` 已区分 `host_platform` 与 `canonical_platform`，并新增 `receipt_evidence + identity_build_recipe` 对账；当前 CI 对账口径改为“不同宿主只比较 Docker canonical 输出与一致的 receipt/build recipe 证据”。
  - runtime `ModuleReleaseSubmitAttestation -> apply` 现已显式绑定 `builder_image_digest + container_platform + canonicalizer_version`；release gate 会拒绝阈值 attestation 间的 receipt evidence 不一致，且要求 attestation 的 `source_hash/build_manifest_hash/wasm_hash` 与 manifest identity 对齐。
  - `ModuleReleaseManifestMappingState` 与节点验收脚本现已补齐 release evidence 摘要：映射状态会落盘 `release_{wasm,source,build_manifest}_hash + builder_image_digest + container_platform + canonicalizer_version + attestation_platforms + proof_cids + receipt_evidence_conflict`，`scripts/module-release-node-acceptance.sh` 也已纳入 receipt mismatch 阻断用例。
  - 新增 `scripts/wasm-release-evidence-report.sh` 作为多 runner fixed entry，可统一收集/校验 `m1/m4/m5` summary 并输出 `summary.md/json`，为后续 Linux + macOS full-tier 证据归档提供固定落点。
  - `compile_module_artifact_from_source` 是 Docker-first 迁移中的最大结构性变更点，因为 production runtime 不应默认持有 Docker daemon 权限。
