# game 文档索引

审计轮次: 6

## 入口
- PRD: `doc/game/prd.md`
- 设计总览: `doc/game/design.md`
- 标准执行入口: `doc/game/project.md`
- 文件级索引: `doc/game/prd.index.md`

## 模块职责
- 维护玩法目标态、核心循环与发布前可玩性口径。
- 汇总 gameplay 主题下的规则、经济、治理、战争与生产闭环专题。
- 承接体验优化、长期在线硬化与发布阻断相关设计追踪。

## 主题文档
- `gameplay/`：玩法、经济、治理、战争、长稳与发布闭环专题。

## 近期专题
- `doc/game/gameplay/gameplay-top-level-design.prd.md`
- `doc/game/gameplay/gameplay-post-onboarding-stage-2026-03-18.prd.md`
- `doc/game/gameplay/gameplay-micro-loop-feedback-visibility-2026-03-05.prd.md`
- `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
- `doc/game/gameplay/gameplay-release-production-closure.prd.md`

## 根目录收口
- 模块根目录主入口保留：`README.md`、`prd.md`、`design.md`、`project.md`、`prd.index.md`。
- 其余专题文档按主题下沉到 `gameplay/`。

## 维护约定
- 新玩法需求先落 PRD，再拆到项目管理文档。
- 玩法行为、发布门禁或体验验收变化需同步回写验收口径与测试引用。
- 新增 gameplay 专题后，需同步回写 `doc/game/prd.index.md` 与本目录索引。
