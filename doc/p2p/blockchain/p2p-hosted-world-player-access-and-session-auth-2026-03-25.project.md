# oasis7 hosted world 玩家访问与会话鉴权（项目管理文档）

- 对应设计文档: `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.design.md`
- 对应需求文档: `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.prd.md`

审计轮次: 1
## 任务拆解（含 PRD-ID 映射）
- [x] TASK-P2P-041-A (PRD-P2P-023-A/B/C) [test_tier_required + test_tier_full]: `runtime_engineer` 拆分 public player plane 与 private control plane，冻结 endpoint taxonomy、`/api/gui-agent/action` split 策略、hosted verdict 与 admission control，并移除 hosted-world 公共路径中的浏览器长期 signer bootstrap。
- [ ] TASK-P2P-041-B (PRD-P2P-023-B/D) [test_tier_required + test_tier_full]: `viewer_engineer` 落地 `guest session -> player session` 网页 join/login/reconnect UX，并按 capability 禁用敏感动作。
- [ ] TASK-P2P-041-C (PRD-P2P-023-B/C) [test_tier_required + test_tier_full]: `runtime_engineer` + `agent_engineer` 落地 session 验证、`player_id -> entity` 绑定、resume/revoke 与 ownership 冲突处理。
- [ ] TASK-P2P-041-D (PRD-P2P-023-B/D) [test_tier_required + test_tier_full]: `runtime_engineer` + `viewer_engineer` 落地 `strong auth` 升级链路，覆盖 `main token transfer` 与敏感 prompt/control 动作。
- [ ] TASK-P2P-041-E (PRD-P2P-023-C/E) [test_tier_required + test_tier_full]: `qa_engineer` 建立 hosted-world abuse suite，覆盖 replay、expired session、revocation、operator/public URL 混淆、admission limit 和 capability bypass。
- [ ] TASK-P2P-041-F (PRD-P2P-023-E) [test_tier_required]: `liveops_community` 建立 hosted operator runbook、分享规范、incident/rotation 流程与 claims boundary。

## 角色拆解
### TASK-P2P-041-A / runtime_engineer
- 输入:
  - `crates/oasis7/src/bin/oasis7_web_launcher.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/server.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/control_plane.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/viewer_auth_bootstrap.rs`
  - `crates/oasis7/src/bin/oasis7_game_launcher.rs`
  - `crates/oasis7/src/bin/oasis7_game_launcher/static_http.rs`
  - `crates/oasis7/src/bin/oasis7_hosted_access.rs`
  - `crates/oasis7/src/bin/oasis7_chain_runtime/node_keypair_config.rs`
- 输出:
  - public/private plane endpoint 清单
  - `/api/gui-agent/action` split 方案
  - join admission control 最小契约
  - hosted-world browser signer bootstrap 退场方案
  - required/full 回归入口
- 完成定义:
  - public join 路径不再依赖长期私钥 bootstrap
  - world/control 接口不再作为 public player origin 默认可达面
  - `/api/gui-agent/action` 未拆分前保持 private，拆分后才允许 player-safe 子集进入 public player plane
  - public join 有显式 session issuance / full-world / rate-limit 规则

### TASK-P2P-041-B / viewer_engineer
- 输入:
  - `crates/oasis7_viewer/src/egui_right_panel_chat_auth.rs`
  - `crates/oasis7_viewer/src/viewer_automation.rs`
  - `crates/oasis7_client_launcher/src/transfer_auth.rs`
  - `crates/oasis7_viewer/software_safe.js`
- 输出:
  - join/login/reconnect UX
  - capability-based button state
  - hosted-world 网页错误文案
- 完成定义:
  - guest/player/strong-auth 三档在 UI 明确可见
  - 没有能力时按钮禁用且错误可读

### TASK-P2P-041-C / runtime_engineer + agent_engineer
- 输入:
  - TASK-P2P-041-A endpoint/signer/admission 边界
  - TASK-P2P-041-B 会话与能力模型
