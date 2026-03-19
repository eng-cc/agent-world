# 纯 API parity 验证证据（2026-03-19）

审计轮次: 1

## Meta
- 关联专题: `PRD-GAME-008 / 纯 API 客户端等价玩法`
- 关联任务: `TASK-GAMEPLAY-API-004`
- 责任角色: `qa_engineer`
- 当前结论: `playable, but parity_verified blocked`
- 目标: 用可复跑的 required/full 证据与 parity matrix 区分“纯 API 已可持续基础游玩”与“纯 API 已达到 UI 等价”。

## 后续修复跟进（2026-03-19）
- `runtime_engineer` 已修复 `world_pure_api_client reconnect-sync --with-snapshot`，现在会显式补拉 `snapshot`，恢复结果可直接携带 `latest_snapshot + player_gameplay`。
- `viewer_engineer` 已把 Mission HUD / PostOnboarding 主卡改为优先消费 canonical `snapshot.player_gameplay`；只有缺少协议级玩家语义时才回退旧事件聚合逻辑。
- 修复后追加 source required 回归：
  - `./scripts/world-pure-api-parity-smoke.sh --tier required --no-llm --viewer-port 4283 --web-bind 127.0.0.1:5123 --live-bind 127.0.0.1:5133 --chain-status-bind 127.0.0.1:5243`
  - 产物：`output/playwright/playability/pure-api-required-20260319-135315/`
  - 新增检查 `recovery_snapshot_present / recovery_player_gameplay_present` 均已通过。
- 定向验证：
  - `env -u RUSTC_WRAPPER cargo test -q -p agent_world --bin world_pure_api_client`
  - `env -u RUSTC_WRAPPER cargo test -q -p agent_world_viewer build_player_post_onboarding_snapshot_prefers_canonical_player_gameplay_snapshot`
  - `env -u RUSTC_WRAPPER cargo test -q -p agent_world_viewer build_player_post_onboarding_snapshot_uses_canonical_blocker_fields`
  - `env -u RUSTC_WRAPPER cargo check -q -p agent_world -p agent_world_viewer`
  - `env -u RUSTC_WRAPPER cargo build -q -p agent_world --bin world_pure_api_client`
  - `target/debug/world_pure_api_client --addr 127.0.0.1:5132 reconnect-sync --player-id player-api-smoke --with-snapshot`

## 执行命令
- source required-tier 预跑:
  - `./scripts/world-pure-api-parity-smoke.sh --tier required --no-llm --viewer-port 4277 --web-bind 127.0.0.1:5117 --live-bind 127.0.0.1:5127 --chain-status-bind 127.0.0.1:5237`
- fresh bundle required-tier 正式口径:
  - `./scripts/build-game-launcher-bundle.sh --out-dir output/release/game-launcher-local`
  - `./scripts/world-pure-api-parity-smoke.sh --tier required --bundle-dir output/release/game-launcher-local --no-llm --viewer-port 4278 --web-bind 127.0.0.1:5118 --live-bind 127.0.0.1:5128 --chain-status-bind 127.0.0.1:5238`
- bundle full-tier 抽样:
  - `./scripts/world-pure-api-parity-smoke.sh --tier full --bundle-dir output/release/game-launcher-local --no-llm --viewer-port 4279 --web-bind 127.0.0.1:5119 --live-bind 127.0.0.1:5129 --chain-status-bind 127.0.0.1:5239`

## 产物路径
- source required-tier:
  - `output/playwright/playability/pure-api-required-20260319-131630/`
- bundle required-tier:
  - `output/playwright/playability/pure-api-required-20260319-132259/`
- bundle full-tier:
  - `output/playwright/playability/pure-api-full-20260319-132316/`
- 对应启动日志:
  - `output/playwright/playability/startup-20260319-131630/`
  - `output/playwright/playability/startup-20260319-132300/`
  - `output/playwright/playability/startup-20260319-132317/`

## required/full 结果摘要
- source required-tier:
  - `initial_stage=first_session_loop`
  - `followup_stage=post_onboarding`
  - `followup_goal_id=post_onboarding.establish_first_capability`
  - `reconnect_sync_ack=catch_up_ready`
- bundle required-tier:
  - 同样通过 `hello_live_profile / step_a_advanced / step_b_advanced / followup_stage_post_onboarding / reconnect_sync_ack`
  - `followup_time=33`
- bundle full-tier:
  - 额外 `step_c_advanced=true`
  - 长步进后仍能继续读取 `player_gameplay`
  - 但 `goal_id` 仍停留在 `post_onboarding.establish_first_capability`，未证明已到“首个持续能力里程碑”

## UI/API parity matrix
| 检查项 | Web/UI 当前状态 | Pure API 当前状态 | 结论 |
| --- | --- | --- | --- |
| 阶段 / 目标 / 进度 / 阻塞 / 下一步建议字段存在 | UI 的 Mission HUD / PostOnboarding 主卡已优先消费 `snapshot.player_gameplay` canonical 语义 | API 已由 `player_gameplay` canonical 字段直接提供 | `pass` |
| 基础推进动作（snapshot / step / play） | 可用 | `world_pure_api_client` 与 `player_gameplay.available_actions` 已可用 | `pass` |
| 最近控制反馈 | UI 可见 | `recent_feedback + control_completion_ack` 已可读 | `pass` |
| 阶段承接（FirstSessionLoop -> PostOnboarding） | UI 已有 headed / no-UI 证据 | source 与 fresh bundle pure API 都已通过 | `pass` |
| 恢复面（重连后直接恢复阶段语义） | UI 重连后可再次看到任务卡 | `reconnect-sync --with-snapshot` 现已直接返回 recovery ack + snapshot + `player_gameplay` | `pass` |
| 正式玩家动作面（聊天 / prompt control / 签名链路） | UI 已有入口 | pure API 客户端已有命令与签名链路，但本轮 no-LLM required/full 未覆盖 LLM 模式真实闭环 | `watch` |
| 到达首个持续能力里程碑 | UI 工业链已有单独专题卡组与 evidence | pure API required/full 本轮只证明到 `PostOnboarding`，未证明达到首个持续能力里程碑 | `block` |
| UI/API 语义是否共用单一事实源 | UI 与 API 已共用 canonical `snapshot.player_gameplay` 作为 PostOnboarding 主语义源 | API 已改为 canonical `player_gameplay` | `pass` |

## QA 判定
- 当前 pure API 路径已经不是 `observer_only`。
  - 原因: 它已经能直接读取 canonical 玩家语义、列出基础动作、推进 live 世界，并拿到 recovery ack。
- 当前 pure API 路径也还不能记为 `parity_verified`。
  - 原因 1: required/full 证据尚未证明 pure API 客户端能到达“首个持续能力里程碑”。
  - 原因 2: 当前 no-LLM 纯 API 长玩仍缺正式动作面来把 `PostOnboarding` 从 `establish_first_capability` 推进到首个持续能力里程碑。

## 结论
- 当前 parity level: `playable`
- 当前 parity verdict: `blocked for parity_verified`
- release / 对外口径:
  - 可以说“纯 API 客户端已经具备基础可玩路径与 canonical 玩家语义”
  - 不能说“纯 API 与 UI 已经达到完整玩法等价”

## 下一步建议
- `runtime_engineer` / `agent_engineer`: 为 no-LLM pure API 补齐能推进首个持续能力里程碑的正式动作面，而不是只剩 `step/play`
- `qa_engineer`: 在动作面补齐后，追加一条 pure API 长玩回归，证明能到达首个持续能力里程碑，再决定是否升级到 `parity_verified`
