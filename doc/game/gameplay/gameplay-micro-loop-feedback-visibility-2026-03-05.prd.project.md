# Gameplay 微循环反馈可见性闭环（项目管理文档）

- 审计轮次: 2

## ROUND-002 主从口径
- 主入口：`doc/game/gameplay/gameplay-top-level-design.prd.md`
- 本文仅维护增量计划，不重复主文档口径。

## 执行目标（How / When / Who）
- How：按“runtime 协议 -> runtime_live 映射 -> viewer 反馈 -> 门禁回归”顺序交付，避免前后口径漂移。
- When：以 5 个任务切片推进（D0 文档基线 + D1/D2 实现 + D3 回归）。
- Who：按责任域分工（runtime / viewer / test-release），每个任务单独提交并记录 devlog。

## 任务拆解（含 PRD-ID 映射）

| Task ID | PRD-ID | Owner | 计划窗口 | 核心改动 | 交付物 | 测试层级 | 状态 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TASK-GAMEPLAY-MLF-000 | PRD-GAME-004 | game-doc-owner | D0 (2026-03-05) | PRD 与项目文档建模、索引挂载 | `gameplay-micro-loop-feedback-visibility-2026-03-05.prd.md` / `.prd.project.md` | `test_tier_required`（文档治理） | completed |
| TASK-GAMEPLAY-MLF-001 | PRD-GAME-004-01/02 | runtime-owner | D1 | 新增 `ActionAccepted` 协议原语并接入 action 流程，保证“动作被接受”可观测 | runtime 事件模型 + action/event 处理代码 | `test_tier_required` | pending |
| TASK-GAMEPLAY-MLF-002 | PRD-GAME-004-02 | runtime-live-owner | D1-D2 | runtime_live 对 gameplay DomainEvent 与 ACK 的结构化映射与兼容降级 | runtime_live mapping + viewer 协议字段 | `test_tier_required` | pending |
| TASK-GAMEPLAY-MLF-003 | PRD-GAME-004-01 | viewer-owner | D2 | 微循环 HUD（倒计时/最近动作状态）与无进展诊断建议 | viewer right panel / i18n / tone 规则 | `test_tier_required` | pending |
| TASK-GAMEPLAY-MLF-004 | PRD-GAME-004-03 | test-release-owner | D3 | required/full 回归与卡片评分复核，形成门禁证据包 | 测试日志 + playability 卡片 + 结论 | `test_tier_required` + `test_tier_full` | pending |

## 执行顺序与依赖
- 串行主链：`MLF-001 -> MLF-002 -> MLF-003 -> MLF-004`。
- 依赖约束：
  - `MLF-002` 依赖 `MLF-001` 的事件 schema 稳定。
  - `MLF-003` 依赖 `MLF-002` 的映射字段与摘要口径。
  - `MLF-004` 依赖 `MLF-001/002/003` 全部合入后执行。

## 任务级完成标准（Definition of Done）

### TASK-GAMEPLAY-MLF-001
- `ActionAccepted` 在 runtime 事件流可持久化与可回放。
- 对旧 schema 兼容，不破坏既有事件解析。
- 关键测试：`env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::gameplay_protocol:: -- --nocapture`。

### TASK-GAMEPLAY-MLF-002
- gameplay 关键 DomainEvent 映射为结构化可读字段，不再仅显示笼统 `RuntimeEvent`。
- ACK 与 DomainEvent 均可被 viewer 协议消费；缺失字段时有降级文案。
- 关键测试：`env -u RUSTC_WRAPPER cargo test -p agent_world viewer::runtime_live::tests:: -- --nocapture`。

### TASK-GAMEPLAY-MLF-003
- 右侧面板可显示 war/governance/crisis/contract 倒计时与最近动作状态。
- 无进展提示包含“原因 + 可执行建议”并支持 `zh/en`。
- 关键测试：`env -u RUSTC_WRAPPER cargo test -p agent_world_viewer -- --nocapture`。

### TASK-GAMEPLAY-MLF-004
- required/full 测试通过且产物可追溯。
- playability 卡片评分达标：题 15/16/17 平均 `>= 4.0`；有效控制命中率 `>= 90%`。
- 关键测试：
  - `./scripts/ci-tests.sh required`
  - `./scripts/ci-tests.sh full`
  - Web 闭环用例按 `testing-manual.md` 执行并归档到 `doc/playability_test_result/`。

## 风险与应对
- 风险: runtime builtin identity 校验失败会阻塞 required tier。
- 应对: 在 `MLF-004` 前先执行最小定向测试；若失败为仓库基线问题，记录阻塞并按团队既定流程处理，不回滚本任务改动。
- 风险: 事件噪声过大影响可读性。
- 应对: viewer 仅突出微循环关键事件，其余降级为信息提示。

## 依赖
- `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.md`
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-war-politics-mvp-baseline.md`
- `doc/playability_test_result/playability_test_card.md`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-06
- 当前状态: active
- 当前完成: `TASK-GAMEPLAY-MLF-000`
- 下一任务: `TASK-GAMEPLAY-MLF-001`
- 阻塞项: 无（若 required tier 触发 builtin identity 基线故障，按“风险与应对”登记）
- 说明: 过程记录在 `doc/devlog/2026-03-06.md`
