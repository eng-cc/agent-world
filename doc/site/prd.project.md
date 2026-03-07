# site PRD Project

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-SITE-001 (PRD-SITE-001) [test_tier_required]: 完成 site PRD 改写，建立站点设计主入口。
- [x] TASK-SITE-002 (PRD-SITE-001/002) [test_tier_required]: 固化站点信息架构与内容同步校验清单。
- [x] TASK-SITE-003 (PRD-SITE-002/003) [test_tier_required]: 补齐发布下载链路与SEO质量门禁说明。
- [x] TASK-SITE-004 (PRD-SITE-003) [test_tier_required]: 建立站点发布后质量回归节奏。
- [x] TASK-SITE-005 (PRD-SITE-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-SITE-006 (PRD-SITE-002) [test_tier_required]: 同步 `site/doc/cn|en/viewer-manual.html` 语义口径（移除过时 `power_storage`，并校准自动目标语法）。
- [x] TASK-SITE-007 (PRD-SITE-003) [test_tier_required]: 回写站点项目状态文档（release pipeline + module 主项目）并与 CI 实况对齐。

## 依赖
- doc/site/prd.index.md
- `site/`
- `site/doc/`
- `doc/site/github-pages/`
- `doc/site/manual/`
- `doc/readme/prd.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-07
- 当前状态: completed
- 下一任务: 无
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- ROUND-002 进展: manual 子簇主从化已完成（`site-manual-static-docs` 主入口，`viewer-manual-content-migration-2026-02-15` 增量维护）。
- ROUND-002 进展: github-pages 子簇主从化已完成（`github-pages-game-engine-reposition-2026-02-25` 主入口，其余专题增量维护）。
- ROUND-003 进展: TASK-SITE-002/003/004 已按既有专题交付收敛为 completed，进入口径同步与状态回写阶段（TASK-SITE-006/007）。
- ROUND-003 进展: TASK-SITE-006 已完成，静态手册已移除过时 `power_storage` 表述并校准自动目标语法。
- ROUND-003 进展: TASK-SITE-007 已完成，`github-pages-release-download-pipeline-2026-03-01.prd.project.md` 状态已与 `Release Packages` 最新成功 run 对齐。
- 说明: 本文档仅维护 site 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
