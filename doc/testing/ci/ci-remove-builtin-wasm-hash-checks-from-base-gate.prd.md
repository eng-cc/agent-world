# Agent World: 基础 CI 门禁移除 Builtin Wasm Hash 校验

- 对应项目管理文档: doc/testing/ci/ci-remove-builtin-wasm-hash-checks-from-base-gate.prd.project.md

## 1. Executive Summary
- Problem Statement: 基础门禁若同时承担格式/测试与 builtin wasm hash 校验，容易拉高日常反馈成本并与分层测试职责耦合。
- Proposed Solution: 从 `scripts/ci-tests.sh` 基础门禁移除 `m1/m4/m5 --check`，将职责收敛到格式与测试链路，并在手册明确覆盖边界。
- Success Criteria:
  - SC-1: `ci-tests.sh` 基础路径不再执行 m1/m4/m5 hash 校验命令。
  - SC-2: required/full 其余测试行为保持不变。
  - SC-3: `testing-manual.md` 覆盖边界与实际执行一致。
  - SC-4: 改造后的门禁行为可在项目文档与日志中追溯。

## 2. User Experience & Functionality
- User Personas:
  - 开发者：希望基础门禁更快反馈核心编译/测试问题。
  - CI 维护者：希望门禁职责边界清晰。
  - 发布负责人：希望知道 hash 校验覆盖盲区与补偿路径。
- User Scenarios & Frequency:
  - 日常提交：执行 required，不再包含 builtin wasm hash check。
  - CI 维护：核对脚本与手册口径一致性。
  - 发布前审查：确认 hash 校验是否由其他链路覆盖。
- User Stories:
  - PRD-TESTING-CI-HASHBASE-001: As a 开发者, I want base gate to focus on fast feedback, so that iteration is not blocked by heavy artifact checks.
  - PRD-TESTING-CI-HASHBASE-002: As a CI 维护者, I want explicit scope boundaries for base gates, so that test responsibilities stay clear.
  - PRD-TESTING-CI-HASHBASE-003: As a 发布负责人, I want documented coverage gaps after removal, so that risk evaluation remains accurate.
- Critical User Flows:
  1. Flow-HASHBASE-001: `执行 ci-tests required -> 跳过 m1/m4/m5 hash -> 完成基础门禁`
  2. Flow-HASHBASE-002: `同步 testing-manual 覆盖说明 -> 校验脚本行为一致`
  3. Flow-HASHBASE-003: `评估发布风险 -> 确认 hash 校验补偿链路 -> 决定放行`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 基础门禁收敛 | required/full 命令、移除命令清单 | 修改 `scripts/ci-tests.sh` 并保留其余路径 | `draft -> applied -> verified` | 先保证 required 快速路径 | CI 维护者维护 |
| 覆盖口径同步 | 手册条目、覆盖/缺口说明 | 更新 `testing-manual.md` | `unsynced -> synced` | 以脚本行为为真值源 | 文档维护者更新 |
| 风险提示 | m4/m5 漂移盲区、发现时点变化 | 在文档中明确风险与应对 | `identified -> documented` | 高风险优先标注 | 发布负责人审阅 |
- Acceptance Criteria:
  - AC-1: `scripts/ci-tests.sh` 移除 m1/m4/m5 hash 校验命令。
  - AC-2: required/full 其他路径不变化。
  - AC-3: `testing-manual.md` 完成覆盖边界同步。
  - AC-4: 变更验证与收口记录完整。
- Non-Goals:
  - 不新增 m4/m5 独立 workflow。
  - 不变更 `builtin-wasm-m1-multi-runner` workflow 行为。
  - 不修改 `sync-m*-builtin-wasm-artifacts.sh` 脚本实现。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本任务为 CI 门禁职责调整）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 将基础门禁定位为“快反馈层”，移除较重的 wasm hash 校验，降低日常执行成本并强调分层职责。
- Integration Points:
  - `scripts/ci-tests.sh`
  - `testing-manual.md`
  - `.github/workflows/rust.yml`
  - `scripts/sync-m1-builtin-wasm-artifacts.sh`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh`
  - `scripts/sync-m5-builtin-wasm-artifacts.sh`
- Edge Cases & Error Handling:
  - hash 漂移发现时点后移：手册明确盲区并要求在其他链路补偿。
  - 团队习惯未更新：继续按旧预期依赖基础门禁时，需在文档与评审中提醒。
  - 脚本/文档不同步：以脚本行为为准并及时回写手册。
- Non-Functional Requirements:
  - NFR-HASHBASE-1: 基础门禁执行时间下降且核心回归覆盖不退化。
  - NFR-HASHBASE-2: 覆盖缺口有明确文档化说明。
  - NFR-HASHBASE-3: 变更行为可审计且可复现。
- Security & Privacy: 本任务不引入新数据处理，仅调整 CI 校验职责。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (M1): 设计文档与项目管理文档落地。
  - v1.1 (M2): 基础门禁脚本移除 hash 校验。
  - v2.0 (M3/M4): 测试手册同步、验证与收口。
- Technical Risks:
  - 风险-1: `m4/m5` hash 漂移无法在基础门禁即时拦截。
  - 风险-2: 团队发现问题时点后移。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-CI-HASHBASE-001 | T1/T2 | `test_tier_required` | `ci-tests.sh` 基础路径命令校验 | 基础门禁执行效率 |
| PRD-TESTING-CI-HASHBASE-002 | T2/T3 | `test_tier_required` | 脚本与手册口径一致性检查 | 测试策略治理一致性 |
| PRD-TESTING-CI-HASHBASE-003 | T3/T4 | `test_tier_required` | 覆盖缺口与风险说明审阅 | 发布风险评估准确性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-HASHBASE-001 | 基础门禁移除 builtin wasm hash 校验 | 保持原状 | 更符合分层门禁职责与反馈效率目标。 |
| DEC-HASHBASE-002 | 手册同步披露覆盖盲区 | 仅改脚本不改文档 | 避免执行口径与文档口径脱节。 |
| DEC-HASHBASE-003 | 保持其他 required/full 测试路径不变 | 同批次大规模改造 | 控制改动面降低回归风险。 |

## 原文约束点映射（内容保真）
- 原“目标（移除 hash 校验、收敛职责、同步手册）” -> 第 1 章与第 2 章 AC。
- 原“In/Out of Scope” -> 第 2 章 AC 与 Non-Goals。
- 原“接口/数据（脚本/手册/入口）” -> 第 4 章 Integration Points。
- 原“里程碑 M1~M4” -> 第 5 章 Phased Rollout。
- 原“风险（盲区、发现后移）” -> 第 4 章 Edge Cases + 第 5 章 Risks。
