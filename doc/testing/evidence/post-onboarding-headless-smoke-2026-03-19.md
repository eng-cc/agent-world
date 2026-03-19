# PostOnboarding 无 UI 协议烟测证据（2026-03-19）

审计轮次: 1

## Meta
- 关联专题: `#46 / PRD-GAME-007`
- 关联任务: `TASK-GAME-022` / `TASK-GAMEPLAY-POD-005`
- 责任角色: `qa_engineer`
- 结论: `pass`
- 目标: 在无浏览器 UI 的前提下，验证 `PostOnboarding` 切换所依赖的 live viewer 协议信号、同会话推进与 runtime 事件流仍然成立。

## 执行命令
- `./scripts/viewer-post-onboarding-headless-smoke.sh --bundle-dir output/release/game-launcher-local --no-llm --viewer-port 4273 --web-bind 127.0.0.1:5111 --live-bind 127.0.0.1:5123 --chain-status-bind 127.0.0.1:5231`

## 产物路径
- `output/playwright/playability/post-onboarding-headless-20260319-101444/post-onboarding-headless-summary.json`
- `output/playwright/playability/post-onboarding-headless-20260319-101444/post-onboarding-headless-summary.md`
- `output/playwright/playability/post-onboarding-headless-20260319-101444/viewer-protocol-transcript.jsonl`
- `output/playwright/playability/post-onboarding-headless-20260319-101444/snapshot-initial.json`
- `output/playwright/playability/post-onboarding-headless-20260319-101444/snapshot-feedback.json`
- `output/playwright/playability/post-onboarding-headless-20260319-101444/snapshot-followup.json`

## 关键结论
- live TCP `hello_ack.control_profile` 为 `live`，说明当前验证链路确实走在线控制协议，而非回放模式。
- 同一 TCP 会话内 `request_snapshot -> step(8) -> request_snapshot -> step(24) -> request_snapshot` 全部成功，两个 `control_completion_ack.status` 都为 `advanced`。
- 快照时间从 `1 -> 9 -> 33` 推进，说明无 UI 模式下依旧能稳定拿到与 `#46` 同源的阶段推进输入。
- 事件流非空，且包含 `RuntimeEvent`，证明 `PostOnboarding` 目标卡所消费的 runtime feed 在无 UI 链路中仍然存在。

## 边界说明
- 该烟测只证明 `#46` 的无 UI 前置条件和协议信号成立，不证明 Mission HUD / `PostOnboarding` 卡片已经渲染到屏幕。
- 屏幕语义、截图和人工复核仍以 `scripts/viewer-post-onboarding-qa.sh` 及 `doc/playability_test_result/card_2026_03_19_09_40_56.md` 为准。