- 输出:
  - session validation
  - entity bind/resume/revoke
  - ownership 冲突规则
- 完成定义:
  - 同一玩家实体 ownership 可验证
  - 断线恢复和撤销不会穿透到其他玩家实体

### TASK-P2P-041-D / runtime_engineer + viewer_engineer
- 输入:
  - `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`
- 输出:
  - hosted-world strong-auth action list
  - challenge/proof/verification 路径
  - Web sensitive-action regression
- 完成定义:
  - `main token transfer` 不再通过浏览器长期私钥默认签名
  - prompt/control 类高风险动作必须明确走强鉴权或 private plane

### TASK-P2P-041-E / qa_engineer
- 输入:
  - TASK-P2P-041-A~D 的平面、session、strong-auth 设计
- 输出:
  - abuse suite
  - failure signature
  - block/pass 判定模板
- 完成定义:
  - replay / revoke / expiry / capability bypass / admission limit 有 required/full 证据

### TASK-P2P-041-F / liveops_community
- 输入:
  - TASK-P2P-041-A~E 结论
  - `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`
  - `doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.prd.md`
- 输出:
  - hosted operator runbook
  - incident/rotation/public claims 模板
  - 分享 URL 规范
- 完成定义:
  - hosted world 分享、误分享、撤销和事故通报均有 runbook

## 当前结论
- 当前阶段:
  - 游戏阶段口径: `limited playable technical preview`
  - 安全阶段口径: `crypto-hardened preview`
  - hosted-world player access verdict: `specified_not_implemented`
- 已实现的 `TASK-P2P-041-A` P0 收口:
  - `oasis7_game_launcher --deployment-mode hosted_public_join` 会停止向公开 viewer HTML 注入长期 signer bootstrap。
  - `oasis7_web_launcher --deployment-mode hosted_public_join` 会把 `/api/state`、`/api/start`、`/api/stop`、`/api/chain/start`、`/api/chain/stop`、`/api/gui-agent/*` 与 console static 路径收口为 loopback-only private control plane。
  - 新增 `/api/public/state`，对外只暴露 join 级 public snapshot，不再把 operator state / logs / config 作为默认公共面。
  - launcher snapshot 现已冻结 `deployment_mode`、hosted verdict、`gui-agent` surface 状态与 admission contract 默认值，供后续 viewer/runtime/QA 接续。
- 已实现的 `TASK-P2P-041-B` viewer first slice:
  - `software_safe.js` 现会显式显示 `guest_session / player_session / strong_auth` 梯度、`deploymentHint`、`auth source` 与 reconnect 提示，不再只显示 `auth=ready|missing`。
  - prompt/chat 现在会按 capability 给出结构化禁用原因：至少区分 `guest_session`、`observer_only` 与 `strong_auth_required` 占位，而不是继续用单一 “viewer auth bootstrap is unavailable”。
  - `__AW_TEST__.getState()` 已补 `authTier`、`authSource`、`authDeploymentHint` 与 `authSurface`，便于后续 QA/agent-browser 对 hosted public join 的 session/capability 状态做证据采样。
  - `software_safe.js` 现在会在 `hosted_public_join` 下优先尝试 `GET /api/public/player-session/issue`，拿到 server-issued `player_id` 后由浏览器本地生成/持久化临时 Ed25519 key，并在 page reload 后复用这份本地会话态。
  - viewer 现已消费 `authoritative_recovery register_session/reconnect_sync` ack/error：首次 issue 后会注册 player session，刷新或断线重连后会先走 `reconnect_sync`，若 runtime 返回 `session_not_found` 则回退到显式重新注册。
  - viewer 现已提供显式 `Release Hosted Player Session` 动作：会向 runtime 发送 `revoke_session`，并向同源 public player plane 发送 `/api/public/player-session/release` 归还 active slot，然后清掉浏览器本地持久化的 hosted player session。
  - viewer 现已可直接读取 `/api/public/player-session/admission`，并在 guest lane 显示当前 `activeSlots/issueBudget`；若先前因为 `world_full/rate_limited` 留在 guest，也可以通过显式 `Acquire Hosted Player Session` 动作重试，不必靠发送 chat/prompt 侧向触发。
  - admission snapshot 现还会回出最近一次 runtime probe 看到的 `runtime_bound_player_sessions`；viewer summary 会并排显示 `activeSlots` 和 `runtimeBound`，便于 QA 区分“issuer 占位”和“runtime 真正还绑着几个玩家”。
  - admission snapshot 现还会回出 `runtime_probe_status/runtime_probe_error/last_runtime_probe_unix_ms`；viewer summary 可直接看 runtime probe 当前是 `ok`、`error` 还是尚未启动，不必只靠外部日志猜测 public player plane 是否还在对账。
  - viewer 现已在 hosted player session 注册成功后自动调用 `/api/public/player-session/refresh` 并启动 lease heartbeat；public admission 也会暴露 `slot_lease_ttl_ms`，让 stale slot 可被自动回收，而不是无限占位。
  - viewer 的 lease heartbeat 现会同时发送轻量 `reconnect_sync` 探针；即便玩家空闲不发 chat/prompt，也能周期性发现 runtime 侧的 `session_revoked/session_not_found`，不再只能等下次主动交互才发现被踢/被撤销。
  - viewer summary 与 `__AW_TEST__.getState()` 现已显式暴露 `authRuntimeStatus/authBoundAgentId` 与 recovery error；WebSocket 断线时也会清掉挂起的 `syncInFlight` 并标记 `runtime=disconnected`，避免 `reconnect_sync` 在短断线后卡死不再自动恢复。
