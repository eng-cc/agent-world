# Agent World: 高频脚本参数契约与失败语义（2026-03-11）设计

- 对应需求文档: `doc/scripts/governance/script-parameter-contracts-2026-03-11.prd.md`
- 对应项目管理文档: `doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
定义 scripts 模块的最小参数契约设计，让高频脚本具备统一的调用说明与失败分类。

## 2. 设计结构
- 契约层：最小命令、关键参数、默认值。
- 失败层：参数误用、环境依赖、门禁失败三类典型语义。
- 追踪层：模块 project / prd.index / handoff 互链。

## 3. 关键接口 / 入口
- `doc/scripts/governance/script-parameter-contracts-2026-03-11.prd.md`
- `doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`
- `doc/scripts/project.md`

## 4. 约束与边界
- 只记录当前可验证的参数与帮助输出。
- 不篡改脚本已有用法。
- 失败语义只描述常见来源，不替代完整 runbook。

## 5. 设计演进计划
- 先补高频脚本。
- 再扩展低频脚本与自动校验。
- 最后考虑将关键 help 输出纳入治理门禁。
