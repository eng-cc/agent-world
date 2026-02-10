# Agent World Runtime：成员目录快照密钥轮换与审计（设计文档）

## 目标
- 让成员目录快照签名支持 `key_id`，使签名密钥可以在不破坏恢复能力的前提下轮换。
- 提供“多密钥验签”能力：新旧密钥共存期可同时验证历史快照与新快照。
- 为恢复链路增加结构化审计输出，记录恢复成功/忽略/拒绝/缺失等状态。

## 范围

### In Scope（本次实现）
- 在成员目录广播与 DHT 快照结构中增加 `signature_key_id`（可选，向后兼容）。
- 新增 `MembershipDirectorySignerKeyring`：
  - 管理多把签名密钥。
  - 指定 active key。
  - 支持按 `key_id` 签名、按 `key_id` 验签、兼容无 `key_id` 的历史签名。
- 扩展恢复策略：
  - `require_signature_key_id`
  - `accepted_signature_key_ids`
- 新增恢复审计结构与接口：
  - `MembershipSnapshotAuditRecord`
  - `MembershipRestoreAuditReport`
  - `restore_membership_from_dht_verified_with_audit`

### Out of Scope（本次不做）
- 自动密钥下发与远程 KMS 集成。
- 密钥失效吊销广播协议。
- 审计日志持久化后端（当前仅结构化返回）。

## 接口 / 数据

### 数据结构
- `MembershipDirectorySnapshot.signature_key_id: Option<String>`
- `MembershipDirectoryAnnounce.signature_key_id: Option<String>`

### Keyring
- `MembershipDirectorySignerKeyring::add_hmac_sha256_key(key_id, key)`
- `MembershipDirectorySignerKeyring::set_active_key(key_id)`
- `MembershipDirectorySignerKeyring::sign_snapshot_with_active_key(snapshot)`
- `MembershipDirectorySignerKeyring::verify_snapshot(snapshot)`

### 恢复策略
- `MembershipSnapshotRestorePolicy`
  - `trusted_requesters`
  - `require_signature`
  - `require_signature_key_id`
  - `accepted_signature_key_ids`

### 审计输出
- `MembershipSnapshotAuditOutcome`：`missing_snapshot/applied/ignored/rejected`
- `MembershipSnapshotAuditRecord`
- `MembershipRestoreAuditReport`

## 里程碑
- **MR1**：数据结构补充 `signature_key_id` 并保持兼容。
- **MR2**：实现 keyring 轮换签名/验签能力。
- **MR3**：恢复策略补充 key_id 控制项。
- **MR4**：恢复审计输出与测试回归。

## 风险
- keyring 仅进程内配置，跨节点配置不一致会导致恢复拒绝。
- 兼容无 `key_id` 历史签名时，策略若过宽可能降低约束力度。
- 审计结果目前未自动落盘，仍需上层系统收集归档。
