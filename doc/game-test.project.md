# 游戏发布前测试（game-test）项目管理文档

## 单次流程
- [x] T1 阅读 `doc/game-test.md` 与可玩性卡片模板，明确执行要求
- [x] T2 启动 Web 闭环测试链路并完成一轮真实游玩
- [x] T3 生成并填写测试卡片到 `doc/playability_test_result/`
- [x] T4 执行“带录屏”复测并补齐故障证据（视频 + 控制台）

## 依赖
- `doc/game-test.md`
- `doc/playability_test_card.md`
- `.codex/skills/playwright/SKILL.md`
- `scripts/run-viewer-web.sh`
- `world_viewer_live` (`cargo run -p agent_world --bin world_viewer_live`)

## 测试记录
- card_2026_02_25_12_20_02.md
- card_2026_02_25_13_22_15.md
- 录屏/截图产物：`output/playwright/playability/20260225-132109/`

## 状态
- 当前阶段：已完成二次测试（第二次含录屏）
- 风险：Web 端启动存在 `copy_deferred_lighting_id_pipeline` 渲染 panic，实测无法进入可玩状态
- 最近更新：2026-02-25
