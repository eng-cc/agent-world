# oasis7: 脚本稳定性趋势跟踪指标（2026-03-11）设计

- 对应需求文档: `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.prd.md`
- 对应项目管理文档: `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
定义 scripts 模块治理趋势结构，持续衡量主入口、参数契约、fallback 围栏与修复时长的收敛程度。

## 2. 设计结构
- 指标层：四项治理趋势指标。
- 样本层：以 scripts 治理任务为样本单元。
- 报告层：输出 baseline Markdown。
- 交接层：回流给 `qa_engineer` 消费。

## 3. 关键接口 / 入口
- `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.prd.md`
- `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.project.md`
- `doc/scripts/evidence/script-stability-trend-baseline-2026-03-11.md`

## 4. 约束与边界
- 只统计 scripts 治理样本，不统计运行时功能结果。
- baseline 可小样本，但必须显式标注。
- 后续续写不应依赖脚本实现改动。

## 5. 设计演进计划
- 先发布首个 baseline。
- 再按周续写样本。
- 后续视需要接入自动化统计。
