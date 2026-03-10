# Agent World: 脚本分层与主入口 / fallback 入口梳理（2026-03-11）设计

- 对应需求文档: `doc/scripts/governance/script-entry-layering-2026-03-11.prd.md`
- 对应项目管理文档: `doc/scripts/governance/script-entry-layering-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
定义 scripts 模块的入口治理结构，把根 `scripts/` 目录映射为稳定主入口、辅助入口与受控 fallback。

## 2. 设计结构
- 入口层：面向开发者 / QA / CI 的推荐主入口。
- 职责层：按开发治理、发布门禁、站点巡检、长跑回归、Viewer 诊断分层。
- fallback 层：仅在主入口不可满足时才允许升级到诊断链路。
- 追踪层：通过 `project.md` 与 `prd.index.md` 建立模块可达引用。

## 3. 关键接口 / 入口
- 主入口专题：`doc/scripts/governance/script-entry-layering-2026-03-11.prd.md`
- 清单与状态：`doc/scripts/governance/script-entry-layering-2026-03-11.project.md`
- 模块入口：`doc/scripts/project.md` / `doc/scripts/prd.index.md`

## 4. 约束与边界
- 不修改现有脚本行为，只定义调用层级与使用优先级。
- fallback 必须晚于主入口出现，且带触发条件。
- 仅覆盖高频根脚本，不在本轮穷尽所有历史低频工具。

## 5. 设计演进计划
- 先固定主入口与 fallback 表。
- 再为高频脚本补参数契约。
- 最后补稳定性趋势与治理节奏。
