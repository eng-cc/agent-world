# hosted world 浏览器 revoke recovery 证据（2026-03-27）

审计轮次: 1

## Meta
- 关联专题: `PRD-P2P-023-B/C/E`
- 关联任务: `TASK-P2P-041-B/C/E/F`
- 责任角色: `qa_engineer`
- 协作角色: `runtime_engineer`, `liveops_community`
- 当前结论: `pass`
- 目标: 用真实 hosted browser session 固化 `operator kick / remote revoke` 之后，公开玩家面会在下一轮 `reconnect_sync` 心跳里掉回 guest，并把 `revoke_reason/revoked_by` 结构化展示到 `Hosted Recovery`。

## 最终结论
- runtime operator 侧下发 `revoke_session` 后，浏览器不会继续静默持有旧 `player_session`。
- 在同一页面等待下一轮 hosted heartbeat 后，`software_safe` 会稳定收敛到:
  - `authTier = guest_session`
  - `authError = session_revoked: ... is revoked`
  - `authRevokeReason = operator_kick_for_drill`
  - `authRevokedBy = hosted-revoke-operator`
  - `hostedRecoveryHint.kind = revoked`
  - `hostedRecoveryHint.cta = Re-acquire Hosted Player Session`
- 页面正文也会直接显示:
  - `Hosted player session was revoked`
  - `The runtime or operator revoked this browser session by hosted-revoke-operator. Reason: operator_kick_for_drill.`
  - `Re-acquire Hosted Player Session`

## 执行命令
- 预编 runtime sibling bin:
  - `env -u RUSTC_WRAPPER cargo build -q -p oasis7 --bin oasis7_viewer_live`
- 启动本地 hosted 栈:
  - `env OASIS7_HOSTED_STRONG_AUTH_PUBLIC_KEY=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa OASIS7_HOSTED_STRONG_AUTH_PRIVATE_KEY=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb OASIS7_HOSTED_STRONG_AUTH_APPROVAL_CODE=preview-code env -u RUSTC_WRAPPER cargo run -q -p oasis7 --bin oasis7_game_launcher -- --deployment-mode hosted_public_join --viewer-static-dir crates/oasis7_viewer --viewer-host 127.0.0.1 --viewer-port 6201 --web-bind 127.0.0.1:6202 --live-bind 127.0.0.1:6203 --chain-disable --no-open-browser`
- 打开浏览器页面:
  - `source scripts/agent-browser-lib.sh && ab_open hosted-revoke-evidence 0 'http://127.0.0.1:6201/software_safe.html?ws=ws%3A%2F%2F127.0.0.1%3A6202&hosted_access=%7B%22deployment_mode%22%3A%22hosted_public_join%22%7D&test_api=1'`
  - `source scripts/agent-browser-lib.sh && ab_eval hosted-revoke-evidence 'window.__AW_TEST__.getState()'`
- operator 侧下发 revoke:
  - `env -u RUSTC_WRAPPER cargo run -q -p oasis7 --bin oasis7_pure_api_client -- --addr 127.0.0.1:6203 --client hosted-revoke-operator revoke-session --player-id hosted-player-0000019d2d3c4a63-00000003 --session-pubkey 5c9228b48496775b2a3410f742e4b20ff31ef9e6e9307bbdf5ab756d77cba4da --revoke-reason operator_kick_for_drill`
- 等待浏览器下一轮 hosted heartbeat:
  - `sleep 35`
  - `source scripts/agent-browser-lib.sh && ab_eval hosted-revoke-evidence 'window.__AW_TEST__.getState()'`
  - `source scripts/agent-browser-lib.sh && ab_run hosted-revoke-evidence get text body`
  - `source scripts/agent-browser-lib.sh && ab_run hosted-revoke-evidence screenshot output/playwright/hosted-p2p-041/hosted-revoke-recovery.png`

## operator 侧佐证
- `oasis7_pure_api_client` 返回的 `authoritative_recovery_ack` 为:
  - `status = session_revoked`
  - `message = operator_kick_for_drill`
  - `revoke_reason = operator_kick_for_drill`
  - `revoked_by = hosted-revoke-operator`
  - `session_epoch = 2`

## 浏览器证据
### 1. revoke 前
- 首轮 `__AW_TEST__.getState()` 返回:
  - `authTier = player_session`
  - `authRegistrationStatus = registered`
  - `authRuntimeStatus = registered_unbound`
  - `authReady = true`
  - `authPlayerId = hosted-player-0000019d2d3c4a63-00000003`
  - `authPublicKey = 5c9228b48496775b2a3410f742e4b20ff31ef9e6e9307bbdf5ab756d77cba4da`

### 2. revoke 后下一轮 heartbeat
- 等待 35s 后，`__AW_TEST__.getState()` 返回:
  - `authTier = guest_session`
  - `authReady = false`
  - `authSource = hosted_access_hint`
  - `authError = session_revoked: player hosted-player-0000019d2d3c4a63-00000003 session_pubkey 5c9228b48496775b2a3410f742e4b20ff31ef9e6e9307bbdf5ab756d77cba4da is revoked`
  - `authRevokeReason = operator_kick_for_drill`
  - `authRevokedBy = hosted-revoke-operator`
  - `hostedAdmission.active_player_sessions = 0`
  - `hostedAdmission.released_players_total = 3`
- `hostedRecoveryHint` 返回:
  - `kind = revoked`
  - `title = Hosted player session was revoked`
  - `cta = Re-acquire Hosted Player Session`
  - `detail = The runtime or operator revoked this browser session by hosted-revoke-operator. Reason: operator_kick_for_drill. You need to acquire a fresh hosted player session before gameplay, chat, or prompt actions can continue.`

### 3. 页面可见文案
- `agent-browser get text body` 直接看到:
  - `Hosted Recovery`
  - `Hosted player session was revoked`
  - `The runtime or operator revoked this browser session by hosted-revoke-operator. Reason: operator_kick_for_drill.`
  - `Re-acquire Hosted Player Session`

## 产物
- 页面截图:
  - `output/playwright/hosted-p2p-041/hosted-revoke-recovery.png`

## 风险与剩余项
- 本证据证明的是“remote revoke / operator kick 的公开玩家面即时回流”已经可见，不代表 hosted handoff / batch kick / mass incident runbook 已全部完成。
- 当前 heartbeat 仍是固定间隔探针，因此玩家侧感知 revoke 的时间上界仍受 probe 周期影响；若后续要进一步缩短体感时延，仍应继续推进更直接的 presence / revoke push 方案。
