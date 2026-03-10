# Legacy Redirect: world-runtime.prd

- 对应设计文档: `doc/world-runtime.design.md`
- 对应项目管理文档: `doc/world-runtime.project.md`

审计轮次: 6
- 对应标准执行入口: `doc/world-runtime.project.md`
## 目标
- 将根目录 `doc/world-runtime.prd.md` 的 redirect 入口迁移为 `.prd` 命名。
- 明确 world-runtime 模块主入口，避免与根目录历史入口混淆。

## 范围
- In Scope:
  - 根目录 redirect 入口说明。
  - 模块主入口与历史归档入口指引。
- Out of Scope:
  - 不承载 world-runtime 业务设计正文。
  - 不替代 `doc/world-runtime/prd.md` 与分册内容。

## 接口 / 数据
- 根目录 redirect：`doc/world-runtime.prd.md`
- 根目录项目管理 redirect：`doc/world-runtime.project.md`
- 当前模块主入口：
  - `doc/world-runtime/prd.md`
  - `doc/world-runtime/project.md`
- 历史完整总览：不再保留归档目录；如需追溯参考 `doc/world-runtime/prd.index.md`。

## 里程碑
- M1（2026-03-03）：根目录 world-runtime redirect 文档完成 `.prd` 命名收口。

## 风险
- 根目录旧命名引用若未回写会造成导航断链，需要在迁移任务内同步处理。
