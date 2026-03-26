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
  - `crates/oasis7/src/bin/oasis7_web_launcher/control_plane.rs`
  - `crates/oasis7/src/bin/oasis7_web_launcher/viewer_auth_bootstrap.rs`
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
- 当前 blocker:
  - browser 仍会收到长期 signer bootstrap
  - public player access 与 control plane 仍未拆分
  - Web 敏感动作尚未切到 session/capability/strong-auth 模型

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

## 验收命令（TASK-P2P-041-A 方案冻结）
- `rg -n "public player plane|private control plane|signer plane|guest session|player session|strong auth|invite-only|gui-agent/action|admission control|specified_not_implemented" doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.prd.md doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.design.md doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.project.md doc/p2p/prd.md doc/p2p/project.md`
- `./scripts/doc-governance-check.sh`
- `git diff --check`

## 状态
- 当前状态: active
- 下一步: 执行 `TASK-P2P-041-A`，先把 public join 与 private control 做硬拆分，冻结 `/api/gui-agent/action` split 与 admission control，并让 hosted-world 浏览器路径停止接收长期 signer bootstrap。
- 最近更新: 2026-03-25
