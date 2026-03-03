# game-test 归档清理（2026-02-28）

## 目标
- 将 `game-test` 主链路中已过时的卡片与文档迁移到归档区，减少执行测试时的误读与误用。
- 保持当前测试入口最小化：玩家只需关注最新模板、最新卡片与当前项目文档。
- 在不修改 `doc/game-test.md`（用户锁定文档）的前提下完成治理。

## 范围
- 归档 `doc/playability_test_result/` 中历史卡片（保留当日最新活跃样本）。
- 归档已过时的 game-test 辅助文档（如旧量化手册、阶段性脚本实验文档）。
- 对 `doc/game-test.project.md` 做“现行视图瘦身”，并保留历史快照到归档。
- 新增归档索引/说明，确保历史可追溯。

不在范围内：
- 不修改 `doc/game-test.md`。
- 不删除任何历史内容（仅迁移到归档并保留引用线索）。
- 不改动运行脚本与功能代码。

## 接口 / 数据
- 输入：
  - `doc/playability_test_result/card_*.md`
  - `doc/game-test.project.md`
  - `doc/playability_test_manual.md`
  - `doc/scripts/run-game-test-ab-no-seek-b-phase-2026-02-27*.md`
- 输出：
  - `doc/playability_test_result/archive/2026-02/`
  - `doc/archive/game-test/`
  - 精简后的 `doc/game-test.project.md`
  - 入口说明 `doc/playability_test_result/README.md`

## 里程碑
- M1：建档并固化归档口径。
- M2：完成卡片归档与活动目录瘦身。
- M3：完成旧文档归档与项目文档现行视图重建。
- M4：校验、日志收口与提交。

## 风险
- 历史路径变化导致旧文档引用路径失效。
  - 缓解：在项目文档与 README 中提供归档路径与规则。
- 过度归档导致近期对比样本不足。
  - 缓解：保留当日活跃样本在主目录。
- 团队误将归档文档当现行口径。
  - 缓解：在归档文件/目录命名中显式标注 `archive`，并在主入口强调“现行文档”。
