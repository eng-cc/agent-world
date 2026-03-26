# hosted world 浏览器访问与会话鉴权证据（2026-03-26）

审计轮次: 1

## Meta
- 关联专题: `PRD-P2P-023-B/C/D/E`
- 关联任务: `TASK-P2P-041-B/C/D/E`
- 责任角色: `qa_engineer`
- 协作角色: `viewer_engineer`
- 当前结论: `pass`
- 目标: 用真实 browser session 固化 hosted public join 的 `Session Ladder`、`Hosted Action Matrix`、`Asset / Governance Lane` 与 `Hosted Recovery` 页面真值，并验证 `pending_registration_ttl_ms` / `release_token` / strong-auth grant route 不会把资产动作误放行。

## 最终结论
- `software_safe` 页面已能在 hosted public join 下稳定暴露三类玩家面真值：
  - `guest_session -> player_session -> strong_auth` 梯度
  - `Hosted Action Matrix` 的真实 `action_id / required_auth / availability`
  - `Asset / Governance Lane` 与 `Hosted Recovery` 的玩家可见阻断/恢复提示
- 浏览器侧证据已确认四个关键约束同时成立：
  - `main_token_transfer` 仍是 `blocked_until_strong_auth`
  - `prompt_control_*` 只处于 `public_player_plane_with_backend_reauth_preview`
  - 未完成 runtime register 的 `player_session` 会在 `pending_registration_ttl_ms=30000` 后失效，旧 `release_token` 不能继续申请 grant
  - 即便带正确 `approval_code`，`main_token_transfer` 仍返回 `strong_auth_action_not_enabled`
- 当前 local hosted 栈的 runtime starvation 已在后续补丁中关闭：
  - fresh `oasis7_game_launcher --deployment-mode hosted_public_join --chain-disable` 栈上，`software_safe` 页面现在可稳定看到 5 个 seeded agents，`debugViewer=debug_viewer:subscribed`
  - 浏览器保持打开时，`/api/public/player-session/admission.runtime_probe_status` 仍可维持 `ok`，不再掉回 `runtime probe timed out waiting for hello_ack`
  - 当前剩余缺口已收敛为 hosted auth 产品证据本身：还没有一份“player_session + backend strong-auth grant + runtime-attached prompt_control success”完整放行证据

## 执行命令
- 本地 hosted 栈:
  - `env OASIS7_HOSTED_STRONG_AUTH_PUBLIC_KEY=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa OASIS7_HOSTED_STRONG_AUTH_PRIVATE_KEY=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb OASIS7_HOSTED_STRONG_AUTH_APPROVAL_CODE=preview-code env -u RUSTC_WRAPPER cargo run -q -p oasis7 --bin oasis7_game_launcher -- --deployment-mode hosted_public_join --viewer-static-dir crates/oasis7_viewer --viewer-host 127.0.0.1 --viewer-port 4186 --web-bind 127.0.0.1:5112 --live-bind 127.0.0.1:5124 --chain-disable --no-open-browser`
- 浏览器会话:
  - `agent-browser --session hosted-p2p-041-evidence2 open 'http://127.0.0.1:4186/software_safe.html?...&ws=ws://127.0.0.1:5112&render_mode=software_safe&test_api=1'`
  - `agent-browser --session hosted-p2p-041-evidence2 get text body`
  - `ab_eval hosted-p2p-041-evidence2 'window.__AW_TEST__.getState()'`
  - `ab_eval hosted-p2p-041-evidence2 'window.__AW_TEST__.logoutHostedPlayerSession().then(() => window.__AW_TEST__.getState())'`
  - `ab_eval hosted-p2p-041-evidence2 'window.__AW_TEST__.retryHostedPlayerIdentityIssue().then(() => window.__AW_TEST__.getState())'`
  - `ab_eval hosted-p2p-041-evidence2 'window.__AW_TEST__.setStrongAuthApprovalCode("preview-code"); window.__AW_TEST__.getState()'`
