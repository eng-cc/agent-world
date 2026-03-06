# Agent World: Builtin Wasm m4/m5 Hash 漂移治理与发布链路收敛（项目管理）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T1 (PRD-TESTING-CI-WASMHARD-001/002/003): 建档（专题设计文档与项目管理文档落地，冻结 1-6 治理项范围）。
- [x] T2 (PRD-TESTING-CI-WASMHARD-001): m4/m5 hash manifest 迁移为 keyed canonical 平台 token。
- [x] T3 (PRD-TESTING-CI-WASMHARD-001): sync 脚本启用 strict 模式并彻底禁止 legacy token 写回。
- [x] T4 (PRD-TESTING-CI-WASMHARD-003): 新增 m4/m5 多 runner 摘要与跨 runner 对账 workflow。
- [x] T5 (PRD-TESTING-CI-WASMHARD-003): required checks 自动化默认上下文扩展到 m1/m4/m5 汇总校验。
- [x] T6 (PRD-TESTING-CI-WASMHARD-002): identity `source_hash` 输入收敛为源码白名单/可追踪文件集合。
- [x] T7 (PRD-TESTING-CI-WASMHARD-002): identity 输入移除 workspace 根 `Cargo.lock`，改为模块级 lockfile 策略。
- [x] T8 (PRD-TESTING-CI-WASMHARD-003): 落地“本地仅 `--check`、写入仅 CI bot”策略并同步测试手册。

## 依赖
- `doc/testing/ci/ci-builtin-wasm-m4-m5-hash-drift-hardening.prd.md`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`
- `scripts/sync-m5-builtin-wasm-artifacts.sh`
- `scripts/ci-m1-wasm-summary.sh`
- `scripts/ci-verify-m1-wasm-summaries.py`
- `scripts/ci-ensure-required-checks.py`
- `crates/agent_world_distfs/src/bin/sync_builtin_wasm_identity.rs`
- `.github/workflows/builtin-wasm-m1-multi-runner.yml`
- `.github/workflows/builtin-wasm-m4-m5-multi-runner.yml`
- `testing-manual.md`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-06
- 当前阶段：已完成（T1~T8 全部完成）
- 阻塞项：无
- 下一步：无（等待新一轮治理需求）
