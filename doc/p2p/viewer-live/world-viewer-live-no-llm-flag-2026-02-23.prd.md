# world_viewer_live `--no-llm` 关闭开关设计文档（2026-02-23）

- 对应设计文档: `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.design.md`
- 对应项目管理文档: `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.project.md`

审计轮次: 5
> 状态更新（2026-03-08）:
> - `world_viewer_live` 已下线 `--release-config`/`--node-*` 控制面入口。
> - 本文继续描述 `--no-llm` 行为语义；release-config 相关口径已失效并归档。

## 1. Executive Summary
- Problem Statement: 在 `world_viewer_live` 默认启用 LLM 决策的前提下，提供显式关闭开关 `--no-llm`，用于本地调试或脚本回退到 Script 决策。
- Proposed Solution: 保持参数语义可预期：默认值稳定、显式参数可覆盖。
- Success Criteria:
  - SC-1: `--no-llm` 可稳定覆盖默认 LLM 决策并切换为 Script 模式。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want world_viewer_live `--no-llm` 关闭开关设计文档（2026-02-23） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world/src/bin/world_viewer_live.rs`
  - AC-2: 新增 `--no-llm` 参数解析，设置 `llm_mode=false`。
  - AC-3: 更新 CLI help/usage 文案，明确 `--llm` 默认开启且可由 `--no-llm` 关闭。
  - AC-4: `crates/agent_world/src/bin/world_viewer_live.rs`（`#[cfg(test)]`）
  - AC-5: 补充 `--no-llm` 参数行为测试。
  - AC-6: 覆盖 `--no-llm` 行为回归，不依赖 release-config 路径。
- Non-Goals:
  - 不恢复 `--release-config` 模式。
  - 不变更场景默认值与拓扑相关参数语义。
  - 不新增额外决策模式枚举，仅在 LLM/Script 之间切换。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.prd.md`
  - `doc/p2p/viewer-live/world-viewer-live-no-llm-flag-2026-02-23.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) CLI 新增参数
- `--no-llm`
  - 语义：关闭 LLM 决策，改用 Script 决策。
  - 默认值：`llm_mode=true`（保持不变）。

### 2) 同时出现 `--llm` 与 `--no-llm` 的处理
- 采用“按出现顺序覆盖，最后出现者生效”语义，符合现有线性解析行为。

### 3) 与发行锁定模式的关系
- `world_viewer_live` 不再支持 `--release-config`，该约束仅保留历史参考意义。

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：完成（设计/项目文档建档）。
  - M1：完成（`--no-llm` 参数解析、help 文案与测试已合入）。
  - M2：完成（Viewer 手册已更新，`world_viewer_live` 定向回归通过）。
  - M3：完成（设计/项目文档与 devlog 已回写收口）。
- Technical Risks:
  - 参数冲突风险：用户同时传 `--llm`/`--no-llm` 时若缺少文档说明，易误解最终行为。
  - 发行配置风险：运维若误以为可在 release-config 运行时覆盖 `--no-llm`，会触发白名单拒绝。
  - 回归风险：参数变更虽小，但需确认 `world_viewer_live` 全量单测通过。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-114-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-114-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
