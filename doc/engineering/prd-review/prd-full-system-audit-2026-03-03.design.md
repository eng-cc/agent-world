# 全量 PRD 体系审读与对齐设计（2026-03-03）

- 对应需求文档: `doc/engineering/prd-review/prd-full-system-audit-2026-03-03.prd.md`
- 对应项目管理文档: `doc/engineering/prd-review/prd-full-system-audit-2026-03-03.project.md`

## 1. 设计定位
定义全量 PRD 审读的执行面、清单结构、偏差回写链路与周度增量机制，确保审读结果可追溯、可复核。

## 2. 设计结构
- 清单层：按模块维护 active checklist，作为逐篇已读执行面。
- 审读层：按模块审查文档与实现、索引、替代链的一致性。
- 修复层：对偏差执行回写、合并、重定向与引用修复。
- 运营层：周度增量巡检承接新增或变更 PRD。

## 3. 关键接口 / 入口
- 审读清单：`doc/engineering/prd-review/checklists/*.md`
- 模块入口：`doc/*/prd.md`、`doc/*/project.md`、`doc/*/prd.index.md`
- 治理门禁：`scripts/doc-governance-check.sh`

## 4. 约束与边界
- 清单必须逐篇可追溯，不能只保留模块级百分比。
- 偏差回写遵循“代码为准、文档收口”的治理策略。
- 历史替代链和引用断链必须在当前批次闭环。

## 5. 设计演进计划
- Phase-0：建立全量清单与首批审读。
- Phase-1：按模块推进并修偏。
- Phase-2：进入周度增量巡检。
