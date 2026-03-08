# world_viewer_live LLM 默认开启（P2P 发行基线）设计文档（2026-02-23）

审计轮次: 5
> 状态更新（2026-03-08）:
> - `world_viewer_live` 已下线 `--release-config`/`--node-*` 控制面入口。
> - 本文仍适用于 `--llm/--no-llm` 行为语义，但不再包含 release-config 运行路径要求。

## 1. Executive Summary
- Problem Statement: 将 `world_viewer_live` 的决策模式默认值从 Script 调整为 LLM，减少发行参数遗漏导致的节点行为偏差。
- Proposed Solution: 在不破坏现有参数模型的前提下，保持 CLI 语义简单且可审计。
- Success Criteria:
  - SC-1: 未显式写入 `--llm` 时，默认进入 LLM 决策。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want world_viewer_live LLM 默认开启（P2P 发行基线）设计文档（2026-02-23） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - AC-2: 调整 `CliOptions::default().llm_mode` 为 `true`。
  - AC-3: 更新 `--llm` 帮助文案，避免“默认关闭”误导。
  - AC-4: `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_tests_split_part1.rs`
  - AC-5: 更新默认解析测试断言，覆盖“默认开启 LLM”。
  - AC-6: 文档
- Non-Goals:
  - 不新增 `--no-llm` 反向开关。
  - 不调整 `WorldScenario` 默认值与场景集合。
  - 不扩展链路控制面参数（由 `world_chain_runtime` 承担）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.prd.md`
  - `doc/p2p/viewer-live/world-viewer-live-llm-default-on-2026-02-23.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) CLI 默认语义
- 变更前：不传 `--llm` 时 `llm_mode=false`（Script）。
- 变更后：不传 `--llm` 时 `llm_mode=true`（LLM）。

### 2) CLI 参数兼容性
- `--llm` 参数继续保留（显式冗余开启，不影响结果）。
- `world_viewer_live` 不再支持 `--release-config`，不再维护 `locked_args` 兼容路径。

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：完成（设计/项目文档建档）。
  - M1：完成（`llm_mode` 默认值切换 + CLI 帮助文案更新）。
  - M2：完成（参数解析测试与手册更新，定向回归通过）。
  - M3：完成（文档状态与 devlog 收口）。
- Technical Risks:
  - 行为变更风险：依赖 Script 默认行为的本地调试脚本将出现行为变化。
  - 可观测性风险：若文档未同步，运维可能误判当前决策模式。
  - 回归风险：虽然改动点小，仍需确认 `world_viewer_live` 全量单测通过。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-113-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-113-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
