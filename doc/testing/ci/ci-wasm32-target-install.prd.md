# Agent World: CI 安装 wasm32-unknown-unknown target

审计轮次: 3

- 对应项目管理文档: doc/testing/ci/ci-wasm32-target-install.prd.project.md

## 1. Executive Summary
- Problem Statement: CI 在执行 builtin wasm 工件校验时依赖 `wasm32-unknown-unknown` target，若 runner 未预装会导致 required/full 门禁随机失败。
- Proposed Solution: 在 GitHub Actions workflow 的关键 job 中显式安装 `wasm32-unknown-unknown` target，消除环境隐式依赖。
- Success Criteria:
  - SC-1: `required-gate` 与 `full-regression` 均在执行测试前完成 wasm target 安装。
  - SC-2: `scripts/ci-tests.sh required/full` 在 Actions 环境下可稳定复现。
  - SC-3: builtin wasm 校验链路不再因缺 target 失败。
  - SC-4: 任务状态、依赖与验证结果在 project/devlog 可追溯。

## 2. User Experience & Functionality
- User Personas:
  - CI 维护者：需要稳定、可复现的门禁环境。
  - 开发者：需要避免因 runner 环境差异导致的非代码失败。
  - 发布负责人：需要可信赖的 CI 结果作为放行依据。
- User Scenarios & Frequency:
  - PR 门禁执行：每次 PR 触发 required/full 流程。
  - CI 故障排查：门禁失败后优先核查环境依赖是否显式声明。
  - 发布前回归：每个发布候选至少执行一轮完整回归。
- User Stories:
  - PRD-TESTING-CI-WASM-001: As a CI 维护者, I want wasm target installation to be explicit in workflows, so that required/full gates are deterministic.
  - PRD-TESTING-CI-WASM-002: As a 开发者, I want CI failures to reflect code issues rather than missing toolchain setup, so that diagnosis stays efficient.
  - PRD-TESTING-CI-WASM-003: As a 发布负责人, I want reproducible gate evidence, so that release decisions are defensible.
- Critical User Flows:
  1. Flow-WASMCI-001: `触发 workflow -> 安装 wasm target -> 执行 required/full -> 输出门禁结论`
  2. Flow-WASMCI-002: `CI 失败 -> 检查 target 安装日志 -> 定位脚本/环境/代码问题 -> 修复并重跑`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| CI target 安装步骤 | target 名称、执行命令、job 位置 | 在 job 中执行 `rustup target add wasm32-unknown-unknown` | `pending -> installed -> verified` | 先安装再执行测试 | workflow 维护者可修改，开发者可查看日志 |
| required/full 门禁执行 | `required` / `full` 命令、校验脚本 | 调用 `scripts/ci-tests.sh` 并输出日志 | `queued -> running -> passed/failed` | required 先于 full | CI 平台自动执行 |
| 故障诊断证据 | 安装日志、测试日志、失败签名 | 汇总后用于排障与复盘 | `captured -> analyzed -> archived` | 按失败链路优先级处理 | 维护者与发布负责人可审阅 |
- Acceptance Criteria:
  - AC-1: `.github/workflows/rust.yml` 的 `required-gate/full-regression` 均显式安装 wasm target。
  - AC-2: 保持现有 CI 分级策略与测试命令不变。
  - AC-3: 最小回归验证可证明目标问题已消除。
  - AC-4: 文档与任务状态同步回写且可追溯。
- Non-Goals:
  - 不调整 CI 缓存、矩阵并行与 runner 类型。
  - 不改造 builtin wasm 构建脚本逻辑。
  - 不引入其他 target（如 `wasm32-wasip1`）治理变更。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本任务为 CI 环境依赖显式化）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 通过 workflow 显式声明 Rust target 依赖，将环境差异前置到安装步骤并纳入门禁日志。
- Integration Points:
  - `.github/workflows/rust.yml`
  - `scripts/ci-tests.sh`
  - `scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh --check`
- Edge Cases & Error Handling:
  - 网络波动导致 `rustup target add` 失败：保留失败日志并重试 job。
  - toolchain 镜像异常：在日志中标记为环境故障，避免误判代码回归。
  - 后续引入新 target：若未显式安装则需在新增任务中补齐。
  - 本地与 CI 不一致：以 CI 安装与执行日志为准进行诊断。
- Non-Functional Requirements:
  - NFR-WASMCI-1: required/full 两条门禁在同一 workflow 下保持可复现。
  - NFR-WASMCI-2: 目标安装步骤失败时具备可诊断日志。
  - NFR-WASMCI-3: 变更后不增加业务脚本复杂度。
- Security & Privacy: 仅涉及公开构建依赖安装，不处理敏感数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (WASMCI-1): 完成设计文档与项目管理文档。
  - v1.1 (WASMCI-2): 在 workflow 的关键 job 中落地 target 安装。
  - v2.0 (WASMCI-3): 完成最小回归验证与文档收口。
- Technical Risks:
  - 风险-1: GitHub Actions 网络抖动导致 target 安装偶发失败。
  - 风险-2: 后续新增 target 仍可能复现同类问题，需保持显式依赖策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-CI-WASM-001 | WASMCI-1/2 | `test_tier_required` | workflow 步骤检查与命令验证 | CI 环境稳定性 |
| PRD-TESTING-CI-WASM-002 | WASMCI-2/3 | `test_tier_required` | required/full 最小回归 | builtin wasm 校验链路 |
| PRD-TESTING-CI-WASM-003 | WASMCI-3 | `test_tier_required` | project/devlog 追溯核验 | 发布门禁可信度 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-WASMCI-001 | 在 workflow 显式安装 wasm target | 依赖 runner 预装环境 | 显式依赖更稳定、可复现。 |
| DEC-WASMCI-002 | 保持 required/full 命令不变 | 同时重构 CI 分级策略 | 控制变更范围，降低回归风险。 |
| DEC-WASMCI-003 | 以最小回归验证收口 | 仅看配置变更静态检查 | 运行验证能直接证明问题消除。 |

## 原文约束点映射（内容保真）
- 原“目标（修复缺 target 导致失败）” -> 第 1 章 Problem/Success。
- 原“In/Out of Scope” -> 第 2 章 AC 与 Non-Goals。
- 原“接口/数据（workflow/命令/脚本）” -> 第 4 章 Integration Points。
- 原“里程碑 WASMCI-1/2/3” -> 第 5 章 Phased Rollout。
- 原“风险（网络波动/后续 target）” -> 第 4 章 Edge Cases + 第 5 章 Risks。
