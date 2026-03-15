# playability_test_result 文档索引

审计轮次: 6

## 入口
- PRD: `doc/playability_test_result/prd.md`
- 设计总览: `doc/playability_test_result/design.md`
- 标准执行入口: `doc/playability_test_result/project.md`
- 文件级索引: `doc/playability_test_result/prd.index.md`

## 模块职责
- 维护可玩性反馈卡、评分口径、高优问题闭环与发布证据包格式。
- 承接 game / testing / core 之间的体验证据互链。
- 统一最近活跃轮次的卡片与正式模板入口。

## 关键文档
- `doc/playability_test_result/game-test.prd.md`
- `doc/playability_test_result/game-test.project.md`
- `doc/playability_test_result/playability_test_card.md`
- `doc/playability_test_result/playability_test_manual.md`
- `doc/playability_test_result/industrial-onboarding-required-tier-cards-2026-03-15.md`
- `doc/playability_test_result/templates/`
- `doc/playability_test_result/evidence/`

## 根目录收口
- 模块根目录主入口保留：`README.md`、`prd.md`、`design.md`、`project.md`、`prd.index.md`，并允许保留当前活跃轮次卡片样本。
- 历史卡片不再保留在仓库（`archive/` 目录已移除）。

## 维护约定
- 可玩性模板、评分口径、专题卡组或发布引用格式变化时，需同步更新 `prd.md` 与 `project.md`。
- 新增专题后，需同步回写 `doc/playability_test_result/prd.index.md` 与本目录索引。
