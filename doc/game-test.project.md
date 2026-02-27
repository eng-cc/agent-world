# 游戏发布前测试（game-test）项目管理文档

## 任务拆解
- [x] T1 阅读 `doc/game-test.md` 与可玩性卡片模板，明确执行要求
- [x] T2 启动 Web 闭环测试链路并完成一轮真实游玩
- [x] T3 生成并填写测试卡片到 `doc/playability_test_result/`
- [x] T4 执行“带录屏”复测并补齐故障证据（视频 + 控制台）
- [x] T5 作为开发者排查“不可玩”根因并补充复现证据
- [x] T6 按用户请求追加一轮“默认 world_viewer_live 链路”真实玩家复测并填写卡片
- [x] T7 按用户再次请求执行夜间轮次真实游玩（带录屏）并填写卡片
- [x] T8 按用户本轮请求再执行一轮“真实玩家”Playwright 试玩并填写卡片
- [x] T9 提供 game-test 一键启动脚本并更新手册，防止启动参数错误
- [x] T10 按用户本轮请求基于 `doc/game-test.md` 再执行一轮真实玩家游玩并填写卡片
- [x] T11 按用户本轮请求基于 `doc/game-test.md` 执行傍晚轮次真实玩家游玩并填写卡片
- [x] T12 按用户本轮请求基于 `doc/game-test.md` 再执行一轮真实玩家游玩并填写卡片
- [x] T13 按用户本轮请求基于 `doc/game-test.md` 再执行一轮真实玩家游玩并填写卡片
- [x] T14 按用户请求将 `scripts/run-game-test.sh` 默认切换为开启 LLM（保留 `--no-llm` 回退）
- [x] T15 按用户本轮请求基于 `doc/game-test.md` 再执行一轮真实玩家游玩并填写卡片
- [x] T16 按用户本轮请求基于 `doc/game-test.md` 再执行一轮真实玩家游玩并填写卡片
- [x] T17 落地 A/B 分段量化复测脚本与卡片量化字段（TTFC/有效控制命中率/无进展窗口）
- [x] T18 同步用户改版 `doc/game-test.md`（最小手册 + 不可改约束）并更新项目状态

## 依赖
- `doc/game-test.md`
- `doc/playability_test_card.md`
- `.codex/skills/playwright/SKILL.md`
- `scripts/run-game-test.sh`
- `scripts/run-game-test-ab.sh`
- `scripts/run-viewer-web.sh`
- `world_viewer_live` (`cargo run -p agent_world --bin world_viewer_live`)
- `doc/world-simulator/viewer-webgl-deferred-compat-2026-02-24.md`

## 测试记录
- card_2026_02_25_12_20_02.md
- card_2026_02_25_13_22_15.md
- card_2026_02_25_16_40_22.md
- card_2026_02_25_22_52_01.md
- card_2026_02_26_11_41_06.md
- card_2026_02_26_13_10_04.md
- card_2026_02_26_16_27_06.md
- card_2026_02_27_13_23_51.md
- card_2026_02_27_14_44_41.md
- card_2026_02_27_15_05_28.md
- card_2026_02_27_16_06_17.md
- 录屏/截图产物：`output/playwright/playability/20260225-132109/`
- 录屏/截图产物：`output/playwright/playability/20260225-163706/`
- 录屏/截图产物：`output/playwright/playability/20260225225029/`
- 录屏/截图产物：`output/playwright/playability/20260226-114106/`
- 录屏/截图产物：`output/playwright/playability/20260226-131004/`
- 录屏/截图产物：`output/playwright/playability/20260226-162706/`
- 录屏/截图产物：`output/playwright/playability/20260227-132351/`
- 录屏/截图产物：`output/playwright/playability/20260227-144441/`
- 录屏/截图产物：`output/playwright/playability/20260227-150528/`
- 录屏/截图产物：`output/playwright/playability/20260227-160617/`
- A/B 量化产物：`output/playwright/playability/20260227-163758/`
- 手册改版记录：`doc/game-test.md`（2026-02-27，用户标注“你不能改这个文档”）
- 开发排查复现：
  - `output/playwright/viewer/webgl-deferred-disable-verify2-20260225-143042/`
  - `output/playwright/viewer/webgl-panic-locate-20260225-143645/`