- 已实现的 `TASK-P2P-041-C` runtime first slice:
  - runtime-live 新增显式 `session_register`，并要求 prompt/chat/gameplay 在 player action 之前先完成 session 注册；原先“第一个签名动作自动注册 active key”的隐式登录已收口。
  - `RuntimeSessionPolicy::validate_known_session_key` 现会在未注册 session 时返回 `session_not_found`，不再把未注册玩家默认为 epoch 0 放行。
  - runtime 现额外维护 `player_id -> agent_id` 单实体占用真值；同一 player 不能静默切到第二个 agent，必须等待后续显式 rebind 设计。
  - `ReconnectSync` / `SessionRegistered` / `SessionRotated` ack 已带回当前 `agent_id`，`RevokeSession` 会清掉该 player 的绑定与 nonce/replay 痕迹，保持“撤销即失效、需重新注册”的 hosted v1 语义。
  - public player plane 的 `refresh/release` 现已收紧为 `player_id + release_token` 双绑定校验，并补充 `player_id_required/player_id_mismatch` 单测，避免 token-only 误归还或误续租其他玩家 slot。
  - `oasis7_game_launcher` 的 public player plane 现在会启动独立 runtime presence monitor 线程，维持到 `live_bind` 的常驻连接，先订阅 runtime 事件流消费 `AgentPlayerBound` 增量，再周期性 `RequestSnapshot` 做全量纠偏；public route 不再自己临时短连 probe。凡是“曾经已在 runtime 里出现过、现在又从 runtime binding 中消失”的 player slot，会被 issuer 立即回收，并对旧 browser session 返回 `session_revoked`，让 operator kick / remote revoke 能更快回流到 `world_full` 判定。
