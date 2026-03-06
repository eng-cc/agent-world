# Gameplay 微循环反馈可见性闭环（Runtime 协议 + Viewer 反馈）

- 审计轮次: 1
- 对应项目管理文档: doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.project.md

## ROUND-002 主从口径
- 主入口：`doc/game/gameplay/gameplay-top-level-design.prd.md`
- 本文仅维护增量，不重复主文档口径。

## 目标
- 修复微循环“动作被接受但无反馈/反馈延迟过长”的断裂，提升控制感与节奏体验。
- 补齐 Runtime 协议的“动作已接受”与 gameplay 事件可见反馈链路，避免玩家把系统延迟误判为“没听懂”。
- 建立可量化的微循环体验指标与回归口径，纳入发行门禁。
- 优先级来源: playability 卡片控制感评分偏低 + release gate 需要可验证微循环反馈。

## 范围

### In Scope
- Runtime 协议扩展：新增“动作接受确认”与可见化字段（action_id/action_kind/eta_ticks/notes）。
- Gameplay DomainEvent 可见性：战争/治理/危机/经济合约/元进度的事件映射与摘要规则。
- Viewer 反馈层：toast/事件行/微循环 HUD（倒计时 + 最近动作状态 + 无进展诊断）。
- 可玩性卡片指标回归：有效控制命中率、卡片 15/16/17 评分目标。

### Out of Scope
- 大规模玩法数值重平衡与规则重写（仅补反馈与协议可见性）。
- 3D 场景与美术资产改版。
- 网络/链上协议改造与跨节点共识流程。

## 接口 / 数据
- Runtime 新增事件（建议落为 DomainEvent）：
  - `ActionAccepted { action_id, action_kind, actor_id, eta_ticks, notes }`
- Gameplay DomainEvent 映射：`WarDeclared/WarConcluded/GovernanceProposalOpened/GovernanceVoteCast/GovernanceProposalFinalized/CrisisSpawned/CrisisResolved/CrisisTimedOut/EconomicContractOpened/Accepted/Settled/Expired/MetaProgressGranted`
- Viewer 显示字段：
  - 微循环 HUD：`last_action_status`、`pending_eta_ticks`、`due_timers`（war/governance/crisis/contract）
  - 反馈 toast：`tone + summary + action_id(optional)`

## 里程碑
- M1: Runtime 协议 ACK + Gameplay 事件映射与摘要规则。
- M2: 微循环 HUD 与无进展诊断提示可见。
- M3: 可玩性卡片回归达标并纳入发行门禁。

## 风险
- 事件映射口径不一致导致“反馈误导”，反而降低控制感。
- 新增 ACK 事件若节奏过密，可能造成信息噪声与 UI 干扰。
- 旧版本 runtime/viewer 兼容不足导致回退体验不一致。

## 1. Executive Summary

- **Problem Statement**: 当前微循环存在“动作被接受但缺少及时可见反馈”的体验断裂，导致控制感与节奏评分偏低，影响可玩性门禁判断。
- **Proposed Solution**: 在 runtime 协议中补齐动作 ACK 与 gameplay 事件摘要映射，在 viewer 中统一渲染微循环反馈（toast + 事件行 + HUD 计时/诊断）。
- **Success Criteria**:
  - SC-1: 有效控制命中率（可推进控制次数 / 预期推进控制次数）`>= 90%`。
  - SC-2: 可玩性卡片题 15/16/17 平均评分 `>= 4.0/5`。
  - SC-3: gameplay 关键动作的可见反馈延迟 `P95 <= 1s`（ActionAccepted 或对应 DomainEvent 进入 UI）。

## 2. User Experience & Functionality

- **User Personas**:
  - 主用户: 玩家/评测者，需要在 5~15 分钟内看到明确的动作反馈与倒计时。
  - 次用户: 玩法开发者，需要稳定的“动作 -> 事件 -> UI”可观测链路用于回归。
  - 次用户: 发行评审者，需要量化指标与证据包判断微循环是否达标。
