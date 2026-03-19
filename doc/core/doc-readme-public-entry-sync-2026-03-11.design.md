# oasis7: 文档总入口公开阅读路径同步（2026-03-11）设计

- 对应需求文档: `doc/core/doc-readme-public-entry-sync-2026-03-11.prd.md`
- 对应项目管理文档: `doc/core/doc-readme-public-entry-sync-2026-03-11.project.md`

审计轮次: 4

## 1. 设计定位
对齐 `doc/README.md` 与当前 repo/site 公开阅读路径，作为工程总入口的轻量同步。

## 2. 设计结构
- 更新时间：同步到当前日期。
- 快速路径：先根 README / site，再 core / module。
- 其余结构保持不变。

## 3. 关键接口 / 入口
- `doc/README.md`
- `README.md`
- `site/index.html`
- `doc/core/project.md`

## 4. 约束与边界
- 不重写模块矩阵。
- 不展开内部治理专题列表。
- 只修正阅读顺序与时间戳。

## 5. 设计演进计划
- 先同步本轮入口。
- 以后随公开阅读路径变化继续小步更新。
