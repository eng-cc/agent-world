# oasis7 hosted world 玩家访问与会话鉴权（Hosted Operator Runbook）

- 对应需求文档: `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.prd.md`
- 对应设计文档: `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.project.md`

审计轮次: 1

## Meta
- Owner Role: `liveops_community`
- Review Role: `producer_system_designer`
- Scope: `hosted_public_join share discipline + operator/public URL boundary + incident first response + public claims`
- Audience: `hosted world host / operator`
- Source Docs:
  - `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.prd.md`
  - `doc/p2p/blockchain/p2p-hosted-world-player-access-and-session-auth-2026-03-25.project.md`
  - `doc/p2p/blockchain/p2p-mainnet-public-claims-policy-2026-03-23.prd.md`

## 1. 适用范围
- 这份 runbook 只覆盖 `deployment_mode=hosted_public_join` 的 hosted world 分享与事故收口。
- 它定义的是 operator 执行方法，不替代 runtime/viewer 的实现细节，也不替代正式账户系统、钱包插件或 invite-only 方案。
- 当前 hosted world 的对外口径仍然是：
  - `limited playable technical preview`
  - `crypto-hardened preview`
  - `hosted-world player access verdict = specified_not_implemented`

## 2. 先认清三类入口
- `public join URL`
  - 给玩家使用。
  - 典型形态是 viewer/game URL，包含公开 `ws` 与 `hosted_access` hint。
- `private control plane`
  - 给 operator 自己使用。
  - 典型是 `oasis7_web_launcher` 的 console / control origin。
  - 当前目标态是 loopback-only；不应该被公网玩家直接访问。
- `signer / approval path`
  - 给敏感动作二次授权使用。
  - 当前只有 preview-grade backend reauth，不是 production custody。

一句话规则：
- 玩家只该拿到 `public join URL`。
- 不要把 operator/control URL 当成玩家分享链接。

## 3. 分享前检查
每次准备把 hosted world 发给别人前，先做下面 6 项：

1. 确认 `deployment_mode` 是 `hosted_public_join`。
2. 确认你准备发出去的是 game/viewer URL，而不是 launcher console 地址。
3. 确认公开页面不会再注入长期 signer bootstrap。
4. 确认 public snapshot 仍显示：
   - `verdict = specified_not_implemented`
   - `main_token_transfer = blocked_until_strong_auth`
5. 确认你没有对外宣称：
   - `hosted-ready`
   - `production-ready`
   - `safe to share with anyone`
6. 如果你走了反向代理或 tunnel，确认公网只暴露玩家 join 面，不暴露 operator/control 面。

## 4. 正确分享方法
- 对外只分享 game/viewer join URL。
- 推荐同时补一句说明：
  - `这是一个 limited playable technical preview。`
  - `如果你能打开页面并进入世界，说明你拿到的是玩家入口，不是 operator 管理入口。`
- 不要分享：
  - launcher console URL
  - 任意 `/api/state`
  - 任意 `/api/start`、`/api/stop`
  - 任意 `/api/chain/start`、`/api/chain/stop`
  - 任意 `/api/gui-agent/*`

## 5. 如何判断自己分享错了
下面任一条成立，都按“误分享 operator URL / operator 面暴露”处理：

- 访客反馈自己打开的是控制台、管理面或非游戏页面。
- 访客能直接命中 `/api/state`、`/api/start|stop`、`/api/chain/start|stop`、`/api/gui-agent/*`。
- 你发现自己发出去的是 launcher/control origin，而不是 game/viewer URL。
- 反向代理或 tunnel 把 operator/control 面一起暴露到了公网。

## 6. 误分享后的第一响应
按顺序执行，不要跳步：

1. 立即停止继续传播错误链接。
2. 撤回或替换所有公开帖子、群消息、文档中的错误 URL。
3. 暂停对外新增玩家流量，直到确认公网只剩 join URL。
4. 重新自查：
   - 远端访问 private-control-plane 应返回 `operator_plane_only`
   - 公开返回体只能带 public snapshot，不应带 operator state / logs / config
5. 如果无法确认是否有人已经命中过私有面：
   - 先按 incident 处理
   - 暂停 public claims
   - 重新开一个干净的分享窗口再恢复

## 7. 最小 Incident 记录
每次误分享或疑似暴露，至少记录以下字段：

- `incident_id`
- `discovered_at`
- `who_found_it`
- `shared_url`
- `intended_join_url`
- `exposed_surface`
- `publicly_visible_duration`
- `immediate_actions`
- `claims_frozen`
- `follow_up_owner`

推荐把结论同步回：
- `doc/devlog/YYYY-MM-DD.md`
- 当前 topic 的 `project.md`

## 8. 何时必须 Freeze 对外口径
出现下面任一情况，立刻冻结对外升级口径，只保留 preview 表述：

- operator/control 面被公网直接访问
- 浏览器或公开 API 返回里出现长期 signer 真值
- 玩家入口与 operator 入口已经无法稳定区分
- hosted strong-auth 或 session 边界出现未解释的穿透

冻结后对外只允许说：
- `当前仍是 limited playable technical preview。`
- `hosted access hardening is in progress。`
- `operator boundary issue is being corrected before wider sharing。`

## 9. 当前已知边界
- 当前不支持 invite-only 作为基础安全方案。
- 当前 `main_token_transfer` 仍不能通过 hosted public join 放行。
- 当前 hosted `prompt_control` 只是 preview-grade backend reauth，不是 production custody。
- 当前 operator 仍以 loopback private control plane 为主；远程 operator URL / tunnel 策略还没有正式完成版。

## 10. 当前推荐执行法
- 小范围分享时：
  - 只发 join URL
  - 不发 operator/control URL
  - 不升级 public claims
- 如果要公开到更大范围：
  - 先完成 QA first slices
  - 再补 operator runbook 演练记录
  - 再由 `producer_system_designer` 决定是否扩大分享范围

## 11. 下一步待补
- 更接近真实部署的 tunnel / reverse proxy 配置示例
- operator mis-share incident 模板
- hosted share announcement / correction 模板
- 远程 operator 值班与 session revoke 实操步骤