- **User Scenarios & Frequency**:
  - 微循环体验（每次会话都发生）：下达动作后需在 1s 内看到“已接受/预计结算”。
  - 版本回归（每个候选版本至少一次）：对照指标与卡片评分验证。
  - 问题诊断（缺陷复盘时）：定位动作失败原因与无进展窗口。
- **User Stories**:
  - PRD-GAME-004-01: As a 玩家/评测者, I want immediate action feedback and timers, so that I trust my control.
  - PRD-GAME-004-02: As a 玩法开发者, I want runtime to emit actionable ACK + gameplay summaries, so that I can debug loops.
  - PRD-GAME-004-03: As a 发行评审者, I want measurable micro-loop metrics, so that release readiness is objective.
- **Critical User Flows**:
  1. Flow-MLF-001: `玩家提交动作 -> Runtime 产生 ActionAccepted -> Viewer 显示 ACK + ETA -> 对应 DomainEvent 到达后更新状态`
  2. Flow-MLF-002: `Gameplay DomainEvent 触发 -> 映射为可读摘要 -> Toast/事件行更新 -> HUD 倒计时刷新`
  3. Flow-MLF-003: `连接正常但无进展 -> 诊断最近动作/拒绝原因/资源缺口 -> 输出可执行建议`
- **Functional Specification Matrix**:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 动作 ACK 事件 | `action_id/action_kind/actor_id/eta_ticks/notes` | 动作通过校验后立即发出 ACK | `submitted -> accepted -> resolved/failed` | 按事件时间排序 | 系统生成，客户端只读 |
| Gameplay 事件摘要 | `domain_kind/summary/key_ids/tone` | DomainEvent 到达后生成摘要 | `raw -> summarized` | 规则化模板映射 | 系统生成，客户端只读 |
| 微循环 HUD 计时 | `pending_eta_ticks/due_timers` | 每 tick 刷新倒计时 | `pending -> due -> cleared` | 最近到期优先 | 客户端显示 |
| 无进展诊断 | `idle_secs/last_action/reject_reason/suggestion` | 无进展 >= 5s 触发提示 | `idle -> hinted -> recovered` | 只保留最新 | 客户端显示 |
- **Acceptance Criteria**:
  - AC-1: Runtime 产生 ActionAccepted 且 viewer 可见，形成“已接受 + ETA”反馈。
  - AC-2: Gameplay DomainEvent 均有可读摘要与 tone 分类。
  - AC-3: HUD 展示至少战争/治理/危机/合约倒计时，并在到期时明确提示。
  - AC-4: 无进展提示包含“原因 + 下一步建议”，不再出现纯“等待/重试”。
  - AC-5: SC-1/SC-2/SC-3 在 playability 测试中达标。
  - AC-6: 完成标准为 required tier 通过 + playability 卡片与指标复核通过。
- **Non-Goals**:
  - 不改动战争/治理/危机/经济规则本体。
  - 不新增玩法 UI 面板与复杂视觉动效。
  - 不引入新的跨节点协议与共识流程。

## 3. AI System Requirements (If Applicable)
- **Tool Requirements**: 不适用（本 PRD 不改动 LLM 决策或工具链）。
- **Evaluation Strategy**: 通过 playability 卡片 + 行为指标回归验证（见第 6 章）。

## 4. Technical Specifications

- **Architecture Overview**:
  - Runtime 动作校验 -> `ActionAccepted` 事件 -> runtime_live 映射 -> viewer toast/HUD。
  - Gameplay DomainEvent -> 映射摘要 -> 事件行/Toast/倒计时刷新。
  - 无进展监控：基于 tick/event 增量与最近动作状态生成诊断提示。
