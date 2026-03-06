# Agent World: Builtin Wasm m4/m5 Hash 漂移治理与发布链路收敛

审计轮次: 1

- 对应项目管理文档: doc/testing/ci/ci-builtin-wasm-m4-m5-hash-drift-hardening.prd.project.md

## 1. Executive Summary
- Problem Statement: 目前 m4/m5 仍使用 legacy 单 token hash 清单，且发布/校验链路对 local 与 CI 的职责边界不够明确，导致 hash 结果在不同 runner 上容易出现漂移覆盖与回归困难。
- Proposed Solution: 将 m4/m5 升级为 keyed 平台 token，统一 sync strict 模式、identity 输入白名单与 CI 多 runner 对账，同时固化 required check 与“本地仅 `--check`、写入仅 CI bot”策略。
- Success Criteria:
  - SC-1: `m4/m5` hash manifest 仅包含 canonical 平台 keyed token（`darwin-arm64`、`linux-x86_64`）。
  - SC-2: `sync-m*-builtin-wasm-artifacts.sh` 禁止 legacy token 写回，并在 strict 模式下拒绝 legacy/mixed 输入。
  - SC-3: `m4/m5` 建立多 runner 对账 workflow，单次运行可定位平台差异。
  - SC-4: required checks 默认包含 `m1/m4/m5` 多 runner 汇总校验上下文。
  - SC-5: `source_hash` 仅基于可追踪源码与模块级 lockfile 输入，不再依赖 workspace 根 `Cargo.lock`。
  - SC-6: 本地默认只读校验，manifest/identity 写入路径限定 CI bot 流程。

## 2. User Experience & Functionality
- User Personas:
  - CI 维护者：需要跨平台 hash 漂移在 PR 阶段被自动拦截并可定位。
  - 发布工程维护者：需要 manifest/identity 的写入来源可审计且不可被本地误覆盖。
  - 模块开发者：需要在本地快速执行 `--check` 获取确定性结果。
- User Scenarios & Frequency:
  - PR 门禁：涉及 wasm 构建链路改动时，每次 PR 触发 multi-runner 对账。
  - 发布更新：仅在发布流程由 CI bot 执行 manifest/identity 写入。
  - 本地调试：开发者高频执行 `sync-m*/--check` 进行只读校验。
- User Stories:
  - PRD-TESTING-CI-WASMHARD-001: As a CI 维护者, I want m4/m5 manifests and sync flow to be strict keyed-format only, so that legacy drift paths are removed.
  - PRD-TESTING-CI-WASMHARD-002: As a 发布工程维护者, I want identity source hash inputs constrained to stable tracked files, so that identity hash no longer drifts on unrelated workspace noise.
  - PRD-TESTING-CI-WASMHARD-003: As a 仓库管理员, I want required checks and write-policy automation for wasm hash flows, so that policy drift and accidental local overwrite are prevented.
- Critical User Flows:
  1. Flow-WASMHARD-001: `PR 触发 -> m1/m4/m5 多 runner 生成摘要 -> 汇总对账 -> required check 放行/阻断`
  2. Flow-WASMHARD-002: `发布触发 -> CI bot 执行 sync 写入 -> keyed manifest + identity 更新 -> 合规提交`
  3. Flow-WASMHARD-003: `本地执行 sync -> 默认仅 --check -> 若请求写入且非 CI bot 则拒绝并提示策略`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| keyed hash manifest | `module_id`, `<platform>=<sha256>` | `sync --check` 校验 token 合法性与平台完备性 | `raw -> validated -> pass/fail` | token 顺序按 canonical 平台固定输出 | 所有人可读，写入受策略约束 |
| sync strict 模式 | `legacy_tokens`, `keyed_tokens`, `current_platform` | 检测 legacy/mixed 时直接失败 | `checking -> rejected/accepted` | 仅 keyed-only 允许进入后续流程 | 本地默认只读，写入需 CI bot |
| identity source_hash 收敛 | 源码白名单、模块 lockfile、hash manifest token | 计算 `source_hash` 与 `identity_hash` | `collecting -> hashing -> emitted` | 输入路径排序稳定，忽略未跟踪文件 | 由构建脚本统一执行 |
| 多 runner 对账 | `runner`, `module_hashes`, `identity_hashes`, `module_set` | runner 导出摘要，汇总脚本执行差异比较 | `generated -> uploaded -> reconciled` | 按 `module_id` 全量对齐比较 | CI workflow 自动执行 |
| required check 保护 | check context 列表、strict 标记 | 自动注入/并集更新 `required_status_checks` | `planned -> applied -> verified` | 保留既有上下文并去重 | 需仓库写权限 |
- Acceptance Criteria:
  - AC-1: `m4_builtin_modules.sha256` 与 `m5_builtin_modules.sha256` 全量迁移到 keyed token 且不含 legacy 单 token。
  - AC-2: sync 脚本在 check/sync 两种模式均拒绝 legacy 或 mixed token 输入。
  - AC-3: 本地执行不带 CI 写入授权时，sync 脚本拒绝写入并提供修复提示。
  - AC-4: 新增 `m4/m5` 多 runner workflow 与摘要对账脚本，PR 触发可稳定运行。
  - AC-5: required checks 自动化默认上下文覆盖 `m1/m4/m5` 汇总校验。
  - AC-6: identity `source_hash` 移除 workspace 根 `Cargo.lock` 依赖，并仅使用可追踪稳定输入。
