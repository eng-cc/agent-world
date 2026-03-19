# oasis7: README 与模块 PRD 口径一致性巡检清单（2026-03-11）设计

- 对应需求文档: `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.prd.md`
- 对应项目管理文档: `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
定义 readme 模块的人工巡检结构，让顶层入口文档与模块 PRD 的一致性复核有固定模板。

## 2. 设计结构
- 检查项层：顶层叙事、状态口径、术语边界、入口链接、触发条件。
- 权威源层：为每个检查项指定回溯文档。
- 处置层：每项检查失败时给出回写动作。

## 3. 关键接口 / 入口
- `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.prd.md`
- `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.project.md`
- `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.md`

## 4. 约束与边界
- 清单只定义检查模板，不直接替代自动化链接校验。
- 权威源必须引用现有主文档，不重复定义模块内容。
- 高优项必须包含产品状态口径。

## 5. 设计演进计划
- 先冻结人工清单模板。
- 再接入自动链接检查。
- 后续纳入季度审查节奏。