- 已实现的 `TASK-P2P-041-D` strong-auth barrier first slice:
  - `oasis7_web_launcher` 在 `deployment_mode=hosted_public_join` 下会显式拒绝 `POST /api/chain/transfer`，返回结构化 `strong_auth_required`，不再让 public join 路径继续借用 trusted-local signer bootstrap。
  - `oasis7_game_launcher -> oasis7_viewer_live -> runtime-live` 现已透传 hosted deployment mode；在 `hosted_public_join` 下，`prompt_control preview/apply/rollback` 会统一返回 `strong_auth_required`，避免敏感 prompt/control 继续凭 `player_session` 直接穿过 hosted 公共玩家面。
  - `software_safe.js` 现会把 `prompt_control` 明确标成 `strong_auth_required`：在 hosted public join 推断路径下不再继续展示成“只差 player_session”，在 remote-origin legacy bootstrap 下也不再把 preview bootstrap 误当成 hosted-ready prompt/control 能力。
  - `oasis7_client_launcher` 已把 `deployment_mode` 透传到启动参数，并在转账窗口对 `hosted_public_join` 显示同口径 strong-auth barrier，不再继续尝试 trusted-local signer bootstrap 提交。
  - `/api/public/state` 的 `hosted_access` contract 现已导出结构化 `action_matrix`，明确 `gameplay_action/agent_chat` 仍是 `player_session`，而 `prompt_control_*` 与 `main_token_transfer` 当前属于 `strong_auth` 且在 hosted public join 下为 `blocked_until_strong_auth`。
  - `oasis7_web_launcher`、`oasis7_game_launcher` 与 `oasis7_client_launcher` 的 game URL 现都已附带精简 `hosted_access` hint；`software_safe.js` 会优先消费这个 query contract，而不是继续只靠 hostname 猜 `deploymentHint`；`__AW_TEST__.getState()` 也会直接回出 `hostedAccess` 供 QA 采样。
- 当前 blocker:
  - `guest session -> player session` 的最小 issuer 已落成，且 `max_player_sessions` 已开始在 public issue 面按 active slot + lease TTL 生效；public player plane 现在也会通过独立后台 runtime presence 常驻连接把已消失的历史绑定玩家回收到 issuer slot；但 `world_full/kick_policy` 仍未完全等价于 runtime authoritative presence，因为 runtime 还没有面向所有客户端的通用 unbind/revoke 广播，而且也还没覆盖“刚 issue 尚未 register”“纯 runtime occupancy 无 issuer slot”的全部状态。
  - hosted v1 目前已支持浏览器本地 player session issue + reconnect/register + local release/logout，并能通过周期性 `reconnect_sync` 探针发现部分 remote revoke；但 operator kick 的公开玩家面即时回流、显式 rebind 流程与更稳定的 resume token 仍未收口。
  - `session_register` 目前仍是 runtime-live 内显式注册；host restart / rollback 之后按 v1 规则仍要求重新注册，不是持久化 session registry。
  - 当前只实现了 hosted `strong_auth` barrier first slice，而不是完整 `strong_auth` challenge/proof/verification lane；`main token transfer` 与 `prompt_control` 现在是显式拒绝/禁用而非 hosted-ready 放行。
  - `agent_chat` 仍归 `player_session` 级低风险交互；更细的 hosted action matrix、resume issuer 与真正 strong-auth proof 仍待后续专题收口。
  - hosted operator 目前仅支持 loopback private control plane；远程 operator URL / tunnel / runbook 仍待 `TASK-P2P-041-F` 收口。

## 依赖
- `doc/p2p/prd.md`
- `doc/p2p/project.md`
- `doc/p2p/token/mainchain-token-signed-transaction-authorization-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-production-signer-custody-keystore-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-governance-signer-externalization-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`
- `doc/p2p/blockchain/p2p-shared-network-release-train-minimum-2026-03-24.prd.md`
- `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.prd.md`
- `testing-manual.md`

## 验收命令（TASK-P2P-041-A P0 实装）
- `rg -n "public player plane|private control plane|signer plane|guest session|player session|strong auth|invite-only|gui-agent/action|admission control|specified_not_implemented" doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.prd.md doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.design.md doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.project.md doc/p2p/prd.md doc/p2p/project.md`
- `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_web_launcher --bin oasis7_game_launcher`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前状态: active
- 下一步: 在 `TASK-P2P-041-C` / `TASK-P2P-041-D` 上继续推进，把当前已落的 hosted v1 `player_id issue + browser-local ephemeral key + reconnect/register` 扩到完整 revoke/world-full/admission enforcement，并把 `strong_auth_required` barrier 升级成正式 challenge/proof/verification lane 与动态 action matrix。
- 最近更新: 2026-03-26
