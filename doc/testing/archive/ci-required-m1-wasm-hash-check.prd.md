# Agent World：Required Tier 接入 M1 Builtin Wasm Hash 校验（归档专题）

> 归档说明（2026-02-20）：该方案已被 `doc/p2p/builtin-wasm-identity-consensus.md` 取代，不再作为现行实现依据。

## 1. Executive Summary
- Problem Statement: 在旧流程中，m1 builtin wasm hash 漂移可能在 PR 合入后才暴露，导致 required 门禁对关键清单一致性的保护不足。
- Proposed Solution: 在 required 前置检查接入 `sync-m1 --check`，并升级 manifest/loader/hydration 为多平台多候选 hash 匹配机制，保证本地与 CI 一致校验。
- Success Criteria:
  - SC-1: `scripts/ci-tests.sh required/full` 均执行 `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`。
  - SC-2: m1 manifest 支持 `module_id <platform>=<hash> ...` 结构。
  - SC-3: DistFS hydration 与 runtime materializer 能匹配候选 hash 列表。
  - SC-4: Linux/macOS hash 可共存并通过 required 回归。
  - SC-5: 本归档专题迁移为 `.prd.md/.prd.project.md` 并保持语义可追溯。

## 2. User Experience & Functionality
- User Personas:
  - CI/测试维护者：需要在 required 阶段尽早发现 m1 清单漂移。
  - Runtime 维护者：需要 loader/hydrator 在多平台 hash 下稳定工作。
  - 开发者：需要在本地快速复现 CI 对 m1 清单的检查规则。
- User Scenarios & Frequency:
  - 每次 required/full 执行时触发 m1 hash 校验。
  - 清单更新或跨平台构建差异出现时更新 manifest。
  - DistFS hydration/runtimeloader 回归时验证候选 hash 匹配。
- User Stories:
  - PRD-TESTING-ARCHIVE-M1-001: As a 测试维护者, I want required tier to gate m1 hash deterministically, so that drift is blocked before merge.
  - PRD-TESTING-ARCHIVE-M1-002: As a runtime maintainer, I want multi-hash compatibility in hydration/materializer, so that cross-platform builds remain valid.
  - PRD-TESTING-ARCHIVE-M1-003: As a developer, I want local check parity with CI, so that failures are reproducible.
- Critical User Flows:
  1. Flow-ARCH-M1-001: `执行 required/full -> 触发 sync-m1 --check -> 判定通过/失败`
  2. Flow-ARCH-M1-002: `构建产物 hash 与 manifest 比对 -> 命中任一候选 hash 即通过`
  3. Flow-ARCH-M1-003: `DistFS hydration 入库 built hash -> 校验在 manifest 允许列表`
  4. Flow-ARCH-M1-004: `runtime materializer DistFS/fetch/compile fallback -> 匹配候选 hash 列表`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| required 前置校验 | `sync-m1 --check`、tier=`required/full` | 在入口脚本前置执行 | `queued -> checked -> pass/fail` | required 与 full 共用同一前置规则 | CI/本地执行者可触发 |
| manifest 结构升级 | `module_id <platform>=<hash> ...` | 支持多平台 hash 声明 | `legacy -> multi-hash` | 命中任一候选 hash 即通过 | 维护者更新清单 |
| DistFS hydration 校验 | built hash、manifest allowlist | 入库并校验合法性 | `built -> hydrated -> verified` | built bytes 实际 hash 为准 | DistFS/测试维护者 |
| runtime materializer 匹配 | 候选 hash 列表、fallback 路径 | DistFS/fetch/compile 回退匹配 | `resolved -> loaded` | 任一候选命中即可 | runtime 维护者 |
- Acceptance Criteria:
  - AC-1: required/full 都能触发 m1 hash 校验。
  - AC-2: manifest 完成多平台 hash 结构迁移。
  - AC-3: hydration/materializer 兼容候选 hash。
  - AC-4: required 回归通过并覆盖 Linux/macOS hash 共存场景。
  - AC-5: 归档专题迁移后仍保留“被新方案替代”的上下文。