## 状态
- 当前阶段：已完成玩家复测 + 开发者排查 + 默认链路复测 + 夜间追加复测 + 本轮日间追加复测 + 本轮午后追加复测 + 本轮傍晚追加复测 + 本轮追加复测 + 本轮下午追加复测 + 本轮黄昏追加复测 + 本轮傍晚新增复测 + A/B 量化脚本化复测 + 用户手册改版同步（2026-02-27 16:48）
- 风险：
  - 运行前置：默认开启 LLM 后，若环境缺失可用 LLM 配置，`run-game-test.sh` 可能在启动阶段失败；可临时使用 `--no-llm` 回退脚本决策。
  - 基线问题：Web 端偶发 `copy_deferred_lighting_id_pipeline`（`wgpu` Validation Error）导致崩溃。
  - 架构约束：`CopyDeferredLightingIdPlugin` 与 `Core3d` render graph 存在硬耦合，单独禁用会触发新的启动 panic（`Option::expect` -> `RuntimeError: unreachable`）。
  - 可玩性闭环问题：夜间复测再次出现 `connectionStatus=connecting` 且 `tick` 持续 `0`，并伴随 WebGL `CONTEXT_LOST_WEBGL`，玩法闭环仍不可用。
  - 本轮新增：`ws://127.0.0.1:5010` 链路下仍复现 `connectionStatus=connecting`/`tick=0`，并出现 `WebSocket opening handshake timed out` 与 wasm panic（`assertion failed: old_size > 0`）。
  - 本轮新增：使用 `scripts/run-game-test.sh --web-bind 127.0.0.1:5311` 链路时可达 `connectionStatus=connected`，但 `tick` 仍停留 `0`，执行 `runSteps(20)` 触发 `RuntimeError: unreachable` + wasm panic（`assertion failed: old_size > 0`），玩法仍不可持续。
  - 缓解：新增 `scripts/run-game-test.sh` 固化启动参数（默认 `--web-bind 127.0.0.1:5011`），降低测试因手工参数错误导致的假故障概率。
  - 本轮观测：`scripts/run-game-test.sh --web-bind 127.0.0.1:6011` 链路下 `connectionStatus=connected`，`tick` 从 `8` 增长到 `41`，`runSteps(20)` 返回 `null`，未再出现 wasm panic；当前主要剩余体验问题是目标引导不足与 `sendControl` 入参语义不清（警告：`sendControl ignored`）。
  - 本轮观测：默认 `scripts/run-game-test.sh` 链路（`ws://127.0.0.1:5011`）下 `connectionStatus=connected`，`tick` 从 `10` 增长到 `49`，`runSteps(25)` 返回 `null`，可玩闭环维持可用；但 `sendControl ignored` 警告仍在，玩家输入语义提示仍需优化。
  - 本轮观测：默认 `scripts/run-game-test.sh` 链路（`ws://127.0.0.1:5011`）下 `connectionStatus=connected`，`tick` 从 `28` 增长到 `287`，`runSteps(25)` 返回 `null`，无致命错误；但 `sendControl ignored` 与 `AudioContext` 警告仍在，控制语义与交互提示仍需加强。
  - 本轮观测：默认 `scripts/run-game-test.sh` 链路（`ws://127.0.0.1:5011`）下 `connectionStatus=connected`，`tick` 从 `24` 增长到 `40`，无崩溃且 `runSteps(30)` 返回 `null`；但后半段 `runSteps(25)` 后 `tick` 停留 `40`，同时出现 `sendControl ignored` 与 `AudioContext` 警告，说明控制语义与停滞反馈仍需强化。
  - 本轮观测：默认 `scripts/run-game-test.sh` 链路（`ws://127.0.0.1:5011`）下 `connectionStatus=connected`，`tick` 从 `6` 增长到 `28`，`sendControl(\"play\", {})` 可接受并返回“world advanced”反馈；`move` 动作被拒并明确提示可用动作集合（`play/pause/step/seek/seek_event`），可解释性提升，但 UI 内动作发现路径仍偏弱。
  - 本轮观测：A/B 量化脚本 `scripts/run-game-test-ab.sh` 可稳定产出 `ab_metrics.json/md` 与卡片量化片段；在 `output/playwright/playability/20260227-163758/` 这轮中捕获到 `tick=0` 持续停滞（TTFC=`null`，有效控制命中率 `0/3`，无进展窗口 `6162ms`），可用于快速识别“连接可达但不可推进”故障。
  - 文档约束：`doc/game-test.md` 当前为用户锁定版本（标题含“你不能改这个文档”）；后续流程优化应通过新增外围设计/项目文档承载，避免直接改手册冲突。
- 最近更新：2026-02-27
