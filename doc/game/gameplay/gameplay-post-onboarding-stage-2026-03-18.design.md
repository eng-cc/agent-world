# PostOnboarding 阶段目标链设计说明

- 对应需求文档: `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md`
- 对应项目管理文档: `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.project.md`

审计轮次: 1

## 1. 设计目标

- 把当前 `FirstSessionLoop` 完成后的“无正式后继阶段”问题收成一个明确的阶段机。
- 保持沙盒自由，不把 `PostOnboarding` 做成强制线性任务，但必须持续给出可执行下一步。
- 让 `viewer_engineer`、`runtime_engineer`、`qa_engineer` 对同一套状态与验收口径工作。

## 2. 阶段机

`first_session_active -> post_onboarding_introduced -> post_onboarding_active -> post_onboarding_blocked -> post_onboarding_milestone_completed -> post_onboarding_branch_ready`

- `post_onboarding_introduced`
  - 进入条件：首次行动闭环完成。
  - 目标：让玩家明确知道“教程结束了，接下来开始经营阶段”。
- `post_onboarding_active`
  - 进入条件：主目标已生成且当前可推进。
  - 目标：显示进度与下一步建议。
- `post_onboarding_blocked`
  - 进入条件：主目标存在明确阻塞。
  - 目标：把“为什么做不下去”解释清楚，不再只给模糊失败提示。
- `post_onboarding_milestone_completed`
  - 进入条件：首个持续能力里程碑达成。
  - 目标：让玩家确认自己已经不只是会点按钮，而是真的建立了组织能力。
- `post_onboarding_branch_ready`
  - 进入条件：完成阶段结算。
  - 目标：向中循环方向承接。

## 3. 目标选择优先级

v1 使用固定优先级，不上 LLM：

1. 工业持续能力
2. 恢复被阻塞产线
3. 首次交易 / 协作
4. 治理建议
5. 冲突 / 安全建议

约束：

- 不允许给出玩家当前无法理解或明显无法执行的长循环目标。
- 不允许重复把“再做一次与 onboarding 完全相同的按钮动作”当成阶段主目标。
- 如果工业线不可行，必须降级到仍属于“持续能力”的替代目标，而不是回到纯提示文案。

## 4. Viewer 表达层

建议分三块，而不是堆成一块：

- 阶段切换卡
  - 只在进入阶段时出现，负责宣告“你进入了下一阶段”。
- 主目标卡
  - 长驻，负责给出当前最重要的目标、进度、阻塞和下一步。
- 阶段完成卡
  - 在首个里程碑达成时出现，负责把玩家从 `PostOnboarding` 接到中循环方向。

这样做的原因：

- 阶段切换和长期跟踪不是同一种信息。
- 阻塞解释不应该淹没阶段宣告。
- 完成时刻需要单独的“反馈峰值”，否则玩家感觉不到成长节点。

## 5. Runtime / Viewer 边界

- `runtime_engineer`
  - 继续负责事实状态、事件与阻塞原因来源。
  - 不在 v1 承担复杂任务编排器。
- `viewer_engineer`
  - 负责阶段机、主目标选择器、卡片表达和恢复逻辑。
- `producer_system_designer`
  - 负责目标优先级、阻塞分类和阶段完成定义。
- `qa_engineer`
  - 负责 required-tier 验证、playability 卡片与 `#46` 回归结论。

## 6. 验收重点

- 玩家在同一会话内是否清楚知道“下一步是什么”。
- 玩家是否能理解自己为什么被卡住。
- 玩家完成首个里程碑后，是否明确感知自己进入了下一层玩法。
- Viewer 表达是否避免把微循环卡、工业卡、阶段卡再次堆成噪音。

## 7. 非目标

- 不在本轮定义完整任务 DSL。
- 不在本轮做跨数周的文明目标面板。
- 不在本轮把所有路线都产品化；先把工业优先的第一段承接做对。
