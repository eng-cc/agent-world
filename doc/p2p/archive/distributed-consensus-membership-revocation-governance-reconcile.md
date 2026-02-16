> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：成员目录吊销授权治理与跨节点对账（设计文档）

## 目标
- 在吊销同步链路中增加“授权治理”约束，避免仅“可信来源”但无治理授权的 requester 触发吊销。
- 提供跨节点 revoked key 集对账机制，识别并收敛节点间吊销状态漂移。
- 保持与既有吊销广播/签名能力兼容，支持渐进启用策略。

## 范围

### In Scope（本次实现）
- 扩展 `MembershipRevocationSyncPolicy`：
  - 新增 `authorized_requesters`
- `sync_key_revocations_with_policy(...)` 增加授权校验：
  - requester 需满足 trusted 与 authorized 策略
- 新增对账 topic 与消息：
  - `aw.<world_id>.membership.reconcile`
  - `MembershipRevocationCheckpointAnnounce`
- 新增对账策略与报告：
  - `MembershipRevocationReconcilePolicy`
  - `MembershipRevocationReconcileReport`
- 新增跨节点对账入口：
  - `publish_revocation_checkpoint(...)`
  - `drain_revocation_checkpoints(...)`
  - `reconcile_revocations_with_policy(...)`
- 对账支持可选自动收敛：
  - `auto_revoke_missing_keys=true` 时，本地补齐远端缺失吊销 key

### Out of Scope（本次不做）
- 吊销授权与 lease holder/BFT 提案的强绑定。
- 吊销对账结果的外部告警系统联动。
- 对账结果写入独立分布式账本或数据库。

## 接口 / 数据

### 吊销授权策略
- `MembershipRevocationSyncPolicy.authorized_requesters`
  - 为空：不启用显式授权名单
  - 非空：requester 必须命中名单

### 对账消息
- `MembershipRevocationCheckpointAnnounce`
  - `world_id`
  - `node_id`
  - `announced_at_ms`
  - `revoked_key_ids`
  - `revoked_set_hash`

### 对账策略与报告
- `MembershipRevocationReconcilePolicy`
  - `trusted_nodes`
  - `auto_revoke_missing_keys`
- `MembershipRevocationReconcileReport`
  - `drained`
  - `in_sync`
  - `diverged`
  - `merged`
  - `rejected`

## 里程碑
- **MR1**：设计/项目文档完成。
- **MR2**：授权治理策略实现与单测。
- **MR3**：跨节点对账消息、策略、报告实现与单测。
- **MR4**：回归验证与总文档/日志更新。

## 风险
- 授权名单配置不一致会造成节点吊销处理分歧。
- 自动收敛策略若配置不当，可能将异常远端状态扩散到本地。
- 对账基于 gossip 广播存在时延窗口，需要配合周期策略持续收敛。
