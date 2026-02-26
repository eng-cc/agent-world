# 游戏发布前测试（game-test）项目管理文档

## 单次流程
- [x] T1 阅读 `doc/game-test.md` 与可玩性卡片模板，明确执行要求
- [x] T2 启动 Web 闭环测试链路并完成一轮真实游玩
- [x] T3 生成并填写测试卡片到 `doc/playability_test_result/`
- [x] T4 执行“带录屏”复测并补齐故障证据（视频 + 控制台）
- [x] T5 作为开发者排查“不可玩”根因并补充复现证据
- [x] T6 按用户请求追加一轮“默认 world_viewer_live 链路”真实玩家复测并填写卡片
- [x] T7 按用户再次请求执行夜间轮次真实游玩（带录屏）并填写卡片
- [x] T8 按用户本轮请求再执行一轮“真实玩家”Playwright 试玩并填写卡片

## 依赖
- `doc/game-test.md`
- `doc/playability_test_card.md`
- `.codex/skills/playwright/SKILL.md`
- `scripts/run-viewer-web.sh`
- `world_viewer_live` (`cargo run -p agent_world --bin world_viewer_live`)
- `doc/world-simulator/viewer-webgl-deferred-compat-2026-02-24.md`

## 测试记录
- card_2026_02_25_12_20_02.md
- card_2026_02_25_13_22_15.md
- card_2026_02_25_16_40_22.md
- card_2026_02_25_22_52_01.md
- card_2026_02_26_11_41_06.md
- 录屏/截图产物：`output/playwright/playability/20260225-132109/`
- 录屏/截图产物：`output/playwright/playability/20260225-163706/`
- 录屏/截图产物：`output/playwright/playability/20260225225029/`
- 录屏/截图产物：`output/playwright/playability/20260226-114106/`
- 开发排查复现：
  - `output/playwright/viewer/webgl-deferred-disable-verify2-20260225-143042/`
  - `output/playwright/viewer/webgl-panic-locate-20260225-143645/`

## 状态
- 当前阶段：已完成玩家复测 + 开发者排查 + 默认链路复测 + 夜间追加复测 + 本轮日间追加复测
- 风险：
  - 基线问题：Web 端偶发 `copy_deferred_lighting_id_pipeline`（`wgpu` Validation Error）导致崩溃。
  - 架构约束：`CopyDeferredLightingIdPlugin` 与 `Core3d` render graph 存在硬耦合，单独禁用会触发新的启动 panic（`Option::expect` -> `RuntimeError: unreachable`）。
  - 可玩性闭环问题：夜间复测再次出现 `connectionStatus=connecting` 且 `tick` 持续 `0`，并伴随 WebGL `CONTEXT_LOST_WEBGL`，玩法闭环仍不可用。
  - 本轮新增：`ws://127.0.0.1:5010` 链路下仍复现 `connectionStatus=connecting`/`tick=0`，并出现 `WebSocket opening handshake timed out` 与 wasm panic（`assertion failed: old_size > 0`）。
- 最近更新：2026-02-26
