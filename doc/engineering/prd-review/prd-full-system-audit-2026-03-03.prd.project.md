# 全量 PRD 体系审读与对齐（2026-03-03）项目管理文档

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-ENGINEERING-020 (PRD-ENGINEERING-012) [test_tier_required]: 建立全量 PRD 逐篇审读机制，生成 active/archive 已读清单并完成模块入口三件套首批审读。
- [ ] TASK-ENGINEERING-021 (PRD-ENGINEERING-013) [test_tier_required]: 逐模块核对 active 专题文档与代码一致性；发现偏差按代码回写并补充处理动作。
- [ ] TASK-ENGINEERING-022 (PRD-ENGINEERING-013/014) [test_tier_required]: 审查跨文档重复与上下游口径漂移，执行合并/重定向/引用修复。
- [ ] TASK-ENGINEERING-023 (PRD-ENGINEERING-014) [test_tier_required]: 完成 archive 专题审读，补齐替代链与历史引用对齐。
- [ ] TASK-ENGINEERING-024 (PRD-ENGINEERING-012/013/014) [test_tier_required]: 建立周度增量审读节奏（新增/变更 PRD 自动入清单）。

## 已读清单（逐篇）
- Active 模块清单：
  - `doc/engineering/prd-review/checklists/active-core.md`
  - `doc/engineering/prd-review/checklists/active-engineering.md`
  - `doc/engineering/prd-review/checklists/active-game.md`
  - `doc/engineering/prd-review/checklists/active-headless-runtime.md`
  - `doc/engineering/prd-review/checklists/active-p2p.md`
  - `doc/engineering/prd-review/checklists/active-playability_test_result.md`
  - `doc/engineering/prd-review/checklists/active-readme.md`
  - `doc/engineering/prd-review/checklists/active-scripts.md`
  - `doc/engineering/prd-review/checklists/active-site.md`
  - `doc/engineering/prd-review/checklists/active-testing.md`
  - `doc/engineering/prd-review/checklists/active-world-runtime.md`
  - `doc/engineering/prd-review/checklists/active-world-simulator.md`
  - `doc/engineering/prd-review/checklists/active-root-legacy.md`
- Archive 清单：
  - `doc/engineering/prd-review/checklists/archive-all.md`

## 依赖
- `doc/engineering/prd-review/prd-full-system-audit-2026-03-03.prd.md`
- `doc/engineering/prd.md`
- `doc/engineering/prd.project.md`
- `doc/engineering/prd.index.md`
- `doc/*/prd.md`
- `doc/*/prd.project.md`
- `doc/*/prd.index.md`
- `scripts/doc-governance-check.sh`
- `scripts/site-manual-sync-check.sh`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 当前完成: 1 / 5（已完成清单搭建与入口文档首批审读）
- 下一任务: TASK-ENGINEERING-021
- 首批发现与修复:
  - 修复 `doc/core/prd.md` 中 `game-test` 路径为 `.prd` 命名入口。
  - 修复 `doc/world-simulator/archive/viewer-chat-agent-prompt-default-values.prd.project.md` 中旧手册与旧设计路径引用。
