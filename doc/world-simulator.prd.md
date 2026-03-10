# Legacy Redirect: world-simulator.prd

- 对应设计文档: `doc/world-simulator.design.md`
- 对应项目管理文档: `doc/world-simulator.project.md`

审计轮次: 6
- 对应标准执行入口: `doc/world-simulator.project.md`
## 目标
- 将根目录 `doc/world-simulator.prd.md` 的 redirect 入口迁移为 `.prd` 命名。
- 固化 world-simulator 模块主入口优先级，减少根目录历史入口误用。

## 范围
- In Scope:
  - 根目录 redirect 入口维护。
  - 模块主入口与历史归档入口指引。
- Out of Scope:
  - 不承载 world-simulator 业务设计正文。
  - 不替代 `doc/world-simulator/prd.md` 与主题分册内容。

## 接口 / 数据
- 根目录 redirect：`doc/world-simulator.prd.md`
- 根目录项目管理 redirect：`doc/world-simulator.project.md`
- 当前模块主入口：
  - `doc/world-simulator/prd.md`
  - `doc/world-simulator/project.md`
- 历史完整总览：不再保留归档目录；如需追溯参考 `doc/world-simulator/prd.index.md`。

## 里程碑
- M1（2026-03-03）：根目录 world-simulator redirect 文档完成 `.prd` 命名收口。

## 风险
- 若 root 旧路径未同步回写，可能导致下游专题文档导航失效。