- Non-Goals:
  - 不变更 runtime 对 builtin wasm manifest 的消费协议。
  - 不扩展 canonical 平台集合（仍为 `darwin-arm64,linux-x86_64`）。
  - 不调整业务模块的 wasm 功能逻辑。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为构建与 CI 治理）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 通过“manifest 格式收敛 + sync 严格策略 + identity 输入白名单 + 多 runner 对账 + required check 治理”形成闭环，消除 m4/m5 hash 漂移的主要制度性来源。
- Integration Points:
  - `scripts/sync-m1-builtin-wasm-artifacts.sh`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh`
  - `scripts/sync-m5-builtin-wasm-artifacts.sh`
  - `scripts/ci-m1-wasm-summary.sh`
  - `scripts/ci-verify-m1-wasm-summaries.py`
  - `scripts/ci-ensure-required-checks.py`
  - `crates/agent_world_distfs/src/bin/sync_builtin_wasm_identity.rs`
  - `.github/workflows/builtin-wasm-m1-multi-runner.yml`
  - `.github/workflows/builtin-wasm-m4-m5-multi-runner.yml`
  - `crates/agent_world/src/runtime/world/artifacts/m4_builtin_modules.sha256`
  - `crates/agent_world/src/runtime/world/artifacts/m5_builtin_modules.sha256`
- Edge Cases & Error Handling:
  - 当前平台不在 canonical 列表：`--check` 直接失败并提示 `AGENT_WORLD_WASM_CANONICAL_PLATFORMS`。
  - manifest 含重复平台 token：严格失败并报告 `module_id + platform`。
  - runner 缺摘要或摘要重复：汇总脚本失败并列出缺失/重复 runner。
  - git 不可用或白名单收集失败：identity 生成失败并输出回退建议。
  - required check 注入时分支未保护：脚本应创建最小保护策略后继续注入。
- Non-Functional Requirements:
  - NFR-WASMHARD-1: keyed manifest 与 identity 计算在同一 commit、同一 runner 上 100% 可复现。
  - NFR-WASMHARD-2: 多 runner 对账失败信息须包含 `runner/module_id/hash`，单次运行内可定位。
  - NFR-WASMHARD-3: 新增治理链路不改变 `scripts/ci-tests.sh required/full` 的职责边界。
  - NFR-WASMHARD-4: 本地默认路径无写权限时不会修改任何 tracked manifest 文件。
- Security & Privacy: 仅处理模块源码路径、hash token 与 CI metadata；不处理敏感业务数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (T1): 专题 PRD/项目文档落地，冻结 1-6 治理项口径。
  - v1.1 (T2/T3/T6/T7): 完成 manifest strict 化与 identity 输入收敛。
  - v2.0 (T4/T5/T8): 接入 m4/m5 多 runner required checks 与发布策略收口。
- Technical Risks:
  - 风险-1: m4/m5 在不同 runner 真实存在编译差异，首次接入 required check 会提高失败率。
  - 风险-2: 白名单策略若遗漏关键输入，可能导致 identity 误判不变。
  - 风险-3: 本地写入策略收紧后，团队需迁移到 CI bot 更新流程。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-CI-WASMHARD-001 | T1/T2/T3 | `test_tier_required` | `sync-m4/m5 --check` + keyed token schema 校验 | m4/m5 manifest 稳定性 |
| PRD-TESTING-CI-WASMHARD-002 | T1/T6/T7 | `test_tier_required` | identity manifest 对账 + source_hash 输入白名单验证 | identity hash 可复现性 |
| PRD-TESTING-CI-WASMHARD-003 | T1/T4/T5/T8 | `test_tier_required` | 多 runner workflow 对账 + required check 注入脚本 dry-run/实跑 | 发布门禁与策略治理 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-WASMHARD-001 | m4/m5 升级 keyed-only manifest | 继续保留 legacy 单 token | keyed-only 可显式表达平台维度并消除覆盖歧义。 |
| DEC-WASMHARD-002 | sync 脚本 strict 拒绝 legacy/mixed | 兼容 legacy 并继续写回 | 兼容写回会持续制造漂移来源。 |
| DEC-WASMHARD-003 | identity 输入改为源码白名单 + 模块 lockfile | 递归目录 + workspace 根 lockfile | 白名单输入更稳定且更贴近模块边界。 |
| DEC-WASMHARD-004 | local 默认只读，写入限定 CI bot | 本地允许直接写入 | 可以避免开发机环境差异污染发布基线。 |
