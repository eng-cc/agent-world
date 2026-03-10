# Agent World: README 入口链接有效性自动检查（2026-03-11）设计

- 对应需求文档: `doc/readme/governance/readme-link-check-automation-2026-03-11.prd.md`
- 对应项目管理文档: `doc/readme/governance/readme-link-check-automation-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
定义 readme 模块最小自动检查脚本，专门守住顶层 README 入口本地链接。

## 2. 设计结构
- 扫描层：提取 Markdown 链接。
- 解析层：过滤外链、锚点和 query。
- 报告层：输出断链来源与目标。

## 3. 关键接口 / 入口
- `scripts/readme-link-check.sh`
- `README.md`
- `doc/README.md`

## 4. 约束与边界
- 只检查本地 Markdown 链接。
- 不做网络请求。
- 不扩展到全仓文档。

## 5. 设计演进计划
- 先守住两份顶层入口文档。
- 后续再按需要扩到更多入口。
- 最终与人工巡检清单联动。
