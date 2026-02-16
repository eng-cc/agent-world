# Agent World Runtime：成员目录吊销来源鉴权与审计落盘归档（设计文档）

## 目标
- 为成员目录 `membership.revoke` 吊销消息增加来源鉴权能力，降低伪造吊销广播风险。
- 在吊销同步阶段提供策略化校验（trusted requester / signature / key_id 白名单与吊销名单）。
- 提供可落盘的审计存储实现，支持跨重启查询成员目录恢复审计记录。

## 范围

### In Scope（本次实现）
- 扩展 `MembershipKeyRevocationAnnounce`：
  - 增加可选 `signature_key_id`
  - 增加可选 `signature`
- 扩展签名器与 keyring：
  - `MembershipDirectorySigner` 增加吊销消息签名/验签能力
  - `MembershipDirectorySignerKeyring` 增加吊销消息签名/验签能力
- 新增吊销同步策略与报告：
  - `MembershipRevocationSyncPolicy`
  - `MembershipRevocationSyncReport`
  - `sync_key_revocations_with_policy(...)`
- 扩展吊销发布入口：
  - `publish_key_revocation_signed(...)`
  - `publish_key_revocation_signed_by_key_id(...)`
  - `publish_key_revocation_signed_with_keyring(...)`
- 新增落盘审计存储：
  - `FileMembershipAuditStore`（JSONL）
  - 支持 append/list 查询

### Out of Scope（本次不做）
- 吊销消息的租约/角色级授权（如必须由 lease holder 发起）。
- 吊销审计与外部 SIEM/数据库集成。
- 分布式多副本审计归档一致性协议。

## 接口 / 数据

### 吊销消息
- `MembershipKeyRevocationAnnounce`
  - `world_id`
  - `requester_id`
  - `requested_at_ms`
  - `key_id`
  - `reason`
  - `signature_key_id?`
  - `signature?`

### 吊销同步策略
- `MembershipRevocationSyncPolicy`
  - `trusted_requesters`
  - `require_signature`
  - `require_signature_key_id`
  - `accepted_signature_key_ids`
  - `revoked_signature_key_ids`

### 吊销同步报告
- `MembershipRevocationSyncReport`
  - `drained`
  - `applied`
  - `ignored`
  - `rejected`

### 审计落盘
- `FileMembershipAuditStore`
  - 文件格式：每行一条 `MembershipSnapshotAuditRecord` JSON（JSONL）
  - 路径约定：`<root>/<world_id>.jsonl`

## 里程碑
- **MR1**：文档与任务拆解完成。
- **MR2**：吊销消息签名与来源校验策略落地。
- **MR3**：吊销同步报告与测试回归通过。
- **MR4**：审计落盘实现、总文档/日志更新完成。

## 风险
- 若各节点 keyring 配置不一致，可能出现“部分节点接受、部分节点拒绝”的吊销分歧。
- JSONL 落盘适合轻量归档，高并发场景需后续替换专用后端。
- 鉴权策略配置过严可能导致合法吊销消息被拒收，需要运维流程配套。
