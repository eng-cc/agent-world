# 游戏发布前测试（game-test）项目管理文档

审计轮次: 1

## 任务拆解
- [x] G1 按 `doc/playability_test_result/game-test.prd.md` 执行真实玩家闭环（Playwright + 录屏）
- [x] G2 填写卡片并沉淀证据到 `doc/playability_test_result/` 与 `output/playwright/playability/`
- [x] G3 维护“活跃样本/归档样本”分层，降低历史噪音对当前测试的干扰
- [x] G4 校验卡片与 `doc/playability_test_result/playability_test_card.md` 最新模板一致性
- [x] G5 补充 2026-03-01 真实玩家复测样本并记录 A/B 波动

## 依赖
- `doc/playability_test_result/game-test.prd.md`（用户锁定，不修改）
- `doc/playability_test_result/playability_test_card.md`
- `doc/playability_test_result/README.md`
- `.codex/skills/playwright/SKILL.md`
- `scripts/run-game-test.sh`

## 测试记录
- 活跃卡片（主目录）：
  - `card_2026_03_01_00_20_13.md`
  - `card_2026_02_28_19_22_20.md`
  - `card_2026_02_28_21_22_51.md`
  - `card_2026_02_28_22_47_14.md`
  - `card_2026_02_28_23_27_06.md`
- 活跃产物目录：
  - `output/playwright/playability/20260301-001603-long/`
  - `output/playwright/playability/startup-20260301-000045/`
  - `output/playwright/playability/20260228-224714-long/`
  - `output/playwright/playability/20260228-231005-long/`
  - `output/playwright/playability/startup-20260228-224626/`
  - `output/playwright/playability/startup-20260228-231005/`
- 历史归档：

## 状态
- 当前阶段：现行视图 + 历史归档模式下持续复测（2026-03-01）
- 当前风险：
  - 控制语义在主界面的可见性仍偏弱，玩家仍会依赖调试信号判断推进状态。
  - 最新样本出现 A 段首控 `play` 已接受但未在窗口内推进（`accepted_no_progress`），可预期性仍有抖动。
  - LLM 前置配置缺失时，`run-game-test.sh` 仍可能启动失败（可用 `--no-llm` 回退）。
  - 历史文档虽已归档，但旧日志中的历史路径仍可能被误引用。
- 最近更新：2026-03-01

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.prd.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
