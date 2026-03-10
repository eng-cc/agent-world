# playability_test_result PRD Project

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-PLAYABILITY_TEST_RESULT-001 (PRD-PLAYABILITY_TEST_RESULT-001) [test_tier_required]: 完成可玩性结果模块 PRD 改写。
- 模块设计总览：`doc/playability_test_result/design.md`
- [x] TASK-PLAYABILITY_TEST_RESULT-002 (PRD-PLAYABILITY_TEST_RESULT-001/002) [test_tier_required]: 固化反馈卡片标准字段与评分口径。
  - 产物文件:
    - `doc/playability_test_result/playability_test_card.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "标准字段|评分口径|继续可玩|需观察|高优先级阻断" doc/playability_test_result/playability_test_card.md`
- [x] TASK-PLAYABILITY_TEST_RESULT-003 (PRD-PLAYABILITY_TEST_RESULT-002/003) [test_tier_required]: 建立高优先级问题闭环追踪模板。
  - 产物文件:
    - `doc/playability_test_result/templates/high-priority-issue-closure-template.md`
  - 验收命令 (`test_tier_required`):
    - `test -f doc/playability_test_result/templates/high-priority-issue-closure-template.md`
    - `rg -n "Issue ID|当前状态|归因标签|复测记录|发布影响" doc/playability_test_result/templates/high-priority-issue-closure-template.md`
- [ ] TASK-PLAYABILITY_TEST_RESULT-004 (PRD-PLAYABILITY_TEST_RESULT-003) [test_tier_required]: 对接发布门禁中的可玩性证据包格式。
- [x] TASK-PLAYABILITY_TEST_RESULT-005 (PRD-PLAYABILITY_TEST_RESULT-001/002) [test_tier_required]: 将 `game-test` 与卡片模板文档迁入模块目录并完成根目录兼容跳转。
- [x] TASK-PLAYABILITY_TEST_RESULT-006 (PRD-PLAYABILITY_TEST_RESULT-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。

## 依赖
- doc/playability_test_result/prd.index.md
- `doc/playability_test_result/game-test.prd.md`
- `doc/playability_test_result/*.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-10
- 当前状态: active
- 下一任务: TASK-PLAYABILITY_TEST_RESULT-004
- 阶段收口优先级: `P0`
- 阶段 owner: `qa_engineer`（联审：`producer_system_designer`）
- 阻断条件: 在 `TASK-PLAYABILITY_TEST_RESULT-002/003` 完成前，可玩性问题不得作为统一格式证据进入发布 go/no-go 评审。
- 承接约束: 先固化反馈卡字段与评分口径，再建立高优问题闭环模板，最后再接入发布证据包格式。
- 专题入口状态: `game-test`/`playability_test_card`/`playability_test_manual` 已收敛到模块目录。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护可玩性结果模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。

## 阶段收口角色交接
## Meta
- Handoff ID: `HO-CORE-20260310-PLAY-001`
- Date: `2026-03-10`
- From Role: `producer_system_designer`
- To Role: `qa_engineer`
- Related Module: `playability_test_result`
- Related PRD-ID: `PRD-PLAYABILITY_TEST_RESULT-001/002/003`
- Related Task ID: `TASK-PLAYABILITY_TEST_RESULT-002/003/004`
- Priority: `P0`
- Expected ETA: `待接收方确认`

## Objective
- 目标描述：建立统一的可玩性反馈卡、评分口径与高优问题闭环模板，使玩法体验可以被稳定记录与发布引用。
- 成功标准：每条高优体验问题都能被固定字段记录、评分、跟踪并引用到发布证据包。
- 非目标：不扩展新的玩法测试主题，只先统一记录口径。

## Current State
- 当前实现 / 文档状态：模块主 PRD 已重写完成，但反馈字段、闭环模板、发布引用格式仍未收口。
- 已确认事实：core 已将 playability 反馈闭环列为 `P0`。
- 待确认假设：评分口径是否需要细分到不同玩家画像。
- 当前失败信号 / 用户反馈：体验问题已有观察，但难以跨轮次比较并进入正式发布评审。

## Scope
- In Scope: `TASK-PLAYABILITY_TEST_RESULT-002/003/004` 的模板与证据格式。
- Out of Scope: 扩展额外玩法机制或独立测试系统实现。

## Inputs
- 关键文件：`doc/playability_test_result/project.md`、`doc/playability_test_result/prd.md`、`doc/playability_test_result/game-test.prd.md`。
- 关键命令：沿用现有游戏测试 / 截图 / 卡片生成流程。
- 上游依赖：`game` 模块的微循环证据、`testing` 模块的证据包格式。
- 现有测试 / 证据：已有 game-test 文档与截图 / 人工观察输出。

## Requested Work
- 工作项 1：完成反馈卡标准字段与评分口径。
- 工作项 2：建立高优问题闭环追踪模板。
- 工作项 3：对接发布门禁中的可玩性证据包格式。

## Expected Outputs
- 代码改动：通常无需代码；如需，仅限卡片生成支撑脚本。
- 文档回写：`doc/playability_test_result/project.md` 与相关模板文档。
- 测试记录：补齐 `test_tier_required` 的字段抽样与模板引用验证。
- devlog 记录：记录评分口径、模板与未决风险。

## Done Definition
- [ ] 输出满足目标与成功标准
- [ ] 影响面已核对 `producer_system_designer` / `qa_engineer` / `viewer_engineer`
- [ ] 对应 `prd.md` / `project.md` 已回写
- [ ] 对应 `doc/devlog/YYYY-MM-DD.md` 已记录
- [ ] required 证据已补齐

## Risks / Decisions
- 已知风险：如果玩法截图已有但反馈字段不统一，发布时仍无法做跨轮次比较。
- 待拍板事项：评分口径是否采用单维分值还是多维评分卡。
- 建议决策：先统一标准字段与闭环模板，再细化评分层次。

## Validation Plan
- 测试层级：`test_tier_required`
- 验证命令：抽样模板字段、问题卡状态机与发布引用路径。
- 预期结果：可玩性问题可被一致记录、追踪并接入发布证据包。
- 回归影响范围：game / testing / 发布评审体验证据链。

## Handoff Acknowledgement
- 接收方确认范围：`已接收 TASK-PLAYABILITY_TEST_RESULT-002/003；当前提交覆盖反馈字段、评分口径与高优问题闭环模板`
- 接收方确认 ETA：`TASK-PLAYABILITY_TEST_RESULT-002/003 已完成，下一步进入 TASK-PLAYABILITY_TEST_RESULT-004`
- 接收方新增风险：`历史卡片与新闭环模板之间暂未建立自动映射，需在后续聚合阶段补引用`
