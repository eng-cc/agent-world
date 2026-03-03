# game-test 归档清理（2026-02-28）

## 1. Executive Summary
- 将 `game-test` 主链路中已过时的卡片与文档迁移到归档区，减少执行测试时的误读与误用。
- 保持当前测试入口最小化：玩家只需关注最新模板、最新卡片与当前项目文档。
- 在不修改 `doc/game-test.md`（用户锁定文档）的前提下完成治理。

## 2. User Experience & Functionality
- 归档 `doc/playability_test_result/` 中历史卡片（保留当日最新活跃样本）。
- 归档已过时的 game-test 辅助文档（如旧量化手册、阶段性脚本实验文档）。
- 对 `doc/game-test.project.md` 做“现行视图瘦身”，并保留历史快照到归档。
- 新增归档索引/说明，确保历史可追溯。

不在范围内：
- 不修改 `doc/game-test.md`。
- 不删除任何历史内容（仅迁移到归档并保留引用线索）。
- 不改动运行脚本与功能代码。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
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

## 5. Risks & Roadmap
- M1：建档并固化归档口径。
- M2：完成卡片归档与活动目录瘦身。
- M3：完成旧文档归档与项目文档现行视图重建。
- M4：校验、日志收口与提交。

### Technical Risks
- 历史路径变化导致旧文档引用路径失效。
  - 缓解：在项目文档与 README 中提供归档路径与规则。
- 过度归档导致近期对比样本不足。
  - 缓解：保留当日活跃样本在主目录。
- 团队误将归档文档当现行口径。
  - 缓解：在归档文件/目录命名中显式标注 `archive`，并在主入口强调“现行文档”。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