- Non-Goals:
  - 不在本专题接入 m4 hash 校验。
  - 不调整 GitHub Actions job 拓扑。
  - 不修改 builtin wasm 构建参数与 hash 算法。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为 CI/脚本/runtime 规则收口，不涉及 AI 推理能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 以 `scripts/ci-tests.sh` 为统一入口，在 required/full 触发 `sync-m1 --check`；manifest 升级为多平台 hash 格式，并同步改造 hydration 与 runtime materializer 候选 hash 匹配链路。
- Integration Points:
  - `scripts/ci-tests.sh`
  - `scripts/sync-m1-builtin-wasm-artifacts.sh`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt`
  - `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
  - `crates/agent_world_distfs/src/bin/hydrate_builtin_wasm.rs`
  - `crates/agent_world/src/runtime/builtin_wasm_materializer.rs`
- Edge Cases & Error Handling:
  - 首次环境缺 nightly/build-std：按脚本提示补齐后重试。
  - 本地 hash 与清单不一致：提交被阻断，需先执行 sync 更新清单。
  - 跨平台 hash 差异：通过 manifest 多候选 hash 保持兼容。
  - 新平台接入：先定义平台键并补齐 canonical hash，再放开校验。
- Non-Functional Requirements:
  - NFR-ARCH-M1-1: required 门禁新增校验后仍需保持可接受执行时间。
  - NFR-ARCH-M1-2: 本地与 CI 校验命令完全一致。
  - NFR-ARCH-M1-3: hash 校验结果必须可复现且可审计。
- Security & Privacy: hash 校验仅面向构建产物完整性，不引入新敏感数据处理流程。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (ARCH-M1-1): 建立设计/项目文档与 required 接入计划。
  - v1.1 (ARCH-M1-2): required/full 前置接入 `sync-m1 --check`。
  - v2.0 (ARCH-M1-3): manifest 多 hash 结构与 runtime/hydration 兼容改造落地。
  - v2.1 (ARCH-M1-4): Linux/macOS hash 共存回归通过并归档。
  - v2.2 (ARCH-M1-5): 本专题 strict schema 迁移与命名统一。
- Technical Risks:
  - 风险-1: required 执行时间因新增构建校验上升。
  - 风险-2: 开发机环境差异导致本地失败率上升。
  - 风险-3: 仅覆盖 m1，m4 仍存在晚发现风险。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-ARCHIVE-M1-001 | ARCH-M1-1/2/5 | `test_tier_required` | required/full 入口校验 + 文档治理检查 | CI 前置门禁一致性 |
| PRD-TESTING-ARCHIVE-M1-002 | ARCH-M1-2/3/4 | `test_tier_required` | manifest/hydration/materializer 候选 hash 匹配验证 | m1 产物完整性与兼容性 |
| PRD-TESTING-ARCHIVE-M1-003 | ARCH-M1-3/4/5 | `test_tier_required` | 本地与 CI 命令口径一致性回归 | 开发与 CI 可复现性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-ARCH-M1-001 | required/full 共用 `sync-m1 --check` 前置 | 仅 CI 校验 | 本地可提前发现漂移。 |
| DEC-ARCH-M1-002 | manifest 支持多平台候选 hash | 单 hash 强绑定 | 适配 Linux/macOS 差异。 |
| DEC-ARCH-M1-003 | hydration/materializer 按候选 hash 匹配 | 仅匹配单一 hash | 避免跨平台构建误判失败。 |
| DEC-ARCH-M1-004 | 归档后保留专题语义与替代关系 | 直接删除旧文档 | 保持历史决策链可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标：required 接入 m1 hash 校验、保持分级结构、兼容多平台 hash” -> 第 1 章 Summary 与第 2 章规格矩阵。
- 原“In/Out Scope（仅 m1、不中改 job 拓扑/算法）” -> 第 2 章 Non-Goals。
- 原“接口数据（入口脚本、manifest 格式、依赖文件）” -> 第 4 章 Integration Points。
- 原“M1~M4 里程碑” -> 第 5 章 phased rollout（ARCH-M1-1~5）。
- 原“风险（耗时、环境依赖、晚发现）” -> 第 4 章边界处理 + 第 5 章风险。
