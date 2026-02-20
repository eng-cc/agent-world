# 文档目录索引

## 目标
- 统一 `doc/` 的主题分层与入口，降低文档检索成本。
- 保留少量顶层“总入口”文档，其余按主题聚类到子目录。

## 顶层入口
- `doc/world-simulator.md`：世界模拟总设计。
- `doc/world-runtime.md`：运行时总设计。
- `doc/viewer-manual.md`：Viewer 使用手册（中文基线）。
- `doc/README.md`：本目录索引（当前文件）。

## 文档治理入口
- `doc/doc-structure-cleanup-2026-02-20.md` 与 `doc/doc-structure-cleanup-2026-02-20.project.md`：目录分层整理方案与执行状态。
- `doc/doc-archive-audit-2026-02-20.md`、`doc/doc-archive-audit-2026-02-20.project.md`、`doc/doc-archive-audit-2026-02-20.result.md`：首轮归档审计方案与结果。
- `doc/doc-structure-freshness-review-round2-2026-02-20.md`、`doc/doc-structure-freshness-review-round2-2026-02-20.project.md`、`doc/doc-structure-freshness-review-round2-2026-02-20.result.md`：第二轮结构与时效复核方案与结果。

## 目录分层
- `doc/readme/`：README 对齐与缺口收口相关设计/项目文档。
- `doc/site/`：GitHub Pages 与静态站手册同步相关文档。
- `doc/world-simulator/`：模拟器专题设计、viewer/llm/scenario 等分册。
- `doc/world-simulator/m4/`：M4 电力与工业经济专题文档。
- `doc/world-runtime/`：运行时专题设计与技术分册。
- `doc/p2p/`：P2P/共识/DistFS 相关文档（含归档目录）。
- `doc/testing/`：测试策略、分层测试、CI 流程与手册分册。
- `doc/scripts/`：脚本相关设计与操作说明。
- `doc/game/`：Gameplay 顶层与治理闭环文档。
- `doc/devlog/`：按日期记录的开发任务日志。

## 命名约定
- 设计文档：`<topic>.md`
- 项目管理文档：`<topic>.project.md`
- 任务日志：`doc/devlog/YYYY-MM-DD.md`

## 维护规则
- 新功能必须先有设计文档与项目管理文档，再落地实现。
- 文档路径变更时，优先修复非 `doc/devlog` 文档中的引用。
- 历史 `devlog` 保持快照属性，不做追溯性路径改写。