- **Integration Points**:
  - Runtime 事件与状态：`crates/agent_world/src/runtime/events.rs`、`crates/agent_world/src/runtime/state/apply_domain_event_gameplay.rs`
  - Viewer 映射与反馈：`crates/agent_world/src/viewer/runtime_live/mapping.rs`、`crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`
  - 玩法设计基线：`doc/game/gameplay/gameplay-top-level-design.prd.md`
  - 可玩性卡片：`doc/playability_test_result/playability_test_card.md`
  - 测试门禁：`testing-manual.md`
- **Edge Cases & Error Handling**:
  - 空数据: 无 action/event 时 HUD 显示“暂无行动/事件”，不出现误导性倒计时。
  - ACK 事件缺失：viewer 降级为“等待 DomainEvent”并提示延迟原因。
  - ACK 超时: 超过 eta_ticks 未见 DomainEvent 时提示“仍在处理中”并引导诊断。
  - DomainEvent 字段缺失：渲染为通用摘要并记录诊断日志。
  - 事件乱序/重复：按 event_id 去重并以时间戳排序。
  - 并发冲突: 同 tick 多事件按 event_id 排序，保持可复现。
  - Viewer 断线或 tick 停滞：提示“连接/推进异常”，不误导为“操作失败”。
  - 旧版本兼容：不支持 ACK 的 runtime 仍可运行，viewer 提示“协议不支持 ACK”。
  - 数据异常：summary 生成失败时回退为原始事件类型字符串。
  - 权限不足: ACK 与摘要仅由系统生成，客户端写入请求拒绝并提示。
  - 数据迁移: 本迭代不引入数据迁移，旧快照保持兼容。
- **Non-Functional Requirements**:
  - NFR-MLF-1: ACK/DomainEvent 到 UI 可见反馈延迟 `P95 <= 1s`。
  - NFR-MLF-2: 新增映射逻辑对 viewer 帧耗时影响 `P95 <= 2ms`。
  - NFR-MLF-3: 事件摘要与提示支持中英文（`zh/en`）。
  - NFR-MLF-4: 兼容旧 runtime/event schema（无 ACK 也可运行）。
  - NFR-MLF-5: 数据规模 10k 事件内，事件行检索不出现可感知卡顿。
  - NFR-MLF-6: 新增 DomainEvent 类型时可扩展摘要模板，不要求改 UI 布局。
- **Security & Privacy**: 不新增敏感字段与玩家私密数据；日志仅记录 action_id 与公开事件信息。

## 5. Risks & Roadmap

- **Phased Rollout**:
  - MVP: ACK 事件 + gameplay 事件摘要 + toast 显示。
  - v1.1: HUD 倒计时 + 无进展诊断建议。
  - v2.0: 指标看板与门禁自动化回归。
- **Technical Risks**:
  - Runtime ACK 与 DomainEvent 口径不一致，导致“确认已接受但后续无结果”的二次困惑。
  - 事件摘要规则过于通用，无法解释关键因果。

## 6. Validation & Decision Record

- **Test Plan & Traceability**:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-GAME-004 | TASK-GAMEPLAY-MLF-001/002 | `test_tier_required` | runtime 事件映射与 viewer toast 单测 | 动作 ACK/事件可见性 |
| PRD-GAME-004 | TASK-GAMEPLAY-MLF-003 | `test_tier_required` | HUD 倒计时与无进展诊断回归 | 微循环节奏与控制感 |
| PRD-GAME-004 | TASK-GAMEPLAY-MLF-004 | `test_tier_required` + `test_tier_full` | playability 卡片与指标复核 | 发行门禁体验 |
- **Decision Log**:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-MLF-001 | 新增 ActionAccepted 事件 | 仅依赖后续 DomainEvent 推断 | 避免“已接受但无反馈”的空窗期 |
| DEC-MLF-002 | Gameplay 事件摘要规则化 | 直接输出 Debug `{:?}` | 反馈可读性与控制感提升 |
| DEC-MLF-003 | HUD 倒计时展示 | 仅通过日志/事件行观察 | 微循环节奏需要即时可见指标 |

审计轮次: 4

