# oasis7: 发布候选 readiness 统一入口（2026-03-11）设计

- 对应需求文档: `doc/core/release-candidate-readiness-entry-2026-03-11.prd.md`
- 对应项目管理文档: `doc/core/release-candidate-readiness-entry-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
定义候选级 readiness 统一入口结构，把分散在 game / playability / testing / runtime / core 的证据链收敛到单一入口。

## 2. 设计结构
- Header 层：候选 ID、阶段、总状态。
- Slot 层：五类证据槽位与 owner / path / blocker。
- Summary 层：聚合规则与下一动作。
- Handoff 层：blocked 项对应的交接入口。

## 3. 关键接口 / 入口
- `doc/core/release-candidate-readiness-entry-2026-03-11.prd.md`
- `doc/core/release-candidate-readiness-entry-2026-03-11.project.md`
- `doc/core/project.md`
- `doc/core/reviews/stage-closure-go-no-go-task-game-018-2026-03-10.md`

## 4. 约束与边界
- 入口只做聚合，不复制证据正文。
- 本任务不实例化具体候选内容。
- blocked/ready 规则必须在文档中显式给出。

## 5. 设计演进计划
- 先冻结入口字段与聚合规则。
- 再实例化首份候选看板。
- 后续评估自动汇总可能性。
