# oasis7: 版本级候选 readiness 扩展（2026-03-11）设计

- 对应需求文档: `doc/core/release-candidate-version-escalation-2026-03-11.prd.md`
- 对应项目管理文档: `doc/core/release-candidate-version-escalation-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
把 task 级候选看板提升为版本级候选入口，聚焦 inherited ready 项与 runtime 长跑新增槽位。

## 2. 设计结构
- Inherited 层：复用 task board 已 ready 槽位。
- Version Extension 层：新增 `footprint` / `GC` / `soak` 三槽位。
- Summary 层：产出版本级总状态与下一动作。

## 3. 关键接口 / 入口
- `doc/core/release-candidate-version-escalation-2026-03-11.prd.md`
- `doc/core/reviews/release-candidate-readiness-board-version-2026-03-11.md`
- `doc/core/reviews/release-candidate-readiness-board-task-game-018-2026-03-11.md`

## 4. 约束与边界
- 版本级 board 不重写 task board 原结论。
- 本任务只挂接现有 runtime 长跑/治理入口，不伪造新运行结果。
- 当前总状态保持保守判定。

## 5. 设计演进计划
- 先完成版本级 board 骨架。
- 再补 runtime 联合证据。
- 后续再考虑正式版本 go/no-go 汇总。
