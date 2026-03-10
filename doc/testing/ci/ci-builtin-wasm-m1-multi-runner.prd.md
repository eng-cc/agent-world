# Agent World: CI 拆分 Builtin Wasm m1 多 Runner 校验

- 对应设计文档: `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.design.md`
- 对应项目管理文档: `doc/testing/ci/ci-builtin-wasm-m1-multi-runner.project.md`

审计轮次: 4


## 1. Executive Summary
- Problem Statement: builtin wasm 的 `m1` 校验原先耦合在主测试流中，难以独立观察多平台差异与定位构建不一致问题。
- Proposed Solution: 新增独立多 runner workflow，在 `ubuntu-latest` 和 `macos-14` 上并行执行 `m1 --check` 并统一摘要对账。
- Success Criteria:
  - SC-1: 独立 workflow 可在两类 runner 稳定执行 `m1` 校验。
  - SC-2: 每个 runner 产出统一 schema 摘要并可被汇总 job 对账。
  - SC-3: 汇总 job 能给出可定位的一致性失败信息。
  - SC-4: 不影响现有 required/full 主测试链路定义。

## 2. User Experience & Functionality
- User Personas:
  - CI 维护者：需要快速定位平台差异导致的 m1 校验问题。
  - 工程维护者：需要可审计的跨 runner 一致性证据。
  - 发布负责人：需要在控制时延的前提下提升链路可信度。
- User Scenarios & Frequency:
  - PR 校验：涉及 builtin wasm 链路改动时执行多 runner 对账。
  - 失败排查：汇总 job 失败后按摘要定位平台差异。
  - 周期性巡检：确认多平台构建结果持续一致。
- User Stories:
  - PRD-TESTING-CI-M1RUNNER-001: As a CI 维护者, I want m1 checks isolated in a dedicated workflow, so that failures are easier to triage.
  - PRD-TESTING-CI-M1RUNNER-002: As an 工程维护者, I want per-runner summaries with strict schema, so that cross-runner comparison is deterministic.
  - PRD-TESTING-CI-M1RUNNER-003: As a 发布负责人, I want m1-only scope in multi-runner checks, so that coverage increases without excessive CI latency.
- Critical User Flows:
  1. Flow-M1RUNNER-001: `触发 workflow -> 两个 runner 执行 m1 --check -> 导出摘要`
  2. Flow-M1RUNNER-002: `汇总 job 读取摘要 -> 执行一致性对账 -> 输出通过/失败详情`
  3. Flow-M1RUNNER-003: `对账失败 -> 按 runner/platform 字段定位差异 -> 修复后复跑`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 多 runner m1 校验 | runner 列表、校验命令 | 每个 runner 执行 `sync-m1 --check` | `queued -> running -> passed/failed` | 仅 m1，避免扩大粒度 | CI 自动执行 |
| runner 摘要导出 | `runner/current_platform/module_hashes/identity_hashes` | 生成 `output/ci/m1-wasm-summary/<runner>.json` | `generated -> uploaded -> consumed` | schema 固定，字段不可缺失 | 脚本统一生成 |
| 跨 runner 对账 | 摘要输入、差异报告 | 运行 `ci-verify-m1-wasm-summaries.py` | `collecting -> comparing -> matched/mismatch` | 按 module_id 对齐比较 | 汇总 job 负责裁决 |
- Acceptance Criteria:
  - AC-1: 新增 `.github/workflows/builtin-wasm-m1-multi-runner.yml`。
  - AC-2: `ubuntu-latest` 与 `macos-14` 均执行 `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`。
  - AC-3: 摘要导出与汇总对账脚本可稳定运行。
  - AC-4: 保持 m1-only 范围，不扩展到 m4/m5。
- Non-Goals:
  - 不新增 m4/m5 多 runner 校验矩阵。
  - 不替换 required/full 主测试流程。
  - 不引入容器化构建链路。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本任务为 CI workflow 与脚本治理）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 通过“runner 执行 + 摘要导出 + 汇总对账”三段式，把 m1 多平台一致性校验从主链路中解耦为独立可观测流程。
- Integration Points:
  - `.github/workflows/builtin-wasm-m1-multi-runner.yml`
  - `scripts/ci-m1-wasm-summary.sh`
  - `scripts/ci-verify-m1-wasm-summaries.py`
  - `scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `output/ci/m1-wasm-summary/<runner>.json`
- Edge Cases & Error Handling:
  - runner 环境抖动：依赖固定 toolchain 与 deterministic guard，失败时保留摘要与日志。
  - 摘要 schema 漂移：脚本做严格字段校验并在缺字段时直接失败。
  - 对账误报：校验器按固定字段与 module_id 比对，输出差异详情。
  - CI 等待时间上升：限制为 m1-only 以控制时延。
- Non-Functional Requirements:
  - NFR-M1RUNNER-1: 多 runner workflow 对主门禁新增时延可控。
  - NFR-M1RUNNER-2: 摘要对账结果可复现且可审计。
  - NFR-M1RUNNER-3: 跨平台一致性问题可在单次运行中定位到 runner 维度。
- Security & Privacy: 仅处理构建产物哈希与平台元信息，不涉及敏感业务数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (M1): 设计文档与项目管理文档落地。
  - v1.1 (M2): 摘要与对账脚本落地。
  - v2.0 (M3/M4): workflow 接入、手册同步与状态收口。
- Technical Risks:
  - 风险-1: macOS 与本地环境差异导致偶发构建抖动。
  - 风险-2: 多 runner 增加 CI 排队与执行时间。
  - 风险-3: 摘要字段不稳定导致误报。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-CI-M1RUNNER-001 | T1/T3 | `test_tier_required` | workflow 触发与 job 执行验证 | CI m1 校验可观测性 |
| PRD-TESTING-CI-M1RUNNER-002 | T2/T3 | `test_tier_required` | 摘要 schema 校验与对账脚本验证 | 多 runner 一致性诊断 |
| PRD-TESTING-CI-M1RUNNER-003 | T3/T4 | `test_tier_required` | m1-only 范围与时延评估 | CI 成本与覆盖平衡 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-M1RUNNER-001 | m1 校验独立 workflow + 多 runner | 继续耦合主测试流 | 独立后更易观测和定位。 |
| DEC-M1RUNNER-002 | 采用摘要对账做跨 runner 比较 | 仅看日志人工比对 | 自动对账更稳定且可审计。 |
| DEC-M1RUNNER-003 | 仅覆盖 m1 | 同步扩到 m4/m5 | 控制执行成本并先验证链路。 |

## 原文约束点映射（内容保真）
- 原“目标（独立 workflow、多 runner、m1-only）” -> 第 1 章与第 2 章 AC。
- 原“In/Out of Scope” -> 第 2 章 AC 与 Non-Goals。
- 原“接口/数据（workflow、脚本、摘要字段）” -> 第 4 章 Integration Points 与规格矩阵。
- 原“里程碑 M1~M4” -> 第 5 章 Phased Rollout。
- 原“风险（环境差异、时延、schema 漂移）” -> 第 4 章 Edge Cases + 第 5 章 Risks。
