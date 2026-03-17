# Agent World: Builtin Wasm Determinism Gate（m1）

- 对应设计文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-m1.design.md`
- 对应项目管理文档: `doc/testing/ci/ci-builtin-wasm-determinism-gate-m1.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: `m1` 曾经使用独立 multi-runner workflow 观察宿主差异，但 builtin wasm 发布级构建已经切到 Docker canonical build；如果继续把旧 workflow 当成现行口径，会把已下线的 host-native hash 对账重新带回 testing 文档。
- Proposed Solution: 保留该专题文件名用于追溯，但把现行执行口径统一到 `.github/workflows/wasm-determinism-gate.yml`。GitHub-hosted 默认只收集 `linux-x86_64` canonical summary 与 release evidence；若需要 full-tier 跨宿主验证，再额外导入 Docker-capable macOS summary 做离线对账。
- Success Criteria:
  - SC-1: `m1` 的独立 gate 入口统一收敛到 `wasm-determinism-gate`，不再依赖 `.github/workflows/builtin-wasm-m1-multi-runner.yml`。
  - SC-2: GitHub-hosted 默认稳定产出 `linux-x86_64` canonical summary 与 release evidence report。
  - SC-3: 跨宿主 full-tier 比较对象是同一 Docker builder 的 canonical 输出，而不是 host-native cargo 输出。
  - SC-4: 不影响现有 required/full 主测试链路定义。

## 2. User Experience & Functionality
- User Personas:
  - CI 维护者：需要一个与当前 workflow 一致的 `m1` gate 说明。
  - 工程维护者：需要可审计的 canonical summary / receipt evidence。
  - 发布负责人：需要区分“GitHub-hosted 默认 gate”和“外部跨宿主补充证据”。
- User Scenarios & Frequency:
  - PR 校验：涉及 builtin wasm 链路改动时执行 `m1` canonical summary gate。
  - 失败排查：verify job 失败后按 summary / receipt evidence 定位差异。
  - 发布候选：需要跨宿主证据时，导入外部 Docker-capable macOS summary 做补充对账。
- User Stories:
  - PRD-TESTING-CI-M1RUNNER-001: As a CI 维护者, I want the m1 gate documented against the current workflow, so that triage follows the real entrypoint.
  - PRD-TESTING-CI-M1RUNNER-002: As an 工程维护者, I want m1 summaries to carry canonical hash and receipt evidence, so that cross-runner comparison is deterministic.
  - PRD-TESTING-CI-M1RUNNER-003: As a 发布负责人, I want GitHub-hosted Linux gate and optional cross-host evidence clearly separated, so that CI cost stays controlled.
- Critical User Flows:
  1. Flow-M1RUNNER-001: `触发 wasm-determinism-gate -> GitHub-hosted Linux 收集 m1 canonical summary -> 生成 release evidence report`
  2. Flow-M1RUNNER-002: `如需 full-tier -> 导入外部 Docker-capable macOS summary -> 以相同 schema 离线对账`
  3. Flow-M1RUNNER-003: `对账失败 -> 按 runner/canonical_platform/receipt evidence 定位差异 -> 修复后复跑`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| m1 gate 执行 | `module_set=m1`、`runner_label=linux-x86_64` | GitHub-hosted Linux 收集 summary | `queued -> collecting -> uploaded` | 默认只校验 canonical Docker 输出 | CI 自动执行 |
| summary/evidence 导出 | `runner/current_platform/canonical_platform/module_hashes/receipt_evidence` | 生成 summary 与 release evidence 报告 | `generated -> uploaded -> consumed` | schema 固定，字段不可缺失 | 脚本统一生成 |
| 离线跨宿主对账 | 导入 summary 集合、差异报告 | 运行 `wasm-release-evidence-report.sh` / `ci-verify-m1-wasm-summaries.py` | `collecting -> comparing -> matched/mismatch` | 比较 canonical Docker 输出，不比较 host-native 输出 | QA / verify job 裁决 |
- Acceptance Criteria:
  - AC-1: 当前现行 workflow 为 `.github/workflows/wasm-determinism-gate.yml`，`m1` 通过 `module_set` 维度进入独立 verify job。
  - AC-2: GitHub-hosted 默认执行 `./scripts/ci-m1-wasm-summary.sh --module-set m1`，并以 `./scripts/wasm-release-evidence-report.sh --expected-runners linux-x86_64` 汇总。
  - AC-3: 若补入外部 Docker-capable macOS summary，现有摘要 schema 与对账脚本可稳定运行。
  - AC-4: 不恢复 `.github/workflows/builtin-wasm-m1-multi-runner.yml`。
