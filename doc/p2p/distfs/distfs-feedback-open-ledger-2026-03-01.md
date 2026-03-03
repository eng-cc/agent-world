# Agent World Runtime：DistFS 公开反馈账本（2026-03-01）设计文档

## 目标
- 在不引入数据库的前提下，基于 `agent_world_distfs` 落地可运行的玩家反馈系统。
- 支持公开写入与公开读取，同时保证反馈记录 append-only，不允许覆盖原始反馈。
- 支持作者控制：反馈追加与删除（tombstone）必须由原作者公钥签名授权。
- 支持基础 anti-abuse：IP 与 pubkey 双限流、内容大小限制、附件数量与大小限制。
- 支持审计：所有变更事件写入审计日志，便于后续排障与治理。

## 范围

### In Scope
- `crates/agent_world_distfs` 新增反馈模块（核心存储与校验逻辑）：
  - feedback create / append / tombstone。
  - Ed25519 签名校验（pubkey + signature）。
  - nonce 重放保护（同 pubkey + nonce 只允许一次）。
  - 基于审计日志的 IP/pubkey 时间窗限流。
  - 公开读接口（list / get）。
- 新增 CLI 工具用于本地闭环调试：
  - `submit` / `append` / `tombstone` / `list` / `read`。
- 单元测试覆盖核心闭环与关键拒绝路径。

### Out of Scope
- 内容审核与 AI 风险过滤。
- 多节点跨机复制一致性策略优化。
- 附件二进制上传与媒体转码流程。
- 权限分级读（本期按需求采用全量公开读）。

## 接口 / 数据

### 目录布局（DistFS FileStore）
- `feedback/records/<feedback_id>/root.json`：反馈创建根记录（不可变）。
- `feedback/records/<feedback_id>/events/<event_id>.json`：追加/删除事件（append-only）。
- `feedback/nonces/<pubkey>/<nonce>.json`：nonce 占位，防重放。
- `feedback/audit/<audit_id>.json`：审计日志（append-only）。

### 关键数据结构（草案）
```rust
FeedbackRootRecord {
  feedback_id: String,
  author_public_key_hex: String,
  created_at_ms: i64,
  content: String,
  category: String,
  platform: String,
  game_version: String,
  attachments: Vec<FeedbackAttachment>,
  signature_hex: String,
  nonce: String,
  expires_at_ms: i64,
}

FeedbackEvent {
  feedback_id: String,
  event_id: String,
  action: "append" | "tombstone",
  actor_public_key_hex: String,
  created_at_ms: i64,
  payload: Option<String>,
  reason: Option<String>,
  signature_hex: String,
  nonce: String,
  expires_at_ms: i64,
}
```

### 签名与防重放
- 仅支持 `Ed25519`。
- 签名 payload 使用 canonical CBOR 编码，固定包含：
  - `action`
  - `feedback_id`
  - `content_hash`（或 tombstone reason hash）
  - `nonce`
  - `timestamp_ms`
  - `expires_at_ms`
- 校验规则：
  - `now_ms <= expires_at_ms`。
  - `nonce` 未被该 pubkey 使用过。
  - 签名与 pubkey 可验证。

### 限流策略（基础）
- 时间窗：`rate_limit_window_ms`（默认 60_000）。
- 阈值：
  - `max_actions_per_ip_window`（默认 20）。
  - `max_actions_per_pubkey_window`（默认 10）。
- 统计来源：扫描时间窗内 `feedback/audit/*.json` 的 accepted mutation 事件。

### 删除语义
- 删除仅写 tombstone 事件，不物理删除 root/event 文件。
- 公共读取视图中返回 `tombstoned=true` 与 `tombstone_reason`。

## 里程碑
- M1：T0 文档建档完成（设计 + 项目管理）。
- M2：T1 核心反馈模块完成并通过 `agent_world_distfs` 定向单测。
- M3：T2 CLI 与闭环测试完成，文档/devlog 收口。

## 风险
- 审计日志扫描限流在高并发场景会有性能开销；本期接受，后续可改为滚动索引。
- 全量公开读可能出现恶意内容暴露；本期按需求保持公开，后续可扩展审核层。
- nonce 占位在并发竞争下依赖文件写时序；本期以单进程/低并发为目标。
