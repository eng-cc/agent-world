> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录审计持久化与吊销传播（设计文档）

## 目标
- 将成员目录恢复审计结果持久化，形成可追溯的恢复记录链。
- 增加签名密钥吊销传播机制，在多节点间同步失效 key_id。
- 在恢复校验阶段拒绝已吊销 key_id，降低已泄露密钥继续生效的风险。

## 范围

### In Scope（本次实现）
- 新增审计存储抽象与内存参考实现：
  - `MembershipAuditStore`（append/list）
  - `InMemoryMembershipAuditStore`
- 新增“恢复 + 审计持久化”入口：
  - `restore_membership_from_dht_verified_with_audit_store(...)`
- 新增密钥吊销广播结构与同步能力：
  - `MembershipKeyRevocationAnnounce`
  - 发布/订阅/消费吊销消息
- 扩展 keyring：
  - 维护 revoked key 集
  - 验签前拒绝吊销 key
- 恢复策略补充吊销名单约束：
  - `MembershipSnapshotRestorePolicy.revoked_signature_key_ids`

### Out of Scope（本次不做）
- 审计日志外部后端（数据库/对象存储）落地。
- 吊销消息跨 world 的层级管理与租户隔离策略。
- 基于硬件可信根（HSM/KMS）的签名密钥托管。

## 接口 / 数据

### 审计持久化
- `trait MembershipAuditStore`：
  - `append(record: &MembershipSnapshotAuditRecord)`
  - `list(world_id: &str)`
- `MembershipSyncClient::restore_membership_from_dht_verified_with_audit_store(...)`
  - 先执行现有 restore+audit
  - 再将 `audit` 写入 store

### 吊销传播
- gossipsub topic：`aw.<world_id>.membership.revoke`
- `MembershipKeyRevocationAnnounce` 字段：
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `key_id`
  - `reason`
- `MembershipSyncClient` 能力：
  - `publish_key_revocation(...)`
  - `drain_key_revocations(...)`
  - `sync_key_revocations(...)`

### keyring 校验
- `MembershipDirectorySignerKeyring` 新增：
  - `revoke_key(key_id)`
  - `is_key_revoked(key_id)`
  - `revoked_keys()`
- 验签路径在以下场景拒绝吊销 key：
  - 快照显式带 `signature_key_id`
  - 按 keyring 遍历尝试时命中吊销 key

### 恢复策略
- `MembershipSnapshotRestorePolicy.revoked_signature_key_ids: Vec<String>`
- restore 校验时若快照 key_id 在策略吊销名单中，直接拒绝。

## 里程碑
- **MR1**：设计文档与项目管理文档完成。
- **MR2**：审计持久化抽象、恢复入口与测试完成。
- **MR3**：吊销传播协议、keyring 吊销校验与测试完成。
- **MR4**：回归验证、总文档和开发日志更新。

## 风险
- 内存审计存储仅用于进程级验证，生产需要替换持久化后端。
- 吊销消息本身若无额外认证，仍依赖上层 requester 信任策略。
- 吊销同步存在传播延迟窗口，需结合恢复策略吊销名单兜底。