- 辅助 admission 取样:
  - `curl -s http://127.0.0.1:4186/api/public/player-session/admission | python3 -m json.tool`
  - follow-up: `agent-browser --session hosted-p2p-041-concurrency open 'http://127.0.0.1:4388/?ws=ws://127.0.0.1:5315'`
  - follow-up: `curl -s http://127.0.0.1:4388/api/public/player-session/admission | python3 -m json.tool`

## 浏览器证据
### 1. player session 页面真值
- `__AW_TEST__.getState()` 在 player lane 返回:
  - `authTier=player_session`
  - `authSource=hosted_player_issue+browser_local_ephemeral_key`
  - `authRegistrationStatus=issued`
  - `debugViewerStatus=detached`
  - `hostedAdmission.active_player_sessions=1`
- 页面文本同时显示:
  - `Release Hosted Player Session`
  - `prompt=enabled`
  - `chat=enabled`
  - `mainToken=strong_auth_required`
  - `Hosted Action Matrix`
  - `Asset / Governance Lane`
- 对应截图:
  - `output/playwright/hosted-p2p-041/hosted-player-session-issued.png`

### 2. recovery / guest 回落页面真值
- 触发 `logoutHostedPlayerSession()` 后，`__AW_TEST__.getState()` 返回:
  - `authTier=guest_session`
  - `authError=hosted player session released locally`
  - `hostedRecoveryHint.kind=released`
  - `hostedRecoveryHint.cta=Acquire Hosted Player Session`
  - `hostedActionMatrix[*].code` 对 `gameplay_action/agent_chat/prompt_control_*` 统一降为 `auth_level_insufficient`
- 页面文本同时显示:
  - `Hosted Recovery`
  - `Hosted player session released`
  - `Acquire Hosted Player Session`
  - `main_token_transfer` 仍是 `strong_auth_required`
- 对应截图:
  - `output/playwright/hosted-p2p-041/hosted-recovery-guest.png`

### 3. pending registration TTL / release_token 失效
- 在 detached world 下，browser-local `player_session` 长时间停留在 `issued_pending_register`
- 之后再次用旧 `release_token` 请求 `/api/public/strong-auth/grant`，页面内 fetch 返回:
  - `ok=false`
  - `error_code=release_token_invalid`
  - `error=release_token does not map to an active player slot`
  - `admission.active_player_sessions=0`
- 这说明:
  - public issuer 的 `pending_registration_ttl_ms=30000` 正在真实回收未完成 register 的 slot
  - strong-auth grant route 没有绕过 active slot / release token 绑定

### 4. strong-auth route 的浏览器侧阻断真值
- 在同一 browser session 里执行:
  - 先 `logoutHostedPlayerSession()`
  - 再 `retryHostedPlayerIdentityIssue()`
  - 再用新 `player_id/public_key/release_token + approval_code=preview-code` 请求 grant
- 对 `prompt_control_apply`:
  - 返回 `ok=false`
  - `error_code=strong_auth_grant_sign_failed`
  - `error=hosted strong-auth agent_id is empty`
- 对 `main_token_transfer`:
  - 返回 `ok=false`
  - `error_code=strong_auth_action_not_enabled`
  - `error=hosted public join does not enable backend strong-auth grant for action_id 'main_token_transfer' yet`
- 这两条结果说明:
  - preview-grade prompt lane 的确要求更完整的 runtime/agent attach 条件，不会在 detached/agentless 页面上误签发
  - 资产动作仍被 hosted public join 显式阻断，grant route 泛化没有把 `main_token_transfer` 顺带打开

## 风险与剩余项
- 本证据覆盖的是 hosted 浏览器访问与鉴权面，不是“world 已可玩”证明；虽然 follow-up 已确认当前 local 栈能稳定给出 agent/world snapshot，但尚未补齐一份带 hosted auth 升级链路的完整 success evidence。
- 下一轮若要补 `prompt_control_*` success path，需要在同一份带真实 agent 的 hosted 页面上保留 `player_session issue -> grant signed -> runtime verify -> prompt action accepted` 证据，而不是只验证阻断路径。
- `main_token_transfer` 当前仍保持阻断，这符合本专题目标，不构成缺陷；真正开放前仍需后续 `strong_auth` 专题和 custody 方案收口。
