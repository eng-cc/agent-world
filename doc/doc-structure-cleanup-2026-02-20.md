# Doc 目录分层整理（2026-02-20）设计文档

## 目标
- 整理 `doc/` 顶层混放文件，降低查找成本与后续维护成本。
- 将同主题文档按目录聚合，保持“设计文档 + 项目管理文档”成对放置。
- 在不修改技术方案语义的前提下，完成路径迁移与引用修复。

## 范围

### In Scope
- 新增并使用以下目录分层：
  - `doc/readme/`：承接 `readme-*` 设计与项目管理文档。
  - `doc/site/`：承接 `github-pages-*`、站点手册相关设计与项目管理文档。
  - `doc/world-simulator/m4/`：承接 `m4-*` 设计与项目管理文档。
- 迁移上述文件并修复非 `doc/devlog/` 文档中的路径引用。
- 新增 `doc/README.md` 作为文档目录入口说明。

### Out of Scope
- 不改写历史 `doc/devlog/*.md` 内容（保持历史记录不变）。
- 不调整 `third_party/` 与业务代码逻辑。
- 不对文档技术结论做语义重写，仅做结构与路径治理。

## 接口 / 数据
- 新目录：
  - `doc/readme/`
  - `doc/site/`
  - `doc/world-simulator/m4/`
- 迁移规则：
  - `doc/readme-*.md` -> `doc/readme/*.md`
  - `doc/readme-*.project.md` -> `doc/readme/*.project.md`
  - `doc/github-pages-*.md` -> `doc/site/*.md`
  - `doc/github-pages-*.project.md` -> `doc/site/*.project.md`
  - `doc/site-manual-static-docs*.md` -> `doc/site/*.md`
  - `doc/viewer-manual-content-migration-2026-02-15*.md` -> `doc/site/*.md`
  - `doc/m4-*.md` -> `doc/world-simulator/m4/*.md`
  - `doc/m4-*.project.md` -> `doc/world-simulator/m4/*.project.md`
- 稳定入口：
  - `doc/README.md`（新增）
  - `doc/viewer-manual.md`、`doc/world-runtime.md`、`doc/world-simulator.md`（保留顶层入口）

## 里程碑
- M1：完成治理文档（设计 + 项目管理）并冻结迁移边界。
- M2：完成文件迁移与路径引用修复。
- M3：完成目录入口文档、校验与项目状态收口。

## 风险
- 风险：路径批量迁移可能遗漏引用，导致跳转失效。
  - 缓解：迁移后执行全仓 `rg` 路径扫描，重点检查非 `doc/devlog` 文档。
- 风险：历史日志中的旧路径与新路径不一致。
  - 缓解：明确 devlog 保持历史快照，不做追溯性改写；新增入口文档说明当前目录基线。