- Non-Goals:
  - 不恢复 host-native m1 multi-runner workflow。
  - 不替换 required/full 主测试流程。
  - 不把 host-native cargo 输出重新定义为发布级 hash 来源。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本任务为 CI workflow 与证据治理）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 通过“Linux canonical summary 收集 + release evidence 汇总 + 可选离线跨宿主导入”三段式，把 `m1` 的独立确定性 gate 收敛到 Docker canonical 输出口径。
- Integration Points:
  - `.github/workflows/wasm-determinism-gate.yml`
  - `scripts/ci-m1-wasm-summary.sh`
  - `scripts/ci-verify-m1-wasm-summaries.py`
  - `scripts/wasm-release-evidence-report.sh`
  - `output/ci/m1-wasm-summary/<runner>.json`
- Edge Cases & Error Handling:
  - GitHub-hosted macOS 无 Docker daemon：默认 gate 不依赖它；需要 full-tier 时改为外部 summary 导入。
  - 摘要 schema 漂移：脚本做严格字段校验并在缺字段时直接失败。
  - 对账误报：校验器按 canonical platform、receipt evidence 与 `module_id` 比对，输出差异详情。
  - CI 等待时间上升：GitHub-hosted 默认只跑 Linux canonical gate 控制时延。
- Non-Functional Requirements:
  - NFR-M1RUNNER-1: `m1` gate 对主门禁新增时延可控。
  - NFR-M1RUNNER-2: summary / evidence 结果可复现且可审计。
  - NFR-M1RUNNER-3: 跨宿主问题可在单次运行中定位到 runner 与 canonical evidence 维度。
- Security & Privacy: 仅处理构建产物哈希、receipt evidence 与平台元信息，不涉及敏感业务数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (M1): 历史专题收口到现行 workflow。
  - v1.1 (M2): summary 与 evidence 报告脚本稳定运行。
  - v2.0 (M3): 外部 Docker-capable macOS summary 作为 full-tier 补充证据纳入流程。
- Technical Risks:
  - 风险-1: 若外部 macOS summary 生成环境不受控，会影响 full-tier 证据可信度。
  - 风险-2: summary schema 继续演化时，旧专题若不回写会再次产生文档漂移。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-CI-M1RUNNER-001 | T1/T3 | `test_tier_required` | workflow 触发与 Linux canonical summary 验证 | CI m1 gate 可观测性 |
| PRD-TESTING-CI-M1RUNNER-002 | T2/T3 | `test_tier_required` | summary schema / receipt evidence 校验 | m1 canonical 对账诊断 |
| PRD-TESTING-CI-M1RUNNER-003 | T3/T4 | `test_tier_required` + `test_tier_full` | GitHub-hosted Linux gate + 外部跨宿主补充证据验证 | CI 成本与覆盖平衡 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-M1RUNNER-001 | 保留历史文件名，但现行口径统一到 `wasm-determinism-gate` | 继续把旧 workflow 当成现行入口 | 现行代码和 workflow 已切换，文档必须跟上。 |
| DEC-M1RUNNER-002 | 默认只跑 GitHub-hosted Linux canonical gate | 恢复 GitHub-hosted macOS 对账 | 当前 GitHub-hosted macOS 无 Docker daemon。 |
| DEC-M1RUNNER-003 | 外部 macOS summary 只作为 full-tier 补充证据 | 把外部 summary 设为默认 required gate | 默认门禁要可稳定执行。 |

## 原文约束点映射（内容保真）
- 原“m1 独立 gate 与摘要对账” -> 保留为独立专题，但执行入口切换到 `wasm-determinism-gate`。
- 原“多 runner 观察差异” -> 调整为“默认 Linux canonical gate + 可选外部跨宿主补证”。
- 原“脚本与工作流入口” -> 收敛到 `ci-m1-wasm-summary.sh` / `ci-verify-m1-wasm-summaries.py` / `wasm-release-evidence-report.sh`。
