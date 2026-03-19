# oasis7: 发布候选 readiness 统一入口（2026-03-11）

- 对应设计文档: `doc/core/release-candidate-readiness-entry-2026-03-11.design.md`
- 对应项目管理文档: `doc/core/release-candidate-readiness-entry-2026-03-11.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: 当前 gameplay / playability / testing / runtime / core 已分别沉淀了任务级证据，但仍缺一个统一入口来回答“某个候选现在到底 ready 到什么程度、还缺哪一块、由谁补”。没有统一入口时，发布判断仍容易依赖人工翻找和口头串联。
- Proposed Solution: 在 core 建立发布候选 readiness 统一入口，定义候选 ID、P0 证据槽位、状态等级、阻断原因、owner 与证据路径，并把现有 go/no-go 文档链挂接成单一阅读入口。
- Success Criteria:
  - SC-1: 候选 readiness 至少覆盖 `gameplay` / `playability` / `testing` / `runtime` / `core` 五类证据槽位。
  - SC-2: 每个槽位都有 `owner`、`状态`、`证据路径`、`阻断原因` 字段。
  - SC-3: core 主项目能够把该主题作为新一轮统一发布入口继续推进。
  - SC-4: 后续任一候选都能按同一模板登记，而无需重新定义字段。

## 2. User Experience & Functionality
- User Personas:
  - `producer_system_designer`：需要快速判断候选是否已具备进入 go/no-go 的最小条件。
  - `qa_engineer`：需要知道当前还缺哪条证据链、阻断是否已解除。
  - 模块 owner：需要知道自己在候选上的明确责任槽位。
- User Scenarios & Frequency:
  - 候选收口前：登记并刷新 readiness 状态。
  - go/no-go 评审前：读取统一入口，而不是手工翻多个模块文档。
  - 阻断解除后：只更新对应槽位与结论，不重写整份评审文档。
- User Stories:
  - PRD-CORE-RR-001: As a `producer_system_designer`, I want one readiness entry per candidate, so that I can judge release posture from a single page.
  - PRD-CORE-RR-002: As a `qa_engineer`, I want explicit evidence slots and blocker fields, so that missing proof is visible instead of implied.
  - PRD-CORE-RR-003: As a 模块 owner, I want my owner slot and expected output stated, so that I know exactly what to deliver next.
- Critical User Flows:
  1. `创建候选 ID -> 填充五类证据槽位 -> 标记 ready/watch/blocked`
  2. `某槽位补齐新证据 -> 更新状态/阻断原因 -> 回写候选总状态`
  3. `go/no-go 评审读取统一入口 -> 决定 pass / conditional / blocked`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| candidate header | 候选 ID、日期、阶段、总状态、summary | 创建候选条目 | `draft -> tracking -> reviewed` | 按日期倒序 | `producer_system_designer` 可建 |
| evidence slot | 槽位名、owner、状态、证据路径、阻断原因、下一动作 | 更新单槽位状态 | `missing -> ready/watch/blocked` | P0 槽位优先 | 对应 owner 更新，`qa_engineer` 复核 |
| readiness summary | ready 数、blocked 数、watch 数、结论 | 聚合所有槽位 | `unknown -> candidate_ready/conditional/blocked` | 任一 P0 blocked 即整体 blocked | core owner 裁定 |
| handoff linkage | 发起角色、接收角色、输入、输出、done | 关联下一步交接 | `planned -> sent -> acknowledged` | blocked 项优先 handoff | 发起方填写 |
- Acceptance Criteria:
  - AC-1: 统一入口明确五类证据槽位及字段。
  - AC-2: 定义候选总状态的聚合规则。
  - AC-3: 产出专题 PRD / Design / Project 与 handoff。
  - AC-4: `doc/core/project.md` 将下一任务推进到“首份候选看板实例化”。
- Non-Goals:
  - 不在本任务直接实例化具体候选看板内容。
  - 不替代模块内证据原文。
  - 不自动抓取证据状态。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: readiness 入口位于 core，聚合模块证据而不复制证据正文；每个槽位只记录 owner、状态、证据路径、阻断和下一动作。
- Integration Points:
  - `doc/core/reviews/stage-closure-go-no-go-task-game-018-2026-03-10.md`
  - `doc/testing/evidence/*.md`
  - `doc/playability_test_result/evidence/*.md`
  - `doc/world-runtime/evidence/*.md`
  - `doc/game/gameplay/*.md`
- Edge Cases & Error Handling:
  - 证据存在但格式不统一：槽位可记 `watch`，不得直接记 `ready`。
  - 多模块同时 claim ready：以槽位 owner 的最新证据为准，由 `qa_engineer` 复核。
  - 某槽位无 owner：整体不得进入 `candidate_ready`。
- Non-Functional Requirements:
  - NFR-CRR-1: 单页阅读时长应 <= 10 分钟。
  - NFR-CRR-2: 任一 blocked 槽位都必须能直接跳转到证据路径或 handoff。
  - NFR-CRR-3: 候选字段必须可被 grep 快速检索。
- Security & Privacy: 只聚合仓内证据路径与状态，不额外复制敏感数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (`CRR-1`): 建立统一入口字段与聚合规则。
  - v1.1 (`CRR-2`): 产出首份候选看板实例。
  - v2.0 (`CRR-3`): 评估是否需要自动化汇总。
- Technical Risks:
  - 风险-1: 若槽位太多，入口会变成第二份 go/no-go 原文。
  - 风险-2: 若没有总状态规则，入口会退化成纯索引页。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-CORE-RR-001 | `TASK-CORE-017` | `test_tier_required` | 检查五类证据槽位与总状态字段存在 | 候选入口一致性 |
| PRD-CORE-RR-002 | `TASK-CORE-017` | `test_tier_required` | 检查 owner / blocker / evidence path 字段存在 | 证据缺口可见性 |
| PRD-CORE-RR-003 | `TASK-CORE-017/018` | `test_tier_required` | 检查下一任务已推进到首份候选看板实例化 | 下一轮执行入口稳定性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| `DEC-CRR-001` | 先做统一入口字段 | 直接做具体候选实例 | 先冻结字段，后续实例才能稳定复用。 |
| `DEC-CRR-002` | 入口只聚合路径与状态，不复制证据正文 | 在 core 重写所有证据摘要 | 避免重复维护与口径漂移。 |
| `DEC-CRR-003` | 任一 P0 槽位 blocked 即整体 blocked | 允许部分 P0 缺失但仍 overall ready | 保持候选级放行口径刚性。 |
